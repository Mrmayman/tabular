use iced::{Background, Border, Color, Theme};

use super::Status;

/// The appearance of a [`Table`].
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The default [`Background`] of the grid.
    pub background: Background,
    /// The [`Border`] of the grid.
    pub border: Border,
    /// The gridlines border
    pub gridlines: Border,
    /// The color of the overlay when hovering a cell.
    pub hovered: Color,
    /// The default [`Color`] of the value of the grid's cells.
    pub value: Color,
    /// The style of some selection of the grid.
    pub selection: SelectionStyle,
}

#[derive(Debug, Clone, Copy)]
/// The appearance of a selection in a [`Table`].
pub struct SelectionStyle {
    /// The fill of the selection
    pub fill: Color,
    /// The stroke of the selection
    pub stroke: Color,
    /// The width of the stroke
    pub stroke_width: f32,
}

/// A styling function for a [`Table`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> <Theme as Catalog>::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &<Theme as Catalog>::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`Table`]
pub fn default(theme: &iced::Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let base = Style {
        background: Color::TRANSPARENT.into(),
        border: Border {
            radius: 0.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        gridlines: Border {
            radius: 0.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        hovered: palette.primary.weak.color.scale_alpha(0.2),
        value: palette.primary.base.text,
        selection: SelectionStyle {
            fill: palette.primary.weak.color.scale_alpha(0.20),
            stroke: palette.primary.weak.color.scale_alpha(0.5),
            stroke_width: 2.0,
        },
    };

    match status {
        Status::Focused => Style {
            selection: SelectionStyle {
                fill: palette.primary.base.color.scale_alpha(0.20),
                stroke: palette.primary.base.color.scale_alpha(1.0),
                stroke_width: 2.0,
            },
            ..base
        },
        Status::Unfocused => Style {
            selection: SelectionStyle {
                fill: base.selection.fill.scale_alpha(0.5),
                stroke: base.selection.stroke.scale_alpha(0.5),
                ..base.selection
            },
            ..base
        },
        Status::Disabled => Style { ..base },
    }
}

/// The theme catalog of a [`Table`].
pub trait Catalog: iced::widget::container::Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> <Self as Catalog>::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &<Self as Catalog>::Class<'_>, status: Status) -> Style;
}
