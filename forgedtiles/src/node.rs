use crate::prelude::*;

#[derive(PartialEq, Clone, Debug)]
pub enum NodeRole {
    Shape,
    Pattern,
}

#[derive(PartialEq, Clone, Debug)]
pub enum NodeSubRole {
    Rect,
    Disc,

    Bricks,
    Tiles,
}

use NodeRole::*;

#[derive(Debug, Clone)]
pub struct Node {
    pub role: NodeRole,
    pub sub_role: NodeSubRole,

    pub name: String,
    pub values: FTValues,

    pub map: FxHashMap<String, Vec<String>>,

    pub indent: usize,
}

impl Node {
    pub fn new(role: NodeRole, sub_role: NodeSubRole) -> Self {
        Self {
            role,
            sub_role,

            name: "".to_string(),
            values: FTValues::default(),

            map: FxHashMap::default(),

            indent: 0,
        }
    }
}