use tokio::sync::RwLock;

use crate::transport::prelude::*;
use core::panic;
use std::{
    collections::{BinaryHeap, HashMap, HashSet, LinkedList},
    fs::File,
    sync::Arc,
};

#[derive(Debug)]
pub enum Error {
    Node,
    AnyPath,
    AnyNode,
}

pub type Result<T> = std::result::Result<T, Error>;

struct SortNode<T>(f64, T);
impl<T> Eq for SortNode<T> {}
impl<T> Ord for SortNode<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.0 == other.0 {
            std::cmp::Ordering::Equal
        } else if self.0 < other.0 {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    }
}

impl<T> PartialOrd for SortNode<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for SortNode<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

pub struct PriorityQueue<T> {
    container: BinaryHeap<SortNode<T>>,
}

impl<T> PriorityQueue<T> {
    pub fn new() -> Self {
        Self {
            container: BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, priority: f64, item: T) {
        self.container.push(SortNode(priority, item));
    }

    pub fn pop(&mut self) -> Option<T> {
        self.container.pop().map(|node| node.1)
    }

    pub fn is_empty(&self) -> bool {
        self.container.is_empty()
    }
}

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Fork,
    ChargingStation,
    ParkingStation,
    ShippingDock(Side),
    Stocker(Side),
}

impl From<Vec<&str>> for NodeType {
    fn from(value: Vec<&str>) -> Self {
        match *value.first().expect("can not get node_type") {
            "stocker" => {
                let side = *value.get(1).expect("can not get machine side");
                Self::Stocker(side.into())
            }
            _ => panic!("no such node_type, {:?}", value),
        }
    }
}

#[derive(Debug)]
pub struct Node {
    name: String,
    node_type: NodeType,
    position: Position,
}

impl Node {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn side(&self) -> Option<&Side> {
        match &self.node_type {
            NodeType::Stocker(side) => Some(side),
            NodeType::ShippingDock(side) => Some(side),
            _ => None,
        }
    }
}

pub type Path = Vec<Arc<Node>>;

#[derive(Debug)]
enum EdgeState {
    Lock,
    UnLock,
}

#[derive(Debug)]
struct Edge {
    from_node: Arc<Node>,
    to_node: Arc<Node>,
    weight: f64,
    state: RwLock<EdgeState>,
}

#[derive(Debug)]
pub struct TrackGraph {
    edges: HashMap<String, Vec<Arc<Edge>>>,
    nodes: HashMap<String, Arc<Node>>,
}

fn heuristic_distance(from_position: &Position, to_position: &Position) -> f64 {
    let (x1, y1, z1) = from_position.into();
    let (x2, y2, z2) = to_position.into();

    // Euclidean Distance
    // let dx = x1 - x2;
    // let dy = y1 - y2;
    // let dz = z1 - z2;
    // f64::sqrt(dx * dx + dy * dy + dz * dz)

    // Manhattan Distance
    (x1 - x2).abs() + (y1 - y2).abs() + (z1 - z2).abs()
}

impl TrackGraph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    fn add_node(&mut self, node: Node) {
        let node_name = node.name.clone();
        if self
            .nodes
            .insert(node_name.clone(), Arc::new(node))
            .is_some()
        {
            panic!("{} Node is forbidden", node_name);
        }
        self.edges.insert(node_name, Vec::new());
    }

    fn add_edge(&mut self, edge: Edge) {
        let from_node_name = &edge.from_node.name.clone();

        let edge = Arc::new(edge);
        self.edges
            .get_mut(from_node_name)
            .unwrap()
            .push(edge.clone());
    }

    pub fn node(&self, name: &String) -> Option<Arc<Node>> {
        self.nodes.get(name).cloned()
    }

    pub async fn lock_node(&self, node_name: &str) {
        for edge in self
            .edges
            .iter()
            .flat_map(|(_, edges)| edges.iter())
            .filter(|edge| edge.to_node.name == node_name)
        {
            *edge.state.write().await = EdgeState::Lock;
        }
    }

    pub async fn unlock_node(&self, node_name: &str) {
        for edge in self
            .edges
            .iter()
            .flat_map(|(_, edges)| edges.iter())
            .filter(|edge| edge.to_node.name == node_name)
        {
            *edge.state.write().await = EdgeState::UnLock;
        }
    }

    pub async fn get_lock_node(&self) -> HashSet<String> {
        let mut result: HashSet<String> = HashSet::new();
        for edge in self.edges.iter().flat_map(|(_, edges)| edges.iter()) {
            if let EdgeState::Lock = *edge.state.read().await {
                result.insert(edge.to_node.name.to_string());
            }
        }
        result
    }

    async fn a_star(&self, begin_node_name: &str, end_node_name: &str) -> Result<Path> {
        let mut open_node: PriorityQueue<Arc<Node>> = PriorityQueue::new();
        let mut close_node: HashSet<String> = HashSet::new();
        let mut came_from: HashMap<String, Arc<Node>> = HashMap::new();
        let mut g_score: HashMap<String, f64> = HashMap::new();
        for (name, _) in self.nodes.iter() {
            g_score.insert(name.clone(), f64::MAX);
        }
        g_score.insert(begin_node_name.to_string(), 0.0);

        let begin_node = self.nodes.get(begin_node_name).ok_or(Error::Node)?;
        let end_node = self.nodes.get(end_node_name).ok_or(Error::Node)?;

        open_node.push(
            heuristic_distance(begin_node.position(), end_node.position()),
            begin_node.clone(),
        );

        while let Some(current_node) = open_node.pop() {
            if current_node.name == end_node_name {
                let mut result: LinkedList<Arc<Node>> = LinkedList::new();
                result.push_back(current_node.clone());

                let mut current_node_name = current_node.name.clone();
                while let Some(prev_node) = came_from.get(current_node_name.as_str()) {
                    current_node_name = prev_node.name.clone();
                    result.push_front(prev_node.clone());
                }
                return Ok(result.into_iter().collect());
            }

            close_node.insert(current_node.name.clone());

            let edges = self.edges.get(current_node.name.as_str()).unwrap();
            for edge in edges {
                if let EdgeState::Lock = *edge.state.read().await {
                    continue;
                }

                let to_node = edge.to_node.clone();
                if close_node.contains(to_node.name.as_str()) {
                    continue;
                }

                let tentative_g_score =
                    g_score.get(current_node.name.as_str()).unwrap() + edge.weight;
                if tentative_g_score >= *g_score.get(to_node.name.as_str()).unwrap() {
                    continue;
                }

                came_from.insert(to_node.name.clone(), current_node.clone());
                g_score.insert(to_node.name.clone(), tentative_g_score);
                let f_score = *g_score.get(&to_node.name).unwrap()
                    + heuristic_distance(current_node.position(), end_node.position());
                open_node.push(f_score, to_node.clone());
            }
        }

        Err(Error::AnyPath)
    }

    async fn dijkstra(&self, begin_node_name: &str, node_type: &NodeType) -> Result<Path> {
        let mut open_node: PriorityQueue<Arc<Node>> = PriorityQueue::new();
        let mut close_node: HashSet<String> = HashSet::new();
        let mut came_from: HashMap<String, Arc<Node>> = HashMap::new();
        let mut g_score: HashMap<String, f64> = HashMap::new();
        for (name, _) in self.nodes.iter() {
            g_score.insert(name.clone(), f64::MAX);
        }
        g_score.insert(begin_node_name.to_string(), 0.0);

        let begin_node = self.nodes.get(begin_node_name).ok_or(Error::Node)?;

        open_node.push(0.0, begin_node.clone());

        while let Some(current_node) = open_node.pop() {
            if std::mem::discriminant(&current_node.node_type) == std::mem::discriminant(node_type) {
            // if current_node.node_type == *node_type {
                let mut result: LinkedList<Arc<Node>> = LinkedList::new();
                result.push_back(current_node.clone());

                let mut current_node_name = current_node.name.clone();
                while let Some(prev_node) = came_from.get(current_node_name.as_str()) {
                    current_node_name = prev_node.name.clone();
                    result.push_front(prev_node.clone());
                }
                return Ok(result.into_iter().collect());
            }

            close_node.insert(current_node.name.clone());

            for edge in self.edges.get(current_node.name()).unwrap() {
                if let EdgeState::Lock = *edge.state.read().await {
                    continue;
                }

                let to_node = edge.to_node.clone();
                if close_node.contains(to_node.name.as_str()) {
                    continue;
                }

                let tentative_g_score =
                    g_score.get(current_node.name.as_str()).unwrap() + edge.weight;
                if tentative_g_score >= *g_score.get(to_node.name.as_str()).unwrap() {
                    continue;
                }

                came_from.insert(to_node.name.clone(), current_node.clone());
                g_score.insert(to_node.name.clone(), tentative_g_score);
                open_node.push(tentative_g_score, to_node.clone());
            }
        }

        Err(Error::AnyPath)
    }

    pub async fn find_path(&self, begin_node_name: &str, end_node_name: &str) -> Result<Path> {
        self.a_star(begin_node_name, end_node_name).await
    }

    pub fn find_shortest_node(&self, position: &Position) -> Result<Arc<Node>> {
        let mut nodes: Vec<(Arc<Node>, f64)> = Vec::with_capacity(self.nodes.len());
        self.nodes.iter().for_each(|(_, to_node)| {
            nodes.push((
                to_node.clone(),
                heuristic_distance(position, to_node.position()),
            ));
        });
        nodes.sort_by(|a, b| a.1.total_cmp(&b.1));
        Ok(nodes.first().ok_or(Error::AnyNode)?.0.clone())
    }

    pub async fn find_path_by_type(
        &self,
        from_node_name: &str,
        node_type: NodeType,
    ) -> Result<Path> {
        self.dijkstra(from_node_name, &node_type).await
    }

    pub async fn find_parking_path(&self, from_node_name: &str) -> Result<Path> {
        self.find_path_by_type(from_node_name, NodeType::ParkingStation)
            .await
    }

    pub async fn find_charging_path(&self, from_node_name: &str) -> Result<Path> {
        self.find_path_by_type(from_node_name, NodeType::ChargingStation)
            .await
    }

    pub async fn find_shipping_dock_path(&self, from_node_name: &str) -> Result<Path> {
        self.find_path_by_type(from_node_name, NodeType::ShippingDock(Side::NegX))
            .await
    }
}

pub(crate) struct TrackGraphBuilder {
    track_graph: TrackGraph,
}

impl TrackGraphBuilder {
    pub(crate) fn new() -> TrackGraphBuilder {
        Self {
            track_graph: TrackGraph::new(),
        }
    }

    pub(crate) fn from_json(file_path: &str) -> Self {
        let file = File::open(file_path).expect("can not open oht_track json file");
        let json: serde_json::Value = serde_json::from_reader(file).expect("can not parse json");

        let mut track_graph = TrackGraph::new();

        json.get("nodes")
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .for_each(|(name, value)| {
                let value = value.as_str().unwrap().to_string();
                let mut value_split: Vec<&str> = value.split(' ').collect();
                let x: f64 = value_split
                    .first()
                    .expect("can not get X string")
                    .parse()
                    .expect("can not parse X");
                let y: f64 = value_split
                    .get(1)
                    .expect("can not get Y string")
                    .parse()
                    .expect("can not parse Y");
                let z: f64 = value_split
                    .get(2)
                    .expect("can not get Z string")
                    .parse()
                    .expect("can not parse Z");
                let node_type = value_split.split_off(3);
                let node_type = node_type.into();
                track_graph.add_node(Node {
                    name: name.clone(),
                    position: (x, y, z).into(),
                    node_type,
                });
            });
        json.get("edges")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .for_each(|value| {
                let value = value.as_str().unwrap().to_string();
                let value_split: Vec<&str> = value.split('-').collect();
                let from_node_name = value_split.first().expect("can not get from_node_name");
                let to_node_name = value_split.get(1).expect("can not get to_node_name");

                let from_node = track_graph.nodes.get(*from_node_name).unwrap();
                let to_node = track_graph.nodes.get(*to_node_name).unwrap();
                let weight = heuristic_distance(&from_node.position, &to_node.position);

                let edge = Edge {
                    from_node: track_graph.nodes.get(*from_node_name).unwrap().clone(),
                    to_node: track_graph.nodes.get(*to_node_name).unwrap().clone(),
                    weight,
                    state: RwLock::new(EdgeState::UnLock),
                };
                track_graph.add_edge(edge);
            });

        Self { track_graph }
    }

    pub fn node(
        mut self,
        name: &str,
        position: impl Into<Position>,
        node_type: NodeType,
    ) -> TrackGraphBuilder {
        let node = Node {
            name: name.to_string(),
            position: position.into(),
            node_type,
        };
        self.track_graph.add_node(node);
        self
    }

    pub fn edge(mut self, from_node_name: &str, to_node_name: &str) -> TrackGraphBuilder {
        let from_node = self
            .track_graph
            .nodes
            .get(from_node_name)
            .unwrap_or_else(|| panic!("No such Node {}, Please check!", from_node_name));
        let to_node = self
            .track_graph
            .nodes
            .get(to_node_name)
            .unwrap_or_else(|| panic!("No such Node {}, Please check!", from_node_name));
        let weight = heuristic_distance(&from_node.position, &to_node.position);

        let edge = Edge {
            from_node: from_node.clone(),
            to_node: to_node.clone(),
            weight,
            state: RwLock::new(EdgeState::UnLock),
        };

        self.track_graph.add_edge(edge);
        self
    }

    pub fn edge_double(mut self, node1: &str, node2: &str) -> TrackGraphBuilder {
        self = self.edge(node1, node2);
        self.edge(node2, node1)
    }

    pub fn build(self) -> TrackGraph {
        self.track_graph
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_grack_graph() -> TrackGraph {
        TrackGraphBuilder::new()
            .node("P2", (0.0, 0.0, 0.0), NodeType::ParkingStation)
            .node("C1", (1.0, 0.0, 0.0), NodeType::ChargingStation)
            .node("P1", (2.0, 0.0, 0.0), NodeType::ParkingStation)
            .node("A1", (2.0, 1.0, 0.0), NodeType::Fork)
            .node("A2", (1.0, 1.0, 0.0), NodeType::Fork)
            .node("A3", (1.0, 2.0, 0.0), NodeType::Fork)
            .node("A4", (2.0, 2.0, 0.0), NodeType::Fork)
            .node("A5", (0.0, 2.0, 0.0), NodeType::Fork)
            .node("A6", (0.0, 1.0, 0.0), NodeType::Fork)
            .edge_double("P2", "A6")
            .edge_double("C1", "A2")
            .edge_double("P1", "A1")
            .edge("A6", "A2")
            .edge("A2", "A1")
            .edge("A1", "A4")
            .edge("A4", "A3")
            .edge("A3", "A2")
            .edge("A3", "A5")
            .edge("A5", "A6")
            .build()
    }

    #[test]
    fn add_node_edge() {
        let track_graph = TrackGraphBuilder::new()
            .node("A", (0.0, 1.0, 2.0), NodeType::Stocker(Side::NegY))
            .node("B", (2.0, 4.0, 6.0), NodeType::Stocker(Side::NegY))
            .edge("A", "B")
            .build();

        let node_b = track_graph.nodes.get("B").unwrap();
        let (x, y, z) = node_b.position().into();
        assert_eq!(node_b.name, "B");
        assert_eq!(x, 2.0);
        assert_eq!(y, 4.0);
        assert_eq!(z, 6.0);
        assert_eq!(node_b.node_type, NodeType::Stocker(Side::NegY));

        let edge = track_graph.edges.get("A").unwrap().get(0).unwrap();
        assert_eq!(edge.from_node.name, "A");
        assert_eq!(edge.to_node.name, "B");
    }

    #[tokio::test]
    async fn a_star() {
        let track_graph = get_grack_graph();
        let mut path: Vec<String> = Vec::new();
        let result = track_graph.a_star("A4", "P1").await.unwrap();
        for node in result {
            path.push(node.name.clone());
        }
        assert_eq!(path, ["A4", "A3", "A2", "A1", "P1"]);
    }

    #[tokio::test]
    async fn find_parking_charging_path() {
        let track_graph = get_grack_graph();

        let mut path: Vec<String> = Vec::new();
        let result = track_graph.find_parking_path("A4").await.unwrap();
        for node in result {
            path.push(node.name.clone());
        }
        assert_eq!(path, ["A4", "A3", "A2", "A1", "P1"]);

        let mut path: Vec<String> = Vec::new();
        let result = track_graph.find_charging_path("A4").await.unwrap();
        for node in result {
            path.push(node.name.clone());
        }
        assert_eq!(path, ["A4", "A3", "A2", "C1"]);
    }

    #[tokio::test]
    async fn load_json() {
        let track_graph = TrackGraphBuilder::from_json("./tests/oht_trackgraph.json").build();
        let mut path: Vec<String> = Vec::new();
        let result = track_graph.find_path("A", "F").await.unwrap();
        for node in result {
            path.push(node.name.clone());
        }
        assert_eq!(path, ["A", "B", "C", "F"]);
    }
}
