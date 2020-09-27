use crate::Color;
use bevy_math::{Mat4, Vec2, Vec3, Vec4};
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum VertexFormat {
    /// Two unsigned bytes (u8). `uvec2` in shaders.
    Uchar2,
    /// Four unsigned bytes (u8). `uvec4` in shaders.
    Uchar4,
    /// Two signed bytes (i8). `ivec2` in shaders.
    Char2,
    /// Four signed bytes (i8). `ivec4` in shaders.
    Char4,
    /// Two unsigned bytes (u8). [0, 255] converted to float [0, 1] `vec2` in shaders.
    Uchar2Norm,
    /// Four unsigned bytes (u8). [0, 255] converted to float [0, 1] `vec4` in shaders.
    Uchar4Norm,
    /// Two signed bytes (i8). [-127, 127] converted to float [-1, 1] `vec2` in shaders.
    Char2Norm,
    /// Four signed bytes (i8). [-127, 127] converted to float [-1, 1] `vec4` in shaders.
    Char4Norm,
    /// Two unsigned shorts (u16). `uvec2` in shaders.
    Ushort2,
    /// Four unsigned shorts (u16). `uvec4` in shaders.
    Ushort4,
    /// Two unsigned shorts (i16). `ivec2` in shaders.
    Short2,
    /// Four unsigned shorts (i16). `ivec4` in shaders.
    Short4,
    /// Two unsigned shorts (u16). [0, 65535] converted to float [0, 1] `vec2` in shaders.
    Ushort2Norm,
    /// Four unsigned shorts (u16). [0, 65535] converted to float [0, 1] `vec4` in shaders.
    Ushort4Norm,
    /// Two signed shorts (i16). [-32767, 32767] converted to float [-1, 1] `vec2` in shaders.
    Short2Norm,
    /// Four signed shorts (i16). [-32767, 32767] converted to float [-1, 1] `vec4` in shaders.
    Short4Norm,
    /// Two half-precision floats (no Rust equiv). `vec2` in shaders.
    Half2,
    /// Four half-precision floats (no Rust equiv). `vec4` in shaders.
    Half4,
    /// One single-precision float (f32). `float` in shaders.
    Float,
    /// Two single-precision floats (f32). `vec2` in shaders.
    Float2,
    /// Three single-precision floats (f32). `vec3` in shaders.
    Float3,
    /// Four single-precision floats (f32). `vec4` in shaders.
    Float4,
    /// One unsigned int (u32). `uint` in shaders.
    Uint,
    /// Two unsigned ints (u32). `uvec2` in shaders.
    Uint2,
    /// Three unsigned ints (u32). `uvec3` in shaders.
    Uint3,
    /// Four unsigned ints (u32). `uvec4` in shaders.
    Uint4,
    /// One signed int (i32). `int` in shaders.
    Int,
    /// Two signed ints (i32). `ivec2` in shaders.
    Int2,
    /// Three signed ints (i32). `ivec3` in shaders.
    Int3,
    /// Four signed ints (i32). `ivec4` in shaders.
    Int4,
}

impl VertexFormat {
    pub fn get_sizes(&self) -> (i32, i32) {
        match *self {
            VertexFormat::Uchar2 => (1, 2),
            VertexFormat::Uchar4 => (1, 4),
            VertexFormat::Char2 => (1, 2),
            VertexFormat::Char4 => (1, 4),
            VertexFormat::Uchar2Norm => (1, 2),
            VertexFormat::Uchar4Norm => (1, 4),
            VertexFormat::Char2Norm => (1, 2),
            VertexFormat::Char4Norm => (1, 4),
            VertexFormat::Ushort2 => (2, 2),
            VertexFormat::Ushort4 => (2, 4),
            VertexFormat::Short2 => (2, 2),
            VertexFormat::Short4 => (2, 4),
            VertexFormat::Ushort2Norm => (2, 2),
            VertexFormat::Ushort4Norm => (2, 4),
            VertexFormat::Short2Norm => (2, 2),
            VertexFormat::Short4Norm => (2, 4),
            VertexFormat::Half2 => (2, 2),
            VertexFormat::Half4 => (2, 4),
            VertexFormat::Float => (4, 1),
            VertexFormat::Float2 => (4, 2),
            VertexFormat::Float3 => (4, 3),
            VertexFormat::Float4 => (4, 4),
            VertexFormat::Uint => (4, 1),
            VertexFormat::Uint2 => (4, 2),
            VertexFormat::Uint3 => (4, 3),
            VertexFormat::Uint4 => (4, 4),
            VertexFormat::Int => (4, 1),
            VertexFormat::Int2 => (4, 2),
            VertexFormat::Int3 => (4, 3),
            VertexFormat::Int4 => (4, 4),
        }
    }
    pub fn get_size(&self) -> u64 {
        let (component_size, components) = self.get_sizes();
        return (component_size * components) as u64;
    }
}

pub trait AsVertexFormats {
    fn as_vertex_formats() -> &'static [VertexFormat];
}

impl AsVertexFormats for f32 {
    fn as_vertex_formats() -> &'static [VertexFormat] {
        &[VertexFormat::Float]
    }
}

impl AsVertexFormats for Vec2 {
    fn as_vertex_formats() -> &'static [VertexFormat] {
        &[VertexFormat::Float2]
    }
}

impl AsVertexFormats for Vec3 {
    fn as_vertex_formats() -> &'static [VertexFormat] {
        &[VertexFormat::Float3]
    }
}

impl AsVertexFormats for Vec4 {
    fn as_vertex_formats() -> &'static [VertexFormat] {
        &[VertexFormat::Float4]
    }
}

impl AsVertexFormats for Mat4 {
    fn as_vertex_formats() -> &'static [VertexFormat] {
        &[
            VertexFormat::Float4,
            VertexFormat::Float4,
            VertexFormat::Float4,
            VertexFormat::Float4,
        ]
    }
}

impl AsVertexFormats for Color {
    fn as_vertex_formats() -> &'static [VertexFormat] {
        &[VertexFormat::Float4]
    }
}

impl AsVertexFormats for [f32; 2] {
    fn as_vertex_formats() -> &'static [VertexFormat] {
        &[VertexFormat::Float2]
    }
}

impl AsVertexFormats for [f32; 3] {
    fn as_vertex_formats() -> &'static [VertexFormat] {
        &[VertexFormat::Float3]
    }
}

impl AsVertexFormats for [f32; 4] {
    fn as_vertex_formats() -> &'static [VertexFormat] {
        &[VertexFormat::Float4]
    }
}
