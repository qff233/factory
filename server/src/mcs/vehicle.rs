use std::collections::LinkedList;
use std::rc::Rc;

use tracing::error;

use super::Position;
use super::Side;
use super::track::{self, TrackGraph};

#[derive(Debug, Clone)]
pub enum Action {
    Move(Rc<track::Node>),
    Drop(Side),
    Suck(Side),
    Drain(Side),
    Fill(Side),
    Use(Side),
}

type ActionSequence = LinkedList<Action>;

#[derive(Debug)]
enum ErrorKind {
    StateError,
    FindPathError,
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
pub enum TransType {
    Item,
    Fluid,
}

#[derive(Debug)]
pub struct Vechicle {
    id: u32,
    trans_type: TransType,
    state: State,
    landmark: Option<Rc<track::Node>>,
}

impl Vechicle {
    fn new(id: u32) -> Self {
        let trans_type = if id > 2000 {
            TransType::Fluid
        } else {
            TransType::Item
        };
        Self {
            id,
            trans_type,
            state: State::Offline,
            landmark: None,
        }
    }

    pub fn offline(&mut self) {
        self.state = State::Offline;
    }

    fn inline(&mut self, current_position: &Position, track_graph: &TrackGraph) {
        if let State::Offline = self.state {
            match track_graph.find_shortest_node(current_position) {
                Ok(node) => {
                    let mut actions = LinkedList::new();
                    actions.push_back(Action::Move(node.clone()));
                    self.landmark = Some(node);
                    self.state = State::Initing(actions.clone());
                }
                Err(_) => error!("{} can't find shortest node.", self.id),
            }
        }
    }

    fn get_move_actions_from_path(&self, path: track::Path) -> ActionSequence {
        let mut actions: ActionSequence = LinkedList::new();
        for node in path {
            actions.push_back(Action::Move(node));
        }
        actions
    }

    fn next_action(
        current_position: &Position,
        landmark: &mut Option<Rc<track::Node>>,
        actions: &mut ActionSequence,
    ) -> Option<Action> {
        match actions.front().unwrap() {
            Action::Move(node) => {
                if current_position == node.position() {
                    *landmark = Some(node.clone());
                    actions.pop_front();
                    return actions.front().cloned();
                } else {
                    return Some(actions.front().unwrap().clone());
                }
            }
            _ => return actions.pop_front(),
        }
    }

    fn parking(&mut self, track_graph: &TrackGraph) -> Result<(), ErrorKind> {
        if let State::ChargeDone = &self.state {
            track_graph.unlock_node(self.landmark.clone().unwrap().name());
        }
        match self.state {
            State::ChargeDone | State::ProcessDone | State::InitDone => {
                let actions =
                    match track_graph.find_parking_path(self.landmark.clone().unwrap().name()) {
                        Ok(path) => {
                            track_graph.lock_node(path.last().unwrap().name());
                            Some(self.get_move_actions_from_path(path))
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
                    None => Err(ErrorKind::FindPathError),
                }
            }
            _ => Err(ErrorKind::StateError),
        }
    }

    fn charging(&mut self, track_graph: &TrackGraph) -> Result<(), ErrorKind> {
        if let State::Parking(actions) = &self.state {
            for station in actions.iter().rev() {
                if let Action::Move(node) = station {
                    track_graph.unlock_node(node.name());
                    break;
                }
            }
        }
        match self.state {
            State::ParkDone | State::Parking(_) | State::ProcessDone => {
                let actions =
                    match track_graph.find_charging_path(self.landmark.clone().unwrap().name()) {
                        Ok(path) => {
                            track_graph.lock_node(path.last().unwrap().name());
                            Some(self.get_move_actions_from_path(path))
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
            _ => Err(ErrorKind::StateError),
        }
    }

    pub fn get_action(
        &mut self,
        current_position: &Position,
        current_battery_level: f32,
        track_graph: &TrackGraph,
    ) -> Option<Action> {
        let require_charge = current_battery_level <= 0.3;
        loop {
            match &mut self.state {
                State::Initing(actions) => {
                    let action = Self::next_action(current_position, &mut self.landmark, actions);
                    if action.is_some() {
                        return action;
                    }
                    self.state = State::InitDone;
                }
                State::Processing(actions) => {
                    let action = Self::next_action(current_position, &mut self.landmark, actions);
                    if action.is_some() {
                        return action;
                    }
                    self.state = State::ProcessDone;
                }
                State::Parking(actions) => {
                    if require_charge {
                        self.charging(track_graph).unwrap();
                    } else {
                        let action =
                            Self::next_action(current_position, &mut self.landmark, actions);
                        if action.is_some() {
                            return action;
                        }
                        self.state = State::ParkDone;
                    }
                }
                State::Charging(actions) => {
                    let action = Self::next_action(current_position, &mut self.landmark, actions);
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
                    self.parking(track_graph).unwrap();
                }
                State::ChargeDone => {
                    self.parking(track_graph).unwrap();
                }
                State::ProcessDone => {
                    if require_charge {
                        self.charging(track_graph).unwrap();
                    } else {
                        self.parking(track_graph).unwrap();
                    }
                }
                State::ParkDone => {
                    if require_charge {
                        self.charging(track_graph).unwrap();
                    } else {
                        return None;
                    }
                }
                State::Offline => unreachable!(),
            }
        }
    }

    pub fn processing(
        &mut self,
        actions: ActionSequence,
        track_graph: &TrackGraph,
    ) -> Result<(), ErrorKind> {
        match &self.state {
            State::ParkDone | State::ChargeDone | State::ProcessDone | State::InitDone => {
                track_graph.unlock_node(self.landmark.clone().unwrap().name());
                self.state = State::Processing(actions);
                Ok(())
            }
            State::Parking(parking_actions) => {
                for station in parking_actions.iter().rev() {
                    if let Action::Move(node) = station {
                        track_graph.unlock_node(node.name());
                        break;
                    }
                }
                self.state = State::Processing(actions);
                Ok(())
            }
            State::Initing(_) | State::Processing(_) | State::Charging(_) | State::Offline => {
                Err(ErrorKind::StateError)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::track::NodeType;
    use super::*;
    #[test]
    fn init() {
        let track_graph = track::TrackGraphBuilder::new()
            .node("P2", (0.0, 0.0, 0.0).into(), NodeType::ParkingStation)
            .node("C1", (1.0, 0.0, 0.0).into(), NodeType::ChargingStation)
            .node("P1", (2.0, 0.0, 0.0).into(), NodeType::ParkingStation)
            .node("A1", (2.0, 1.0, 0.0).into(), NodeType::Fork)
            .node("A2", (1.0, 1.0, 0.0).into(), NodeType::Fork)
            .node("A3", (1.0, 2.0, 0.0).into(), NodeType::Fork)
            .node("A4", (2.0, 2.0, 0.0).into(), NodeType::Fork)
            .node("A5", (0.0, 2.0, 0.0).into(), NodeType::Fork)
            .node("A6", (0.0, 1.0, 0.0).into(), NodeType::Fork)
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
            .build();

        let mut vehicle = Vechicle::new(1000);
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
            .unwrap()
        {
            assert_eq!(node.name(), "A3");
        }

        // arrive A3, action move to A2
        let current_position = (1.0, 2.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .unwrap()
        {
            assert_eq!(node.name(), "A2");
        }

        // arrive A2, action move to A1
        let current_position = (1.0, 1.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .unwrap()
        {
            assert_eq!(node.name(), "A1");
        }

        // arrive A1, action move to P1
        let current_position = (2.0, 1.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .unwrap()
        {
            assert_eq!(node.name(), "P1");
        }

        // arrive P1, check state
        let current_position = (2.0, 0.0, 0.0).into();
        assert!(
            vehicle
                .get_action(&current_position, 1.0, &track_graph)
                .is_none()
        );
        if let State::ParkDone = vehicle.state {
        } else {
            assert!(false)
        }

        // Check P1 locked
        assert!(track_graph.get_lock_node().contains("P1"));
    }

    #[test]
    fn auto_charging() {
        let track_graph = track::TrackGraphBuilder::new()
            .node("P2", (0.0, 0.0, 0.0).into(), NodeType::ParkingStation)
            .node("C1", (1.0, 0.0, 0.0).into(), NodeType::ChargingStation)
            .node("P1", (2.0, 0.0, 0.0).into(), NodeType::ParkingStation)
            .node("A1", (2.0, 1.0, 0.0).into(), NodeType::Fork)
            .node("A2", (1.0, 1.0, 0.0).into(), NodeType::Fork)
            .node("A3", (1.0, 2.0, 0.0).into(), NodeType::Fork)
            .node("A4", (2.0, 2.0, 0.0).into(), NodeType::Fork)
            .node("A5", (0.0, 2.0, 0.0).into(), NodeType::Fork)
            .node("A6", (0.0, 1.0, 0.0).into(), NodeType::Fork)
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
            .build();

        let mut vehicle = Vechicle::new(1000);

        let current_position = (2.0, 2.0, 0.0).into();
        vehicle.inline(&current_position, &track_graph);

        vehicle.get_action(&current_position, 0.1, &track_graph);
        if let State::Charging(_) = vehicle.state {
        } else {
            assert!(false)
        }

        // arrive A3, action move to A2
        let current_position = (1.0, 2.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 0.2, &track_graph)
            .unwrap()
        {
            assert_eq!(node.name(), "A2");
        }

        // arrive A2, action move to C1
        let current_position = (1.0, 1.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 0.2, &track_graph)
            .unwrap()
        {
            assert_eq!(node.name(), "C1");
        }

        assert_eq!(track_graph.get_lock_node().len(), 1);

        // charge over
        let current_position = (1.0, 0.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .unwrap()
        {
            assert_eq!(node.name(), "A2");
        }
        assert_eq!(track_graph.get_lock_node().len(), 1);

        // arrive A2, action move to A1
        let current_position = (1.0, 1.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .unwrap()
        {
            assert_eq!(node.name(), "A1");
        }
        assert_eq!(track_graph.get_lock_node().len(), 1);

        println!("{:#?} \n {:#?}", vehicle, track_graph.get_lock_node());
        // arrive A1, action move to P1
        let current_position = (2.0, 1.0, 0.0).into();
        if let Action::Move(node) = vehicle
            .get_action(&current_position, 1.0, &track_graph)
            .unwrap()
        {
            assert_eq!(node.name(), "P1");
        }
        assert_eq!(track_graph.get_lock_node().len(), 1);

        // arrive P1
        let current_position = (2.0, 0.0, 0.0).into();
        vehicle.get_action(&current_position, 1.0, &track_graph);

        if let State::ParkDone = vehicle.state {
        } else {
            assert!(false)
        }
    }
}
