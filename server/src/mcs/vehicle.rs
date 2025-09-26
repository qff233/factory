use std::collections::HashMap;
use std::collections::LinkedList;

use super::Position;
use super::Side;
use super::track::TrackGraph;

enum Command {
    UseToolIn(String, Side),
}

enum Action {
    Move(Position),
    Use(Side),
}

struct ActionBuilder {
    actions: LinkedList<Action>,
}

impl ActionBuilder {
    fn new() -> Self {
        Self {
            actions: LinkedList::new(),
        }
    }

    fn move_to(mut self, position: Position) -> Self {
        self.actions.push_back(Action::Move(position));
        self
    }

    fn use_tool(mut self, side: Side) -> Self {
        self.actions.push_back(Action::Use(side));
        self
    }

    fn build(self) -> LinkedList<Action> {
        self.actions
    }
}

enum ProcessError {
    StateError,
}

enum Vechicle {
    Offline,
    Idle,
    Processing(LinkedList<Action>),
    Parking,
    Charging,
}

impl Vechicle {
    fn new() -> Self {
        Self::Offline
    }

    fn offline(&mut self) {
        *self = Self::Offline;
    }

    fn update(
        &mut self,
        current_position: Position,
        current_battery_level: f32,
        track_graph: &TrackGraph,
    ) -> Option<Action> {
        match self {
            Vechicle::Parking => {
                // return Action
                todo!()
            }
            Vechicle::Processing(actions) => {
                while let Some(action) = actions.front() {
                    match action {
                        Action::Move(position) => {
                            if current_position == *position {
                                actions.pop_front();
                            }
                        }
                        Action::Use(_side) => return Some(actions.pop_front().unwrap()),
                    }
                }

                // TODO 路径规划停站点
                // reeturn Action::Move()
                *self = Self::Parking;
            }
            Vechicle::Charging => {
                if current_battery_level >= 0.95 {
                    *self = Self::Idle
                }
            }
            Vechicle::Idle => (),
            Vechicle::Offline => (),
        }

        if current_battery_level <= 0.3 {
            // TODO 路径规划到充电站
            // return Action::Move()
            *self = Self::Charging;
        }

        None
    }

    fn process(&mut self, actions: LinkedList<Action>) -> Result<(), ProcessError> {
        match self {
            Self::Idle | Self::Parking => {
                *self = Self::Processing(actions);
                Ok(())
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

    pub fn update(&mut self, id: u32, position: Position, battery_level: f32) {
        self.pingpong.update_timestamp(id);
        self.vechicles
            .get_mut(&id)
            .unwrap()
            .update(position, battery_level, &self.track_graph);
        // self.pingpong.offline_overtime_vechicle(&mut self.vechicles);  //TODO  run once every 5 seconds
    }

    pub fn tasking(&mut self) {}
}
