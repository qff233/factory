mod action_planner;
mod adder;
mod exec;
mod state_update;

#[derive(Debug)]
pub enum Error {
    VehicleBusy,
    PathFind,
    Db(sqlx::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
