use super::{Position, Side, queue::PriorityQueue};
use core::panic;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, LinkedList},
    fs::File,
    rc::Rc,
};

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Fork,
    ChargingStation,
    ParkingStation,
    Stocker(Side),
    Machine(Side),
}

impl From<Vec<&str>> for NodeType {
    fn from(value: Vec<&str>) -> Self {
        match *value.get(0).expect("can not get node_type") {
            "stocker" => {
                let side = *value.get(1).expect("can not get machine side");
                Self::Stocker(side.into())
            }
            "machine" => {
                let side = *value.get(1).expect("can not get machine side");
                Self::Machine(side.into())
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
}

pub type Path = Vec<Rc<Node>>;

#[derive(Debug)]
enum EdgeState {
    Lock,
    UnLock,
}

#[derive(Debug)]
struct Edge {
    from_node: Rc<Node>,
    to_node: Rc<Node>,
    weight: f64,
    state: RefCell<EdgeState>,
}

#[derive(Debug)]
pub struct TrackGraph {
    edges: HashMap<String, Vec<Rc<Edge>>>,
    nodes: HashMap<String, Rc<Node>>,
}

#[derive(Debug)]
pub enum FindError {
    NotFindNode,
    NotFindAnyPath,
    NotFindAnyNode,
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
        if let Some(_) = self.nodes.insert(node_name.clone(), Rc::new(node)) {
            assert!(false, "Same Node is forbidden")
        }
        self.edges.insert(node_name, Vec::new());
    }

    fn add_edge(&mut self, edge: Edge) {
        let from_node_name = &edge.from_node.name.clone();

        let edge = Rc::new(edge);
        self.edges
            .get_mut(from_node_name)
            .unwrap()
            .push(edge.clone());
    }

    pub fn lock_node(&self, node_name: &str) {
        self.edges
            .iter()
            .flat_map(|(_, edges)| edges.iter())
            .filter(|edge| edge.to_node.name == node_name)
            .for_each(|edge| *edge.state.borrow_mut() = EdgeState::Lock);
    }

    pub fn unlock_node(&self, node_name: &str) {
        self.edges
            .iter()
            .flat_map(|(_, edges)| edges.iter())
            .filter(|edge| edge.to_node.name == node_name)
            .for_each(|edge| *edge.state.borrow_mut() = EdgeState::UnLock);
    }

    pub fn get_lock_node(&self) -> HashSet<String> {
        let mut result: HashSet<String> = HashSet::new();
        self.edges
            .iter()
            .flat_map(|(_, edges)| edges.iter())
            .filter(|edge| match *edge.state.borrow() {
                EdgeState::Lock => true,
                EdgeState::UnLock => false,
            })
            .for_each(|edge| {
                result.insert(edge.to_node.name.to_string());
            });
        result
    }

    fn a_star(&self, begin_node_name: &str, end_node_name: &str) -> Result<Path, FindError> {
        let mut open_node: PriorityQueue<Rc<Node>> = PriorityQueue::new();
        let mut close_node: HashSet<String> = HashSet::new();
        let mut came_from: HashMap<String, Rc<Node>> = HashMap::new();
        let mut g_score: HashMap<String, f64> = HashMap::new();
        for (name, _) in self.nodes.iter() {
            g_score.insert(name.clone(), f64::MAX);
        }
        g_score.insert(begin_node_name.to_string(), 0.0);

        let begin_node = self
            .nodes
            .get(begin_node_name)
            .ok_or(FindError::NotFindNode)?;
        let end_node = self
            .nodes
            .get(end_node_name)
            .ok_or(FindError::NotFindNode)?;

        open_node.push(
            heuristic_distance(begin_node.position(), end_node.position()),
            begin_node.clone(),
        );

        while let Some(current_node) = open_node.pop() {
            if current_node.name == end_node_name {
                let mut result: LinkedList<Rc<Node>> = LinkedList::new();
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
                if let EdgeState::Lock = *edge.state.borrow() {
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

        Err(FindError::NotFindAnyPath)
    }

    fn dijkstra(&self, begin_node_name: &str, node_type: &NodeType) -> Result<Path, FindError> {
        let mut open_node: PriorityQueue<Rc<Node>> = PriorityQueue::new();
        let mut close_node: HashSet<String> = HashSet::new();
        let mut came_from: HashMap<String, Rc<Node>> = HashMap::new();
        let mut g_score: HashMap<String, f64> = HashMap::new();
        for (name, _) in self.nodes.iter() {
            g_score.insert(name.clone(), f64::MAX);
        }
        g_score.insert(begin_node_name.to_string(), 0.0);

        let begin_node = self
            .nodes
            .get(begin_node_name)
            .ok_or(FindError::NotFindNode)?;

        open_node.push(0.0, begin_node.clone());

        while let Some(current_node) = open_node.pop() {
            if current_node.node_type == *node_type {
                let mut result: LinkedList<Rc<Node>> = LinkedList::new();
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
                if let EdgeState::Lock = *edge.state.borrow() {
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

        Err(FindError::NotFindAnyPath)
    }

    pub fn find_path(&self, begin_node_name: &str, end_node_name: &str) -> Result<Path, FindError> {
        self.a_star(begin_node_name, end_node_name)
    }

    pub fn find_shortest_node(&self, position: &Position) -> Result<Rc<Node>, FindError> {
        let mut nodes: Vec<(Rc<Node>, f64)> = Vec::new();
        nodes.reserve(self.nodes.len());
        self.nodes.iter().for_each(|(_, to_node)| {
            nodes.push((
                to_node.clone(),
                heuristic_distance(&position, to_node.position()),
            ));
        });
        nodes.sort_by(|a, b| a.1.total_cmp(&b.1));
        Ok(nodes.get(0).ok_or(FindError::NotFindAnyNode)?.0.clone())
    }

    pub fn find_path_by_type(
        &self,
        from_node_name: &str,
        node_type: NodeType,
    ) -> Result<Path, FindError> {
        self.dijkstra(from_node_name, &node_type)
    }

    pub fn find_parking_path(&self, from_node_name: &str) -> Result<Path, FindError> {
        self.find_path_by_type(from_node_name, NodeType::ParkingStation)
    }

    pub fn find_charging_path(&self, from_node_name: &str) -> Result<Path, FindError> {
        self.find_path_by_type(from_node_name, NodeType::ChargingStation)
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

        let _add_nodes = json
            .get("nodes")
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .for_each(|(name, value)| {
                let value = value.as_str().unwrap().to_string();
                let mut value_split: Vec<&str> = value.split(' ').collect();
                let x: f64 = value_split
                    .get(0)
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
        let _add_edges = json
            .get("edges")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .for_each(|value| {
                let value = value.as_str().unwrap().to_string();
                let value_split: Vec<&str> = value.split('-').collect();
                let from_node_name = value_split.get(0).expect("can not get from_node_name");
                let to_node_name = value_split.get(1).expect("can not get to_node_name");

                let from_node = track_graph.nodes.get(*from_node_name).unwrap();
                let to_node = track_graph.nodes.get(*to_node_name).unwrap();
                let weight = heuristic_distance(&from_node.position, &to_node.position);

                let edge = Edge {
                    from_node: track_graph.nodes.get(*from_node_name).unwrap().clone(),
                    to_node: track_graph.nodes.get(*to_node_name).unwrap().clone(),
                    weight,
                    state: RefCell::new(EdgeState::UnLock),
                };
                track_graph.add_edge(edge);
            });

        Self { track_graph }
    }

    pub fn node(
        mut self,
        name: &str,
        position: Position,
        node_type: NodeType,
    ) -> TrackGraphBuilder {
        let node = Node {
            name: name.to_string(),
            position,
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
            .expect(format!("No such Node {}, Please check!", from_node_name).as_str());
        let to_node = self
            .track_graph
            .nodes
            .get(to_node_name)
            .expect(format!("No such Node {}, Please check!", from_node_name).as_str());
        let weight = heuristic_distance(&from_node.position, &to_node.position);

        let edge = Edge {
            from_node: from_node.clone(),
            to_node: to_node.clone(),
            weight,
            state: RefCell::new(EdgeState::UnLock),
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

    #[test]
    fn add_node_edge() {
        let track_graph = TrackGraphBuilder::new()
            .node("A", (0.0, 1.0, 2.0).into(), NodeType::Stocker(Side::NegY))
            .node("B", (2.0, 4.0, 6.0).into(), NodeType::Stocker(Side::NegY))
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

    #[test]
    fn a_star() {
        let track_graph = TrackGraphBuilder::new()
            .node("P2", (0.0, 0.0, 0.0).into(), NodeType::ParkingStation)
            .node("C1", (1.0, 0.0, 0.0).into(), NodeType::ChargingStation)
            .node("P1", (2.0, 0.0, 0.0).into(), NodeType::ParkingStation)
            .node("A1", (2.0, 1.0, 0.0).into(), NodeType::Fork)
            .node("A2", (1.0, 1.0, 0.0).into(), NodeType::Fork)
            .node("A3", (1.0, 2.0, 0.0).into(), NodeType::Fork)
            .node("A4", (2.0, 2.0, 0.0).into(), NodeType::Fork)
            .node("A5", (0.0, 2.0, 0.0).into(), NodeType::Fork)
            .node("A6", (0.0, 1.0, 0.0).into(), NodeType::Fork)
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
            .build();

        let mut path: Vec<String> = Vec::new();
        let result = track_graph.a_star("A4", "P1").unwrap();
        for node in result {
            path.push(node.name.clone());
        }
        assert_eq!(path, ["A4", "A3", "A2", "A1", "P1"]);
    }

    #[test]
    fn find_parking_charging_path() {
        let track_graph = TrackGraphBuilder::new()
            .node("P2", (0.0, 0.0, 0.0).into(), NodeType::ParkingStation)
            .node("C1", (1.0, 0.0, 0.0).into(), NodeType::ChargingStation)
            .node("P1", (2.0, 0.0, 0.0).into(), NodeType::ParkingStation)
            .node("A1", (2.0, 1.0, 0.0).into(), NodeType::Fork)
            .node("A2", (1.0, 1.0, 0.0).into(), NodeType::Fork)
            .node("A3", (1.0, 2.0, 0.0).into(), NodeType::Fork)
            .node("A4", (2.0, 2.0, 0.0).into(), NodeType::Fork)
            .node("A5", (0.0, 2.0, 0.0).into(), NodeType::Fork)
            .node("A6", (0.0, 1.0, 0.0).into(), NodeType::Fork)
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
            .build();

        let mut path: Vec<String> = Vec::new();
        let result = track_graph.find_parking_path("A4").unwrap();
        for node in result {
            path.push(node.name.clone());
        }
        assert_eq!(path, ["A4", "A3", "A2", "A1", "P1"]);

        let mut path: Vec<String> = Vec::new();
        let result = track_graph.find_charging_path("A4").unwrap();
        for node in result {
            path.push(node.name.clone());
        }
        assert_eq!(path, ["A4", "A3", "A2", "C1"]);
    }

    #[test]
    fn load_json() {
        let track_graph = TrackGraphBuilder::from_json("./tests/oht_trackgraph.json").build();
        let mut path: Vec<String> = Vec::new();
        let result = track_graph.find_path("A", "F").unwrap();
        for node in result {
            path.push(node.name.clone());
        }
        assert_eq!(path, ["A", "B", "C", "F"]);
    }
}
