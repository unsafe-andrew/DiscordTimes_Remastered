use derive_more::{Add, AddAssign, Mul, Neg, Sub, SubAssign, Sum};
use std::ops::Div;

#[derive(Add, AddAssign, Sub, SubAssign, Neg, Mul, PartialEq, Copy, Clone, Debug, Sum)]
pub struct Position(pub f32, pub f32);
#[derive(Add, AddAssign, Sub, SubAssign, Neg, Mul, PartialEq, Copy, Clone, Debug, Sum)]
pub struct Size(pub f32, pub f32);
#[derive(Add, AddAssign, Sub, SubAssign, Neg, Mul, PartialEq, Copy, Clone, Debug, Sum)]
pub struct Rect {
    pub pos: Position,
    pub size: Size,
}
impl Rect {
    pub fn collides(&self, pos: Position) -> bool {
        pos.0 >= self.pos.0
            && pos.0 <= self.pos.0 + self.size.0
            && pos.1 >= self.pos.1
            && pos.1 <= self.pos.1 + self.size.1
    }
}
impl Default for Rect {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            size: Default::default(),
        }
    }
}
impl Into<(f32, f32)> for Position {
    fn into(self) -> (f32, f32) {
        (self.0, self.1)
    }
}
impl From<(f32, f32)> for Position {
    fn from(values: (f32, f32)) -> Self {
        Self(values.0, values.1)
    }
}
impl Into<Size> for Position {
    fn into(self) -> Size {
        Size(self.0, self.1)
    }
}
impl Default for Position {
    fn default() -> Self {
        Self(0., 0.)
    }
}
impl Into<(f32, f32)> for Size {
    fn into(self) -> (f32, f32) {
        (self.0, self.1)
    }
}
impl From<(f32, f32)> for Size {
    fn from(values: (f32, f32)) -> Self {
        Self(values.0, values.1)
    }
}
impl Into<Position> for Size {
    fn into(self) -> Position {
        Position(self.0, self.1)
    }
}
impl Default for Size {
    fn default() -> Self {
        Self(0., 0.)
    }
}

impl Div<f32> for Position {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        (self.0 / rhs, self.1 / rhs).into()
    }
}
impl Div<f32> for Size {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        (self.0 / rhs, self.1 / rhs).into()
    }
}
