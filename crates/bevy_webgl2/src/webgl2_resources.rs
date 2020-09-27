use crate::renderer::{
    compile_shader, link_program, Gl, WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture,
};
use bevy_asset::{Assets, Handle, HandleUntyped};
use bevy_render::shader::{ShaderSource, ShaderStage};
use bevy_render::{
    pipeline::{
        BindGroupDescriptor, BindGroupDescriptorId, BindingShaderStage, PipelineDescriptor,
        VertexBufferDescriptor,
    },
    renderer::{
        BindGroup, BindGroupId, BufferId, BufferInfo, BufferUsage, RenderResourceBinding,
        RenderResourceContext, RenderResourceId, SamplerId, TextureId,
    },
    shader::Shader,
    texture::{Extent3d, SamplerDescriptor, TextureDescriptor},
};
use bevy_utils::HashMap;
use bevy_window::{Window, WindowId};
use parking_lot::RwLock;
use std::{borrow::Cow, ops::Range, rc::Rc, sync::Arc};

#[derive(Debug)]
pub struct WebGL2Pipeline {
    pub vertex_shader: WebGlShader,
    pub fragment_shader: Option<WebGlShader>,
    pub program: WebGlProgram,
    pub vertex_buffer_descriptors: Vec<VertexBufferDescriptor>,
}

#[derive(Debug)]
pub struct WebGL2BindGroup {
    pub binding_point: u32,
    pub buffer: BufferId,
    pub size: u64,
}

#[derive(Default, Debug, Clone)]
pub struct WebGL2Resources {
    pub next_binding_point: Arc<RwLock<u32>>,
    pub binding_points: Arc<RwLock<HashMap<(u32, u32), u32>>>,
    pub bind_groups: Arc<RwLock<HashMap<BindGroupId, Vec<WebGL2BindGroup>>>>,
    pub buffer_info: Arc<RwLock<HashMap<BufferId, BufferInfo>>>,
    pub buffers: Arc<RwLock<HashMap<BufferId, WebGlBuffer>>>,
    pub texture_descriptors: Arc<RwLock<HashMap<TextureId, TextureDescriptor>>>,
    pub textures: Arc<RwLock<HashMap<TextureId, WebGlTexture>>>,
    pub asset_resources: Arc<RwLock<HashMap<(HandleUntyped, usize), RenderResourceId>>>,
    pub bind_group_layouts: Arc<RwLock<HashMap<BindGroupDescriptorId, BindGroupDescriptor>>>,
    pub pipelines: Arc<RwLock<HashMap<Handle<PipelineDescriptor>, WebGL2Pipeline>>>,
}

impl WebGL2Resources {
    pub fn get_or_create_binding_point(&self, binding_group_index: u32, index: u32) -> u32 {
        let mut binding_points = self.binding_points.write();
        *binding_points
            .entry((binding_group_index, index))
            .or_insert_with(|| {
                let mut last_binding_point = self.next_binding_point.write();
                let ret = *last_binding_point;
                *last_binding_point += 1;
                ret
            })
    }
}
