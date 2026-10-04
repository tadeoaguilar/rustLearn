//! Exercise 5: Associated Types.
//!
//! Associated type vs generic parameter: `Iterator` has `type Item` rather than
//! being `Iterator<T>` because a given iterator yields exactly one item type.
//! A generic parameter would let one type implement `Iterator<u32>` AND
//! `Iterator<String>`, and every use would need annotations to pick one.

use std::collections::HashMap;

/// Task 1.
#[derive(Debug, Clone)]
pub struct Counter {
    count: u32,
    max: u32,
}

impl Counter {
    pub fn new(max: u32) -> Counter {
        todo!("Exercise 5")
    }
}

impl Iterator for Counter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        todo!("Exercise 5")
    }
}

/// Task 2: a graph trait whose node and edge types are chosen by each
/// implementation.
pub trait Graph {
    type Node;
    type Edge;

    fn has_edge(&self, from: &Self::Node, to: &Self::Node) -> bool;
    fn edges(&self, node: &Self::Node) -> Vec<&Self::Edge>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Road {
    pub from: String,
    pub to: String,
    pub km: u32,
}

/// "Implement for a specific graph type": a directed road map, stored as an
/// adjacency list -- each city maps to the roads leaving it.
#[derive(Debug, Default)]
pub struct RoadMap {
    roads: HashMap<String, Vec<Road>>,
}

impl RoadMap {
    pub fn new() -> Self {
        todo!("Exercise 5")
    }

    pub fn add_road(&mut self, from: &str, to: &str, km: u32) {
        todo!("Exercise 5")
    }
}

impl Graph for RoadMap {
    type Node = String;
    type Edge = Road;

    fn has_edge(&self, from: &String, to: &String) -> bool {
        todo!("Exercise 5")
    }

    fn edges(&self, node: &String) -> Vec<&Road> {
        todo!("Exercise 5")
    }
}

/// Generic code over *any* Graph. `G::Node` names the associated type.
pub fn out_degree<G: Graph>(graph: &G, node: &G::Node) -> usize {
    todo!("Exercise 5")
}

pub fn run() {
    todo!("Exercise 5")
}
