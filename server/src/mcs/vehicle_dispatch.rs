use std::collections::HashMap;
use std::rc::Rc;

use super::{
    Position,
    track::{self, TrackGraph},
    vehicle::{Action, TransType, Vechicle},
};

struct PingPong {
    time_stamps: HashMap<u32, chrono::DateTime<chrono::Local>>,
}

impl PingPong {
    fn new(vechicles: &HashMap<u32, Vechicle>, time_out: i64) -> Self {
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

    fn offline_overtime_vechicle(&self, vechicles: &mut HashMap<u32, Vechicle>, time_out: i64) {
        let now = chrono::Local::now();
        self.time_stamps
            .iter()
            .filter(|(_, after)| {
                let dt = now - *after;
                dt.num_seconds() > time_out
            })
            .for_each(|(id, _)| {
                vechicles.get_mut(id).unwrap().offline();
            });
    }
}

pub struct VehicleDispatch {
    vechicles: HashMap<u32, Vechicle>,
    pingpong: PingPong,
    track_graph: TrackGraph,
    time_out: i64,
}

impl VehicleDispatch {
    pub fn new(track_graph: TrackGraph, time_out: i64) -> Self {
        let vechicles = HashMap::new();
        let pingpong = PingPong::new(&vechicles, time_out);
        Self {
            vechicles,
            pingpong,
            track_graph,
            time_out,
        }
    }

    pub fn timer_tick(&mut self) {
        self.pingpong
            .offline_overtime_vechicle(&mut self.vechicles, self.time_out);
    }

    pub fn get_action(
        &mut self,
        id: u32,
        position: &Position,
        battery_level: f32,
    ) -> Option<Action> {
        self.pingpong.update_timestamp(id);
        self.vechicles
            .get_mut(&id)?
            .get_action(position, battery_level, &self.track_graph)
    }

    pub fn transport(&mut self, from: Rc<track::Node>, to: Rc<track::Node>, trans_type: TransType) {
    }
}
