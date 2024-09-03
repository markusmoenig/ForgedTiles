use crate::prelude::*;

/// FTHitStruct
#[derive(Serialize, Deserialize, PartialEq, Debug, Copy, Clone)]
pub struct FTHitStruct {
    pub distance: f32,
    pub node: usize,
    pub pattern_id: i32,
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
        }
    }
}
