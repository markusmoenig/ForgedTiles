pub mod bsdf;
pub mod camera;
pub mod compiler;
pub mod context;
pub mod hit;
pub mod material;
pub mod node;
pub mod ray;
pub mod scanner;
pub mod sdf;
pub mod value;

use std::path::PathBuf;

pub mod prelude {
    pub use ::serde::{Deserialize, Serialize};

    pub use crate::compiler::FTError;
    pub use crate::context::FTContext;
    pub use crate::hit::*;
    pub use crate::material::*;
    pub use crate::node::*;
    pub use crate::scanner::*;
    pub use crate::value::*;
    pub use crate::ForgedTiles;
    pub use maths_rs::prelude::*;
    pub use rand::prelude::*;
    pub use rustc_hash::*;
}

use compiler::Compiler;
use prelude::*;

pub struct ForgedTiles {}

impl Default for ForgedTiles {
    fn default() -> Self {
        Self::new()
    }
}

impl ForgedTiles {
    pub fn new() -> Self {
        Self {}
    }

    /// Compile the given script.
    pub fn compile(&self, path: PathBuf, file_name: String) -> Result<FTContext, FTError> {
        let main_path = path.join(file_name.clone());

        if let Ok(code) = std::fs::read_to_string(main_path) {
            self.compile_code(code)
        } else {
            Err(FTError::new(
                format!("Error reading file `{}`", file_name),
                0,
            ))
        }
    }

    /// Compile the given code.
    pub fn compile_code(&self, code: String) -> Result<FTContext, FTError> {
        let mut compiler = Compiler::new();
        compiler.compile(code)
    }
}
