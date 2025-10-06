use std::fmt;
use std::sync::Arc;

use tokio::sync::{RwLock, mpsc};
use tracing::error;

use super::track;
use crate::transport::prelude::*;
use crate::transport::track::Graph;
pub use crate::transport::vehicle::action::{Action, ActionSequence, ActionSequenceBuilder};
pub use crate::transport::vehicle::skill::Skill;
pub use crate::transport::vehicle::skill::ToolType;
use crate::transport::vehicle::timeout::Timeout;

mod action;
mod skill;
mod timeout;

#[derive(Debug)]
pub enum Error {
    State,
    TrackGraph,
    NotInTrackGraph,
    Db(sqlx::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub enum Event {
    ProcessStart {
        vehicle_id: i32,
        vehicle_skill: Skill,
        task_id: i32,
    },
    ProcessDone {
        vehicle_id: i32,
        vehicle_skill: Skill,
        task_id: i32,
    },
    ChargeStart,
    ChargeDone,
}

#[derive(Debug)]
enum State {
    Initing(ActionSequence),
    InitDone,
    ChargeDone,
    Charging(ActionSequence),
    Offline,
    ParkDone,
    Parking(ActionSequence),
    ProcessDone,
    Processing(ActionSequence),
}

pub struct Vehicle {
    id: i32,
    state: Arc<RwLock<State>>,
    skill: Skill,
    overtime: Timeout,
    track_graph: Arc<Graph>,
    node: Option<Arc<track::Node>>,
    current_task_id: Option<i32>,
    sender: Option<mpsc::Sender<Event>>,
}

impl Vehicle {
    pub async fn new(id: i32, track_graph: Arc<Graph>) -> Self {
        let skill = Skill::from_id(&id);
        let state = Arc::new(RwLock::new(State::Offline));
        Self {
            id,
            overtime: Timeout::new(state.clone()),
            state,
            skill,
            track_graph,
            node: None,
            current_task_id: None,
            sender: None,
        }
    }

    pub fn set_event_sender(&mut self, sender: mpsc::Sender<Event>) {
        self.sender = Some(sender)
    }

    pub async fn offline(&mut self) {
        *self.state.write().await = State::Offline;
    }

    async fn initing(&self, current_position: &Position, state: &mut State) -> Result<()> {
        if let State::Offline = state {
        } else {
            error!(
                "vehicle({}): state error before initing. current status is {:?}, expect offline.",
                self.id,
                self.state.read().await
            );
            return Err(Error::State);
        }

        let shortest_node = self
            .track_graph
            .find_shortest_node(current_position).await
            .map_err(|e| {
                error!(
                    "vehicle({}): find shortest node path error in initing. error type: {:?}.current position is {:?}.",
                    self.id, e,current_position
                );
                Error::Db(e)
            })?;
        let shortest_node_to_shipping_dock_path = self
            .track_graph
            .find_shipping_dock_path(&shortest_node.name)
            .await
            .map_err(|e| {
                error!(
                    "vehicle({}): find shortest node to shipping dock path error in initing. error type: {:?}.current position is {:?}.",
                    self.id, e,current_position
                );
                Error::Db(e)
            })?;
        let mut actions = ActionSequenceBuilder::new().move_to(shortest_node.clone());
        match self.skill {
            Skill::Item => {
                actions = actions
                    .move_path(&shortest_node_to_shipping_dock_path)
                    .drop();
            }
            Skill::Fluid => {
                actions = actions
                    .move_path(&shortest_node_to_shipping_dock_path)
                    .fill();
            }
            _ => (),
        }
        *state = State::Initing(actions.build());
        Ok(())
    }

    pub fn node(&self) -> Result<Arc<track::Node>> {
        self.node.clone().ok_or(Error::NotInTrackGraph)
    }

    pub fn skill(&self) -> &Skill {
        &self.skill
    }

    pub async fn idle(&self) -> bool {
        match *self.state.read().await {
            State::InitDone
            | State::ChargeDone
            | State::ProcessDone
            | State::ParkDone
            | State::Parking(_) => true,
            State::Initing(_) | State::Charging(_) | State::Processing(_) | State::Offline => false,
        }
    }

    fn next_action(
        current_position: &Position,
        landmark: &mut Option<Arc<track::Node>>,
        actions: &mut ActionSequence,
    ) -> Option<Action> {
        match actions.next_action()? {
            Action::Move(node) => {
                if current_position == &node.position {
                    *landmark = Some(node.clone());
                    actions.pop_next_action();
                    actions.next_action().cloned()
                } else {
                    Some(actions.next_action()?.clone())
                }
            }
            _ => {
                actions.pop_next_action();
                actions.next_action().cloned()
            }
        }
    }

    async fn parking(&self, state: &mut State) -> Result<()> {
        if let State::ChargeDone = *state {
            self.track_graph
                .unlock_node(self.node()?.id)
                .await
                .map_err(Error::Db)?;
        }
        match *state {
            State::ChargeDone | State::ProcessDone | State::InitDone => {
                let path = self
                    .track_graph
                    .find_parking_path(
                        &self
                            .node()
                            .map_err(|e| {
                                error!(
                                    "vehicle({}): find current node in parking. error type: {:?}.",
                                    self.id, e
                                );
                                Error::NotInTrackGraph
                            })?
                            .name,
                    )
                    .await
                    .map_err(Error::Db)?;
                self.track_graph
                    .lock_node(path.last().ok_or_else(|| {
                        error!("vehicle({}): get parking node from path error. current state is {:?}.",self.id, self.state);
                        Error::TrackGraph
                    })?.id)
                    .await.map_err(Error::Db)?;
                let actions = ActionSequenceBuilder::new().move_path(&path).build();
                *state = State::Parking(actions);
                Ok(())
            }
            _ => {
                error!(
                    "vehicle({}): state error before parking. current status is {:?}, expect ChargeDone|ProcessDone|InitDone.",
                    self.id,
                    self.state.read().await
                );
                Err(Error::State)
            }
        }
    }

    async fn charging(&self, state: &mut State) -> Result<()> {
        if let State::Parking(actions) = state
            && let Some(node) = actions.last_move_node()
        {
            self.track_graph
                .unlock_node(node.id)
                .await
                .map_err(Error::Db)?;
        }
        match *state {
            State::ParkDone | State::Parking(_) | State::ProcessDone | State::InitDone => {
                let path = self
                    .track_graph
                    .find_charging_path(
                        &self
                            .node()
                            .map_err(|e| {
                                error!(
                                    "vehicle({}): find current node in parking. error type: {:?}.",
                                    self.id, e
                                );
                                Error::NotInTrackGraph
                            })?
                            .name,
                    )
                    .await
                    .map_err(Error::Db)?;
                self.track_graph
                    .lock_node(path.last().ok_or_else(|| {
                        error!("vehicle({}): get parking node from path error. current state is {:?}.",self.id, self.state);
                        Error::TrackGraph
                    })?.id)
                    .await
                    .map_err(Error::Db)?;
                let actions = ActionSequenceBuilder::new().move_path(&path).build();
                *state = State::Charging(actions);
                Ok(())
            }
            _ => {
                error!(
                    "vehicle({}): state error before charging. current status is {:?}, expect ParkDone|Parking|ProcessDone.",
                    self.id,
                    self.state.read().await
                );
                Err(Error::State)
            }
        }
    }

    pub async fn get_action(
        &mut self,
        current_position: &Position,
        current_battery_level: f32,
    ) -> Option<Action> {
        self.overtime.update().await;
        let require_charge = current_battery_level <= 0.3;
        let mut state = self.state.write().await;
        loop {
            match &mut *state {
                State::Initing(actions) => {
                    let action = Self::next_action(current_position, &mut self.node, actions);
                    if action.is_some() {
                        return action;
                    }
                    *state = State::InitDone;
                }
                State::Processing(actions) => {
                    let action = Self::next_action(current_position, &mut self.node, actions);
                    if action.is_some() {
                        return action;
                    }
                    *state = State::ProcessDone;
                    Self::send_event(
                        &mut self.sender,
                        Event::ProcessDone {
                            vehicle_id: self.id,
                            vehicle_skill: self.skill.clone(),
                            task_id: self.current_task_id.unwrap(),
                        },
                    )
                    .await;
                }
                State::Parking(actions) => {
                    if require_charge {
                        self.charging(&mut state).await.ok()?;
                        Self::send_event(&mut self.sender, Event::ChargeStart).await;
                    } else {
                        let action = Self::next_action(current_position, &mut self.node, actions);
                        if action.is_some() {
                            return action;
                        }
                        *state = State::ParkDone;
                    }
                }
                State::Charging(actions) => {
                    let action = Self::next_action(current_position, &mut self.node, actions);
                    if action.is_some() {
                        return action;
                    }

                    if current_battery_level >= 0.95 {
                        *state = State::ChargeDone;
                        Self::send_event(&mut self.sender, Event::ChargeDone).await;
                    } else {
                        return None;
                    }
                }
                State::InitDone => {
                    if require_charge {
                        self.charging(&mut state).await.ok()?;
                        Self::send_event(&mut self.sender, Event::ChargeStart).await;
                    } else {
                        self.parking(&mut state).await.ok()?;
                    }
                }
                State::ChargeDone => {
                    self.parking(&mut state).await.ok()?;
                }
                State::ProcessDone => {
                    self.current_task_id = None;
                    if require_charge {
                        self.charging(&mut state).await.ok()?;
                        Self::send_event(&mut self.sender, Event::ChargeStart).await;
                    } else {
                        self.parking(&mut state).await.ok()?;
                    }
                }
                State::ParkDone => {
                    if require_charge {
                        self.charging(&mut state).await.ok()?;
                        Self::send_event(&mut self.sender, Event::ChargeStart).await;
                    } else {
                        return None;
                    }
                }
                State::Offline => {
                    self.initing(current_position, &mut state).await.ok()?;
                }
            }
        }
    }

    async fn send_event(sender: &mut Option<mpsc::Sender<Event>>, event: Event) {
        if let Some(event_sender) = sender {
            if let Err(_) = event_sender.send(event).await {
                *sender = None;
            }
        }
    }

    pub async fn processing(&mut self, task_id: i32, actions: ActionSequence) -> Result<()> {
        let mut state = self.state.write().await;
        match &*state {
            State::ParkDone | State::ChargeDone | State::ProcessDone | State::InitDone => {
                self.track_graph
                    .unlock_node(self.node()?.id)
                    .await
                    .map_err(Error::Db)?;
                *state = State::Processing(actions);
                Self::send_event(
                    &mut self.sender,
                    Event::ProcessStart {
                        vehicle_id: self.id,
                        vehicle_skill: self.skill.clone(),
                        task_id,
                    },
                )
                .await;
                self.current_task_id = Some(task_id);
                Ok(())
            }
            State::Parking(parking_actions) => {
                if let Some(node) = parking_actions.last_move_node() {
                    self.track_graph
                        .unlock_node(node.id)
                        .await
                        .map_err(Error::Db)?;
                }
                *state = State::Processing(actions);
                Self::send_event(
                    &mut self.sender,
                    Event::ProcessStart {
                        vehicle_id: self.id,
                        vehicle_skill: self.skill.clone(),
                        task_id,
                    },
                )
                .await;
                self.current_task_id = Some(task_id);
                Ok(())
            }
            State::Initing(_) | State::Processing(_) | State::Charging(_) | State::Offline => {
                error!(
                    "vehicle({}): state error before processing. current status is {:?}, expect ParkDone|Parking|ProcessDone|ChargeDone|InitDone",
                    self.id,
                    self.state.read().await
                );
                Err(Error::State)
            }
        }
    }
}

impl fmt::Debug for Vehicle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id:{} state:{:#?}", self.id, self.state)
    }
}

#[cfg(test)]
mod tests {
    use dotenvy::dotenv;
    use sqlx::postgres::PgPoolOptions;

    use crate::db_manager::DbManager;

    use super::*;

    async fn get_track_graph() -> Graph {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        println!("Connecting to database: {}", database_url);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create pool");
        let db_manaer = DbManager::new(pool);
        Graph::new(db_manaer).await
    }

    #[tokio::test]
    async fn init() {
        let track_graph = get_track_graph().await;
        let track_graph = Arc::new(track_graph);

        let mut vehicle = Vehicle::new(2000, track_graph.clone()).await;

        // move to shortest node
        assert!(
            matches!(vehicle.get_action(&(2.0, 4.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name == "S1")
        );
        assert!(matches!(*vehicle.state.read().await, State::Initing(_)));

        assert!(
            matches!(vehicle.get_action(&(1.0, 3.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name == "A3")
        );

        assert!(
            matches!(vehicle.get_action(&(1.0, 2.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name == "A5")
        );

        assert!(
            matches!(vehicle.get_action(&(0.0, 2.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name == "S3")
        );

        // move to shortest node   action
        assert!(matches!(
            vehicle
                .get_action(&(-1.0, 2.0, 0.0).into(), 1.0)
                .await
                .unwrap(),
            Action::Drop
        ));

        assert!(
            matches!(vehicle.get_action(&(-1.0, 2.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name == "A5")
        );

        assert!(matches!(*vehicle.state.read().await, State::Parking(_)));

        assert!(
            matches!(vehicle.get_action(&(0.0, 2.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name == "A6")
        );

        assert!(
            matches!(vehicle.get_action(&(0.0, 1.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name == "P2")
        );
        assert!(matches!(
            vehicle.get_action(&(0.0, 0.0, 0.0).into(), 1.0).await,
            None
        ));
        assert!(matches!(*vehicle.state.read().await, State::ParkDone));
    }

    #[tokio::test]
    async fn auto_charging() {
        let track_graph = get_track_graph().await;
        let track_graph = Arc::new(track_graph);

        let mut vehicle = Vehicle::new(2000, track_graph.clone()).await;

        assert!(matches!(vehicle
            .get_action(&(-2.0, 2.0, 0.0).into(), 0.1)
            .await.unwrap(), Action::Move(node) if node.name == "S3"));
        assert!(matches!(*vehicle.state.read().await, State::Initing(_)));

        assert!(matches!(
            vehicle
                .get_action(&(-1.0, 2.0, 0.0).into(), 0.8)
                .await
                .unwrap(),
            Action::Drop
        ));

        vehicle.get_action(&(-1.0, 2.0, 0.0).into(), 0.8).await;
        assert!(matches!(*vehicle.state.read().await, State::Parking(_)));

        vehicle.get_action(&(-1.0, 2.0, 0.0).into(), 0.1).await;
        assert!(matches!(*vehicle.state.read().await, State::Charging(_)));

        assert!(
            matches!(vehicle.get_action(&(-1.0, 2.0, 0.0).into(), 0.2).await.unwrap(), Action::Move(node) if node.name == "A5")
        );

        assert!(
            matches!(vehicle.get_action(&(0.0, 2.0, 0.0).into(), 0.2).await.unwrap(), Action::Move(node) if node.name == "A6")
        );

        assert!(
            matches!(vehicle.get_action(&(0.0, 1.0, 0.0).into(), 0.2).await.unwrap(), Action::Move(node) if node.name == "A2")
        );

        assert!(
            matches!(vehicle.get_action(&(1.0, 1.0, 0.0).into(), 0.2).await.unwrap(), Action::Move(node) if node.name == "C1")
        );

        assert!(matches!(
            vehicle.get_action(&(1.0, 0.0, 0.0).into(), 0.2).await,
            None
        ));

        // charge over
        assert!(
            matches!(vehicle.get_action(&(1.0, 0.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name == "A2")
        );

        // arrive A2, action move to A1
        assert!(
            matches!(vehicle.get_action(&(1.0, 1.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name == "A1")
        );

        // arrive A1, action move to P1
        assert!(
            matches!(vehicle.get_action(&(2.0, 1.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name == "P1")
        );

        // arrive P1
        assert!(
            vehicle
                .get_action(&(2.0, 0.0, 0.0).into(), 1.0)
                .await
                .is_none()
        );

        assert!(matches!(*vehicle.state.read().await, State::ParkDone));
    }
}
