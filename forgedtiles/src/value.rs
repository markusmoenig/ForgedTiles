// use crate::prelude::*;

use FTValueRole::*;

#[derive(Debug, PartialEq, Clone)]
pub enum FTValueRole {
    Color,
    Width,
    Height,
    Radius,
    Thickness,
    Content,
    Length,
    Ratio,
    Rotation,
    Rounding,
    Gap,
    Cell,
}

impl FTValueRole {
    pub fn from_string(s: &str) -> Option<FTValueRole> {
        match s {
            "color" => Some(Color),
            "width" => Some(Width),
            "height" => Some(Height),
            "radius" => Some(Radius),
            "thickness" => Some(Thickness),
            "content" => Some(Content),
            "length" => Some(Length),
            "ratio" => Some(Ratio),
            "rotation" => Some(Rotation),
            "rounding" => Some(Rounding),
            "gap" => Some(Gap),
            "cell" => Some(Cell),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FTValues {
    pub values: Vec<(FTValueRole, Vec<f32>)>,
}

impl Default for FTValues {
    fn default() -> Self {
        Self::new()
    }
}

impl FTValues {
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    /// Add a value.
    pub fn add(&mut self, role: FTValueRole, value: Vec<f32>) {
        self.values.push((role, value));
    }

    /// Add a string based value.
    pub fn add_string_based(&mut self, role_string: &str, value: Vec<f32>) -> bool {
        if let Some(role) = FTValueRole::from_string(role_string) {
            self.add(role, value);
            true
        } else {
            false
        }
    }

    /// Get the values of the given role or return the default value.
    pub fn get(&self, role: FTValueRole, default: Vec<f32>) -> Vec<f32> {
        for (r, values) in &self.values {
            if *r == role {
                return values.clone();
            }
        }
        default.clone()
    }
}
