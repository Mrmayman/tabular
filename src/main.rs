use iced::Length::Fill;
use iced::widget::{column, text};
use iced::{Element, Size, Task};

mod table;
use table::table;

fn main() -> iced::Result {
    iced::application(
        "iced • how fast can you make a table widget",
        App::update,
        App::view,
    )
    .window_size(Size::new(400.0, 400.0))
    .centered()
    .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {}

#[derive(Default)]
struct App {
    cells: Cells,
}

#[derive(Debug, Clone)]
pub struct Cells {
    values: Vec<Vec<String>>,
    row_sizes: Vec<f32>,
    col_sizes: Vec<f32>,
}

impl Cells {
    fn with_size(rows: usize, cols: usize) -> Self {
        Self {
            values: vec![vec!["Hello, world".to_string(); cols]; rows],
            row_sizes: vec![20.0; rows],
            col_sizes: vec![60.0; cols],
        }
    }
}

impl Default for Cells {
    fn default() -> Self {
        Self::with_size(10, 10)
    }
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        // match message {}
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        column![
            text("Custom table widget"),
            table(
                &self.cells.values,
                |cell| text(cell)
                    .size(12)
                    .width(Fill)
                    .height(Fill)
                    .wrapping(text::Wrapping::None)
                    .center()
                    .into(),
                self.cells.col_sizes.as_slice(),
                self.cells.row_sizes.as_slice()
            ),
        ]
        .padding(20)
        .spacing(20)
        .into()
    }
}
