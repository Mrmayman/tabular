use iced::keyboard::key;
use iced::widget::{center, checkbox, column, container, row, slider, text, text_input};
use iced::{Center, Element, Function, Task, keyboard};

use tabular::list::{Cell, Content};
use tabular::reference::*;
use tabular::{Address, Tabular, tabular};

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("iced • how quickly can I make a grid/table widget")
        .window(iced::window::Settings {
            size: (800.0, 600.0).into(),
            ..Default::default()
        })
        .theme(|_| iced::Theme::Light)
        .centered()
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Action(tabular::Action),
    Instruction(tabular::Instruction),
    Spacing(f32),
    ShowGridlines(bool),
    Edit(Address, String),
    FocusSelection,
    FocusTable,
}

struct App {
    cells: Content,
    clipboard: String,
    spacing: f32,
    show_gridlines: bool,
}

use std::sync::LazyLock;
static TABLE: LazyLock<widget::Id> = LazyLock::new(|| widget::Id::new("board"));

impl App {
    fn new() -> (Self, Task<Message>) {
        let mut cells = Content::default()
            .with_row_heights(vec![5.0, 20.0, 20.0, 20.0])
            .with_column_widths(vec![50.0, 50.0, 50.0, 50.0]);

        [
            ["Item", "Price", "Qty", "Total"],
            ["Apple", "$1.00", "2", "$2.00"],
            ["Banana", "$0.50", "3", "$1.50"],
            ["Cherry", "$2.00", "1", "$2.00"],
        ]
        .iter()
        .enumerate()
        .for_each(|(y, row)| {
            row.iter()
                .enumerate()
                .for_each(|(x, &content)| cells.insert((x, y), content.to_string()));
        });

        (
            Self {
                cells,
                clipboard: String::new(),
                spacing: 0.0,
                show_gridlines: true,
            },
            tabular::focus(TABLE.clone()),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Spacing(value) => {
                self.spacing = value;
            }
            Message::ShowGridlines(value) => {
                self.show_gridlines = value;
            }
            Message::Action(action) => self.cells.perform(action),
            Message::Instruction(instruction) => match instruction {
                tabular::Instruction::Cut => self.cut(),
                tabular::Instruction::Copy => self.copy(),
                tabular::Instruction::Paste => self.paste(),
                tabular::Instruction::Activate(address) => {
                    if address.y() != 0 {
                        return text_input::focus(address);
                    }
                }
            },
            Message::Edit(address, content) => {
                if let Some(cell) = self.cells.get_mut(address) {
                    cell.content = content;
                }
            }
            Message::FocusTable => return tabular::focus(TABLE.clone()),
            Message::FocusSelection => {
                let address = self.cells.selection().start();
                return text_input::focus(address);
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        column![
            text("How quickly can I make a grid/table widget?"),
            row![
                "Spacing:",
                slider(0.0..=50.0, self.spacing, Message::Spacing),
                checkbox("Show gridlines", self.show_gridlines).on_toggle(Message::ShowGridlines)
            ]
            .spacing(5)
            .align_y(Center),
            tabular(&self.cells, |address, cell| view_cell(address, cell))
                .id(TABLE.clone())
                .on_action(Message::Action)
                .on_instruction(Message::Instruction)
                .key_binding(|key_press| match key_press.key.as_ref() {
                    keyboard::Key::Named(key::Named::Enter)
                        if key_press.status == tabular::Status::Focused =>
                        Some(tabular::Binding::Custom(Message::FocusSelection)),
                    keyboard::Key::Named(key::Named::Escape)
                        if key_press.status == tabular::Status::Unfocused =>
                        Some(tabular::Binding::Custom(Message::FocusTable)),
                    _ => tabular::Binding::from_key_press(key_press),
                })
                .show_gridlines(self.show_gridlines)
                .with_spacing((self.spacing, self.spacing)),
        ]
        .padding(20)
        .spacing(20)
        .into()
    }

    fn cut(&mut self) {
        self.copy();

        for cell in self.cells.selection().normalize().iter() {
            self.cells.insert(cell, String::new());
        }
    }

    fn copy(&mut self) {
        let range = self.cells.selection().normalize();
        let mut all_content = Vec::new();

        for y in range.start().y()..=range.end().unwrap_or(range.start()).y() {
            let mut row_content = Vec::new();

            for x in range.start().x()..=range.end().unwrap_or(range.start()).x() {
                let content = self
                    .cells
                    .get((y, x))
                    .cloned()
                    .unwrap_or(Default::default());
                row_content.push(content.content);
            }

            // row cells (column-wise) are joined by tabs
            all_content.push(row_content.join("\t"));
        }

        // rows are joined by newlines
        self.clipboard = all_content.join("\n");
    }

    fn paste(&mut self) {
        let start = self.cells.selection().start();
        let rows: Vec<&str> = self.clipboard.split('\n').collect();

        let max_rows = self.cells.row_count();
        let max_cols = self.cells.column_count();

        for (row_offset, row_content) in rows.iter().enumerate() {
            let target_row = start.y() + row_offset; // bounds check
            if target_row >= max_rows {
                break;
            }

            let columns: Vec<&str> = row_content.split('\t').collect();
            for (col_offset, content) in columns.iter().enumerate() {
                let target_col = start.x() + col_offset; // bounds check
                if target_col >= max_cols {
                    break;
                }

                self.cells
                    .insert((target_row, target_col), content.to_string());
            }
        }
    }
}

fn view_cell(address: Address, cell: &Cell) -> Element<'_, Message> {
    if address.y() == 0 {
        return center(text(&cell.content).size(13)).style(header).into();
    }

    center(
        text_input("", &cell.content)
            .on_input(Message::Edit.with(address))
            .on_submit(Message::FocusTable)
            .size(12)
            .style(transparent)
            .id(text_input::Id::new(address.to_string())),
    )
    .style(body)
    .into()
}

fn header(theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(theme.extended_palette().background.weak.color.into()),
        text_color: theme.extended_palette().background.weak.text.into(),
        ..Default::default()
    }
}
fn body(theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(theme.extended_palette().background.base.color.into()),
        text_color: theme.extended_palette().background.base.text.into(),
        ..Default::default()
    }
}

fn transparent(theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::default(theme, status);

    text_input::Style {
        border: Default::default(),
        background: iced::Color::TRANSPARENT.into(),
        value: iced::Color::BLACK,
        ..base
    }
}

use iced::advanced::widget;
pub fn unfocus<Message>() -> iced::Task<Message>
where
    Message: Send + 'static,
{
    widget::operate(widget::operation::focusable::unfocus::<Message>())
}
