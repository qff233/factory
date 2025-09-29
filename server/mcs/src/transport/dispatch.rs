use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{error, warn};

use crate::transport::vehicle::ToolType;
use crate::{
    transport::prelude::*,
    transport::track::{self, TrackGraph},
    transport::vehicle::{Action, ActionSequenceBuilder, Vehicle},
};

#[derive(Debug)]
pub enum Error {
    VehicleBusy,
    NodeFind,
    PathFind,
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Event {
    TransItem {
        begin_node_name: String,
        end_node_name: String,
    },
    TransFluid {
        begin_node_name: String,
        end_node_name: String,
    },
    UseTool {
        end_node_name: String,
        tool_type: ToolType,
    },
    Timeout(u32),
}

impl Event {
    fn contain_id(&self, id: &u32) -> bool {
        match self {
            Event::TransItem { .. } => (2000..4000).contains(id),
            Event::TransFluid { .. } => (4000..6000).contains(id),
            Event::UseTool { tool_type, .. } => match tool_type {
                ToolType::Wrench => (000..100).contains(id),
                ToolType::Solder => (100..200).contains(id),
                ToolType::Crowbar => (200..300).contains(id),
                ToolType::Screwdriver => (300..400).contains(id),
                ToolType::WireNipper => (400..500).contains(id),
                ToolType::SoftHammer => (500..600).contains(id),
            },
            Event::Timeout(_id) => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct DispatchRequest {
    sender: mpsc::Sender<Event>,
}

impl DispatchRequest {
    pub fn new(sender: mpsc::Sender<Event>) -> Self {
        Self { sender }
    }

    pub async fn trans_items(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Result<()> {
        self.sender
            .send(Event::TransItem {
                begin_node_name: from.into(),
                end_node_name: to.into(),
            })
            .await
            .map_err(|_| Error::VehicleBusy)?;
        Ok(())
    }

    pub async fn trans_fluid(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Result<()> {
        self.sender
            .send(Event::TransFluid {
                begin_node_name: from.into(),
                end_node_name: to.into(),
            })
            .await
            .map_err(|_| Error::VehicleBusy)?;
        Ok(())
    }

    pub async fn use_tool(&mut self, pos: impl Into<String>, tool_type: ToolType) -> Result<()> {
        self.sender
            .send(Event::UseTool {
                end_node_name: pos.into(),
                tool_type,
            })
            .await
            .map_err(|_| Error::VehicleBusy)?;
        Ok(())
    }
}

struct ActionPlanner {
    vehicles: Arc<RwLock<HashMap<u32, Vehicle>>>,
    track_graph: Arc<TrackGraph>,
}

impl ActionPlanner {
    async fn find_idle_vehicle_shortest_path(
        &self,
        to: &str,
        exec: &Event,
    ) -> Option<(u32, track::Path)> {
        let mut result: Vec<(u32, track::Path)> = Vec::new();
        for (id, vehicle) in self
            .vehicles
            .read()
            .await
            .iter()
            .filter(|(id, vehicle)| exec.contain_id(id) && vehicle.idle())
        {
            let path = self
                .track_graph
                .find_path(vehicle.node()?.name(), to)
                .await
                .ok()?;
            result.push((*id, path));
        }
        result.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
        result.first().cloned()
    }

    fn node_side(&self, node_name: &String) -> Result<Side> {
        Ok(self
            .track_graph
            .node(node_name)
            .ok_or(Error::NodeFind)?
            .side()
            .ok_or(Error::NodeFind)?
            .clone())
    }

    async fn plan(&mut self, exec: &Event) -> Result<()> {
        match exec {
            Event::TransItem {
                begin_node_name,
                end_node_name,
            } => {
                let (id, to_begin_path) = self
                    .find_idle_vehicle_shortest_path(begin_node_name, exec)
                    .await
                    .ok_or(Error::VehicleBusy)?;
                let begin_side = self.node_side(begin_node_name)?;
                let end_side = self.node_side(end_node_name)?;
                let begin_to_end_path = self
                    .track_graph
                    .find_path(begin_node_name, end_node_name)
                    .await
                    .map_err(|_| Error::PathFind)?;

                let actions = ActionSequenceBuilder::new()
                    .move_path(&to_begin_path)
                    .suck(&begin_side)
                    .move_path(&begin_to_end_path)
                    .drop(&end_side)
                    .build();

                self.vehicles
                    .write()
                    .await
                    .get_mut(&id)
                    .ok_or(Error::VehicleBusy)?
                    .processing(actions, &self.track_graph)
                    .await
                    .map_err(|_| Error::VehicleBusy)?
            }
            Event::TransFluid {
                begin_node_name,
                end_node_name,
            } => {
                let (id, to_begin_path) = self
                    .find_idle_vehicle_shortest_path(begin_node_name, exec)
                    .await
                    .ok_or(Error::VehicleBusy)?;
                let begin_side = self.node_side(begin_node_name)?;
                let end_side = self.node_side(end_node_name)?;
                let begin_to_end_path = self
                    .track_graph
                    .find_path(begin_node_name, end_node_name)
                    .await
                    .map_err(|_| Error::PathFind)?;

                let actions = ActionSequenceBuilder::new()
                    .move_path(&to_begin_path)
                    .drain(&begin_side)
                    .move_path(&begin_to_end_path)
                    .fill(&end_side)
                    .build();

                self.vehicles
                    .write()
                    .await
                    .get_mut(&id)
                    .ok_or(Error::VehicleBusy)?
                    .processing(actions, &self.track_graph)
                    .await
                    .map_err(|_| Error::VehicleBusy)?
            }
            Event::UseTool { end_node_name, .. } => {
                let (id, to_end_path) = self
                    .find_idle_vehicle_shortest_path(end_node_name, exec)
                    .await
                    .ok_or(Error::VehicleBusy)?;
                let end_side = self.node_side(end_node_name)?;

                let actions = ActionSequenceBuilder::new()
                    .move_path(&to_end_path)
                    .use_tool(&end_side)
                    .build();

                self.vehicles
                    .write()
                    .await
                    .get_mut(&id)
                    .ok_or(Error::VehicleBusy)?
                    .processing(actions, &self.track_graph)
                    .await
                    .map_err(|_| Error::VehicleBusy)?
            }
            Event::Timeout(id) => {
                if let Some(vehicle) = self.vehicles.write().await.get_mut(&id) {
                    vehicle.offline();
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DispatchExec {
    tool_warn_level: f32,
    track_graph: Arc<TrackGraph>,
    vehicles: Arc<RwLock<HashMap<u32, Vehicle>>>,
}

impl DispatchExec {
    pub async fn new(
        mut receiver: mpsc::Receiver<Event>,
        tool_warn_level: f32,
        track_graph: TrackGraph,
    ) -> Self {
        let vehicles = Arc::new(RwLock::new(HashMap::new()));
        let track_graph = Arc::new(track_graph);

        let mut action_panner = ActionPlanner {
            vehicles: vehicles.clone(),
            track_graph: track_graph.clone(),
        };
        tokio::spawn(async move {
            loop {
                while let Some(exec) = receiver.recv().await {
                    if let Err(e) = action_panner.plan(&exec).await {
                        warn!(
                            "trans suffer {:?}\n{:#?}",
                            e,
                            action_panner.vehicles.read().await
                        );
                    }
                }
            }
        })
        .await
        .unwrap();

        Self {
            tool_warn_level,
            track_graph,
            vehicles,
        }
    }

    pub async fn get_action(
        &mut self,
        id: u32,
        position: impl Into<Position>,
        battery_level: f32,
        tool_level: Option<f32>,
    ) -> Option<Action> {
        if let Some(tool_level) = tool_level
            && tool_level < self.tool_warn_level
        {
            warn!("{} suffer low tool level", id);
        }
        let position = &position.into();
        let mut vehicles = self.vehicles.write().await;
        match vehicles.get_mut(&id) {
            Some(vehicle) => {
                vehicle
                    .get_action(position, battery_level, &self.track_graph)
                    .await
            }
            None => {
                let mut vehicle = Vehicle::new(id);
                if let Err(e) = vehicle.initing(position, &self.track_graph).await {
                    assert!(false, "{} suffer {:#?}", id, e);
                    error!("{} suffer {:#?}", id, e);
                }
                let action = vehicle
                    .get_action(position, battery_level, &self.track_graph)
                    .await;
                vehicles.insert(id, vehicle);
                action
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::time;

    use super::*;
    use crate::transport::track::{NodeType, TrackGraphBuilder};

    #[test]
    fn vehicle_type_contain_id() {
        // match self {
        //     VehicleType::Item => (2000..4000).contains(id),
        //     VehicleType::Fluid => (4000..6000).contains(id),
        //     VehicleType::Trolley => (6000..8000).contains(id),
        //     VehicleType::Tool(tool_type) => match tool_type {
        //         ToolType::Wrench => (000..100).contains(id),
        //         ToolType::Solder => (100..200).contains(id),
        //         ToolType::Crowbar => (200..300).contains(id),
        //         ToolType::Screwdriver => (300..400).contains(id),
        //         ToolType::WireNipper => (400..500).contains(id),
        //         ToolType::SoftHammer => (500..600).contains(id),
        //     },
        // }
        assert!(
            Event::TransItem {
                begin_node_name: "".to_string(),
                end_node_name: "".to_string()
            }
            .contain_id(&2050)
        );
        assert!(
            Event::TransFluid {
                begin_node_name: "".to_string(),
                end_node_name: "".to_string()
            }
            .contain_id(&4050)
        );

        let tool_type = ToolType::Wrench;
        assert!(
            Event::UseTool {
                tool_type,
                end_node_name: "".to_string()
            }
            .contain_id(&50)
        );

        let tool_type = ToolType::Solder;
        assert!(
            Event::UseTool {
                tool_type,
                end_node_name: "".to_string()
            }
            .contain_id(&150)
        );

        let tool_type = ToolType::Crowbar;
        assert!(
            Event::UseTool {
                tool_type,
                end_node_name: "".to_string()
            }
            .contain_id(&250)
        );

        let tool_type = ToolType::Screwdriver;
        assert!(
            Event::UseTool {
                tool_type,
                end_node_name: "".to_string()
            }
            .contain_id(&350)
        );

        let tool_type = ToolType::WireNipper;
        assert!(
            Event::UseTool {
                tool_type,
                end_node_name: "".to_string()
            }
            .contain_id(&450)
        );

        let tool_type = ToolType::SoftHammer;
        assert!(
            Event::UseTool {
                tool_type,
                end_node_name: "".to_string()
            }
            .contain_id(&550)
        );
    }

    #[tokio::test]
    async fn channel() {
        let (sender, mut receiver) = mpsc::channel(200);

        sender
            .send(Event::TransItem {
                begin_node_name: "S2".to_string(),
                end_node_name: "S1".to_string(),
            })
            .await
            .unwrap();
        time::sleep(time::Duration::from_secs(3)).await;

        tokio::spawn(async move {
            loop {
                while let Some(exec) = receiver.recv().await {
                    println!("{:#?}", exec);
                }
            }
        })
        .await
        .unwrap();

        assert!(false);
    }

    #[tokio::test]
    async fn dispatch() {
        let track_graph = TrackGraphBuilder::new()
            .node("P2", (0.0, 0.0, 0.0), NodeType::ParkingStation)
            .node("C1", (1.0, 0.0, 0.0), NodeType::ChargingStation)
            .node("P1", (2.0, 0.0, 0.0), NodeType::ParkingStation)
            .node("S3", (-1.0, 0.0, 0.0), NodeType::ShippingDock(Side::PosZ))
            .node("A1", (2.0, 1.0, 0.0), NodeType::Fork)
            .node("A2", (1.0, 1.0, 0.0), NodeType::Fork)
            .node("A3", (1.0, 2.0, 0.0), NodeType::Fork)
            .node("A4", (2.0, 2.0, 0.0), NodeType::Fork)
            .node("A5", (0.0, 2.0, 0.0), NodeType::Fork)
            .node("A6", (0.0, 1.0, 0.0), NodeType::Fork)
            .node("S1", (1.0, 3.0, 0.0), NodeType::Stocker(Side::PosZ))
            .node("S2", (-1.0, 1.0, 0.0), NodeType::Stocker(Side::PosZ))
            .edge_double("P2", "A6")
            .edge_double("C1", "A2")
            .edge_double("P1", "A1")
            .edge_double("S1", "A3")
            .edge_double("S2", "A6")
            .edge_double("S3", "P2")
            .edge("A6", "A2")
            .edge("A2", "A1")
            .edge("A1", "A4")
            .edge("A4", "A3")
            .edge("A3", "A2")
            .edge("A3", "A5")
            .edge("A5", "A6")
            .build();

        let (sender, receiver) = mpsc::channel(200);
        sender
            .send(Event::TransItem {
                begin_node_name: "S2".to_string(),
                end_node_name: "S1".to_string(),
            })
            .await
            .unwrap();

        let mut dispatch = DispatchExec::new(receiver, 0.1, track_graph).await;

        // Item

        assert!(
            matches!(dispatch .get_action(2500, (0.0, 0.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "S3")
        );
        assert!(matches!(
            dispatch
                .get_action(2500, (-1.0, 0.0, 0.0), 1.0, Some(1.0))
                .await
                .unwrap(),
            Action::Drop(_)
        ));

        assert!(
            matches!(dispatch .get_action(2500, (-1.0, 0.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "P2")
        );

        //TODO ParkDone
        dispatch
            .get_action(2500, (0.0, 0.0, 0.0), 1.0, Some(1.0))
            .await;
        println!("{:#?}", dispatch.vehicles.read().await.get(&2500).unwrap());
        assert!(
            matches!(dispatch .get_action(2500, (0.0, 0.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A6")
        );
        assert!(
            matches!(dispatch.get_action(2500, (0.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "S2")
        );
        assert!(
            matches!(dispatch.get_action(2500, (-1.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Suck(side) if side == Side::PosZ)
        );
        assert!(
            matches!(dispatch.get_action(2500, (-1.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A6")
        );
        assert!(
            matches!(dispatch.get_action(2500, (0.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A2")
        );
        assert!(
            matches!(dispatch.get_action(2500, (1.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A1")
        );
        assert!(
            matches!(dispatch.get_action(2500, (2.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A4")
        );
        assert!(
            matches!(dispatch.get_action(2500, (2.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A3")
        );
        assert!(
            matches!(dispatch.get_action(2500, (1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "S1")
        );
        assert!(
            matches!(dispatch.get_action(2500, (1.0, 3.0, 0.0), 1.0, Some(1.0)).await.unwrap(), Action::Drop(side) if side == Side::PosZ)
        );
        assert!(
            matches!(dispatch.get_action(2500, (1.0, 3.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A3")
        );
        assert!(
            matches!(dispatch.get_action(2500, (1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A2")
        );
        assert!(
            matches!(dispatch.get_action(2500, (1.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A1")
        );
        assert!(
            matches!(dispatch.get_action(2500, (2.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "P1")
        );
        assert!(
            dispatch
                .get_action(2500, (2.0, 0.0, 0.0), 1.0, Some(1.0))
                .await
                .is_none()
        );
        assert_eq!(dispatch.track_graph.get_lock_node().await.len(), 1);

        // Trolly
        let action = dispatch
            .get_action(5500, (2.0, 1.0, 0.0), 1.0, Some(1.0))
            .await;
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A4"));
        sender
            .send(Event::TransFluid {
                begin_node_name: "S1".to_string(),
                end_node_name: "S2".to_string(),
            })
            .await
            .unwrap();

        println!("{:#?}", dispatch.vehicles.read().await.get(&6500).unwrap());

        assert!(
            matches!(dispatch.get_action(6500, (2.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A4")
        );
        assert!(
            matches!(dispatch.get_action(6500, (2.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A3")
        );
        assert!(
            matches!(dispatch .get_action(6500, (1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "S1")
        );
        assert!(
            matches!(dispatch .get_action(6500, (1.0, 3.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Use(side) if side == Side::PosZ)
        );
        assert!(
            matches!(dispatch .get_action(6500, (1.0, 3.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A3")
        );
        assert!(
            matches!(dispatch .get_action(6500, (1.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A5")
        );
        assert!(
            matches!(dispatch .get_action(6500, (0.0, 2.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A6")
        );
        assert!(
            matches!(dispatch .get_action(6500, (0.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "S2")
        );
        assert!(
            matches!(dispatch .get_action(6500, (-1.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Use(side) if side == Side::PosZ)
        );

        assert_eq!(dispatch.track_graph.get_lock_node().await.len(), 1);
        assert!(
            matches!(dispatch .get_action(6500, (-1.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "A6")
        );
        assert_eq!(dispatch.track_graph.get_lock_node().await.len(), 2);
        assert!(
            matches!(dispatch .get_action(6500, (0.0, 1.0, 0.0), 1.0, Some(1.0))
            .await.unwrap(), Action::Move(node) if node.name() == "P2")
        );
        assert!(
            dispatch
                .get_action(6500, (0.0, 0.0, 0.0), 1.0, Some(1.0))
                .await
                .is_none()
        );
        assert_eq!(dispatch.track_graph.get_lock_node().await.len(), 2);
    }
}
