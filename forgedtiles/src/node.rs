use crate::prelude::*;

#[derive(PartialEq, Clone, Debug)]
pub enum NodeRole {
    Shape,
}

use NodeRole::*;

#[derive(Debug, Clone)]
pub struct Node {
    pub role: NodeRole,

    pub name: String,
    pub childs: Vec<usize>,
    pub elements: Vec<usize>,

    pub values: FTValues,

    //pub object: Object,
    pub texture: Option<usize>,

    pub indent: usize,
}

impl Node {
    pub fn new(role: NodeRole) -> Self {
        Self {
            role,
            name: "".to_string(),
            childs: vec![],
            elements: vec![],

            values: FTValues::default(),

            //object: Object::Empty,
            texture: None,
            indent: 0,
        }
    }

    // Returns the type of the node object.
    // pub fn get_node_type(&self) -> NodeType {
    //     match &self.object {
    //         Object::AnalyticalObject(_v) => Object3D,
    //         Object::SDF3D(_v) => Object3D,
    //         Object::Element2D(_v) => Element2D,
    //         _ => Unknown,
    //     }
    // }
}
