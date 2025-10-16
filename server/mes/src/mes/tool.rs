use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

struct ProductId(String);

impl From<String> for ProductId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

enum State {
    StandBy,
    Processing {
        product_id: ProductId,
        recipe_name: String,
        count: u32,
    },
    Offline,
}

#[derive(PartialEq, Eq, Hash)]
struct Id(String);

struct Tool {
    pub id: Id,
    pub state: RwLock<State>,
    pub recipe_names: Vec<String>,
}

struct ToolManager {
    tools: HashMap<Id, Arc<Tool>>,
}

impl ToolManager {
    fn new() -> Self {
        todo!()
    }
}
