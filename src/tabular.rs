//! A table widget for `iced` which is generic over the data type in cells
//! and the types for referencing cells and ranges of cells.
//!
//! The data is expected to implement the `Tabular` trait, which provides
//! methods for accessing and modifying the data in the table.
//!
//! A default implementation of the `Tabular` trait is provided for a
//! `Vec<Vec<T>>` in `content::list`, which may be used as a reference
//! (no pun intended) for implementing the `Tabular` trait for other data
//! structures.
//!
//! Usage:
//!
//! ```rust
//! use tabular::list::{Cell, Content};
//! use tabular::reference::*;
//! use tabular::{Address, Tabular, tabular};
//!
//! fn main() -> iced::Result {
//!     iced::application(App::new, App::update, App::view).run()
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!    Action(tabular::Action),
//!    Instruction(tabular::Instruction),
//! }
//!
//! struct App {
//!    cells: Content,
//! }
//!
//! TODO: finish writing this example.....
//!
//! ```
use action::Edit;
use iced::advanced::widget::{self, Tree, Widget, operation, tree};
use iced::advanced::{Clipboard, Layout, Renderer, Shell, clipboard, layout, mouse, renderer};
use iced::{Border, Color, Element, Length, Point, Rectangle, Size, Vector};

mod action;
mod content;
pub mod reference;
mod theme;
mod update;
mod utils;

use reference::{Reference, ReferenceRange};

pub trait Cell {
    // Method to check if the cell has any borders to draw
    fn has_borders(&self) -> bool;

    // Method to draw border quads for the cell
    fn fill_border_quads<R: Renderer>(
        &self,
        renderer: &mut R,
        bounds: iced::Rectangle,
        scaling: f32,
    );
}

pub use action::{Action, Instruction};
pub use content::{Internal, Tabular, list};
pub use reference::{Address, Range};
pub use theme::*;
pub use update::{Binding, KeyPress, Update};
pub use utils::*;

pub struct Table<'a, Data, T, K, R, Message, Theme, Renderer>
where
    Data: Tabular<T, K, R>,
    T: Cell + Default + 'a,
    K: Reference,
    R: ReferenceRange<K>,
    Message: Clone,
    Theme: Catalog,
{
    // The id of the [`Table`]
    id: Option<widget::Id>,
    // The source data
    data: &'a Data,
    // The cells in the grid
    cells: Vec<(K, Element<'a, Message, Theme, Renderer>)>,
    // The number of columns in the grid
    columns: usize,
    // The width of the grid
    width: Length,
    // The height of the grid
    height: Length,
    // The spacing amount between cells
    spacing: Size,
    // Whether to show gridlines
    show_gridlines: bool,
    // If true, all single clicks will passthrough to children. If not,
    // only double clicks will.
    passthrough: bool,
    // The function that is called when an action is performed in the grid
    on_edit: Option<Box<dyn Fn(Action<K, R>) -> Message + 'a>>,
    // The function that is called when an instruction is emitted by the grid
    on_instruction: Option<Box<dyn Fn(Instruction<K>) -> Message + 'a>>,
    // The function that is called to produce key bindings on key presses
    key_binding: Option<Box<dyn Fn(KeyPress) -> Option<Binding<Message>> + 'a>>,
    // The style class of the grid
    class: <Theme as Catalog>::Class<'a>,

    _phantom: std::marker::PhantomData<T>,
}

impl<'a, Data, T, K, R, Message, Theme, Renderer> Table<'a, Data, T, K, R, Message, Theme, Renderer>
where
    Data: Tabular<T, K, R>,
    T: Cell + Default,
    K: Reference,
    R: ReferenceRange<K>,
    Message: Clone,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub fn new(
        data: &'a Data,
        view_cell: impl Fn(K, &'a T) -> Element<'a, Message, Theme, Renderer> + 'static,
    ) -> Self {
        let columns = data.column_count();
        Self {
            id: None,
            data,
            cells: data
                .iter()
                .map(|(cell_ref, cell)| (cell_ref, view_cell(cell_ref, cell)))
                .collect::<Vec<_>>(),
            columns,
            width: Length::Fill,
            height: Length::Fill,
            show_gridlines: true,
            spacing: Size::ZERO,
            passthrough: false,
            on_edit: None,
            on_instruction: None,
            key_binding: None,
            class: <Theme as Catalog>::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Sets the [`Id`] of the [`Table`].
    pub fn id(mut self, id: widget::Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the spacing between cells in the [`Table`].
    pub fn with_width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Table`].
    pub fn with_height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the spacing between cells in the [`Table`].
    pub fn with_spacing(mut self, spacing: impl Into<Size>) -> Self {
        self.spacing = spacing.into();
        self
    }

    /// Sets whether to show gridlines in the [`Table`].
    pub fn show_gridlines(mut self, show: bool) -> Self {
        self.show_gridlines = show;
        self
    }

    /// Sets whether the [`Table`] should pass through single clicks to its
    /// children.
    pub fn passthrough(mut self, passthrough: bool) -> Self {
        self.passthrough = passthrough;
        self
    }

    /// Sets the message that should be produced when some action is performed
    /// in the [`Table`].
    ///
    /// If this method is not called, the [`Table`] will be disabled.
    pub fn on_action(mut self, on_edit: impl Fn(Action<K, R>) -> Message + 'a) -> Self {
        self.on_edit = Some(Box::new(on_edit));
        self
    }

    /// Sets the message that should be produced when some instruction is
    /// given by the [`Table`].
    pub fn on_instruction(
        mut self,
        on_instruction: impl Fn(Instruction<K>) -> Message + 'a,
    ) -> Self {
        self.on_instruction = Some(Box::new(on_instruction));
        self
    }

    /// Sets the closure to produce key bindings on key presses.
    ///
    /// See [`Binding`] for the list of available bindings.
    pub fn key_binding(
        mut self,
        key_binding: impl Fn(KeyPress) -> Option<Binding<Message>> + 'a,
    ) -> Self {
        self.key_binding = Some(Box::new(key_binding));
        self
    }

    /// Sets the style of the [`Table`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        <Theme as Catalog>::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Table`].
    #[must_use]
    pub fn class(mut self, class: impl Into<<Theme as Catalog>::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Data, T, K, R, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Table<'a, Data, T, K, R, Message, Theme, Renderer>
where
    Data: Tabular<T, K, R>,
    T: Cell + Default,
    K: Reference,
    R: ReferenceRange<K>,
    Message: Clone + 'a,
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::from_data(self.data, self.spacing))
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_ref::<State>();

        // check if dimensions have changed
        if self.data.row_count() != state.region.row_count
            || self.data.column_count() != state.region.column_count
        {
            tree.state = tree::State::new(State::from_data(self.data, self.spacing));
        }

        tree.diff_children(&self.cells.iter().map(|(_, el)| el).collect::<Vec<_>>());
    }

    fn children(&self) -> Vec<widget::Tree> {
        self.cells.iter().map(|(_, el)| Tree::new(el)).collect()
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();

        let size = limits.resolve(
            self.width,
            self.height,
            Size::new(
                state.region.total_raw_width(),
                state.region.total_raw_height(),
            ),
        );

        state.region.scale_to_bounds(size, self.spacing);

        let mut cells = Vec::with_capacity(self.cells.len());
        let rows = (self.cells.len() + self.columns.saturating_sub(1))
            .checked_div(self.columns)
            .unwrap_or(0);

        // Create Rectangle from size for offset calculation
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: size.width,
            height: size.height,
        };

        for col in 0..self.columns {
            for row in 0..rows {
                if col * rows + row >= self.cells.len() {
                    break;
                }

                let pos = state.region.cell_position(row, col);
                let cell_pos = Point::new(bounds.x + pos.x, bounds.y + pos.y);

                cells.push((col * rows + row, cell_pos, state.region.cell_size(row, col)));
            }
        }

        let children = cells
            .into_iter()
            .map(|(idx, position, cell_size)| {
                let cell_limits = layout::Limits::new(cell_size, cell_size);
                let node = self.cells[idx].1.as_widget().layout(
                    &mut tree.children[idx],
                    renderer,
                    &cell_limits,
                );
                node.move_to(position)
            })
            .collect();

        layout::Node::with_children(size, children)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let offset = Vector::new(bounds.x, bounds.y);
        let status = if self.on_edit.is_none() {
            Status::Disabled
        } else if state.is_focused {
            Status::Focused
        } else {
            Status::Unfocused
        };

        let style = Catalog::style(theme, &self.class, status);

        // Draw children
        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            for (child, (state, layout)) in self
                .cells
                .iter()
                .zip(tree.children.iter().zip(layout.children()))
                .filter(|(_, (_, layout))| layout.bounds().intersects(&clipped_viewport))
            {
                child.1.as_widget().draw(
                    state,
                    renderer,
                    theme,
                    defaults,
                    layout,
                    cursor,
                    &clipped_viewport,
                );

                // FIXME: consider computing these at the layout pass instead and rendering simpler quads here.
                if let Some(cell) = self.data.get(child.0) {
                    if cell.has_borders() {
                        cell.fill_border_quads(renderer, layout.bounds(), 1.0);
                    }
                }
            }
        }

        // Draw intermediate vertical lines using cumulative positions
        if self.show_gridlines && self.columns.checked_sub(1).is_some() {
            for x in state
                .region
                .cumulative_x
                .iter()
                .take(self.columns.checked_sub(1).unwrap_or(0))
            {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + x - style.gridlines.width / 2.0,
                            y: bounds.y,
                            width: style.gridlines.width,
                            height: bounds.height,
                        },
                        border: Border::default(),
                        ..Default::default()
                    },
                    style.gridlines.color,
                );
            }

            // Draw intermediate horizontal lines using cumulative positions
            for y in state
                .region
                .cumulative_y
                .iter()
                .take(state.region.scaled_rows.len().checked_sub(1).unwrap_or(0))
            {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y: bounds.y + y - style.gridlines.width / 2.0,
                            width: bounds.width,
                            height: style.gridlines.width,
                        },
                        border: Border::default(),
                        ..Default::default()
                    },
                    style.gridlines.color,
                );
            }
        }

        // Draw outer border
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..Default::default()
            },
            Color::TRANSPARENT,
        );

        let selection_bounds = state.region.selection_bounds(*self.data.selection()) + offset;
        renderer.fill_quad(
            renderer::Quad {
                bounds: selection_bounds,
                border: Border {
                    width: style.selection.stroke_width,
                    color: style.selection.stroke,
                    radius: 0.0.into(),
                },
                ..Default::default()
            },
            style.selection.fill,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
            // Check if we're hovering over a divider
            if let Some(divider_hit) = state.region.find_nearest_divider(cursor_position) {
                return match divider_hit.axis {
                    Axis::Row => mouse::Interaction::ResizingVertically,
                    Axis::Column => mouse::Interaction::ResizingHorizontally,
                };
            }
        }
        mouse::Interaction::default()
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        raw_cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let mut _cells = layout.children();

        let filtered = if self.on_edit.as_ref().is_some() {
            match &event {
                iced::Event::Mouse(mouse_event) => {
                    if matches!(mouse_event, mouse::Event::ButtonPressed(_))
                        && raw_cursor.position_in(layout.bounds()).is_none()
                    {
                        state.interaction = Interaction::None;
                        state.unfocus();
                        return;
                    }
                    Some(event)
                }
                iced::Event::Touch(_) | iced::Event::Keyboard(_) => Some(event),
                iced::Event::Window(iced::window::Event::RedrawRequested(_)) => Some(event),
                _ => None,
            }
        } else {
            eprintln!("Table is disabled. Enable it by calling `.on_action()`");
            None
        };

        if let Some(filtered) = filtered {
            let on_edit = self.on_edit.as_ref().unwrap();

            if let Some(update) = Update::from_event::<K>(
                filtered,
                state,
                layout.bounds(),
                raw_cursor,
                self.key_binding.as_deref(),
            ) {
                match update {
                    Update::RedrawRequested => {
                        if self.data.internal().is_dirty() {
                            self.data.internal().set_clean();
                            *state = State::from_data(self.data, self.spacing);
                            shell.invalidate_layout();
                        }
                    }
                    Update::Click(click) => match click.kind() {
                        mouse::click::Kind::Single => {
                            state.last_click = Some(click);
                            state.drag_click = Some(click.kind());

                            if let Some(divider_hit) =
                                state.region.find_nearest_divider(click.position())
                            {
                                state.interaction = Interaction::ResizeDivider(divider_hit);
                                state.focus();
                                return; // don't click through cells
                            } else {
                                let cell_ref = K::from(state.region.find_cell(click.position()));
                                if !self.data.selection().contains(&cell_ref) {
                                    state.focus();
                                    shell.publish(on_edit(Action::Select(cell_ref.as_range())));
                                }
                                if self.passthrough {
                                    for ((child, state), child_layout) in self
                                        .cells
                                        .iter_mut()
                                        .zip(tree.children.iter_mut())
                                        .zip(layout.children())
                                    {
                                        child.1.as_widget_mut().update(
                                            state,
                                            event,
                                            child_layout,
                                            raw_cursor,
                                            renderer,
                                            clipboard,
                                            shell,
                                            viewport,
                                        );
                                    }
                                }

                                return; // early to avoid second child update()
                            }
                        }
                        mouse::click::Kind::Double | mouse::click::Kind::Triple => {
                            let cell_ref = K::from(state.region.find_cell(click.position()));
                            if !self.data.selection().contains(&cell_ref) {
                                shell.publish(on_edit(Action::Select(cell_ref.as_range())));
                            }
                            if let Some(on_instruction) = self.on_instruction.as_ref() {
                                shell.publish(on_instruction(Instruction::Activate(cell_ref)));
                            }
                            for ((child, state), child_layout) in self
                                .cells
                                .iter_mut()
                                .zip(tree.children.iter_mut())
                                .zip(layout.children())
                            {
                                child.1.as_widget_mut().update(
                                    state,
                                    event,
                                    child_layout,
                                    raw_cursor,
                                    renderer,
                                    clipboard,
                                    shell,
                                    viewport,
                                );
                            }
                            shell.capture_event();
                            return;
                        }
                    },
                    Update::Release => match state.interaction {
                        Interaction::ResizeDivider(hit) => {
                            if let Some(start) = state.last_click.map(|c| c.position()) {
                                if let Some(current) = raw_cursor.position_in(layout.bounds()) {
                                    let raw_delta = match hit.axis {
                                        Axis::Column => {
                                            (current.x - start.x) / state.region.scale_factor_x
                                        }
                                        Axis::Row => {
                                            (current.y - start.y) / state.region.scale_factor_y
                                        }
                                    };

                                    shell.publish(on_edit(Action::ResizeDivider(
                                        hit.axis, hit.index, raw_delta,
                                    )));
                                }
                            }
                            state.interaction = Interaction::None;
                            state.drag_click = None;

                            shell.invalidate_layout();
                            shell.invalidate_widgets();
                            shell.capture_event();
                            return;
                        }
                        Interaction::None => {
                            state.drag_click = None;
                        }
                    },
                    Update::Drag(raw_end) => {
                        if let Some(start) = state.last_click.map(|c| c.position()) {
                            match state.interaction {
                                Interaction::ResizeDivider(hit) => {
                                    // Convert pixel delta to raw delta using stored scale factor
                                    let current = raw_end
                                        - Vector::new(layout.position().x, layout.position().y);
                                    let raw_delta = match hit.axis {
                                        Axis::Column => {
                                            (current.x - start.x) / state.region.scale_factor_x
                                        }
                                        Axis::Row => {
                                            (current.y - start.y) / state.region.scale_factor_y
                                        }
                                    };

                                    // Update the raw sizes
                                    match hit.axis {
                                        Axis::Column => {
                                            if hit.index < state.region.raw_columns.len() {
                                                state.region.raw_columns[hit.index] =
                                                    (hit.original_size + raw_delta).max(0.0);
                                            }
                                        }
                                        Axis::Row => {
                                            if hit.index < state.region.raw_rows.len() {
                                                state.region.raw_rows[hit.index] =
                                                    (hit.original_size + raw_delta).max(0.0);
                                            }
                                        }
                                    }

                                    // Rescale everything based on the new raw sizes
                                    state
                                        .region
                                        .scale_to_bounds(layout.bounds().size(), self.spacing);

                                    shell.invalidate_layout();
                                    shell.invalidate_widgets();
                                    shell.capture_event();
                                    return;
                                }
                                Interaction::None => {
                                    // Only create a new selection if we've actually dragged to a different position
                                    if let Some(end) = raw_cursor.position_in(layout.bounds()) {
                                        if end != start {
                                            let range_start = state.region.find_cell(start);
                                            let range_end = state.region.find_cell(end);
                                            let range = <R as ReferenceRange<K>>::new(
                                                K::new(range_start.0, range_start.1),
                                                Some(K::new(range_end.0, range_end.1)),
                                            )
                                            .normalize();

                                            if range != *self.data.selection() {
                                                shell.publish(on_edit(Action::Select(range)));
                                                shell.invalidate_layout();
                                            }
                                        }
                                        shell.capture_event();
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    Update::Binding(binding) => {
                        fn apply_binding<Data, T, K, R, Message, Theme>(
                            binding: Binding<Message>,
                            _bounds: Rectangle,
                            data: &Data,
                            state: &mut State,
                            on_edit: &dyn Fn(Action<K, R>) -> Message,
                            on_instruction: Option<&dyn Fn(Instruction<K>) -> Message>,
                            _clipboard: &mut dyn clipboard::Clipboard,
                            shell: &mut Shell<'_, Message>,
                        ) -> bool
                        where
                            Data: Tabular<T, K, R>,
                            T: Default,
                            K: Reference,
                            R: ReferenceRange<K>,
                            Message: Clone,
                        {
                            let mut action = |action| shell.publish(on_edit(action));
                            match binding {
                                Binding::Focus => {
                                    state.interaction = Interaction::None;
                                    state.focus();
                                    shell.invalidate_layout();
                                }
                                Binding::ClickedOutside => {
                                    state.interaction = Interaction::None;
                                    state.unfocus();
                                    shell.invalidate_layout();
                                    return true;
                                }
                                Binding::Unfocus => {
                                    state.interaction = Interaction::None;
                                    state.unfocus();
                                    shell.invalidate_layout();
                                }
                                Binding::Cut => {
                                    if state.is_focused() {
                                        if let Some(on_instruction) = on_instruction.as_ref() {
                                            shell.publish(on_instruction(Instruction::Cut));
                                        }
                                    }
                                }
                                Binding::Copy => {
                                    if state.is_focused() {
                                        if let Some(on_instruction) = on_instruction.as_ref() {
                                            shell.publish(on_instruction(Instruction::Copy));
                                        }
                                    }
                                }
                                Binding::Paste => {
                                    if state.is_focused() {
                                        if let Some(on_instruction) = on_instruction.as_ref() {
                                            shell.publish(on_instruction(Instruction::Paste));
                                        }
                                    }
                                }
                                Binding::Custom(message) => {
                                    shell.publish(message);
                                }
                                Binding::Delete => {
                                    if state.is_focused() {
                                        action(Edit::Delete.into());
                                    }
                                }
                                Binding::StartEdit => {
                                    if let Some(on_instruction) = on_instruction.as_ref() {
                                        shell.publish(on_instruction(Instruction::Activate(
                                            data.selection().start(),
                                        )));
                                    }
                                }
                                Binding::Enter => {
                                    if state.is_focused() {
                                        state.interaction = Interaction::None;
                                        state.focus();
                                        if let Some(on_instruction) = on_instruction.as_ref() {
                                            shell.publish(on_instruction(Instruction::Activate(
                                                data.selection().start(),
                                            )));
                                        }
                                        shell.invalidate_layout();
                                        shell.invalidate_widgets();
                                    }
                                }
                                Binding::MoveSelection(motion) => {
                                    if state.is_focused() {
                                        action(Action::MoveSelection(motion))
                                    }
                                }
                                Binding::ExpandSelection(motion) => {
                                    if state.is_focused() {
                                        action(Action::ExpandSelection(motion))
                                    }
                                }
                                Binding::SelectAll => {
                                    if state.is_focused() {
                                        action(Action::SelectAll)
                                    }
                                }
                            }
                            false
                        }

                        let terminate = apply_binding::<Data, T, K, R, Message, Theme>(
                            binding,
                            layout.bounds(),
                            self.data,
                            state,
                            on_edit,
                            self.on_instruction.as_deref(),
                            clipboard,
                            shell,
                        );

                        if terminate {
                            return;
                        }
                    }
                }
            }
        }

        for ((child, state), child_layout) in self
            .cells
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            child.1.as_widget_mut().update(
                state,
                event,
                child_layout,
                raw_cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<State>();
        operation.focusable(self.id.as_ref(), layout.bounds(), state);
        operation.custom(self.id.as_ref(), layout.bounds(), state);
        operation.container(self.id.as_ref(), layout.bounds(), &mut |operation| {
            self.cells
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), child_layout)| {
                    child
                        .1
                        .as_widget()
                        .operate(state, child_layout, renderer, operation);
                });
        })
    }
}

#[derive(Debug)]
struct State {
    last_click: Option<mouse::Click>,
    drag_click: Option<mouse::click::Kind>,
    interaction: Interaction,
    is_focused: bool,

    region: Region,
}

#[derive(Debug)]
struct Region {
    row_count: usize,
    column_count: usize,
    raw_rows: Vec<f32>,
    raw_columns: Vec<f32>,
    scaled_rows: Vec<f32>,
    scaled_columns: Vec<f32>,
    scale_factor_x: f32,
    scale_factor_y: f32,
    cumulative_x: Vec<f32>,
    cumulative_y: Vec<f32>,
    spacing: Size<f32>,
}

impl Region {
    const RESIZE_AREA: f32 = 4.0;

    fn new(
        columns: &[f32],
        rows: &[f32],
        spacing: Size<f32>,
        row_count: usize,
        column_count: usize,
    ) -> Self {
        let raw_columns = columns.iter().map(|&w| w + spacing.width).collect();
        let raw_rows = rows.iter().map(|&h| h + spacing.height).collect();

        Self {
            row_count,
            column_count,
            raw_columns,
            raw_rows,
            scaled_columns: vec![0.0; column_count],
            scaled_rows: vec![0.0; row_count],
            cumulative_x: vec![0.0; column_count],
            cumulative_y: vec![0.0; row_count],
            scale_factor_x: 1.0,
            scale_factor_y: 1.0,
            spacing,
        }
    }

    fn scale_to_bounds(&mut self, bounds: Size, spacing: Size<f32>) {
        self.spacing = spacing;
        let total_raw_width = self.total_raw_width();
        let total_raw_height = self.total_raw_height();

        // Scale columns including spacing
        for (scaled, raw) in self.scaled_columns.iter_mut().zip(self.raw_columns.iter()) {
            *scaled = (raw / total_raw_width) * bounds.width;
        }

        // Scale rows including spacing
        for (scaled, raw) in self.scaled_rows.iter_mut().zip(self.raw_rows.iter()) {
            *scaled = (raw / total_raw_height) * bounds.height;
        }

        // Precompute cumulative positions
        let mut x = 0.0;
        for (i, width) in self.scaled_columns.iter().enumerate() {
            x += width;
            self.cumulative_x[i] = x;
        }

        let mut y = 0.0;
        for (i, height) in self.scaled_rows.iter().enumerate() {
            y += height;
            self.cumulative_y[i] = y;
        }

        self.scale_factor_x = bounds.width / total_raw_width;
        self.scale_factor_y = bounds.height / total_raw_height;
    }

    fn total_raw_width(&self) -> f32 {
        self.raw_columns.iter().sum()
    }

    fn total_raw_height(&self) -> f32 {
        self.raw_rows.iter().sum()
    }

    // Get actual cell size (without spacing)
    fn cell_size(&self, row: usize, col: usize) -> Size {
        let width = self.scaled_columns.get(col).unwrap_or(&0.0) - self.spacing.width;
        let height = self.scaled_rows.get(row).unwrap_or(&0.0) - self.spacing.height;

        Size::new(width.max(0.0), height.max(0.0))
    }

    // Get cell position using precomputed cumulative positions
    fn cell_position(&self, row: usize, col: usize) -> Point {
        let x = if col == 0 {
            &0.0
        } else {
            self.cumulative_x
                .get(col.checked_sub(1).unwrap_or(0))
                .unwrap_or(&0.0)
        };
        let y = if row == 0 {
            &0.0
        } else {
            self.cumulative_y
                .get(row.checked_sub(1).unwrap_or(0))
                .unwrap_or(&0.0)
        };

        Point::new(x + self.spacing.width / 2.0, y + self.spacing.height / 2.0)
    }

    // Find cell indices for a given point in widget bounds
    fn find_cell(&self, pos: Point) -> (usize, usize) {
        fn find_index(pos: f32, cumulative: &[f32]) -> usize {
            match cumulative
                .binary_search_by(|cum| cum.partial_cmp(&pos).unwrap_or(std::cmp::Ordering::Equal))
            {
                Ok(idx) => idx,
                Err(idx) => idx.min(cumulative.len().checked_sub(1).unwrap_or(0)),
            }
        }

        let col = find_index(pos.x, &self.cumulative_x);
        let row = find_index(pos.y, &self.cumulative_y);

        (col, row)
    }

    // Find the nearest divider to a given point
    fn find_nearest_divider(&self, pos: Point) -> Option<DividerHit> {
        fn find_nearest(
            pos: f32,
            cumulative: &[f32],
            spacing: f32,
            raw_sizes: &[f32],
        ) -> Option<(usize, f32)> {
            // Get the index where this position would be inserted
            let idx = match cumulative
                .binary_search_by(|cum| cum.partial_cmp(&pos).unwrap_or(std::cmp::Ordering::Equal))
            {
                Ok(idx) => idx,
                Err(idx) => idx,
            };

            // Handle all bounds cases together
            let candidates = if idx >= cumulative.len() {
                cumulative.len().checked_sub(1).map(|i| vec![i])
            } else if idx == 0 {
                Some(vec![0])
            } else {
                Some(vec![idx - 1, idx])
            }?;

            candidates.into_iter().find_map(|i| {
                let cum_pos = cumulative.get(i)?;
                let raw_size = raw_sizes.get(i)?;

                let divider_pos = cum_pos - spacing / 2.0;
                ((pos - divider_pos).abs() <= Region::RESIZE_AREA)
                    .then_some((i, raw_size - spacing))
            })
        }

        // Try vertical dividers first, then horizontal
        find_nearest(
            pos.x,
            &self.cumulative_x,
            self.spacing.width,
            &self.raw_columns,
        )
        .map(|(idx, original_size)| DividerHit {
            axis: Axis::Column,
            index: idx,
            original_size,
        })
        .or_else(|| {
            find_nearest(
                pos.y,
                &self.cumulative_y,
                self.spacing.height,
                &self.raw_rows,
            )
            .map(|(idx, original_size)| DividerHit {
                axis: Axis::Row,
                index: idx,
                original_size,
            })
        })
    }

    /// Calculate the bounding rectangle for a selection range
    pub fn selection_bounds<K: Reference, R: ReferenceRange<K>>(&self, selection: R) -> Rectangle {
        let start = selection.start();
        let end = selection.end().unwrap_or(selection.start());

        // Get the minimum and maximum cell coordinates
        let min_row = start.y().min(end.y());
        let max_row = start.y().max(end.y());
        let min_col = start.x().min(end.x());
        let max_col = start.x().max(end.x());

        // Get the top-left position using precomputed cumulative positions
        let x = if min_col == 0 {
            0.0
        } else {
            self.cumulative_x
                .get(min_col.saturating_sub(1))
                .copied()
                .unwrap_or(0.0)
        };

        let y = if min_row == 0 {
            0.0
        } else {
            self.cumulative_y
                .get(min_row.saturating_sub(1))
                .copied()
                .unwrap_or(0.0)
        };

        // Calculate width and height using the difference in cumulative positions
        if let Some(width) = self.cumulative_x.get(max_col) {
            if let Some(height) = self.cumulative_y.get(max_row) {
                Rectangle {
                    x: x + self.spacing.width / 2.0,
                    y: y + self.spacing.height / 2.0,
                    width: *width - x - self.spacing.width / 2.0,
                    height: *height - y - self.spacing.height / 2.0,
                }
            } else {
                Rectangle::default()
            }
        } else {
            Rectangle::default()
        }
    }
}

#[derive(Debug, Clone, Default)]
enum Interaction {
    #[default]
    None,
    ResizeDivider(DividerHit),
}

impl State {
    // Create a new State with the given column and row sizes
    fn new(
        col_sizes: &[f32],
        row_sizes: &[f32],
        spacing: Size,
        row_count: usize,
        column_count: usize,
    ) -> Self {
        Self {
            last_click: None,
            drag_click: None,
            interaction: Interaction::default(),
            is_focused: false,
            region: Region::new(col_sizes, row_sizes, spacing, row_count, column_count),
        }
    }

    fn from_data<Data, T, K, R>(data: &Data, spacing: Size) -> Self
    where
        Data: Tabular<T, K, R>,
        T: Default,
        K: Reference,
        R: ReferenceRange<K>,
    {
        Self::new(
            data.column_sizes(),
            data.row_sizes(),
            spacing,
            data.row_count(),
            data.column_count(),
        )
    }

    /// Returns whether the [`Table`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Focuses the [`Table`].
    pub fn focus(&mut self) {
        self.is_focused = true;
    }

    /// Unfocuses the [`Table`].
    pub fn unfocus(&mut self) {
        self.is_focused = false;
        self.drag_click = None;
        self.last_click = None;
    }
}

impl operation::Focusable for State {
    fn is_focused(&self) -> bool {
        State::is_focused(self)
    }

    fn focus(&mut self) {
        State::focus(self);
    }

    fn unfocus(&mut self) {
        State::unfocus(self);
    }
}

impl std::fmt::Display for Interaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Interaction::None => write!(f, "None"),
            Interaction::ResizeDivider(DividerHit {
                axis,
                index,
                original_size,
            }) => write!(
                f,
                "Resize({:?}, {}, {})",
                axis,
                index,
                original_size.trunc() as i32
            ),
        }
    }
}

pub fn focus<Message>(id: impl Into<widget::Id>) -> iced::Task<Message>
where
    Message: Send + 'static,
{
    widget::operate(widget::operation::focusable::focus::<Message>(id.into()))
}

impl<'a, Data, T, K, R, Message, Theme, Renderer>
    From<Table<'a, Data, T, K, R, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Data: Tabular<T, K, R>,
    T: Cell + Default,
    K: Reference + 'a,
    R: ReferenceRange<K> + 'a,
    Renderer: renderer::Renderer + 'a,
    Message: Clone + 'a,
    Theme: Catalog + 'a,
{
    fn from(data: Table<'a, Data, T, K, R, Message, Theme, Renderer>) -> Self {
        Self::new(data)
    }
}
