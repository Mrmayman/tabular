use iced::advanced::graphics::core::SmolStr;
use iced::advanced::mouse;
use iced::keyboard::{self, key};
use iced::{Point, Rectangle};

use super::reference::Reference;
use super::{Interaction, State, Status};

#[derive(Clone)]
pub enum Update<Message: Clone> {
    /// RedrawRequested
    RedrawRequested,
    /// Click
    Click(mouse::Click),
    /// Drag
    Drag(Point),
    /// Release the mouse
    Release,
    /// Call some binding
    Binding(Binding<Message>),
}

#[derive(Clone, PartialEq)]
pub enum Binding<Message: Clone> {
    /// Copy the selection of the [`Spreadsheet`].
    Copy,
    /// Cut the selection of the [`Spreadsheet`].
    Cut,
    /// Paste the clipboard contents in the [`Spreadsheet`].
    Paste,
    /// Move the cursor by the given [`Motion`].
    MoveSelection(Motion),
    /// Expand the selection by the given [`Motion`].
    ExpandSelection(Motion),
    /// Select the entire buffer.
    SelectAll,
    /// Break the current line.
    Enter,
    /// Delete the selection.
    Delete,
    /// Focus the widget
    Focus,
    /// Unfocus the widget
    Unfocus,
    /// Clicked outside the widget
    ClickedOutside,
    /// Produce the given message.
    Custom(Message),
    /// Start editing the active cell
    StartEdit,
}

/// A key press.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPress {
    /// The key pressed.
    pub key: keyboard::Key,
    /// The state of the keyboard modifiers.
    pub modifiers: keyboard::Modifiers,
    /// The text produced by the key press.
    pub text: Option<SmolStr>,
    /// The current [`Status`] of the [`Board`].
    pub status: Status,
}

impl<Message: Clone> Update<Message> {
    pub(super) fn from_event<K: Reference>(
        event: &iced::Event,
        state: &State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        key_binding: Option<&dyn Fn(KeyPress) -> Option<Binding<Message>>>,
    ) -> Option<Self>
    where
        Message: Clone,
    {
        let binding = |binding| Some(Update::Binding(binding));

        match &event {
            iced::Event::InputMethod(_) | iced::Event::Window(_) => None,
            iced::Event::Mouse(event) => match event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(position) = cursor.position_in(bounds) {
                        let click =
                            mouse::Click::new(position, mouse::Button::Left, state.last_click);

                        Some(Update::Click(click))
                    } else if state.is_focused() {
                        binding(Binding::ClickedOutside)
                    } else {
                        None
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => Some(Update::Release),
                mouse::Event::CursorMoved { .. } => match state.drag_click {
                    Some(mouse::click::Kind::Single) => {
                        match state.interaction {
                            // If we're resizing, return the update regardless of bounds
                            Interaction::ResizeDivider(_) => Some(Update::Drag(cursor.position()?)),
                            Interaction::None => Some(Update::Drag(cursor.position()?)),
                        }
                    }
                    _ => None,
                },
                _ => None,
            },
            iced::Event::Keyboard(event) => match event {
                keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    text,
                    ..
                } => {
                    let status = if state.is_focused() {
                        Status::Focused
                    } else {
                        Status::Unfocused
                    };

                    let key_press = KeyPress {
                        key: key.clone(),
                        modifiers: *modifiers,
                        text: text.clone(),
                        status,
                    };

                    if let Some(key_binding) = key_binding {
                        key_binding(key_press)
                    } else {
                        Binding::from_key_press(key_press)
                    }
                    .map(Self::Binding)
                }
                _ => None,
            },
            iced::Event::Touch(event) => match event {
                _ => None,
            },
        }
    }
}

impl<Message: Clone> Binding<Message> {
    /// Returns the default [`Binding`] for the given key press.
    pub fn from_key_press(event: KeyPress) -> Option<Self> {
        let KeyPress {
            key,
            modifiers,
            status,
            ..
        } = event;

        if status != Status::Focused {
            return None;
        }

        match key.as_ref() {
            keyboard::Key::Named(key::Named::F2) => match status {
                Status::Focused => Some(Self::StartEdit),
                Status::Unfocused => Some(Self::Focus),
                Status::Disabled => None,
            },
            keyboard::Key::Named(key::Named::Enter) => Some(Self::Enter),
            keyboard::Key::Named(key::Named::Delete)
            | keyboard::Key::Named(key::Named::Backspace) => Some(Self::Delete),
            keyboard::Key::Named(key::Named::Escape) => Some(Self::Focus),
            keyboard::Key::Character("c") if modifiers.command() => Some(Self::Copy),
            keyboard::Key::Character("x") if modifiers.command() => Some(Self::Cut),
            keyboard::Key::Character("v") if modifiers.command() && !modifiers.alt() => {
                Some(Self::Paste)
            }
            keyboard::Key::Character("a") if modifiers.command() => Some(Self::SelectAll),
            _ => {
                if let keyboard::Key::Named(named_key) = key.as_ref() {
                    let motion = motion(named_key)?;

                    let motion = if modifiers.macos_command() {
                        match motion {
                            Motion::Left => Motion::Home,
                            Motion::Right => Motion::End,
                            _ => motion,
                        }
                    } else {
                        motion
                    };

                    let motion = if modifiers.jump() {
                        motion.widen()
                    } else {
                        motion
                    };

                    Some(if modifiers.shift() {
                        Self::ExpandSelection(motion)
                    } else {
                        Self::MoveSelection(motion)
                    })
                } else {
                    None
                }
            }
        }
    }
}

fn motion(key: key::Named) -> Option<Motion> {
    match key {
        key::Named::ArrowLeft => Some(Motion::Left),
        key::Named::ArrowRight => Some(Motion::Right),
        key::Named::ArrowUp => Some(Motion::Up),
        key::Named::ArrowDown => Some(Motion::Down),
        key::Named::Home => Some(Motion::Home),
        key::Named::End => Some(Motion::End),
        _ => None,
    }
}

/// A cursor movement on the table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Motion {
    /// Move left.
    Left,
    /// Move right.
    Right,
    /// Move up.
    Up,
    /// Move down.
    Down,
    /// Move to the start of the row.
    Home,
    /// Move to the end of the row.
    End,
    /// Move to the next cell on the table.
    Forward,
    /// Move to the previous cell on the table.
    Back,
    /// Move to the start of the document.
    DocumentStart,
    /// Move to the end of the document.
    DocumentEnd,
}

impl Motion {
    /// Widens the [`Motion`], if possible.
    pub fn widen(self) -> Self {
        match self {
            Self::Home => Self::DocumentStart,
            Self::End => Self::DocumentEnd,
            _ => self,
        }
    }

    /// Returns the [`Direction`] of the [`Motion`].
    pub fn direction(&self) -> Direction {
        match self {
            Self::Left | Self::Up | Self::Home | Self::DocumentStart => Direction::Left,
            Self::Right | Self::Down | Self::End | Self::DocumentEnd => Direction::Right,
            Self::Forward | Self::Back => Direction::Arbitrary,
        }
    }
}

/// A direction in some text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// <-
    Left,
    /// ->
    Right,
    /// Depends on context
    Arbitrary,
}

impl<Message: Clone> std::fmt::Debug for Update<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RedrawRequested => write!(f, "RedrawRequested"),
            Self::Click(click) => write!(f, "Click({:?})", click),
            Self::Drag(position) => write!(f, "Drag({:?})", position),
            Self::Release => write!(f, "Release"),
            Self::Binding(binding) => write!(f, "Binding({:?})", binding),
        }
    }
}

impl<Message: Clone> std::fmt::Debug for Binding<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Copy => write!(f, "Copy"),
            Self::Cut => write!(f, "Cut"),
            Self::Paste => write!(f, "Paste"),
            Self::MoveSelection(motion) => write!(f, "MoveSelection({:?})", motion),
            Self::ExpandSelection(motion) => write!(f, "ExpandSelection({:?})", motion),
            Self::SelectAll => write!(f, "SelectAll"),
            Self::Enter => write!(f, "Enter"),
            Self::Delete => write!(f, "Delete"),
            Self::Focus => write!(f, "Focus"),
            Self::Unfocus => write!(f, "Unfocus"),
            Self::ClickedOutside => write!(f, "ClickedOutside"),
            Self::Custom(_) => write!(f, "Custom"),
            Self::StartEdit => write!(f, "StartEdit"),
        }
    }
}
