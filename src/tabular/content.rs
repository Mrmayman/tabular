use std::cell::RefCell;
use std::slice::SliceIndex;

use super::action::Edit;
use super::update::Motion;
use super::{Action, Axis, Reference, ReferenceRange};

pub mod list;

pub trait Tabular<T: Default, K: Reference, R: ReferenceRange<K>> {
    /// Create a new table.
    fn from_range(range: &R) -> Self;
    /// Get the range
    fn range(&self) -> &R;

    /// Get the item at the given cell.
    fn get(&self, cell: impl Into<K>) -> Option<&T>
    where
        usize: SliceIndex<[Vec<T>]>;
    /// Get the mutable item at the given cell.
    fn get_mut(&mut self, cell: impl Into<K>) -> Option<&mut T>
    where
        usize: SliceIndex<[Vec<T>]>;
    /// Insert an item at the given cell.
    fn insert(&mut self, cell: impl Into<K>, item: impl Into<T>);

    /// The number of rows in the table.
    fn row_count(&self) -> usize;
    /// The number of columns in the table.
    fn column_count(&self) -> usize;
    /// The sizes of the columns.
    fn column_sizes(&self) -> &[f32];
    /// The sizes of the rows.
    fn row_sizes(&self) -> &[f32];
    /// A mutable reference to the sizes of the columns.
    fn column_sizes_mut(&mut self) -> &mut Vec<f32>;
    /// A mutable reference to the sizes of the rows.
    fn row_sizes_mut(&mut self) -> &mut Vec<f32>;

    /// A reference to the internal state of the widget (the impure bits?)
    fn internal(&self) -> &Internal;

    /// An iterator over the items in the table.
    fn iter(&self) -> Box<dyn Iterator<Item = (K, &T)> + '_>;
    /// A mutable iterator over the items in the table.
    fn iter_mut(&mut self) -> Box<dyn Iterator<Item = (K, &mut T)> + '_>;
    /// An iterator over the items in the table, relative to their position in the source sheet.
    fn iter_relative(&self) -> Box<dyn Iterator<Item = (K, &T)> + '_>;
    /// A mutable iterator over the items in the table, relative to their position in the source sheet.
    fn iter_relative_mut(&mut self) -> Box<dyn Iterator<Item = (K, &mut T)> + '_>;

    /// Apply a function to the item at the given cell
    fn with_reference(&mut self, cell: impl Into<K>, f: impl Fn(&K, &mut T));

    /// Apply a function to the items in a given range
    fn with_reference_range(&mut self, range: &R, f: impl Fn(&K, &mut T));

    /// Select the given cell.
    fn select_cell(&mut self, cell: K);
    /// Select the given range.
    fn select_range(&mut self, range: R);
    /// Select all cells.
    fn select_all(&mut self);
    /// The currently selected cells.
    fn selection(&self) -> &R;
    /// A mutable reference to the selection.
    fn selection_mut<'a>(&'a mut self) -> &'a mut R
    where
        K: 'a;

    /// Perform a grid [`Action`].
    fn perform(&mut self, action: Action<K, R>) {
        match action {
            // editing actions will trigger a recalculation.
            Action::Edit(edit) => match edit {
                Edit::Delete => self.selection_mut().iter().for_each(|cell| {
                    self.get_mut(cell).map(|item| *item = T::default());
                }),
            },
            Action::Select(range) => self.select_range(range),
            Action::SelectAll => self.select_all(),
            Action::MoveSelection(motion) => self.move_selection(motion),
            Action::ExpandSelection(motion) => self.expand_selection(motion),
            Action::ResizeDivider(axis, index, size) => {
                let size = size.clamp(0.0, f32::INFINITY);
                match axis {
                    Axis::Column => {
                        self.column_sizes_mut()
                            .get_mut(index)
                            .map(|col| *col += size);
                    }
                    Axis::Row => {
                        self.row_sizes_mut().get_mut(index).map(|row| *row += size);
                    }
                }
            }
            Action::_Phantom(_) => {}
        }
    }

    /// Do something with the selected cells.
    fn with_selection<O>(&self, f: impl FnOnce(&R) -> O) -> O {
        f(&self.selection())
    }

    /// Moves the active cell within or beyond the current selection based on the motion
    fn move_selection(&mut self, motion: Motion) {
        let (start_col, start_row) = self.selection().start().as_tuple();
        let (end_col, end_row) = self
            .selection()
            .end()
            .unwrap_or(self.selection().start())
            .as_tuple();

        // "start" should always be before "end", so rename them "first" and "last"
        let (first_col, first_row) = (start_col.min(end_col), start_row.min(end_row));
        let (last_col, last_row) = (start_col.max(end_col), start_row.max(end_row));

        // prevent out-of-bounds movement
        let max_col = self.column_count().saturating_sub(1);
        let max_row = self.row_count().saturating_sub(1);

        // Handle tab behavior for single-cell selections
        let motion = match motion {
            Motion::Forward => {
                if self.selection().end().is_none() {
                    Motion::Right
                } else {
                    Motion::Forward
                }
            }
            Motion::Back => {
                if self.selection().end().is_none() {
                    Motion::Left
                } else {
                    Motion::Back
                }
            }
            _ => motion,
        };

        let new_cell = match motion {
            Motion::Forward => {
                let num_cols = last_col - first_col + 1;
                let next_col = (start_col - first_col + 1) % num_cols + first_col;
                let next_row = if next_col == first_col {
                    start_row.saturating_add(1)
                } else {
                    start_row
                };
                let wrapped_row = if next_row > last_row {
                    first_row
                } else {
                    next_row
                };
                K::new(next_col.min(max_col), wrapped_row.min(max_row))
            }
            Motion::Back => {
                let prev_col = if start_col == first_col {
                    last_col
                } else {
                    start_col.saturating_sub(1)
                };
                let prev_row = if prev_col == last_col && start_row == first_row {
                    last_row
                } else if prev_col == last_col {
                    start_row.saturating_sub(1)
                } else {
                    start_row
                };
                let wrapped_row = if prev_row < first_row {
                    last_row
                } else {
                    prev_row
                };
                K::new(prev_col.min(max_col), wrapped_row.min(max_row))
            }
            Motion::Up => K::new(start_col.min(max_col), start_row.saturating_sub(1)),
            Motion::Down => K::new(
                start_col.min(max_col),
                start_row.saturating_add(1).min(max_row),
            ),
            Motion::Right => K::new(
                start_col.saturating_add(1).min(max_col),
                start_row.min(max_row),
            ),
            Motion::Left => K::new(start_col.saturating_sub(1), start_row.min(max_row)),
            Motion::Home => K::new(first_col.min(max_col), start_row.min(max_row)),
            Motion::End => K::new(last_col.min(max_col), start_row.min(max_row)),
            Motion::DocumentStart => K::new(0, 0),
            Motion::DocumentEnd => K::new(max_col, max_row),
        };

        // Update the region's selection state with the new active cell
        self.select_range(R::new(new_cell, None));
    }

    /// Expands the current selection in the specified direction
    fn expand_selection(&mut self, motion: Motion) {
        let (add_x, add_y): (i16, i16) = match motion {
            Motion::Left => (-1, 0),
            Motion::Right => (1, 0),
            Motion::Up => (0, -1),
            Motion::Down => (0, 1),
            _ => (0, 0),
        };

        // Get grid bounds to prevent out-of-bounds expansion
        let max_col = self.column_count().saturating_sub(1);
        let max_row = self.row_count().saturating_sub(1);

        let start = self.selection().start();
        let end = self.selection().end().unwrap_or(start);

        // Calculate new end position while keeping within grid bounds
        let new_y = ((end.y() as i16).saturating_add(add_y))
            .max(0)
            .min(max_row as i16) as usize;
        let new_x = ((end.x() as i16).saturating_add(add_x))
            .max(0)
            .min(max_col as i16) as usize;

        let new_end = K::new(new_x, new_y);

        self.select_range(R::new(start, Some(new_end)))
    }
}

pub struct Internal {
    /// Whether the `Tabular` content is dirty and needs to be rebuilt by the
    /// widget.
    is_dirty: RefCell<bool>,
}

impl Clone for Internal {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl Internal {
    /// Mark the content as dirty.
    pub fn set_dirty(&self) {
        *self.is_dirty.borrow_mut() = true;
    }

    /// Mark the content as clean.
    pub fn set_clean(&self) {
        *self.is_dirty.borrow_mut() = false;
    }

    /// Check if the content is dirty.
    pub fn is_dirty(&self) -> bool {
        *self.is_dirty.borrow()
    }
}

impl Default for Internal {
    fn default() -> Self {
        Self {
            is_dirty: RefCell::new(true),
        }
    }
}
