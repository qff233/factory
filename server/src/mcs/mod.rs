use std::rc::Rc;

mod queue;
mod track;
mod vehicle;
mod vehicle_dispatch;

#[derive(Debug)]
pub(crate) struct Position(f64, f64, f64);

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

#[derive(Debug, PartialEq, Clone)]
pub enum Side {
    NegY,
    PosY,
    NegZ,
    PosZ,
    NegX,
    PosX,
}

impl From<&str> for Side {
    fn from(value: &str) -> Self {
        match value {
            "negy" => Self::NegY,
            "posy" => Self::PosY,
            "negz" => Self::NegZ,
            "posz" => Self::PosZ,
            "negx" => Self::NegX,
            "posx" => Self::PosX,
            _ => panic!("no such side, {}", value),
        }
    }
}

struct MCS {
    sql: Rc<sqlx::PgPool>,
}
