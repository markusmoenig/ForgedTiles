use crate::prelude::*;

#[derive(PartialEq, Clone, Debug)]
pub enum NodeRole {
    Shape,
    Pattern,
    Face,
    Material,
}

#[derive(PartialEq, Clone, Debug)]
pub enum NodeSubRole {
    Disc,
    Box,

    Repeat,
    Offset,
    Stack,

    Floor,
    Left,
    Top,
    Right,
    Bottom,
    MiddleX,
    MiddleY,

    BSDF,
}

use FTValueRole::*;
use NodeRole::*;
use NodeSubRole::*;

#[derive(Debug, Clone)]
pub struct Node {
    pub role: NodeRole,
    pub sub_role: NodeSubRole,

    pub name: String,

    /// Arrays of f32 values associated with a given role (width, height, etc)
    pub values: FTValues,
    /// The map contains String lists.
    pub map: FxHashMap<String, Vec<String>>,
    /// Array of indices to other nodes.
    pub links: Vec<u16>,
    /// Material index
    pub material: Option<u8>,
}

impl Node {
    pub fn new(role: NodeRole, sub_role: NodeSubRole) -> Self {
        Self {
            role,
            sub_role,

            name: "".to_string(),

            values: FTValues::default(),
            map: FxHashMap::default(),
            links: vec![],

            material: None,
        }
    }

    pub fn content_indices(&self, index: usize) -> Vec<usize> {
        match &self.role {
            Face => {
                let indices = self.values.get(FTValueRole::Content, vec![]);

                let mut out: Vec<usize> = vec![];

                for i in indices {
                    out.push(i as usize);
                }

                out
            }
            _ => {
                vec![index]
            }
        }
    }

    // pub fn distance_3d(&self, p: Vec3f) -> f32 {
    //     match &self.role {
    //         Face => {}
    //         _ => f32::MAX,
    //     }
    // }
}
