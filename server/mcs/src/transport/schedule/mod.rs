use crate::transport::vehicle::ToolType;
use std::collections::LinkedList;

mod action_planner;
mod exec;
mod request;
mod db_client;

#[derive(Debug)]
pub enum Error {
    VehicleBusy,
    NodeFind,
    PathFind,
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Task {
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
}

#[derive(Debug)]
struct TaskList {
    pub trans_item_task: LinkedList<Task>,
    pub trans_fluid_task: LinkedList<Task>,
    pub use_tool_task: LinkedList<Task>,
}

impl TaskList {
    fn new() -> Self {
        Self {
            trans_item_task: LinkedList::new(),
            trans_fluid_task: LinkedList::new(),
            use_tool_task: LinkedList::new(),
        }
    }
}
