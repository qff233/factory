use std::collections::BinaryHeap;

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
