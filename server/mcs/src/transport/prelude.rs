#[derive(Debug)]
pub(crate) struct Position(pub f64, pub f64, pub f64);

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < 0.1
            && (self.1 - other.1).abs() < 0.1
            && (self.2 - other.2).abs() < 0.1
    }
}

impl From<&Position> for (f64, f64, f64) {
    fn from(value: &Position) -> Self {
        (value.0, value.1, value.2)
    }
}

impl From<(f64, f64, f64)> for Position {
    fn from(value: (f64, f64, f64)) -> Self {
        Self(value.0, value.1, value.2)
    }
}
