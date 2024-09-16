use crate::prelude::*;

/// FTHitStruct
#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub struct FTHitStruct {
    pub distance: f32,
    pub min_distance: f32,
    pub group_distance: Option<f32>,

    pub node: usize,

    pub last_size: Vec2f,

    pub pattern_id: i32,
    pub pattern_hash: f32,

    pub face: Vec3f,

    pub origin: Vec2f,

    pub tile_id: Vec2f,

    pub working_seed: f32,
    pub working_seed_id: i32,

    pub seed: f32,
    pub seed_id: i32,
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
            min_distance: f32::MAX,
            group_distance: None,

            node: 0,

            last_size: Vec2f::zero(),

            pattern_id: 0,
            pattern_hash: 0.0,

            face: Vec3f::zero(),

            origin: Vec2f::zero(),

            tile_id: Vec2f::zero(),

            working_seed: 0.0,
            working_seed_id: 0,

            seed: 0.0,
            seed_id: 0,
        }
    }
}
