use std::rc::Rc;

mod queue;
mod track;
mod vehicle;

#[derive(Debug, PartialEq)]
pub(crate) struct Position(f64, f64, f64);

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
