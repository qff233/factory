use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::time;

use crate::mcs::transport::dispatch;

pub struct PingPong {
    sender: mpsc::Sender<dispatch::Event>,
    time_stamps: HashMap<u32, chrono::DateTime<chrono::Local>>,
    timeout: i64,
}

impl PingPong {
    pub fn new(sender: mpsc::Sender<dispatch::Event>, timeout: i64) -> Self {
        let time_stamps = HashMap::new();
        Self {
            sender,
            time_stamps,
            timeout,
        }
    }

    pub fn update(&mut self, id: u32) {
        self.time_stamps.insert(id, chrono::Local::now());
    }

    fn overtime_id(&self) -> Vec<u32> {
        let mut result: Vec<u32> = Vec::new();
        let now = chrono::Local::now();
        self.time_stamps
            .iter()
            .filter(|(_, after)| (now - *after).num_seconds() > self.timeout)
            .for_each(|(id, _)| result.push(*id));
        result
    }

    pub async fn task(&self) {
        let mut interval = time::interval(time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            for id in self.overtime_id() {
                self.sender.send(dispatch::Event::Timeout(id)).await.ok();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn overtime_vehicle_id() {
        let (sender, mut receiver) = mpsc::channel(5);
        let mut ping = PingPong::new(sender, 1);

        ping.update(1000);
        tokio::spawn(async move {
            ping.task().await;
        });

        while let Some(event) = receiver.recv().await {
            match event {
                dispatch::Event::Timeout(id) => {
                    assert_eq!(id, 1000);
                    break;
                }
                _ => (),
            }
        }
    }
}
