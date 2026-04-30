use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteOffset(usize);

impl ByteOffset {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for ByteOffset {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl fmt::Display for ByteOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Add<usize> for ByteOffset {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for ByteOffset {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Sub<usize> for ByteOffset {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(
            self.0
                .checked_sub(rhs)
                .expect("ByteOffset subtraction underflow"),
        )
    }
}

impl SubAssign<usize> for ByteOffset {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 = self
            .0
            .checked_sub(rhs)
            .expect("ByteOffset subtraction underflow");
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CharOffset(usize);

impl CharOffset {
    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for CharOffset {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl fmt::Display for CharOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Add<usize> for CharOffset {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for CharOffset {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Sub<usize> for CharOffset {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(
            self.0
                .checked_sub(rhs)
                .expect("CharOffset subtraction underflow"),
        )
    }
}

impl SubAssign<usize> for CharOffset {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 = self
            .0
            .checked_sub(rhs)
            .expect("CharOffset subtraction underflow");
    }
}

pub struct CharCounter<'a> {
    text: &'a str,
}

impl<'a> CharCounter<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub fn char_offset(&mut self, byte_offset: ByteOffset) -> Option<CharOffset> {
        let byte_offset = byte_offset.as_usize();
        if byte_offset > self.text.len() || !self.text.is_char_boundary(byte_offset) {
            return None;
        }

        Some(CharOffset(self.text[..byte_offset].chars().count()))
    }
}
