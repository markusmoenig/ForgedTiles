use crate::prelude::*;

/// FTHitStruct
#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub struct FTHitStruct {
    pub distance: f32,
    pub node: usize,
    pub pattern_id: i32,
    pub pattern_hash: f32,

    pub face: Vec3f,

    pub tile_id: Vec2f,
}

impl Default for FTHitStruct {
    fn default() -> Self {
        Self::new()
    }
}

impl FTHitStruct {
    pub fn new() -> Self {
        Self {
            distance: f32::MAX,
            node: 0,
            pattern_id: 0,
            pattern_hash: 0.0,

            face: Vec3f::zero(),

            tile_id: Vec2f::zero(),
        }
    }
}
