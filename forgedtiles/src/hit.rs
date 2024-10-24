use crate::prelude::*;

/// FTHitStruct
#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub struct FTHitStruct {
    pub distance: f32,
    pub min_distance: f32,

    pub node: Option<usize>,

    pub last_size: Vec2f,

    pub face: Vec3f,

    pub origin: Vec2f,

    pub tile_id: Vec2f,

    pub pattern_id: i32,
    pub pattern_hash: f32,

    pub working_pattern_id: i32,
    pub working_pattern_hash: f32,

    pub working_seed: f32,
    pub working_seed_id: i32,

    pub seed: f32,
    pub seed_id: i32,

    pub is_cut_out: bool,
    pub shape_adder: f32,

    pub group_uv: Vec2f,
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

            node: None,

            last_size: Vec2f::zero(),

            face: Vec3f::zero(),

            origin: Vec2f::zero(),

            tile_id: Vec2f::zero(),

            pattern_id: 0,
            pattern_hash: 0.0,

            working_pattern_id: 0,
            working_pattern_hash: 0.0,

            working_seed: 0.0,
            working_seed_id: 0,

            seed: 0.0,
            seed_id: 0,

            is_cut_out: false,
            shape_adder: 0.0,

            group_uv: Vec2f::zero(),
        }
    }
}
