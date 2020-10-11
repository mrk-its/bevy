use crate::shader::ShaderLayout;

impl ShaderLayout {
    pub fn from_spirv(spirv_data: &[u32], bevy_conventions: bool) -> ShaderLayout {
        ShaderLayout {
            bind_groups: vec![],
            vertex_buffer_descriptors: vec![],
            entry_point: "".to_string(),
        }
    }
}
