use tokio::sync::mpsc;

use crate::transport::schedule::Error;
use crate::transport::schedule::Result;
use crate::transport::schedule::Task;
use crate::transport::vehicle::ToolType;

#[derive(Debug)]
pub struct ScheduleRequest {
    sender: mpsc::Sender<Task>,
}

impl ScheduleRequest {
    pub fn new(sender: mpsc::Sender<Task>) -> Self {
        Self { sender }
    }

    pub async fn trans_items(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Result<()> {
        self.sender
            .send(Task::TransItem {
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
            .send(Task::TransFluid {
                begin_node_name: from.into(),
                end_node_name: to.into(),
            })
            .await
            .map_err(|_| Error::VehicleBusy)?;
        Ok(())
    }

    pub async fn use_tool(&mut self, pos: impl Into<String>, tool_type: ToolType) -> Result<()> {
        self.sender
            .send(Task::UseTool {
                end_node_name: pos.into(),
                tool_type,
            })
            .await
            .map_err(|_| Error::VehicleBusy)?;
        Ok(())
    }
}
