use std::collections::HashMap;

pub struct PingPong {
    time_stamps: HashMap<u32, chrono::DateTime<chrono::Local>>,
    timeout: i64,
}

impl PingPong {
    pub fn new(timeout: i64) -> Self {
        let time_stamps = HashMap::new();
        Self {
            time_stamps,
            timeout,
        }
    }

    pub fn update(&mut self, id: u32) {
        self.time_stamps.insert(id, chrono::Local::now());
    }

    pub fn overtime_id(&self) -> Vec<u32> {
        let mut result: Vec<u32> = Vec::new();
        let now = chrono::Local::now();
        self.time_stamps
            .iter()
            .filter(|(_, after)| (now - *after).num_seconds() > self.timeout)
            .for_each(|(id, _)| result.push(*id));
        result
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time};

    use super::*;
    #[test]
    fn overtime_vehicle_id() {
        let mut ping = PingPong::new(1);
        ping.update(1000);

        let ping = ping;
        assert_eq!(ping.overtime_id().len(), 0);

        sleep(time::Duration::from_secs(2));
        assert_eq!(ping.overtime_id(), vec![1000]);
    }
}
