use core::f32;

use crate::prelude::*;

#[derive(PartialEq, Clone, Debug)]
pub enum NodeRole {
    Shape,
    Pattern,
    Face,
}

#[derive(PartialEq, Clone, Debug)]
pub enum NodeSubRole {
    Disc,
    Box,

    Bricks,
    Tiles,

    Floor,
    Left,
    Top,
    Right,
    Bottom,
    MiddleX,
    MiddleY,
}

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

    pub fn distance(&self, p: Vec2f) -> f32 {
        match &self.role {
            Shape => match &self.sub_role {
                Disc => length(p) - 0.5,
                Box => sdf_box2d(p, 0.5, 0.5, 0.0),
                _ => f32::MAX,
            },
            Pattern => match &self.sub_role {
                Bricks => {
                    let ratio = 3.0; //params[0];
                    let round = 0.0; //params[1];
                    let rotation = 0.1; //params[2];
                    let gap = 0.1; //params[3];
                    let cell = 6.0; //params[4];

                    let mut u = p + 10.0;

                    let w = vec2f(ratio, 1.0);
                    u *= vec2f(cell, cell) / w;

                    u.x += 0.5 * u.y.floor() % 2.0;

                    // let id = hash21(floor(u));
                    let id_float = hash21(floor(u));

                    // let id = (id_float * 10000.0).floor() as i32;
                    // let id = id % 10000;

                    let mut p = frac(u);
                    p = rot((id_float - 0.5) * rotation) * (p - 0.5);
                    sdf_box2d(p, 0.5 - gap, 0.5 - gap, round)
                }
                Tiles => f32::MAX,
                _ => f32::MAX,
            },
            Face => f32::MAX,
        }
    }

    // pub fn distance_3d(&self, p: Vec3f) -> f32 {
    //     match &self.role {
    //         Face => {}
    //         _ => f32::MAX,
    //     }
    // }
}
