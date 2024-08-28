pub mod compiler;
pub mod context;
pub mod node;
pub mod scanner;
pub mod sdf;
pub mod value;

use std::path::PathBuf;

pub mod prelude {
    pub use crate::compiler::FTError;
    pub use crate::context::FTContext;
    pub use crate::node::*;
    pub use crate::scanner::*;
    pub use crate::sdf::*;
    pub use crate::value::*;
    pub use crate::ForgedTiles;
    pub use maths_rs::prelude::*;
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
            self.compile_code(code, file_name.to_string())
        } else {
            Err(FTError::new(
                format!("Error reading file `{}`", file_name),
                0,
            ))
        }
    }

    /// Compile the given code.
    pub fn compile_code(&self, code: String, _file_name: String) -> Result<FTContext, FTError> {
        let mut compiler = Compiler::new();
        compiler.compile(code)
    }
}
