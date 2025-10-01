use std::fmt;
use std::sync::Arc;

use tokio::sync::RwLock;
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
    FindPath,
    TrackGraph,
    NotInTrackGraph,
}

pub type Result<T> = std::result::Result<T, Error>;

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
    id: u32,
    state: Arc<RwLock<State>>,
    skill: Skill,
    overtime: Timeout,
    track_graph: Arc<Graph>,
    node: Option<Arc<track::Node>>,
}

impl Vehicle {
    pub async fn new(id: u32, track_graph: Arc<Graph>) -> Self {
        let skill = Skill::from_id(&id);
        let state = Arc::new(RwLock::new(State::Offline));
        Self {
            id,
            overtime: Timeout::new(state.clone()),
            state,
            skill,
            track_graph,
            node: None,
        }
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
            .find_shortest_node(current_position)
            .map_err(|e| {
                error!(
                    "vehicle({}): find shortest node path error in initing. error type: {:?}.current position is {:?}.",
                    self.id, e,current_position
                );
                Error::FindPath
            })?;
        let shortest_node_to_shipping_dock_path = self
            .track_graph
            .find_shipping_dock_path(shortest_node.name())
            .await
            .map_err(|e| {
                error!(
                    "vehicle({}): find shortest node to shipping dock path error in initing. error type: {:?}.current position is {:?}.",
                    self.id, e,current_position
                );
                Error::FindPath
            })?;
        let mut actions = ActionSequenceBuilder::new().move_to(shortest_node.clone());
        match self.skill {
            Skill::Item => {
                actions = actions
                    .move_path(&shortest_node_to_shipping_dock_path)
                    .drop(
                        shortest_node_to_shipping_dock_path
                            .last()
                            .ok_or_else(|| {
                                error!("vehicle({}): get shipping node from path error. current position is {:?}. current state is {:?}.",self.id, current_position, self.state);
                                Error::TrackGraph
                            })?
                            .side()
                            .ok_or_else(|| {
                                error!("vehicle({}): get side of shipping dock error. current position is {:?}. current state is {:?}.",self.id, current_position, self.state);
                                Error::TrackGraph
                            })?,
                    );
            }
            Skill::Fluid => {
                actions = actions
                    .move_path(&shortest_node_to_shipping_dock_path)
                    .fill(
                        shortest_node_to_shipping_dock_path
                            .last()
                            .ok_or_else(|| {
                                error!("vehicle({}): get shipping node from path error. current position is {:?}. current state is {:?}.",self.id, current_position, self.state);
                                Error::TrackGraph
                            })?
                            .side()
                            .ok_or_else(|| {
                                error!("vehicle({}): get side of shipping dock error. current position is {:?}. current state is {:?}",self.id, current_position, self.state);
                                Error::TrackGraph
                            })?,
                    );
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
                if current_position == node.position() {
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
            self.track_graph.unlock_node(self.node()?.name()).await;
        }
        match *state {
            State::ChargeDone | State::ProcessDone | State::InitDone => {
                let path = self
                    .track_graph
                    .find_parking_path(
                        self.node()
                            .map_err(|e| {
                                error!(
                                    "vehicle({}): find current node in parking. error type: {:?}.",
                                    self.id, e
                                );
                                Error::NotInTrackGraph
                            })?
                            .name(),
                    )
                    .await
                    .map_err(|_| Error::FindPath)?;
                self.track_graph
                    .lock_node(path.last().ok_or_else(|| {
                        error!("vehicle({}): get parking node from path error. current state is {:?}.",self.id, self.state);
                        Error::TrackGraph
                    })?.name())
                    .await;
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
            self.track_graph.unlock_node(node.name()).await;
        }
        match *state {
            State::ParkDone | State::Parking(_) | State::ProcessDone | State::InitDone => {
                let path = self
                    .track_graph
                    .find_charging_path(
                        self.node()
                            .map_err(|e| {
                                error!(
                                    "vehicle({}): find current node in parking. error type: {:?}.",
                                    self.id, e
                                );
                                Error::NotInTrackGraph
                            })?
                            .name(),
                    )
                    .await
                    .map_err(|_| Error::FindPath)?;
                self.track_graph
                    .lock_node(path.last().ok_or_else(|| {
                        error!("vehicle({}): get parking node from path error. current state is {:?}.",self.id, self.state);
                        Error::TrackGraph
                    })?.name())
                    .await;
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
                }
                State::Parking(actions) => {
                    if require_charge {
                        self.charging(&mut state).await.ok()?;
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
                    } else {
                        return None;
                    }
                }
                State::InitDone => {
                    if require_charge {
                        self.charging(&mut state).await.ok()?;
                    } else {
                        self.parking(&mut state).await.ok()?;
                    }
                }
                State::ChargeDone => {
                    self.parking(&mut state).await.ok()?;
                }
                State::ProcessDone => {
                    if require_charge {
                        self.charging(&mut state).await.ok()?;
                    } else {
                        self.parking(&mut state).await.ok()?;
                    }
                }
                State::ParkDone => {
                    if require_charge {
                        self.charging(&mut state).await.ok()?;
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

    pub async fn processing(&mut self, actions: ActionSequence) -> Result<()> {
        let mut state = self.state.write().await;
        match &*state {
            State::ParkDone | State::ChargeDone | State::ProcessDone | State::InitDone => {
                self.track_graph.unlock_node(self.node()?.name()).await;
                *state = State::Processing(actions);
                Ok(())
            }
            State::Parking(parking_actions) => {
                if let Some(node) = parking_actions.last_move_node() {
                    self.track_graph.unlock_node(node.name()).await;
                }
                *state = State::Processing(actions);
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
    use super::track::NodeType;
    use super::*;

    fn get_tarck_graph() -> track::Graph {
        track::TrackGraphBuilder::new()
            .node("S1", (2.0, 3.0, 0.0), NodeType::ShippingDock(Side::PosZ))
            .node("P2", (0.0, 0.0, 0.0), NodeType::ParkingStation)
            .node("C1", (1.0, 0.0, 0.0), NodeType::ChargingStation)
            .node("P1", (2.0, 0.0, 0.0), NodeType::ParkingStation)
            .node("A1", (2.0, 1.0, 0.0), NodeType::Fork)
            .node("A2", (1.0, 1.0, 0.0), NodeType::Fork)
            .node("A3", (1.0, 2.0, 0.0), NodeType::Fork)
            .node("A4", (2.0, 2.0, 0.0), NodeType::Fork)
            .node("A5", (0.0, 2.0, 0.0), NodeType::Fork)
            .node("A6", (0.0, 1.0, 0.0), NodeType::Fork)
            .edge_double("P2", "A6")
            .edge_double("C1", "A2")
            .edge_double("P1", "A1")
            .edge_double("S1", "A4")
            .edge("A6", "A2")
            .edge("A2", "A1")
            .edge("A1", "A4")
            .edge("A4", "A3")
            .edge("A3", "A2")
            .edge("A3", "A5")
            .edge("A5", "A6")
            .build()
    }

    #[tokio::test]
    async fn init() {
        let track_graph = get_tarck_graph();
        let track_graph = Arc::new(track_graph);

        let mut vehicle = Vehicle::new(2000, track_graph.clone()).await;

        // move to shortest node
        // move to S1 Shipping dock
        assert!(
            matches!(vehicle.get_action(&(2.0, 4.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name() == "S1")
        );
        assert!(matches!(*vehicle.state.read().await, State::Initing(_)));

        // move to shortest node   action
        assert!(matches!(
            vehicle
                .get_action(&(2.0, 3.0, 0.0).into(), 1.0)
                .await
                .unwrap(),
            Action::Drop(Side::PosZ)
        ));

        // yet to A4, still action move to A4
        assert!(
            matches!(vehicle.get_action(&(2.0, 3.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name() == "A4")
        );

        assert!(matches!(*vehicle.state.read().await, State::Parking(_)));

        // arrive A4, action move to A3
        assert!(
            matches!(vehicle.get_action(&(2.0, 2.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name() == "A3")
        );
        // arrive A3, action move to A2
        assert!(
            matches!(vehicle.get_action(&(1.0, 2.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name() == "A2")
        );
        // arrive A2, action move to A1
        assert!(
            matches!(vehicle.get_action(&(1.0, 1.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name() == "A1")
        );

        // arrive A1, action move to P1
        assert!(
            matches!(vehicle.get_action(&(2.0, 1.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name() == "P1")
        );

        // arrive P1, check state
        assert!(
            vehicle
                .get_action(&(2.0, 0.0, 0.0).into(), 1.0)
                .await
                .is_none()
        );
        assert!(matches!(*vehicle.state.read().await, State::ParkDone));

        // Check P1 locked
        assert!(track_graph.get_lock_nodes().await.contains("P1"));
    }

    #[tokio::test]
    async fn auto_charging() {
        let track_graph = get_tarck_graph();
        let track_graph = Arc::new(track_graph);

        let mut vehicle = Vehicle::new(2000, track_graph.clone()).await;

        assert!(matches!(vehicle
            .get_action(&(2.0, 2.0, 0.0).into(), 0.1)
            .await.unwrap(), Action::Move(node) if node.name() == "S1"));
        assert!(matches!(*vehicle.state.read().await, State::Initing(_)));

        assert!(matches!(
            vehicle
                .get_action(&(2.0, 3.0, 0.0).into(), 0.8)
                .await
                .unwrap(),
            Action::Drop(_)
        ));

        vehicle.get_action(&(2.0, 3.0, 0.0).into(), 0.8).await;
        assert!(matches!(*vehicle.state.read().await, State::Parking(_)));

        vehicle.get_action(&(2.0, 2.0, 0.0).into(), 0.1).await;
        assert!(matches!(*vehicle.state.read().await, State::Charging(_)));
        assert_eq!(track_graph.get_lock_nodes().await.len(), 1);
        assert!(track_graph.get_lock_nodes().await.contains("C1"));

        // arrive A3, action move to A2
        assert!(
            matches!(vehicle.get_action(&(1.0, 2.0, 0.0).into(), 0.2).await.unwrap(), Action::Move(node) if node.name() == "A2")
        );
        assert_eq!(track_graph.get_lock_nodes().await.len(), 1);
        assert!(track_graph.get_lock_nodes().await.contains("C1"));

        // arrive A2, action move to C1
        assert!(
            matches!(vehicle.get_action(&(1.0, 1.0, 0.0).into(), 0.2).await.unwrap(), Action::Move(node) if node.name() == "C1")
        );
        assert_eq!(track_graph.get_lock_nodes().await.len(), 1);
        assert!(track_graph.get_lock_nodes().await.contains("C1"));

        // charge over
        assert!(
            matches!(vehicle.get_action(&(1.0, 0.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name() == "A2")
        );
        assert_eq!(track_graph.get_lock_nodes().await.len(), 1);
        assert!(track_graph.get_lock_nodes().await.contains("P1"));

        // arrive A2, action move to A1
        assert!(
            matches!(vehicle.get_action(&(1.0, 1.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name() == "A1")
        );
        assert_eq!(track_graph.get_lock_nodes().await.len(), 1);
        assert!(track_graph.get_lock_nodes().await.contains("P1"));

        println!(
            "{:#?} \n {:#?}",
            vehicle,
            track_graph.get_lock_nodes().await
        );
        // arrive A1, action move to P1
        assert!(
            matches!(vehicle.get_action(&(2.0, 1.0, 0.0).into(), 1.0).await.unwrap(), Action::Move(node) if node.name() == "P1")
        );
        assert_eq!(track_graph.get_lock_nodes().await.len(), 1);
        assert!(track_graph.get_lock_nodes().await.contains("P1"));

        // arrive P1
        assert!(
            vehicle
                .get_action(&(2.0, 0.0, 0.0).into(), 1.0)
                .await
                .is_none()
        );
        assert!(track_graph.get_lock_nodes().await.contains("P1"));

        assert!(matches!(*vehicle.state.read().await, State::ParkDone));
        assert!(track_graph.get_lock_nodes().await.contains("P1"));
    }
}
