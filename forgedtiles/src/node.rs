use crate::prelude::*;

#[derive(PartialEq, Clone, Debug)]
pub enum NodeRole {
    Shape,
    Pattern,
}

#[derive(PartialEq, Clone, Debug)]
pub enum NodeSubRole {
    Disc,
    Box,

    Bricks,
    Tiles,
}

use NodeRole::*;
use NodeSubRole::*;

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
                    let round = 0.1; //params[1];
                    let rotation = 0.0; //params[2];
                    let gap = 0.1; //params[3];
                    let cell = 6.0; //params[4];

                    let mut u = p + 10.0;

                    let w = vec2f(ratio, 1.0);
                    u *= vec2f(cell, cell) / w;

                    u.x += 0.5 * u.y.floor() % 2.0;

                    //let id = hash21(floor(u));
                    let id_float = hash21(floor(u));

                    let id = (id_float * 10000.0).floor() as i32; // Scale factor can be adjusted
                    let id = id % 10000; // For example, to keep it within 0 - 9999

                    //let mut p = frac(u);
                    //p = rot((id_float - 0.5) * rotation) * (p - 0.5);

                    sdf_box2d(p, 0.5 - gap, 0.5 - gap, round)
                }
                Tiles => f32::MAX,
                _ => f32::MAX,
            },
        }
    }
}
