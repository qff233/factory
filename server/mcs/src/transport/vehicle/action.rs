use std::{collections::LinkedList, sync::Arc};

use crate::transport::{prelude::Side, track};

#[derive(Debug, Clone)]
pub enum Action {
    Move(Arc<track::Node>),
    Drop(Side),
    Suck(Side),
    Drain(Side),
    Fill(Side),
    Use(Side),
}

#[derive(Debug)]
pub struct ActionSequence(LinkedList<Action>);

impl ActionSequence {
    pub fn next_action(&self) -> Option<&Action> {
        self.0.front()
    }

    pub fn pop_next_action(&mut self) -> Option<Action> {
        self.0.pop_front()
    }

    pub fn last_move_node(&self) -> Option<Arc<track::Node>> {
        for action in self.0.iter().rev() {
            if let Action::Move(node) = action {
                return Some(node.clone());
            }
        }
        None
    }
}

pub struct ActionSequenceBuilder(LinkedList<Action>);

impl ActionSequenceBuilder {
    pub fn new() -> Self {
        Self(LinkedList::new())
    }

    pub fn move_path(mut self, path: &track::Path) -> Self {
        for node in path.iter().skip(1) {
            self.0.push_back(Action::Move(node.clone()));
        }
        self
    }

    pub fn move_to(mut self, node: Arc<track::Node>) -> Self {
        self.0.push_back(Action::Move(node));
        self
    }

    pub fn drop(mut self, side: &Side) -> Self {
        self.0.push_back(Action::Drop(side.clone()));
        self
    }

    pub fn suck(mut self, side: &Side) -> Self {
        self.0.push_back(Action::Suck(side.clone()));
        self
    }

    pub fn drain(mut self, side: &Side) -> Self {
        self.0.push_back(Action::Drain(side.clone()));
        self
    }

    pub fn fill(mut self, side: &Side) -> Self {
        self.0.push_back(Action::Fill(side.clone()));
        self
    }

    pub fn use_tool(mut self, side: &Side) -> Self {
        self.0.push_back(Action::Use(side.clone()));
        self
    }

    pub fn chain(mut self, mut sequence: Self) -> Self {
        self.0.append(&mut sequence.0);
        self
    }

    pub fn build(self) -> ActionSequence {
        ActionSequence(self.0)
    }
}
