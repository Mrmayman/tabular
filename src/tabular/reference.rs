//! Reference cells and ranges in a table.
use std::fmt;
use std::hash::Hash;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::to_column_name;

/// A trait for a reference to a cell in a table.
pub trait Reference:
    Sized
    + Clone
    + Copy
    + PartialEq
    + Eq
    + Hash
    + PartialOrd
    + Ord
    + Default
    + fmt::Debug
    + fmt::Display
    + std::convert::From<(usize, usize)>
{
    fn new(x: usize, y: usize) -> Self;
    fn x(&self) -> usize;
    fn y(&self) -> usize;
    fn as_tuple(&self) -> (usize, usize) {
        (self.x(), self.y())
    }
    fn as_range<R>(self) -> R
    where
        R: ReferenceRange<Self>,
    {
        R::new(self, None)
    }
}

/// A trait for a range of references to cells in a table.
pub trait ReferenceRange<K: Reference>:
    Sized + Clone + Copy + Default + PartialEq + fmt::Debug + fmt::Display
{
    type Iterator: Iterator<Item = K>;

    fn new(start: K, end: Option<K>) -> Self;
    fn normalize(&self) -> Self;
    fn iter(&self) -> Self::Iterator;
    fn start(&self) -> K;
    fn end(&self) -> Option<K>;
    fn contains(&self, other: &K) -> bool {
        let start = self.start();
        let end = self.end().unwrap_or(start);
        (start.x() <= other.x() && other.x() <= end.x())
            && (start.y() <= other.y() && other.y() <= end.y())
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A standard implementation of `Reference` for a cell in a table.
pub struct Address {
    x: usize,
    y: usize,
}

use iced::advanced::widget;
impl From<Address> for widget::Id {
    fn from(address: Address) -> Self {
        widget::Id::new(address.to_string())
    }
}

use iced::widget::container;
impl From<Address> for container::Id {
    fn from(address: Address) -> Self {
        container::Id::new(address.to_string())
    }
}

use iced::widget::text_input;
impl From<Address> for text_input::Id {
    fn from(address: Address) -> Self {
        text_input::Id::new(address.to_string())
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
/// A standard implementation of `ReferenceRange` for a range of cells in a table.
pub struct Range {
    start: Address,
    end: Option<Address>,
}

impl Reference for Address {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    fn x(&self) -> usize {
        self.x
    }

    fn y(&self) -> usize {
        self.y
    }
}

impl From<(usize, usize)> for Address {
    fn from((x, y): (usize, usize)) -> Self {
        Self { x, y }
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", to_column_name(self.x), self.y + 1)
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.start, self.end.unwrap_or(self.start))
    }
}

impl ReferenceRange<Address> for Range {
    type Iterator = std::vec::IntoIter<Address>;

    fn new(start: Address, end: Option<Address>) -> Self {
        Self { start, end }
    }

    fn normalize(&self) -> Self {
        let mut start = self.start;
        let mut end = self.end.unwrap_or(start);
        if end.x < start.x {
            std::mem::swap(&mut start.x, &mut end.x);
        }
        if end.y < start.y {
            std::mem::swap(&mut start.y, &mut end.y);
        }
        Self {
            start,
            end: Some(end),
        }
    }

    fn iter(&self) -> Self::Iterator {
        let start = self.start;
        let end = self.end.unwrap_or(start);
        let x_range = if start.x <= end.x {
            start.x..=end.x
        } else {
            end.x..=start.x
        };
        let y_range = if start.y <= end.y {
            start.y..=end.y
        } else {
            end.y..=start.y
        };

        x_range
            .flat_map(move |x| y_range.clone().map(move |y| Address { x, y }))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn start(&self) -> Address {
        self.start
    }

    fn end(&self) -> Option<Address> {
        self.end
    }
}
