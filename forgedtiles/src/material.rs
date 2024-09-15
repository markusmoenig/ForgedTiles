use crate::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct BSDFMedium {
    pub type_: i32,
    pub density: f32,
    pub color: Vec3f,
    pub anisotropy: f32,
}

impl Default for BSDFMedium {
    fn default() -> Self {
        Self::new()
    }
}

impl BSDFMedium {
    pub fn new() -> Self {
        Self {
            type_: 0,
            density: 0.0,
            color: Vec3f::zero(),
            anisotropy: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct BSDFMaterial {
    pub base_color: Vec3f,
    pub opacity: f32,
    pub alpha_mode: i32,
    pub alpha_cutoff: f32,
    pub emission: Vec3f,
    pub anisotropic: f32,
    pub metallic: f32,
    pub roughness: f32,
    pub subsurface: f32,
    pub specular_tint: f32,
    pub sheen: f32,
    pub sheen_tint: f32,
    pub clearcoat: f32,
    pub clearcoat_roughness: f32,
    pub spec_trans: f32,
    pub ior: f32,
    pub ax: f32,
    pub ay: f32,
    pub medium: BSDFMedium,
}

impl Default for BSDFMaterial {
    fn default() -> Self {
        Self::new()
    }
}

impl BSDFMaterial {
    pub fn new() -> Self {
        Self {
            base_color: Vec3f::new(0.5, 0.5, 0.5),
            opacity: 1.0,
            alpha_mode: 0,
            alpha_cutoff: 0.0,
            emission: Vec3f::zero(),
            anisotropic: 0.0,
            metallic: 0.0,
            roughness: 0.5,
            subsurface: 0.0,
            specular_tint: 0.0,
            sheen: 0.0,
            sheen_tint: 0.0,
            clearcoat: 0.0,
            clearcoat_roughness: 0.0,
            spec_trans: 0.0,
            ior: 1.5,
            ax: 0.0,
            ay: 0.0,
            medium: BSDFMedium::new(),
        }
    }

    /// Mixes two materials.
    pub fn mix(&mut self, mat1: &BSDFMaterial, mat2: &BSDFMaterial, t: f32) {
        self.base_color = lerp(mat1.base_color, mat2.base_color, t);
        self.emission = lerp(mat1.emission, mat2.emission, t);
        self.anisotropic = lerp(mat1.anisotropic, mat2.anisotropic, t);
        self.metallic = lerp(mat1.metallic, mat2.metallic, t);
        self.roughness = lerp(mat1.roughness, mat2.roughness, t);
        self.subsurface = lerp(mat1.subsurface, mat2.subsurface, t);
        self.specular_tint = lerp(mat1.specular_tint, mat2.specular_tint, t);
        self.sheen = lerp(mat1.sheen, mat2.sheen, t);
        self.sheen_tint = lerp(mat1.sheen_tint, mat2.sheen_tint, t);
        self.clearcoat = lerp(mat1.clearcoat, mat2.clearcoat, t);
        self.clearcoat_roughness = lerp(mat1.clearcoat_roughness, mat2.clearcoat_roughness, t);
        self.spec_trans = lerp(mat1.spec_trans, mat2.spec_trans, t);
        self.ior = lerp(mat1.ior, mat2.ior, t);
    }
}
