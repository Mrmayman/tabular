// //! A default implementation of `Tabular` in a `Vec<Vec<T>>` format.
use super::{Internal, Reference, ReferenceRange, Tabular};
use crate::{Address, Range};
use iced::advanced::{Renderer, renderer};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone)]
pub struct Content<T = Cell, K: Reference = Address, R: ReferenceRange<K> = Range> {
    columns: Vec<Vec<T>>,
    selection: R,
    col_widths: Vec<f32>,
    row_heights: Vec<f32>,
    range: R,
    internal: Internal,
    _phantom: std::marker::PhantomData<K>,
}

#[derive(Default, Clone)]
pub struct Cell<T = String> {
    pub content: T,
    pub border: Option<iced::Border>,
}

impl<T> From<T> for Cell<T> {
    fn from(content: T) -> Self {
        Self {
            content,
            border: None,
        }
    }
}
impl crate::tabular::Cell for Cell {
    fn has_borders(&self) -> bool {
        self.border.is_some()
    }

    fn fill_border_quads<R: Renderer>(
        &self,
        renderer: &mut R,
        bounds: iced::Rectangle,
        scaling: f32,
    ) {
        if let Some(border) = &self.border {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: bounds * scaling,
                    border: *border,
                    ..Default::default()
                },
                iced::Color::TRANSPARENT,
            )
        }
    }
}

impl<T, K: Reference, R: ReferenceRange<K>> Tabular<T, K, R> for Content<T, K, R>
where
    T: Default,
{
    fn from_range(range: &R) -> Self {
        Self {
            columns: Vec::new(),
            selection: R::default(),
            col_widths: vec![],
            row_heights: vec![],
            range: range.clone(),
            internal: Internal::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    fn range(&self) -> &R {
        &self.range
    }

    fn get(&self, cell: impl Into<K>) -> Option<&T> {
        let (x, y) = cell.into().as_tuple();
        self.columns.get(x).and_then(|col| col.get(y))
    }

    fn get_mut(&mut self, cell: impl Into<K>) -> Option<&mut T> {
        let (x, y) = cell.into().as_tuple();
        self.columns.get_mut(x).and_then(|col| col.get_mut(y))
    }

    fn insert(&mut self, cell: impl Into<K>, item: impl Into<T>) {
        let (x, y) = cell.into().as_tuple();

        // Find the max dimensions needed
        let needed_cols = (x + 1).max(self.column_count());
        let needed_rows = (y + 1).max(self.row_count());

        // Ensure we have a uniform grid
        self.ensure_uniform_grid(needed_rows, needed_cols);

        // Now we can safely insert
        self.columns[x][y] = item.into();
    }

    fn row_count(&self) -> usize {
        self.columns.first().map(|col| col.len()).unwrap_or(0)
    }

    fn column_count(&self) -> usize {
        self.columns.len()
    }

    fn column_sizes(&self) -> &[f32] {
        &self.col_widths
    }

    fn row_sizes(&self) -> &[f32] {
        &self.row_heights
    }

    fn column_sizes_mut(&mut self) -> &mut Vec<f32> {
        &mut self.col_widths
    }

    fn row_sizes_mut(&mut self) -> &mut Vec<f32> {
        &mut self.row_heights
    }

    fn internal(&self) -> &Internal {
        &self.internal
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (K, &T)> + '_> {
        Box::new(ContentIterator {
            columns: &self.columns,
            col: 0,
            row: 0,
            offset_col: 0,
            offset_row: 0,
            _phantom: std::marker::PhantomData,
        })
    }

    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = (K, &mut T)> + '_> {
        Box::new(ContentIteratorMut {
            columns: &mut self.columns,
            col: 0,
            row: 0,
            offset_col: 0,
            offset_row: 0,
            _phantom: std::marker::PhantomData,
        })
    }

    fn iter_relative(&self) -> Box<dyn Iterator<Item = (K, &T)> + '_> {
        Box::new(ContentIterator {
            columns: &self.columns,
            col: 0,
            row: 0,
            offset_col: self.range.start().x(),
            offset_row: self.range.start().y(),
            _phantom: std::marker::PhantomData,
        })
    }

    fn iter_relative_mut(&mut self) -> Box<dyn Iterator<Item = (K, &mut T)> + '_> {
        Box::new(ContentIteratorMut {
            columns: &mut self.columns,
            col: 0,
            row: 0,
            offset_col: self.range.start().x(),
            offset_row: self.range.start().y(),
            _phantom: std::marker::PhantomData,
        })
    }

    fn selection(&self) -> &R {
        &self.selection
    }

    fn selection_mut<'a>(&'a mut self) -> &'a mut R
    where
        K: 'a,
    {
        &mut self.selection
    }

    fn select_cell(&mut self, cell: K) {
        self.selection = cell.as_range();
    }

    fn select_range(&mut self, range: R) {
        self.selection = range;
    }

    fn select_all(&mut self) {
        let start = K::from((0, 0));
        let end = K::from((self.column_count(), self.row_count()));
        self.selection = R::new(start, Some(end));
    }

    fn with_reference(&mut self, cell: impl Into<K>, f: impl Fn(&K, &mut T)) {
        let key = cell.into();
        if let Some(value) = self.get_mut(key) {
            f(&key, value);
        }
    }

    fn with_reference_range(&mut self, range: &R, f: impl Fn(&K, &mut T)) {
        let normalized = range.normalize();
        for cell in normalized.iter() {
            if let Some(value) = self.get_mut(cell) {
                f(&cell, value);
            }
        }
    }
}

impl<T, K: Reference, R: ReferenceRange<K>> Content<T, K, R> {
    /// Set the row heights for the grid.
    #[allow(unused)]
    pub fn with_row_heights(self, sizes: Vec<f32>) -> Self {
        Self {
            row_heights: sizes,
            ..self
        }
    }

    /// Set the column widths for the grid.
    #[allow(unused)]
    pub fn with_column_widths(self, sizes: Vec<f32>) -> Self {
        Self {
            col_widths: sizes,
            ..self
        }
    }

    pub fn with_range(range: R) -> Self
    where
        T: Default,
    {
        // Normalize the range to get consistent dimensions
        let normalized = range.normalize();
        let start = normalized.start();
        let end = normalized.end().unwrap_or(start);

        // Calculate dimensions (add 1 since end is inclusive)
        let cols = end.x().saturating_sub(start.x()) + 1;
        let rows = end.y().saturating_sub(start.y()) + 1;

        // Create the grid with default values
        let mut columns = Vec::with_capacity(cols);
        for _ in 0..cols {
            let mut column = Vec::with_capacity(rows);
            column.resize_with(rows, T::default);
            columns.push(column);
        }

        Self {
            columns,
            selection: R::default(),
            col_widths: vec![100.0; cols],
            row_heights: vec![20.0; rows],
            range,
            internal: Internal::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    fn ensure_uniform_grid(&mut self, rows: usize, cols: usize)
    where
        T: Default,
    {
        // First ensure we have enough column capacity
        if self.columns.len() < cols {
            self.columns.resize_with(cols, || Vec::with_capacity(rows));
            self.col_widths.resize(cols, 100.0);
        }

        // Ensure each column has the right number of rows
        for column in &mut self.columns {
            column.resize_with(rows, T::default);
        }

        // Update row heights array
        self.row_heights.resize(rows, 20.0);
    }

    /// Set the height for a specific row
    pub fn set_row_height(&mut self, row: usize, height: f32) {
        if row < self.row_heights.len() {
            self.row_heights[row] = height;
        }
    }

    /// Set the width for a specific column
    pub fn set_column_width(&mut self, col: usize, width: f32) {
        if col < self.col_widths.len() {
            self.col_widths[col] = width;
        }
    }
}

struct ContentIterator<'a, T, K: Reference> {
    columns: &'a Vec<Vec<T>>,
    col: usize,
    row: usize,
    offset_col: usize,
    offset_row: usize,
    _phantom: std::marker::PhantomData<K>,
}

struct ContentIteratorMut<'a, T, K: Reference> {
    columns: &'a mut Vec<Vec<T>>,
    col: usize,
    row: usize,
    offset_col: usize,
    offset_row: usize,
    _phantom: std::marker::PhantomData<K>,
}

impl<'a, T, K: Reference> Iterator for ContentIterator<'a, T, K> {
    type Item = (K, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.col >= self.columns.len() {
            return None;
        }

        let current_col = &self.columns[self.col];
        if self.row >= current_col.len() {
            self.col += 1;
            self.row = 0;
            return self.next();
        }

        let item = &current_col[self.row];
        let cell_ref = K::new(self.col + self.offset_col, self.row + self.offset_row);
        self.row += 1;
        Some((cell_ref, item))
    }
}

impl<'a, T, K: Reference> Iterator for ContentIteratorMut<'a, T, K> {
    type Item = (K, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.col >= self.columns.len() {
            return None;
        }

        let current_col = &mut self.columns[self.col];
        if self.row >= current_col.len() {
            self.col += 1;
            self.row = 0;
            return self.next();
        }

        // Create the cell reference
        let cell_ref = K::new(self.col + self.offset_col, self.row + self.offset_row);
        self.row += 1;

        // SAFETY: We know this reference is unique because we're iterating through the Vec mutably
        // and we increment the indices before returning
        let item = unsafe {
            let ptr = &mut self.columns[self.col][self.row - 1] as *mut T;
            &mut *ptr
        };

        Some((cell_ref, item))
    }
}

impl<T, K: Reference, R: ReferenceRange<K>> Default for Content<T, K, R>
where
    T: Default,
{
    fn default() -> Self {
        Self::from_range(&R::default())
    }
}

#[cfg(feature = "serde")]
impl<T: Serialize, K: Reference, R: ReferenceRange<K> + Serialize> Serialize for Content<T, K, R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Content", 5)?;
        state.serialize_field("items", &self.columns)?;
        state.serialize_field("selection", &self.selection)?;
        state.serialize_field("col_widths", &self.col_widths)?;
        state.serialize_field("row_heights", &self.row_heights)?;
        state.serialize_field("range", &self.range)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de>, K: Reference, R: ReferenceRange<K> + Deserialize<'de>>
    Deserialize<'de> for Content<T, K, R>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ContentHelper<T, R> {
            items: Vec<Vec<T>>,
            selection: R,
            col_widths: Vec<f32>,
            row_heights: Vec<f32>,
            range: R,
        }

        let helper = ContentHelper::deserialize(deserializer)?;

        Ok(Content {
            columns: helper.items,
            selection: helper.selection,
            col_widths: helper.col_widths,
            row_heights: helper.row_heights,
            range: helper.range,
            internal: Internal::default(),
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<T: PartialEq, K: Reference, R: ReferenceRange<K>> PartialEq for Content<T, K, R> {
    // Ignore internal
    fn eq(&self, other: &Self) -> bool {
        self.columns == other.columns
            && self.selection == other.selection
            && self.col_widths == other.col_widths
            && self.row_heights == other.row_heights
    }
}
