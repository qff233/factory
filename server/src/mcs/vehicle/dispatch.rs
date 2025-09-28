use std::collections::{HashMap, LinkedList};
use std::rc::Rc;

use jsonrpsee::core::id_providers;
use tracing::warn;

use crate::mcs::{
    Position,
    track::{self, TrackGraph},
    vehicle::vehicle::{Action, Vehicle},
};

#[derive(Debug)]
pub enum Error {
    VehicleBusy,
    NodeError,
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum ToolType {
    Wrench,      // 扳手
    Solder,      // 烙铁
    Crowbar,     // 撬棍
    Screwdriver, // 螺丝刀
    WireNipper,  // 剪线钳
    SoftHammer,  // 软锤
}

#[derive(Debug)]
pub enum VehicleType {
    Item,
    Fluid,
    Trolley,
    Tool(ToolType),
}

impl VehicleType {
    fn contain_id(&self, id: &u32) -> bool {
        match self {
            VehicleType::Item => (2000..4000).contains(id),
            VehicleType::Fluid => (4000..6000).contains(id),
            VehicleType::Trolley => (6000..8000).contains(id),
            VehicleType::Tool(tool_type) => match tool_type {
                ToolType::Wrench => (000..100).contains(id),
                ToolType::Solder => (100..200).contains(id),
                ToolType::Crowbar => (200..300).contains(id),
                ToolType::Screwdriver => (300..400).contains(id),
                ToolType::WireNipper => (400..500).contains(id),
                ToolType::SoftHammer => (500..600).contains(id),
            },
        }
    }
}

#[derive(Debug)]
pub struct Dispatch {
    tool_warn_level: f32,
    track_graph: TrackGraph,
    vehicles: HashMap<u32, Vehicle>,
}

impl Dispatch {
    pub fn new(tool_warn_level: f32, track_graph: TrackGraph) -> Self {
        let vehicles = HashMap::new();
        Self {
            tool_warn_level,
            vehicles,
            track_graph,
        }
    }

    pub fn offline(&mut self, id: u32) {
        if let Some(vehicle) = self.vehicles.get_mut(&id) {
            vehicle.offline();
        }
    }

    pub fn get_action(
        &mut self,
        id: u32,
        position: impl Into<Position>,
        battery_level: f32,
        tool_level: Option<f32>,
    ) -> Option<Action> {
        if let Some(tool_level) = tool_level {
            if tool_level < self.tool_warn_level {
                warn!("{} suffer low tool level", id);
            }
        }
        let position = &position.into();
        match self.vehicles.get_mut(&id) {
            Some(vehicle) => vehicle.get_action(position, battery_level, &self.track_graph),
            None => {
                let mut vehicle = Vehicle::new(id);
                vehicle.inline(position, &self.track_graph);
                let action = vehicle.get_action(position, battery_level, &self.track_graph);
                self.vehicles.insert(id, vehicle);
                action
            }
        }
    }

    fn find_shortest_idle_vehicle(&self, to: &str) -> Option<u32> {
        let mut result: Vec<(u32, track::Path)> = Vec::new();
        for (id, vehicle) in self.vehicles.iter().filter(|(_, vehicle)| vehicle.idle()) {
            let path = self
                .track_graph
                .find_path(vehicle.node()?.name(), to)
                .ok()?;
            result.push((*id, path));
        }
        result.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
        result.first().map(|(id, _)| *id)
    }

    fn trans(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        vehicle_type: VehicleType,
    ) -> Result<()> {
        let from = from.into();
        let to = to.into();

        let id = self
            .find_shortest_idle_vehicle(&from)
            .ok_or(Error::VehicleBusy)?;
        let vehicle = self.vehicles.get_mut(&id).ok_or(Error::VehicleBusy)?;

        let from = self.track_graph.node(&from).ok_or(Error::NodeError);
        let to = self.track_graph.node(&to).ok_or(Error::NodeError);

        let actions: LinkedList<Action> = match vehicle_type {
            VehicleType::Item => {
                let from = from?;
                let from_side = from.side().ok_or(Error::NodeError)?;
                let to = to?;
                let to_side = to.side().ok_or(Error::NodeError)?;
                [
                    Action::Move(from.clone()),
                    Action::Suck(from_side.clone()),
                    Action::Move(to.clone()),
                    Action::Drop(to_side.clone()),
                ]
            }
            .into(),
            VehicleType::Fluid => {
                let from = from?;
                let from_side = from.side().ok_or(Error::NodeError)?;
                let to = to?;
                let to_side = to.side().ok_or(Error::NodeError)?;
                [
                    Action::Move(from.clone()),
                    Action::Drain(from_side.clone()),
                    Action::Move(to.clone()),
                    Action::Fill(to_side.clone()),
                ]
                .into()
            }
            VehicleType::Trolley => {
                let from = from?;
                let from_side = from.side().ok_or(Error::NodeError)?;
                let to = to?;
                let to_side = to.side().ok_or(Error::NodeError)?;
                [
                    Action::Move(from.clone()),
                    Action::Use(from_side.clone()),
                    Action::Move(to.clone()),
                    Action::Use(to_side.clone()),
                ]
                .into()
            }
            VehicleType::Tool(_) => {
                let from = from?;
                let from_side = from.side().ok_or(Error::NodeError)?;
                [Action::Move(from.clone()), Action::Use(from_side.clone())].into()
            }
        };
        if let Ok(()) = vehicle.processing(actions, &self.track_graph) {
            return Ok(());
        }

        Err(Error::VehicleBusy)
    }

    pub fn trans_items(&mut self, from: impl Into<String>, to: impl Into<String>) -> Result<()> {
        self.trans(from, to, VehicleType::Item)
    }

    pub fn trans_items_by_trolley(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Result<()> {
        self.trans(from, to, VehicleType::Trolley)
    }

    pub fn trans_fluid(&mut self, from: impl Into<String>, to: impl Into<String>) -> Result<()> {
        self.trans(from, to, VehicleType::Fluid)
    }

    pub fn use_tool(&mut self, pos: impl Into<String>, tool_type: ToolType) -> Result<()> {
        self.trans("P1", pos, VehicleType::Tool(tool_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcs::{
        prelude::Side,
        track::{NodeType, TrackGraphBuilder},
    };

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
        assert!(VehicleType::Item.contain_id(&2050));
        assert!(VehicleType::Fluid.contain_id(&4050));
        assert!(VehicleType::Trolley.contain_id(&6050));

        let tool_type = ToolType::Wrench;
        assert!(VehicleType::Tool(tool_type).contain_id(&50));

        let tool_type = ToolType::Solder;
        assert!(VehicleType::Tool(tool_type).contain_id(&150));

        let tool_type = ToolType::Crowbar;
        assert!(VehicleType::Tool(tool_type).contain_id(&250));

        let tool_type = ToolType::Screwdriver;
        assert!(VehicleType::Tool(tool_type).contain_id(&350));

        let tool_type = ToolType::WireNipper;
        assert!(VehicleType::Tool(tool_type).contain_id(&450));

        let tool_type = ToolType::SoftHammer;
        assert!(VehicleType::Tool(tool_type).contain_id(&550));
    }

    #[test]
    fn dispatch() {
        let track_graph = TrackGraphBuilder::new()
            .node("P2", (0.0, 0.0, 0.0), NodeType::ParkingStation)
            .node("C1", (1.0, 0.0, 0.0), NodeType::ChargingStation)
            .node("P1", (2.0, 0.0, 0.0), NodeType::ParkingStation)
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
            .edge("A6", "A2")
            .edge("A2", "A1")
            .edge("A1", "A4")
            .edge("A4", "A3")
            .edge("A3", "A2")
            .edge("A3", "A5")
            .edge("A5", "A6")
            .build();

        let mut dispatch = Dispatch::new(0.1, track_graph);

        // Item
        let action = dispatch.get_action(2500, (0.0, 0.0, 0.0), 1.0, Some(1.0));
        assert!(action.is_none());
        dispatch.trans_items("S2", "S1").unwrap();

        println!("{:#?}", dispatch);

        let action = dispatch.get_action(2500, (0.0, 0.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A6"));
        let action = dispatch.get_action(2500, (0.0, 1.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "S2"));
        let action = dispatch.get_action(2500, (-1.0, 1.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Suck(side) if side == Side::PosZ));
        let action = dispatch.get_action(2500, (-1.0, 1.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A6"));
        let action = dispatch.get_action(2500, (0.0, 1.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A2"));
        let action = dispatch.get_action(2500, (1.0, 1.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A1"));
        let action = dispatch.get_action(2500, (2.0, 1.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A4"));
        let action = dispatch.get_action(2500, (2.0, 2.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A3"));
        let action = dispatch.get_action(2500, (1.0, 2.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "S1"));
        let action = dispatch.get_action(2500, (1.0, 3.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Drop(side) if side == Side::PosZ));
        let action = dispatch.get_action(2500, (1.0, 3.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A3"));
        let action = dispatch.get_action(2500, (1.0, 2.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A5"));
        let action = dispatch.get_action(2500, (0.0, 2.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A6"));
        let action = dispatch.get_action(2500, (0.0, 1.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "P2"));
        let action = dispatch.get_action(2500, (0.0, 0.0, 0.0), 1.0, Some(1.0));
        assert!(action.is_none());

        // Trolly
        let action = dispatch.get_action(6500, (2.0, 0.0, 0.0), 1.0, Some(1.0));
        assert!(action.is_none());
        dispatch.trans_items_by_trolley("S1", "S2").unwrap();
        let action = dispatch.get_action(6500, (2.0, 0.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A1"));
        let action = dispatch.get_action(6500, (2.0, 1.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A4"));
        let action = dispatch.get_action(6500, (2.0, 2.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A3"));
        let action = dispatch.get_action(6500, (1.0, 2.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "S1"));
        let action = dispatch.get_action(6500, (1.0, 3.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Use(side) if side == Side::PosZ));
        let action = dispatch.get_action(6500, (1.0, 3.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A3"));
        let action = dispatch.get_action(6500, (1.0, 2.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A2"));
        let action = dispatch.get_action(6500, (1.0, 1.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "A1"));
        let action = dispatch.get_action(6500, (2.0, 1.0, 0.0), 1.0, Some(1.0));
        assert!(matches!(action.unwrap(), Action::Move(node) if node.name() == "P1"));
        let action = dispatch.get_action(6500, (2.0, 0.0, 0.0), 1.0, Some(1.0));
        assert!(action.is_none());
    }
}
