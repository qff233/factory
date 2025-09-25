use super::queue::PriorityQueue;
use core::panic;
use std::{
    collections::{HashMap, HashSet, LinkedList},
    fs::File,
    rc::Rc,
};

#[derive(Debug, PartialEq)]
pub enum Side {
    NegY,
    PosY,
    NegZ,
    PosZ,
    NegX,
    PosX,
}

impl From<&str> for Side {
    fn from(value: &str) -> Self {
        match value {
            "negy" => Self::NegY,
            "posy" => Self::PosY,
            "negz" => Self::NegZ,
            "posz" => Self::PosZ,
            "negx" => Self::NegX,
            "posx" => Self::PosX,
            _ => panic!("no such side, {}", value),
        }
    }
}

#[derive(Debug, PartialEq)]
enum NodeType {
    Stocker,
    Machine(Side),
}

impl From<Vec<&str>> for NodeType {
    fn from(value: Vec<&str>) -> Self {
        match *value.get(0).expect("can not get node_type") {
            "stocker" => Self::Stocker,
            "machine" => {
                let side = *value.get(1).expect("can not get machine side");
                Self::Machine(side.into())
            }
            _ => panic!("no such node_type, {:?}", value),
        }
    }
}

#[derive(Debug)]
struct Node {
    name: String,
    position: (f64, f64, f64),
    node_type: NodeType,
}

fn heuristic_distance(from_node: &Node, to_node: &Node) -> f64 {
    let (x1, y1, z1) = from_node.position;
    let (x2, y2, z2) = to_node.position;

    let dx = x1 - x2;
    let dy = y1 - y2;
    let dz = z1 - z2;

    f64::sqrt(dx * dx + dy * dy + dz * dz)
}

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
    state: EdgeState,
}

#[derive(Debug)]
pub(crate) struct TrackGraph {
    adjacency_edge: HashMap<String, Vec<Rc<Edge>>>,
    edges: Vec<Rc<Edge>>,
    nodes: HashMap<String, Rc<Node>>,
}

#[derive(Debug)]
enum FindPathError {
    NotFindBeginNode,
    NotFindEndNode,
}

impl TrackGraph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            adjacency_edge: HashMap::new(),
        }
    }

    fn add_node(&mut self, node: Node) {
        let node_name = node.name.clone();
        if let Some(_) = self.nodes.insert(node_name.clone(), Rc::new(node)) {
            assert!(false, "Same Node is forbidden")
        }
        self.adjacency_edge.insert(node_name, Vec::new());
    }

    fn add_edge(&mut self, edge: Edge) {
        let from_node_name = &edge.from_node.name.clone();

        let edge = Rc::new(edge);
        self.adjacency_edge
            .get_mut(from_node_name)
            .unwrap()
            .push(edge.clone());

        self.edges.push(edge);
    }

    pub(crate) fn find_shortest_path(
        self,
        begin_node_name: &str,
        end_node_name: &str,
    ) -> Result<Vec<Rc<Node>>, FindPathError> {
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
            .ok_or(FindPathError::NotFindBeginNode)?;
        let end_node = self
            .nodes
            .get(end_node_name)
            .ok_or(FindPathError::NotFindEndNode)?;

        open_node.push(heuristic_distance(begin_node, end_node), begin_node.clone());

        while let Some(current_node) = open_node.pop() {
            if current_node.name == end_node_name {
                let mut current_node = current_node;
                let mut result: LinkedList<Rc<Node>> = LinkedList::new();
                result.push_back(current_node.clone());
                while let Some(prev_node) = came_from.get(current_node.name.as_str()) {
                    current_node = prev_node.clone();
                    result.push_front(current_node.clone());
                }
                return Ok(result.into_iter().collect());
            }

            close_node.insert(current_node.name.clone());

            let edges = self.adjacency_edge.get(current_node.name.as_str()).unwrap();
            for edge in edges {
                if let EdgeState::Lock = edge.state {
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
                    + heuristic_distance(&current_node, end_node);
                open_node.push(f_score, to_node.clone());
            }
        }

        todo!()
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
                    position: (x, y, z),
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
                let weight = heuristic_distance(from_node, to_node);

                let edge = Edge {
                    from_node: track_graph.nodes.get(*from_node_name).unwrap().clone(),
                    to_node: track_graph.nodes.get(*to_node_name).unwrap().clone(),
                    weight,
                    state: EdgeState::UnLock,
                };
                track_graph.add_edge(edge);
            });

        Self { track_graph }
    }

    pub(crate) fn node(
        mut self,
        name: &str,
        position: (f64, f64, f64),
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

    pub(crate) fn edge(mut self, from_node_name: &str, to_node_name: &str) -> TrackGraphBuilder {
        let from_node = self.track_graph.nodes.get(from_node_name).unwrap();
        let to_node = self.track_graph.nodes.get(to_node_name).unwrap();
        let weight = heuristic_distance(from_node, to_node);

        let edge = Edge {
            from_node: from_node.clone(),
            to_node: to_node.clone(),
            weight,
            state: EdgeState::UnLock,
        };

        self.track_graph.add_edge(edge);
        self
    }

    pub(crate) fn build(self) -> TrackGraph {
        self.track_graph
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_node_edge() {
        let track_graph = TrackGraphBuilder::new()
            .node("A", (0.0, 1.0, 2.0), NodeType::Stocker)
            .node("B", (2.0, 4.0, 6.0), NodeType::Stocker)
            .edge("A", "B")
            .build();

        let node_b = track_graph.nodes.get("B").unwrap();
        let (x, y, z) = node_b.position;
        assert_eq!(node_b.name, "B");
        assert_eq!(x, 2.0);
        assert_eq!(y, 4.0);
        assert_eq!(z, 6.0);
        assert_eq!(node_b.node_type, NodeType::Stocker);

        let edge = track_graph.adjacency_edge.get("A").unwrap().get(0).unwrap();
        assert_eq!(edge.from_node.name, "A");
        assert_eq!(edge.to_node.name, "B");
    }

    #[test]
    fn a_star() {
        let track_graph = TrackGraphBuilder::new()
            .node("A", (0.0, 0.0, 0.0), NodeType::Stocker)
            .node("B", (0.0, 3.0, 0.0), NodeType::Machine(Side::NegY))
            .node("C", (6.5, 5.5, 0.0), NodeType::Machine(Side::NegY))
            .node("D", (3.0, 0.0, 7.0), NodeType::Machine(Side::NegY))
            .node("E", (18.0, 0.0, 0.0), NodeType::Machine(Side::NegY))
            .node("F", (17.0, 5.0, 0.0), NodeType::Machine(Side::NegY))
            .edge("A", "B")
            .edge("A", "D")
            .edge("B", "C")
            .edge("C", "D")
            .edge("C", "E")
            .edge("C", "F")
            .edge("D", "C")
            .edge("D", "E")
            .edge("E", "F")
            .build();

        let mut path: Vec<String> = Vec::new();
        let result = track_graph.find_shortest_path("A", "F").unwrap();
        for node in result {
            path.push(node.name.clone());
        }
        assert_eq!(path, ["A", "B", "C", "F"]);
    }

    #[test]
    fn load_json() {
        let track_graph = TrackGraphBuilder::from_json("./tests/oht_trackgraph.json").build();
        let mut path: Vec<String> = Vec::new();
        let result = track_graph.find_shortest_path("A", "F").unwrap();
        for node in result {
            path.push(node.name.clone());
        }
        assert_eq!(path, ["A", "B", "C", "F"]);
    }
}
