use std::rc::Rc;

mod queue;
mod track;
mod vehicle;

#[derive(Debug, PartialEq)]
pub(crate) struct Position(f64, f64, f64);

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

#[derive(Debug, PartialEq)]
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
