use super::update::Motion;
use super::{Axis, Reference, ReferenceRange};

use crate::{Address, Range};

/// An interaction with a [`Table`] editor. These are handled by calling
/// .perform() on the [`Table`] widget.
#[derive(Debug, Clone)]
pub enum Action<K: Reference = Address, R: ReferenceRange<K> = Range> {
    /// Apply a [`Motion`].
    MoveSelection(Motion),
    /// Apply a [`Motion`] while selecting cells.
    ExpandSelection(Motion),
    /// Select a range of cells.
    Select(R),
    /// Select the entire table
    SelectAll,
    /// Edit the table
    Edit(Edit),
    /// Resize a divider with index `usize` to the given `f32` value.
    ResizeDivider(Axis, usize, f32),

    _Phantom(K), // marker for K
}

impl<K: Reference, R: ReferenceRange<K>> Action<K, R> {
    pub fn is_edit(&self) -> bool {
        matches!(self, Self::Edit(_))
    }
}

/// An edit action that can be performed on a [`Table`].
#[derive(Debug, Clone)]
pub enum Edit {
    /// Delete the selected cells.
    Delete,
}

impl<K: Reference, R: ReferenceRange<K>> From<Edit> for Action<K, R> {
    fn from(edit: Edit) -> Self {
        Self::Edit(edit)
    }
}

/// An instruction to the app resulting from a [`Table`] action being performed.
/// These will require more context from the app to be handled, such as access
/// to the app's clipboard, and therefore cannot directly be .perform()ed on the
/// [`Table`] widget.
#[derive(Debug, Clone)]
pub enum Instruction<K: Reference = Address> {
    /// The app should paste the clipboard contents.
    Paste,
    /// The app should cut the selection.
    Cut,
    /// The app should copy the selection.
    Copy,
    /// The app should activate the given cell, such as focusing it.
    Activate(K),
}
