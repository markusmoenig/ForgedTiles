use core::f32;

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

    Bricks,
    Tiles,

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

    pub fn distance(&self, p: Vec2f, pos: Vec2f, hit: &mut FTHitStruct) {
        #[allow(clippy::single_match)]
        match &self.role {
            Shape => match &self.sub_role {
                Disc => hit.distance = length(p) - 0.5,
                Box => {
                    let length = self.values.get(FTValueRole::Length, vec![1.0])[0];
                    let height = self.values.get(FTValueRole::Height, vec![1.0])[0];
                    hit.distance = crate::sdf::sdf_box2d(p, pos, length / 2.0, height / 2.0, 0.0)
                }
                _ => hit.distance = f32::MAX,
            },
            Pattern => match &self.sub_role {
                Bricks => {
                    let length = self.values.get(FTValueRole::Length, vec![1.0])[0];
                    let height = self.values.get(FTValueRole::Height, vec![1.0])[0];
                    let cell = self.values.get(Cell, vec![3.0])[0];

                    //let height = height / cell;

                    //let pos = pos
                    hit.distance = crate::sdf::sdf_box2d(p, pos, length / 2.0, height / 2.0, 0.0)

                    /*
                    let ratio = self.values.get(Ratio, vec![3.0])[0];
                    let round = self.values.get(Rounding, vec![0.0])[0];
                    let rotation = self.values.get(Rotation, vec![1.0])[0] / 10.0;
                    let gap = self.values.get(Gap, vec![1.0])[0] / 10.0;
                    let cell = self.values.get(Cell, vec![3.0])[0];

                    let mut u = p - pos + 10.0; // + hit.tile_id; // + vec2f(0.0, 0.5);

                    let w = vec2f(ratio, 1.0);
                    u *= vec2f(cell, cell) / w;

                    u.x += 0.5 * u.y.floor() % 2.0;

                    // let id = hash21(floor(u));
                    let hash = crate::sdf::hash21(floor(u));

                    let id = (hash * 10000.0).floor() as i32;
                    let id = id % 10000;

                    let mut p = frac(u);
                    p = crate::sdf::rot((hash - 0.5) * rotation) * (p - 0.5);

                    hit.distance =
                        crate::sdf::sdf_box2d(p, Vec2f::zero(), 0.5 - gap, 0.5 - gap, round);
                    hit.pattern_id = id;
                    hit.pattern_hash = hash;
                    */
                }
                _ => {}
            },
            _ => {}
        }
    }

    // pub fn distance_3d(&self, p: Vec3f) -> f32 {
    //     match &self.role {
    //         Face => {}
    //         _ => f32::MAX,
    //     }
    // }
}
