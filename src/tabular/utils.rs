/// An axis of a [`Table`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Column,
    Row,
}

/// The possible statuses of a [`Table`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The table is focused.
    Focused,
    /// The table is unfocused.
    Unfocused,
    /// The table cannot be interacted with.
    Disabled,
}

#[derive(Debug, Clone, Copy)]
pub struct DividerHit {
    pub axis: Axis,
    pub index: usize,
    pub original_size: f32,
}

impl PartialEq for DividerHit {
    // ignore original_size
    fn eq(&self, other: &Self) -> bool {
        self.axis == other.axis && self.index == other.index
    }
}

pub fn to_column_name(mut n: usize) -> String {
    let mut name = String::new();

    loop {
        let rem = n % 26;
        name.push((b'A' + rem as u8) as char);
        n = n / 26;
        if n == 0 {
            break;
        }
        n -= 1;
    }

    name.chars().rev().collect()
}

pub fn from_column_name(s: &str) -> Result<usize, String> {
    let mut column = 0u32;

    for c in s.chars() {
        let c = if c.is_ascii_lowercase() {
            ((c as u8) - (b'a' - b'A')) as char
        } else {
            c
        };
        column = column
            .checked_mul(26)
            .ok_or_else(|| format!("Column index overflow while parsing: {}", s))?;
        column = column
            .checked_add((c as u32) - (b'A' as u32) + 1)
            .ok_or_else(|| format!("Column index overflow while parsing: {}", s))?;
    }

    if column == 0 {
        return Err(format!("Invalid column name: {}", s));
    }

    Ok(usize::try_from(column - 1).map_err(|_| format!("Column index too large: {}", column))?)
}
