use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Layout, layout, renderer};
use iced::{Border, Color, Element, Length, Point, Rectangle, Size, mouse};

pub struct Table<'a, Message, Theme, Renderer> {
    cells: Vec<Element<'a, Message, Theme, Renderer>>,
    columns: usize,
    col_sizes: &'a [f32],
    row_sizes: &'a [f32],
    width: Length,
    height: Length,
}

impl<'a, Message, Theme, Renderer> Table<'a, Message, Theme, Renderer> {
    pub fn new<T>(
        cells: &'a Vec<Vec<T>>,
        view_cell: impl Fn(&'a T) -> Element<'a, Message, Theme, Renderer> + 'static,
        col_sizes: &'a [f32],
        row_sizes: &'a [f32],
    ) -> Self {
        let columns = col_sizes.len();
        Self {
            cells: cells
                .iter()
                .flat_map(|row| row.iter().map(&view_cell))
                .collect(),
            columns,
            col_sizes,
            row_sizes,
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    pub fn with_width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

pub fn table<'a, T, Message, Theme, Renderer>(
    cells: &'a Vec<Vec<T>>,
    view_cell: impl Fn(&'a T) -> Element<'a, Message, Theme, Renderer> + 'static,
    col_sizes: &'a [f32],
    row_sizes: &'a [f32],
) -> Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Message: 'a,
    Theme: 'a,
    T: 'a,
{
    Table::new(cells, view_cell, col_sizes, row_sizes).into()
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Table<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.cells.iter().collect::<Vec<_>>());
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.cells.iter().map(Tree::new).collect()
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width: f32 = self.col_sizes.iter().sum();
        let height: f32 = self.row_sizes.iter().sum();

        let size = limits.resolve(self.width, self.height, Size::new(width, height));

        let children = self
            .cells
            .iter()
            .enumerate()
            .map(|(idx, cell)| {
                let row = idx / self.columns;
                let col = idx % self.columns;

                // Calculate cell size
                let col_width = size.width * self.col_sizes[col] / width;
                let row_height = size.height * self.row_sizes[row] / height;

                // Calculate position
                let x = self
                    .col_sizes
                    .iter()
                    .take(col)
                    .map(|cs| size.width * cs / width)
                    .sum();
                let y = self
                    .row_sizes
                    .iter()
                    .take(row)
                    .map(|rs| size.height * rs / height)
                    .sum();

                let cell_size = Size::new(col_width, row_height);
                let cell_limits = layout::Limits::new(cell_size, cell_size);

                let node = cell
                    .as_widget()
                    .layout(&mut tree.children[idx], renderer, &cell_limits);

                node.move_to(Point::new(x, y))
            })
            .collect();

        layout::Node::with_children(size, children)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        // Draw intermediate horizontal lines
        let total_height = self.row_sizes.iter().sum::<f32>();
        let mut y = bounds.y;
        for row_height in self.row_sizes.iter().take(self.row_sizes.len() - 1) {
            y += row_height * bounds.height / total_height;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x,
                        y,
                        width: bounds.width,
                        height: 1.0,
                    },
                    border: Border::default(),
                    ..Default::default()
                },
                Color::from_rgb(0.7, 0.7, 0.7),
            );
        }

        // Draw intermediate vertical lines
        let total_width = self.col_sizes.iter().sum::<f32>();
        let mut x = bounds.x;
        for col_width in self.col_sizes.iter().take(self.columns - 1) {
            x += col_width * bounds.width / total_width;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x,
                        y: bounds.y,
                        width: 1.0,
                        height: bounds.height,
                    },
                    border: Border::default(),
                    ..Default::default()
                },
                Color::from_rgb(0.7, 0.7, 0.7),
            );
        }

        // Draw outer border
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    width: 1.0,
                    color: Color::from_rgb(0.7, 0.7, 0.7),
                    radius: 0.0.into(),
                },
                ..Default::default()
            },
            Color::TRANSPARENT,
        );

        // Draw children
        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            for (child, (state, layout)) in self
                .cells
                .iter()
                .zip(tree.children.iter().zip(layout.children()))
                .filter(|(_, (_, layout))| layout.bounds().intersects(&clipped_viewport))
            {
                child.as_widget().draw(
                    state,
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor,
                    &clipped_viewport,
                );
            }
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Table<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + 'a,
    Message: 'a,
    Theme: 'a,
{
    fn from(table: Table<'a, Message, Theme, Renderer>) -> Self {
        Self::new(table)
    }
}
