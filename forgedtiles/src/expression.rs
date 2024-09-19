use crate::prelude::*;

use exmex::Express;
use FTExpressionRole::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum FTExpressionRole {
    Extrusion,
    Modifier,
    Rounding,
    Annular,
    Border,

    Anisotropic,
    Metallic,
    Roughness,
    Subsurface,
    SpecularTint,
    Sheen,
    SheenTint,
    Clearcoat,
    ClearcoatGloss,
    Emission,
    Transmission,
    IOR,
}

impl FTExpressionRole {
    pub fn from_string(s: &str) -> Option<FTExpressionRole> {
        match s {
            "extrusion" => Some(Extrusion),
            "modifier" => Some(Modifier),
            "rounding" => Some(Rounding),
            "annular" => Some(Annular),
            "border" => Some(Border),

            "anisotropic" => Some(Anisotropic),
            "metallic" => Some(Metallic),
            "roughness" => Some(Roughness),
            "subsurface" => Some(Subsurface),
            "specular_tint" => Some(SpecularTint),
            "sheen" => Some(Sheen),
            "sheen_tint" => Some(SheenTint),
            "clearcoat" => Some(Clearcoat),
            "clearcoat_gloss" => Some(ClearcoatGloss),
            "emission" => Some(Emission),
            "transmission" => Some(Transmission),
            "ior" => Some(IOR),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum FTExpressionParam {
    Hash,
    Thickness,
}

impl FTExpressionParam {
    pub fn from_string(s: &str) -> Option<FTExpressionParam> {
        match s {
            "hash" => Some(FTExpressionParam::Hash),
            "thickness" => Some(FTExpressionParam::Thickness),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct FTExpressions {
    pub expressions: Vec<(FTExpressionRole, exmex::FlatEx<f32>, Vec<FTExpressionParam>)>,
}

impl Default for FTExpressions {
    fn default() -> Self {
        Self::new()
    }
}

impl FTExpressions {
    pub fn new() -> Self {
        Self {
            expressions: vec![],
        }
    }

    /// Add an expression.
    pub fn add(&mut self, role: FTExpressionRole, expression: &str) {
        match exmex::parse::<f32>(expression) {
            Ok(expr) => {
                let params_in_expr = expr.var_names();
                let mut params = vec![];

                for p in params_in_expr {
                    if let Some(ftp) = FTExpressionParam::from_string(p) {
                        params.push(ftp);
                    }
                }
                self.expressions.push((role, expr, params));
            }
            Err(err) => {
                println!("Expression error: {} for {:?}", err, role);
            }
        }
    }

    /// Evaluate the expression of the given role or return the default value.
    pub fn eval(
        &self,
        role: FTExpressionRole,
        parameters: Vec<(FTExpressionParam, f32)>,
        default: f32,
    ) -> f32 {
        for (r, expr, params) in &self.expressions {
            if *r == role {
                // Only use the variables which are contained in the expression
                let mut out_params = vec![];
                for (prole, pvalue) in &parameters {
                    if params.contains(prole) {
                        out_params.push(*pvalue);
                    }
                }

                match expr.eval(&out_params) {
                    Ok(val) => {
                        return val;
                    }
                    Err(err) => {
                        println!("Error in expression '{:?}' : {}.", role, err);
                        return default;
                    }
                }
            }
        }
        default
    }
}
