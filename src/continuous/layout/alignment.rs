use std::collections::HashSet;

#[derive(Clone, Debug)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right
}

#[derive(Clone, Debug)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom
}

#[derive(Debug, Clone, std::hash::Hash, PartialEq, Eq)]
pub enum Edge {
    Top,
    Right,
    Bottom,
    Left
}

impl Edge {
    pub fn all() -> HashSet<Edge> {
        hash_set!(Edge::Top, Edge::Right, Edge::Bottom, Edge::Left)
    }

    pub fn horizontal() -> HashSet<Edge> {
        hash_set!(Edge::Right, Edge::Left)
    }

    pub fn vertical() -> HashSet<Edge> {
        hash_set!(Edge::Top, Edge::Bottom)
    }
}