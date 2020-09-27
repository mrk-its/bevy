#[allow(clippy::module_inception)]

#[cfg(feature = "naga-glsl")]
mod preprocessor;
mod shader;
mod shader_defs;

#[cfg(feature = "spirv-reflect")]
mod shader_reflect;

#[cfg(feature = "naga-reflect")]
mod shader_reflect_naga;

pub use shader::*;
pub use shader_defs::*;

#[cfg(feature = "spirv-reflect")]
pub use shader_reflect::*;

#[cfg(feature = "naga-reflect")]
pub use shader_reflect_naga::*;

use crate::pipeline::{BindGroupDescriptor, VertexBufferDescriptor};

/// Defines the memory layout of a shader
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderLayout {
    pub bind_groups: Vec<BindGroupDescriptor>,
    pub vertex_buffer_descriptors: Vec<VertexBufferDescriptor>,
    pub entry_point: String,
}

pub const GL_VERTEX_INDEX: &str = "gl_VertexIndex";
