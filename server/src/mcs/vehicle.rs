use std::collections::HashMap;
use std::collections::LinkedList;
use std::rc::Rc;

use tracing::error;

use super::Position;
use super::Side;
use super::track::{self, TrackGraph};

#[derive(Debug)]
enum Command {
    UseToolIn(String, Side),
}

#[derive(Debug)]
enum Action {
    Move(Rc<track::Node>),
    Use(Side),
}

type ActionSequence = LinkedList<Action>;

#[derive(Debug)]
enum ErrorKind {
    StateError,
}

#[derive(Debug)]
enum State {
    ChargeDone,
    Charging(ActionSequence),
    Offline,
    ParkDone,
    Parking(ActionSequence),
    ProcessDone,
    Processing(ActionSequence),
}

#[derive(Debug)]
struct Vechicle {
    id: u32,
    state: State,
    landmark: Option<Rc<track::Node>>,
}

impl Vechicle {
    fn new(id: u32) -> Self {
        Self {
            id,
            state: State::Offline,
            landmark: None,
        }
    }

    fn offline(&mut self) {
        self.state = State::Offline;
    }

    fn inline(&mut self, current_position: &Position, track_graph: &TrackGraph) {
        if let State::Offline = self.state {
            self.state = State::ParkDone;
            if self.landmark.is_none() {
                match track_graph.find_shortest_node(current_position) {
                    Ok(node) => {
                        let mut actions = LinkedList::new();
                        actions.push_back(Action::Move(node));
                        self.processing(actions, track_graph).unwrap();
                    }
                    Err(_) => error!("{} can't find shortest node.", self.id),
                }
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

    fn next_action(&mut self, current_position: &Position) -> Option<Action> {
        match &mut self.state {
            State::Processing(actions) | State::Parking(actions) => {
                let action = actions.front().unwrap();
                match action {
                    Action::Move(node) if current_position == node.position() => {
                        self.landmark = Some(node.clone());
                        return actions.pop_front();
                    }
                    Action::Use(_side) => return actions.pop_front(),
                    _ => return None,
                }
            }
            _ => None,
        }
    }

    fn parking(&mut self, track_graph: &TrackGraph) -> Result<(), ErrorKind> {
        // unlock
        if let State::ChargeDone = &self.state {
            track_graph.unlock_node(self.landmark.clone().unwrap().name());
        }
        match self.state {
            State::ChargeDone | State::ProcessDone => {
                let actions =
                    match track_graph.find_parking_path(self.landmark.clone().unwrap().name()) {
                        Ok(path) => {
                            // TODO check unlock
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
                    self.state = State::Parking(actions)
                }
                Ok(())
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
                            // TODO check unlock
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
                    self.state = State::Parking(actions)
                }

                Ok(())
            }
            _ => Err(ErrorKind::StateError),
        }
    }

    pub fn update(
        &mut self,
        current_position: &Position,
        current_battery_level: f32,
        track_graph: &TrackGraph,
    ) -> Option<Action> {
        let require_charge = current_battery_level <= 0.3;
        match &mut self.state {
            State::Processing(actions) => {
                while !actions.is_empty() {
                    return self.next_action(current_position);
                }
                self.state = State::ProcessDone;
            }
            State::Parking(actions) => {
                if require_charge {
                    self.charging(track_graph).unwrap();
                } else {
                    while !actions.is_empty() {
                        return self.next_action(current_position);
                    }
                    self.state = State::ParkDone;
                }
            }
            State::Charging(actions) => {
                while !actions.is_empty() {
                    return self.next_action(current_position);
                }
                if current_battery_level >= 0.95 {
                    self.state = State::ChargeDone;
                }
            }
            State::ChargeDone => self.parking(track_graph).unwrap(),
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
                }
            }
            State::Offline => unreachable!(),
        }
        None
    }

    pub fn processing(
        &mut self,
        actions: ActionSequence,
        track_graph: &TrackGraph,
    ) -> Result<(), ErrorKind> {
        match &self.state {
            State::ParkDone | State::ChargeDone | State::ProcessDone => {
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
            State::Processing(_) | State::Charging(_) | State::Offline => {
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
    fn status_update() {
        let track_graph = track::TrackGraphBuilder::new()
            .node(
                "P2",
                (0.0, 0.0, 0.0).into(),
                track::NodeType::ParkingStation,
            )
            .node(
                "C1",
                (1.0, 0.0, 0.0).into(),
                track::NodeType::ChargingStation,
            )
            .node(
                "P1",
                (3.0, 0.0, 0.0).into(),
                track::NodeType::ParkingStation,
            )
            .node("A1", (2.0, 1.0, 0.0).into(), track::NodeType::Fork)
            .node("A2", (1.0, 1.0, 0.0).into(), track::NodeType::Fork)
            .node("A3", (1.0, 2.0, 0.0).into(), track::NodeType::Fork)
            .node("A4", (2.0, 2.0, 0.0).into(), track::NodeType::Fork)
            .node("A5", (0.0, 2.0, 0.0).into(), track::NodeType::Fork)
            .node("A6", (0.0, 1.0, 0.0).into(), track::NodeType::Fork)
            .edge_double("P2", "A6")
            .edge_double("C2", "A2")
            .edge_double("P2", "A1")
            .edge("A6", "A2")
            .edge("A2", "A1")
            .edge("A1", "A4")
            .edge("A4", "A3")
            .edge("A3", "A2")
            .edge("A3", "A5")
            .edge("A5", "A6")
            .build();

        let mut vehicle = Vechicle::new(1000);
        vehicle.inline(&(0.0, 0.0, 0.0).into(), &track_graph);

        if let State::ParkDone = vehicle.state {
        } else {
            assert!(false)
        }
    }
}

struct PingPong {
    time_stamps: HashMap<u32, chrono::DateTime<chrono::Local>>,
}

impl PingPong {
    fn new(vechicles: &HashMap<u32, Vechicle>) -> Self {
        let mut time_stamps = HashMap::new();
        let now = chrono::Local::now();
        for (id, _) in vechicles {
            time_stamps.insert(*id, now);
        }
        Self { time_stamps }
    }

    fn update_timestamp(&mut self, id: u32) {
        self.time_stamps.insert(id, chrono::Local::now());
    }

    fn offline_overtime_vechicle(&self, vechicles: &mut HashMap<u32, Vechicle>) {
        let now = chrono::Local::now();
        self.time_stamps
            .iter()
            .filter(|(_, after)| {
                let dt = now - *after;
                dt.num_seconds() > 5
            })
            .for_each(|(id, _)| {
                vechicles.get_mut(id).unwrap().offline();
            });
    }
}

pub struct VechicleManager {
    vechicles: HashMap<u32, Vechicle>,
    pingpong: PingPong,
    track_graph: TrackGraph,
}

impl VechicleManager {
    pub fn new(track_graph: TrackGraph) -> Self {
        let vechicles = HashMap::new();
        let pingpong = PingPong::new(&vechicles);
        Self {
            vechicles,
            pingpong,
            track_graph,
        }
    }

    pub fn update(&mut self, id: u32, position: &Position, battery_level: f32) -> Option<Action> {
        self.pingpong.update_timestamp(id);
        self.vechicles
            .get_mut(&id)
            .unwrap()
            .update(position, battery_level, &self.track_graph)
        // self.pingpong.offline_overtime_vechicle(&mut self.vechicles);  //TODO  run once every 5 seconds
    }

    pub fn tasking(&mut self) {}
}
