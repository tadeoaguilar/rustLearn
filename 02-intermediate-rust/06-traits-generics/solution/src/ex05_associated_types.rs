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
        Counter { count: 0, max }
    }
}

impl Iterator for Counter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.max {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
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
        Self::default()
    }

    pub fn add_road(&mut self, from: &str, to: &str, km: u32) {
        self.roads.entry(from.to_string()).or_default().push(Road {
            from: from.to_string(),
            to: to.to_string(),
            km,
        });
    }
}

impl Graph for RoadMap {
    type Node = String;
    type Edge = Road;

    fn has_edge(&self, from: &String, to: &String) -> bool {
        self.roads
            .get(from)
            .is_some_and(|rs| rs.iter().any(|r| &r.to == to))
    }

    fn edges(&self, node: &String) -> Vec<&Road> {
        self.roads
            .get(node)
            .map(|rs| rs.iter().collect())
            .unwrap_or_default()
    }
}

/// Generic code over *any* Graph. `G::Node` names the associated type.
pub fn out_degree<G: Graph>(graph: &G, node: &G::Node) -> usize {
    graph.edges(node).len()
}

pub fn run() {
    for num in Counter::new(5) {
        print!("{num} ");
    }
    println!();
    // Implementing `next` gives you every Iterator adaptor for free:
    let sum: u32 = Counter::new(5)
        .zip(Counter::new(5).skip(1))
        .map(|(a, b)| a * b)
        .filter(|x| x % 3 == 0)
        .sum();
    println!("Rust Book's Counter challenge: {sum}");

    let mut map = RoadMap::new();
    map.add_road("Paris", "Lyon", 465);
    map.add_road("Paris", "Lille", 225);
    map.add_road("Lyon", "Marseille", 315);
    let paris = "Paris".to_string();
    println!(
        "Paris -> Lyon? {}",
        map.has_edge(&paris, &"Lyon".to_string())
    );
    println!(
        "Lyon -> Paris? {}",
        map.has_edge(&"Lyon".to_string(), &paris)
    );
    println!("roads out of Paris: {}", out_degree(&map, &paris));
}
