use std::cell::RefCell;
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
    Move(Rc<RefCell<track::Node>>),
    Use(Side),
}

#[derive(Debug)]
enum ProcessError {
    StateError,
}

#[derive(Debug)]
enum State {
    Offline,
    Idle,
    Processing(LinkedList<Action>),
    Parking(LinkedList<Action>),
    Charging,
}

#[derive(Debug)]
struct Vechicle {
    id: u32,
    state: State,
    landmark: Option<Rc<RefCell<track::Node>>>,
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
            self.state = State::Idle;
            if self.landmark.is_none() {
                match track_graph.find_shortest_node(current_position) {
                    Ok(node) => {
                        let mut actions = LinkedList::new();
                        actions.push_back(Action::Move(node));
                        self.process(actions)
                            .map_err(|_| error!("{} can't process actions.", self.id))
                            .ok();
                    }
                    Err(_) => error!("{} can't find shortest node.", self.id),
                }
            }
        }
    }

    fn update(
        &mut self,
        current_position: &Position,
        current_battery_level: f32,
        track_graph: &TrackGraph,
    ) -> Option<Action> {
        let mut process_actions = |actions: &mut LinkedList<Action>| -> Option<Action> {
            let action = actions.front().unwrap();
            match action {
                Action::Move(node) if current_position == node.borrow().position() => {
                    self.landmark = Some(node.clone());
                    return actions.pop_front();
                }
                Action::Use(_side) => return actions.pop_front(),
                _ => return None,
            }
        };

        match &mut self.state {
            State::Parking(actions) => {
                while !actions.is_empty() {
                    return process_actions(actions);
                }
                self.state = State::Idle;
            }
            State::Processing(actions) => {
                while !actions.is_empty() {
                    return process_actions(actions);
                }

                match track_graph.find_shortest_path_by_type(
                    self.landmark.clone().unwrap().borrow().name(),
                    track::NodeType::ChargingStation(false),
                ) {
                    Ok(path) => {
                        // 锁住该充电站，禁止其他载具来
                        let charging_station = path.last().unwrap();
                        charging_station.borrow_mut().lock();
                        let mut actions: LinkedList<Action> = LinkedList::new();
                        for node in path {
                            actions.push_back(Action::Move(node));
                        }
                        self.state = State::Parking(actions);
                    }
                    Err(_) => error!("{} can't not find charging station", self.id),
                }
            }
            State::Charging => {
                if current_battery_level >= 0.95 {
                    self.state = State::Idle
                }
            }
            State::Idle => (),
            State::Offline => (),
        }

        if current_battery_level <= 0.3 {
            // TODO 路径规划到充电站
            // return Action::Move()
            self.state = State::Charging;
        }

        None
    }

    fn process(&mut self, actions: LinkedList<Action>) -> Result<(), ProcessError> {
        match &self.state {
            State::Idle => {
                todo!()
            }
            State::Parking(actions) => {
                todo!()
            }
            _ => Err(ProcessError::StateError),
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
