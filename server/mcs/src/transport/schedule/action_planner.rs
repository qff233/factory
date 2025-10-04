use std::{
    collections::{HashMap, LinkedList},
    sync::Arc,
};

use tokio::{
    sync::RwLock,
    time::{self},
};
use tracing::error;

use crate::{
    constant,
    transport::{
        schedule::{Error, Result, Task, TaskList},
        track,
        track::Graph,
        vehicle::{ActionSequence, ActionSequenceBuilder, Skill, ToolType, Vehicle},
    },
};

#[derive(Debug)]
pub struct ActionPlanner {
    vehicles: Arc<RwLock<HashMap<u32, Vehicle>>>,
    track_graph: Arc<Graph>,
    pending_tasks: Arc<RwLock<TaskList>>,
}

impl ActionPlanner {
    pub fn run(
        vehicles: Arc<RwLock<HashMap<u32, Vehicle>>>,
        track_graph: Arc<Graph>,
        pending_tasks: Arc<RwLock<TaskList>>,
    ) {
        let planner = Self {
            vehicles,
            track_graph,
            pending_tasks,
        };
        tokio::spawn(async move { planner.task().await });
    }

    async fn task(mut self) {
        let mut interval =
            time::interval(time::Duration::from_secs(constant::VEHICLE_SCHEDULE_TIME));
        loop {
            interval.tick().await;
            self.plan().await;
        }
    }

    async fn find_idle_vehicle_shortest_path_by_skill(
        &self,
        to: &str,
        skill: Skill,
    ) -> Option<(u32, track::Path)> {
        let mut result: Vec<(u32, track::Path)> = Vec::new();
        for (id, vehicle) in self.vehicles.read().await.iter() {
            if skill != *vehicle.skill() || !vehicle.idle().await {
                continue;
            }

            if let Ok(path) = self.track_graph.find_path(&vehicle.node().map_err(|e|{
                error!("vehicle({}): current node not find in idle. may be not in trackgraph or dont init. error type is {:?}.", {id}, {e});
            }).ok()?.name, to).await {
                result.push((*id, path));
            }
        }
        result.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
        result.first().cloned()
    }

    async fn trans_item_actions(
        &self,
        begin_node_name: &str,
        end_node_name: &str,
    ) -> Result<(u32, ActionSequence)> {
        let (id, to_begin_path) = self
            .find_idle_vehicle_shortest_path_by_skill(begin_node_name, Skill::Item)
            .await
            .ok_or(Error::VehicleBusy)?;
        let begin_to_end_path = self
            .track_graph
            .find_path(begin_node_name, end_node_name)
            .await
            .map_err(|_| Error::PathFind)?;

        Ok((
            id,
            ActionSequenceBuilder::new()
                .move_path(&to_begin_path)
                .suck()
                .move_path(&begin_to_end_path)
                .drop()
                .build(),
        ))
    }

    async fn trans_fluid_actions(
        &self,
        begin_node_name: &str,
        end_node_name: &str,
    ) -> Result<(u32, ActionSequence)> {
        let (id, to_begin_path) = self
            .find_idle_vehicle_shortest_path_by_skill(begin_node_name, Skill::Fluid)
            .await
            .ok_or(Error::VehicleBusy)?;

        let begin_to_end_path = self
            .track_graph
            .find_path(begin_node_name, end_node_name)
            .await
            .map_err(|_| Error::PathFind)?;

        Ok((
            id,
            ActionSequenceBuilder::new()
                .move_path(&to_begin_path)
                .drain()
                .move_path(&begin_to_end_path)
                .fill()
                .build(),
        ))
    }

    async fn use_tool_actions(
        &self,
        node_name: &str,
        tool_type: ToolType,
    ) -> Result<(u32, ActionSequence)> {
        let (id, to_end_path) = self
            .find_idle_vehicle_shortest_path_by_skill(node_name, Skill::UseTool(tool_type))
            .await
            .ok_or(Error::VehicleBusy)?;

        Ok((
            id,
            ActionSequenceBuilder::new()
                .move_path(&to_end_path)
                .use_tool()
                .build(),
        ))
    }

    async fn plan_for_vehicle(&self, task: &Task) -> Result<()> {
        match task {
            Task::TransItem {
                begin_node_name,
                end_node_name,
            } => {
                let (id, actions) = self
                    .trans_item_actions(begin_node_name, end_node_name)
                    .await?;

                self.vehicles
                    .write()
                    .await
                    .get_mut(&id)
                    .ok_or(Error::VehicleBusy)?
                    .processing(actions)
                    .await
                    .map_err(|_| Error::VehicleBusy)?
            }
            Task::TransFluid {
                begin_node_name,
                end_node_name,
            } => {
                let (id, actions) = self
                    .trans_fluid_actions(begin_node_name, end_node_name)
                    .await?;

                self.vehicles
                    .write()
                    .await
                    .get_mut(&id)
                    .ok_or(Error::VehicleBusy)?
                    .processing(actions)
                    .await
                    .map_err(|_| Error::VehicleBusy)?
            }
            Task::UseTool {
                end_node_name,
                tool_type,
            } => {
                let (id, actions) = self
                    .use_tool_actions(end_node_name, tool_type.clone())
                    .await?;

                self.vehicles
                    .write()
                    .await
                    .get_mut(&id)
                    .ok_or(Error::VehicleBusy)?
                    .processing(actions)
                    .await
                    .map_err(|_| Error::VehicleBusy)?
            }
        }

        Ok(())
    }

    async fn plan_from_tasks(&self, tasks: &mut LinkedList<Task>) {
        while let Some(task) = tasks.front() {
            match self.plan_for_vehicle(task).await {
                Ok(_) => {
                    tasks.pop_front();
                }
                Err(_) => break,
            }
        }
    }

    async fn plan(&mut self) {
        let mut tasks = self.pending_tasks.write().await;
        self.plan_from_tasks(&mut tasks.trans_item_task).await;
        self.plan_from_tasks(&mut tasks.trans_fluid_task).await;
        self.plan_from_tasks(&mut tasks.use_tool_task).await;
    }
}
