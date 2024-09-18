use crate::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum NodeRole {
    Shape,
    Pattern,
    Face,
    Material,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum NodeSubRole {
    Disc,
    Box,

    Repeat,
    Offset,
    Stack,
    Group,

    Floor,
    Left,
    Top,
    Right,
    Bottom,
    MiddleX,
    MiddleY,

    BSDF,
}

//use FTValueRole::*;
use NodeRole::*;
use NodeSubRole::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Node {
    pub role: NodeRole,
    pub sub_role: NodeSubRole,

    pub name: String,

    /// Arrays of f32 values associated with a given role (length, height, etc)
    pub values: FTValues,
    /// Arrays of expressions for a given role (extrusion etc)
    pub expressions: FTExpressions,
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
            expressions: FTExpressions::default(),
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

    /// Return the length and height of the shape.
    pub fn get_shape_dim(&self) -> Vec2f {
        let mut dim = Vec2f::zero();

        match &self.sub_role {
            Disc => {
                let radius = self.values.get(FTValueRole::Radius, vec![0.5])[0] * 2.0;
                dim.x = radius;
                dim.y = radius;
            }
            _ => {
                dim.x = self.values.get(FTValueRole::Length, vec![1.0])[0];
                dim.y = self.values.get(FTValueRole::Height, vec![1.0])[0];
            }
        }

        dim
    }
}
