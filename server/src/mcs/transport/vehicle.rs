use std::collections::LinkedList;
use std::sync::Arc;

use tracing::error;

use super::track;
use super::track::TrackGraph;
use crate::mcs::Position;
use crate::mcs::Side;

#[derive(Debug)]
pub enum Error {
    StateError,
    FindPathError,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Action {
    Move(Arc<track::Node>),
    Drop(Side),
    Suck(Side),
    Drain(Side),
    Fill(Side),
    Use(Side),
}

#[derive(Debug)]
pub struct ActionSequence(LinkedList<Action>);

impl ActionSequence {
    pub fn next_action(&self) -> Option<&Action> {
        self.0.front()
    }

    pub fn pop_next_action(&mut self) -> Option<Action> {
        self.0.pop_front()
    }

    pub fn last_move_node(&self) -> Option<Arc<track::Node>> {
        for action in self.0.iter().rev() {
            if let Action::Move(node) = action {
                return Some(node.clone());
            }
        }
        None
    }
}

pub struct ActionSequenceBuilder(LinkedList<Action>);

impl ActionSequenceBuilder {
    pub fn new() -> Self {
        Self(LinkedList::new())
    }

    pub fn move_path(mut self, path: &track::Path) -> Self {
        for node in path.iter().skip(1) {
            self.0.push_back(Action::Move(node.clone()));
        }
        self
    }

    pub fn move_to(mut self, node: Arc<track::Node>) -> Self {
        self.0.push_back(Action::Move(node));
        self
    }

    pub fn drop(mut self, side: &Side) -> Self {
        self.0.push_back(Action::Drop(side.clone()));
        self
    }

    pub fn suck(mut self, side: &Side) -> Self {
        self.0.push_back(Action::Suck(side.clone()));
        self
    }

    pub fn drain(mut self, side: &Side) -> Self {
        self.0.push_back(Action::Drain(side.clone()));
        self
    }

    pub fn fill(mut self, side: &Side) -> Self {
        self.0.push_back(Action::Fill(side.clone()));
        self
    }

    pub fn use_tool(mut self, side: &Side) -> Self {
        self.0.push_back(Action::Use(side.clone()));
        self
    }

    pub fn chain(mut self, mut sequence: Self) -> Self {
        self.0.append(&mut sequence.0);
        self
    }

    pub fn build(self) -> ActionSequence {
        ActionSequence(self.0)
    }
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

#[derive(Debug)]
pub struct Vehicle {
    id: u32,
    state: State,
    node: Option<Arc<track::Node>>,
}

impl Vehicle {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            state: State::Offline,
            node: None,
        }
    }

    pub fn offline(&mut self) {
        self.state = State::Offline;
    }

    pub fn inline(&mut self, current_position: &Position, track_graph: &TrackGraph) {
        if let State::Offline = self.state {
            match track_graph.find_shortest_node(current_position) {
                Ok(node) => {
                    let actions = ActionSequenceBuilder::new().move_to(node.clone()).build();
                    self.node = Some(node);
                    self.state = State::Initing(actions);
                }
                Err(_) => error!("{} can't find shortest node.", self.id),
            }
        }
    }

    pub fn node(&self) -> Option<Arc<track::Node>> {
        self.node.clone()
    }

    pub fn idle(&self) -> bool {
        match &self.state {
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
                    Some(actions.next_action().unwrap().clone())
                }
            }
            _ => {
                actions.pop_next_action();
                actions.next_action().cloned()
            }
        }
    }

    async fn parking(&mut self, track_graph: &TrackGraph) -> Result<()> {
        if let State::ChargeDone = &self.state {
            track_graph
                .unlock_node(self.node.clone().unwrap().name())
                .await;
        }
        match self.state {
            State::ChargeDone | State::ProcessDone | State::InitDone => {
                let actions = match track_graph
                    .find_parking_path(self.node.clone().unwrap().name())
                    .await
                {
                    Ok(path) => {
                        track_graph.lock_node(path.last().unwrap().name()).await;
                        Some(ActionSequenceBuilder::new().move_path(&path).build())
                    }
                    Err(e) => {
                        error!(
                            "{} suffer {:?}, current state: {:?}",
                            self.id, e, self.state
                        );
                        None
                    }
                };

                match actions {
                    Some(actions) => {
                        self.state = State::Parking(actions);
                        Ok(())
                    }
                    None => Err(Error::FindPathError),
                }
            }
            _ => Err(Error::StateError),
        }
    }

    async fn charging(&mut self, track_graph: &TrackGraph) -> Result<()> {
        if let State::Parking(actions) = &self.state
            && let Some(node) = actions.last_move_node()
        {
            track_graph.unlock_node(node.name()).await;
        }
        match self.state {
            State::ParkDone | State::Parking(_) | State::ProcessDone => {
                let actions = match track_graph
                    .find_charging_path(self.node.clone().unwrap().name())
                    .await
                {
                    Ok(path) => {
                        track_graph.lock_node(path.last().unwrap().name()).await;
                        Some(ActionSequenceBuilder::new().move_path(&path).build())
                    }
                    Err(e) => {
                        error!(
                            "{} suffer {:?}, current state: {:?}",
                            self.id, e, self.state
                        );
                        None
                    }
                };
                if let Some(actions) = actions {
                    self.state = State::Charging(actions)
                }

                Ok(())
            }
            _ => Err(Error::StateError),
        }
    }

    pub async fn get_action(
        &mut self,
        current_position: &Position,
        current_battery_level: f32,
        track_graph: &TrackGraph,
    ) -> Option<Action> {
        let require_charge = current_battery_level <= 0.3;
        loop {
            match &mut self.state {
                State::Initing(actions) => {
                    let action = Self::next_action(current_position, &mut self.node, actions);
                    if action.is_some() {
                        return action;
                    }
                    self.state = State::InitDone;
                }
                State::Processing(actions) => {
                    let action = Self::next_action(current_position, &mut self.node, actions);
                    if action.is_some() {
                        return action;
                    }
                    self.state = State::ProcessDone;
                }
                State::Parking(actions) => {
                    if require_charge {
                        self.charging(track_graph).await.unwrap();
                    } else {
                        let action = Self::next_action(current_position, &mut self.node, actions);
                        if action.is_some() {
                            return action;
                        }
                        self.state = State::ParkDone;
                    }
                }
                State::Charging(actions) => {
                    let action = Self::next_action(current_position, &mut self.node, actions);
                    if action.is_some() {
                        return action;
                    }

                    if current_battery_level >= 0.95 {
                        self.state = State::ChargeDone;
                    } else {
                        return None;
                    }
                }
                State::InitDone => {
                    self.parking(track_graph).await.unwrap();
                }
                State::ChargeDone => {
                    self.parking(track_graph).await.unwrap();
                }
                State::ProcessDone => {
                    if require_charge {
                        self.charging(track_graph).await.unwrap();
                    } else {
                        self.parking(track_graph).await.unwrap();
                    }
                }
                State::ParkDone => {
                    if require_charge {
                        self.charging(track_graph).await.unwrap();
                    } else {
                        return None;
                    }
                }
                State::Offline => unreachable!(),
            }
        }
    }

    pub async fn processing(
        &mut self,
        actions: ActionSequence,
        track_graph: &TrackGraph,
    ) -> Result<()> {
        match &self.state {
            State::ParkDone | State::ChargeDone | State::ProcessDone | State::InitDone => {
                track_graph
                    .unlock_node(self.node.clone().unwrap().name())
                    .await;
                self.state = State::Processing(actions);
                Ok(())
            }
            State::Parking(parking_actions) => {
                if let Some(node) = parking_actions.last_move_node() {
                    track_graph.unlock_node(node.name()).await;
                }
                self.state = State::Processing(actions);
                Ok(())
            }
            State::Initing(_) | State::Processing(_) | State::Charging(_) | State::Offline => {
                Err(Error::StateError)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::track::NodeType;
    use super::*;

    fn get_tarck_graph() -> track::TrackGraph {
        track::TrackGraphBuilder::new()
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

        let mut vehicle = Vehicle::new(1000);
        let current_position = (8.0, 8.0, 0.0).into();
        vehicle.inline(&current_position, &track_graph);

        // move to shortest node
        if let State::Initing(_) = vehicle.state {
        } else {
            assert!(false)
        }

        // move to shortest node   action
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .await
            .unwrap()
        {
            assert_eq!(node.name(), "A4");
        } else {
            assert!(false)
        }

        // yet to A4, still action move to A4
        let current_position = (6.0, 0.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .await
            .unwrap()
        {
            assert_eq!(node.name(), "A4");
        }

        if let State::Initing(_) = vehicle.state {
        } else {
            assert!(false)
        }

        // arrive A4, action move to A3
        let current_position = (2.0, 2.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .await
            .unwrap()
        {
            assert_eq!(node.name(), "A3");
        }

        // arrive A3, action move to A2
        let current_position = (1.0, 2.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .await
            .unwrap()
        {
            assert_eq!(node.name(), "A2");
        }

        // arrive A2, action move to A1
        let current_position = (1.0, 1.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .await
            .unwrap()
        {
            assert_eq!(node.name(), "A1");
        }

        // arrive A1, action move to P1
        let current_position = (2.0, 1.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .await
            .unwrap()
        {
            assert_eq!(node.name(), "P1");
        }

        // arrive P1, check state
        let current_position = (2.0, 0.0, 0.0).into();
        assert!(
            vehicle
                .get_action(&current_position, 1.0, &track_graph)
                .await
                .is_none()
        );
        if let State::ParkDone = vehicle.state {
        } else {
            assert!(false)
        }

        // Check P1 locked
        assert!(track_graph.get_lock_node().await.contains("P1"));
    }

    #[tokio::test]
    async fn auto_charging() {
        let track_graph = get_tarck_graph();

        let mut vehicle = Vehicle::new(1000);

        let current_position = (2.0, 2.0, 0.0).into();
        vehicle.inline(&current_position, &track_graph);

        vehicle.get_action(&current_position, 0.1, &track_graph).await;
        if let State::Charging(_) = vehicle.state {
        } else {
            assert!(false)
        }
        assert_eq!(track_graph.get_lock_node().await.len(), 1);
        assert!(track_graph.get_lock_node().await.contains("C1"));

        // arrive A3, action move to A2
        let current_position = (1.0, 2.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 0.2, &track_graph)
            .await
            .unwrap()
        {
            assert_eq!(node.name(), "A2");
        }
        assert_eq!(track_graph.get_lock_node().await.len(), 1);
        assert!(track_graph.get_lock_node().await.contains("C1"));

        // arrive A2, action move to C1
        let current_position = (1.0, 1.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 0.2, &track_graph)
            .await
            .unwrap()
        {
            assert_eq!(node.name(), "C1");
        }
        assert_eq!(track_graph.get_lock_node().await.len(), 1);
        assert!(track_graph.get_lock_node().await.contains("C1"));

        // charge over
        let current_position = (1.0, 0.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .await
            .unwrap()
        {
            assert_eq!(node.name(), "A2");
        }
        assert_eq!(track_graph.get_lock_node().await.len(), 1);
        assert!(track_graph.get_lock_node().await.contains("P1"));

        // arrive A2, action move to A1
        let current_position = (1.0, 1.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .await
            .unwrap()
        {
            assert_eq!(node.name(), "A1");
        }
        assert_eq!(track_graph.get_lock_node().await.len(), 1);
        assert!(track_graph.get_lock_node().await.contains("P1"));

        println!("{:#?} \n {:#?}", vehicle, track_graph.get_lock_node().await);
        // arrive A1, action move to P1
        let current_position = (2.0, 1.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .await
            .unwrap()
        {
            assert_eq!(node.name(), "P1");
        }
        assert_eq!(track_graph.get_lock_node().await.len(), 1);
        assert!(track_graph.get_lock_node().await.contains("P1"));

        // arrive P1
        let current_position = (2.0, 0.0, 0.0).into();
        vehicle.get_action(&current_position, 1.0, &track_graph).await;
        assert!(track_graph.get_lock_node().await.contains("P1"));

        if let State::ParkDone = vehicle.state {
        } else {
            assert!(false)
        }
        assert!(track_graph.get_lock_node().await.contains("P1"));
    }
}
