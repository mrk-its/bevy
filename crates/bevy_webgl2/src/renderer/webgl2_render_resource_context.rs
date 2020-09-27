use super::{compile_shader, link_program, Gl, WebGlBuffer, WebGlProgram, WebGlShader};
use crate::{Device, WebGL2BindGroup, WebGL2Pipeline, WebGL2Resources};
use bevy_asset::{Assets, Handle, HandleUntyped};
use bevy_render::shader::{ShaderSource, ShaderStage};
use bevy_render::{
    pipeline::{
        BindGroupDescriptor, BindGroupDescriptorId, BindingShaderStage, PipelineDescriptor,
    },
    renderer::{
        BindGroup, BufferId, BufferInfo, BufferUsage, RenderResourceBinding, RenderResourceContext,
        RenderResourceId, SamplerId, TextureId,
    },
    shader::Shader,
    texture::{Extent3d, SamplerDescriptor, TextureDescriptor},
};
use bevy_utils::HashMap;
use bevy_window::{Window, WindowId};
use parking_lot::RwLock;
use std::{borrow::Cow, ops::Range, rc::Rc, sync::Arc};

#[derive(Clone, Debug)]
pub struct WebGL2RenderResourceContext {
    pub device: Arc<Device>,
    pub resources: WebGL2Resources,
}

unsafe impl Send for WebGL2RenderResourceContext {}
unsafe impl Sync for WebGL2RenderResourceContext {}

impl WebGL2RenderResourceContext {
    pub fn new(device: Arc<crate::Device>) -> Self {
        WebGL2RenderResourceContext {
            device,
            resources: WebGL2Resources::default(),
        }
    }
    pub fn add_buffer_info(&self, buffer: BufferId, info: BufferInfo) {
        self.resources.buffer_info.write().insert(buffer, info);
    }

    pub fn add_texture_descriptor(&self, texture: TextureId, descriptor: TextureDescriptor) {
        self.resources
            .texture_descriptors
            .write()
            .insert(texture, descriptor);
    }
    pub fn create_bind_group_layout(&self, descriptor: &BindGroupDescriptor) {
        if self.bind_group_descriptor_exists(descriptor.id) {
            return;
        };
        // log::info!(
        //     "resources: create bind group layoyt, descriptor: {:?}",
        //     descriptor
        // );
        self.resources
            .bind_group_layouts
            .write()
            .insert(descriptor.id, descriptor.clone());
    }
    pub fn compile_shader(&self, shader: &Shader) -> WebGlShader {
        let shader_type = match shader.stage {
            ShaderStage::Vertex => Gl::VERTEX_SHADER,
            ShaderStage::Fragment => Gl::FRAGMENT_SHADER,
            ShaderStage::Compute => panic!("compute shaders are not supported!"),
        };

        match &shader.source {
            ShaderSource::Glsl(source, Some(layout)) => {
                compile_shader(&self.device.context, shader_type, source).unwrap()
            }
            _ => {
                panic!("unsupported shader format");
            }
        }
    }
}

fn show_data(data: &[u8]) {
    // log::info!("len: {:?} data: {:?}", data.len(), data);
    // let mut f32_view = unsafe { std::mem::transmute::<&[u8], &[f32]>(data) };
    // let mut f32_view = &f32_view[0..data.len() / 4];
    // log::info!("f32 len: {:?}, data: {:?}", f32_view.len(), &f32_view);
}

impl RenderResourceContext for WebGL2RenderResourceContext {
    fn create_swap_chain(&self, _window: &Window) {
        //log::info!("create_swap_chain");
    }

    fn next_swap_chain_texture(&self, _window: &Window) -> TextureId {
        //log::info!("next_swap_chain_texture");
        TextureId::new()
    }

    fn drop_swap_chain_texture(&self, _render_resource: TextureId) {
        //log::info!("drop_swap_chain_texture");
    }

    fn drop_all_swap_chain_textures(&self) {
        // log::info!("drop_all_swap_chain_textures");
    }

    fn create_sampler(&self, _sampler_descriptor: &SamplerDescriptor) -> SamplerId {
        // log::info!("create_sampler");
        SamplerId::new()
    }

    fn create_texture(&self, texture_descriptor: TextureDescriptor) -> TextureId {
        // log::info!("create_texture: {:?}", texture_descriptor);
        let texture_id = TextureId::new();
        self.add_texture_descriptor(texture_id, texture_descriptor);
        let gl = &self.device.context;
        let texture = crate::gl_call!(gl.create_texture()).unwrap();
        self.resources.textures.write().insert(texture_id, texture);
        texture_id
    }

    fn create_buffer(&self, buffer_info: BufferInfo) -> BufferId {
        let id = BufferId::new();
        // log::info!("create_buffer: {:?} -> {:?}", buffer_info, id);
        assert!(buffer_info.size > 8); // TODO - remove
        let gl = &self.device.context;
        let buffer = crate::gl_call!(gl.create_buffer())
            .ok_or("failed to create_buffer")
            .unwrap();
        crate::gl_call!(gl.bind_buffer(Gl::UNIFORM_BUFFER, Some(&buffer)));
        crate::gl_call!(gl.buffer_data_with_i32(
            Gl::UNIFORM_BUFFER,
            buffer_info.size as i32,
            Gl::DYNAMIC_DRAW,
        ));
        self.resources.buffers.write().insert(id, buffer);
        self.add_buffer_info(id, buffer_info);
        id
    }

    fn write_mapped_buffer(
        &self,
        id: BufferId,
        range: Range<u64>,
        write: &mut dyn FnMut(&mut [u8], &dyn RenderResourceContext),
    ) {
        // log::info!("write_mapped_buffer {:?}, {:?}", id, range);
        let gl = &self.device.context;

        let mut data = vec![0; (range.end - range.start) as usize];

        write(&mut data, self);
        show_data(&data);

        let buffers = self.resources.buffers.read();
        let buffer = buffers.get(&id).unwrap();
        crate::gl_call!(gl.bind_buffer(Gl::COPY_WRITE_BUFFER, Some(&buffer)));
        crate::gl_call!(
            gl.buffer_sub_data_with_i32_and_u8_array_and_src_offset_and_length(
                Gl::COPY_WRITE_BUFFER,
                range.start as i32,
                &data,
                0,
                data.len() as u32,
            )
        );
    }

    fn map_buffer(&self, _id: BufferId) {
        // log::info!("map buffer {:?}", _id);
    }

    fn unmap_buffer(&self, _id: BufferId) {
        // log::info!("unmap buffer {:?}", _id);
    }

    fn create_buffer_with_data(&self, buffer_info: BufferInfo, data: &[u8]) -> BufferId {
        let id = BufferId::new();
        // log::info!("create_buffer_with_data: {:?} -> {:?}", buffer_info, id);
        // show_data(&data);
        let gl = &self.device.context;

        let buffer = gl.create_buffer().ok_or("failed to create_buffer").unwrap();
        let (target, usage) =
            if buffer_info.buffer_usage & BufferUsage::VERTEX == BufferUsage::VERTEX {
                (Gl::ARRAY_BUFFER, Gl::DYNAMIC_DRAW)
            } else if buffer_info.buffer_usage & BufferUsage::INDEX == BufferUsage::INDEX {
                (Gl::ELEMENT_ARRAY_BUFFER, Gl::DYNAMIC_DRAW)
            } else {
                (Gl::PIXEL_UNPACK_BUFFER, Gl::STATIC_COPY)
            };

        crate::gl_call!(gl.bind_buffer(target, Some(&buffer)));
        crate::gl_call!(gl.buffer_data_with_u8_array(target, &data, usage));

        self.resources.buffers.write().insert(id, buffer);
        self.add_buffer_info(id, buffer_info);
        id
    }

    fn create_shader_module(&self, shader_handle: Handle<Shader>, _shaders: &Assets<Shader>) {}

    fn remove_buffer(&self, buffer: BufferId) {
        self.resources.buffer_info.write().remove(&buffer);
    }

    fn remove_texture(&self, texture: TextureId) {
        self.resources.texture_descriptors.write().remove(&texture);
    }

    fn remove_sampler(&self, _sampler: SamplerId) {}

    fn set_asset_resource_untyped(
        &self,
        handle: HandleUntyped,
        render_resource: RenderResourceId,
        index: usize,
    ) {
        self.resources
            .asset_resources
            .write()
            .insert((handle, index), render_resource);
    }

    fn get_asset_resource_untyped(
        &self,
        handle: HandleUntyped,
        index: usize,
    ) -> Option<RenderResourceId> {
        self.resources
            .asset_resources
            .write()
            .get(&(handle, index))
            .cloned()
    }

    fn create_render_pipeline(
        &self,
        pipeline_handle: Handle<PipelineDescriptor>,
        pipeline_descriptor: &PipelineDescriptor,
        shaders: &Assets<Shader>,
    ) {
        // log::info!(
        //     "create render pipeline: {:?}, {:#?}",
        //     pipeline_handle,
        //     pipeline_descriptor
        // );
        let layout = pipeline_descriptor.get_layout().unwrap();
        for bind_group_descriptor in layout.bind_groups.iter() {
            self.create_bind_group_layout(&bind_group_descriptor);
        }
        let vertex_buffer_descriptors = pipeline_descriptor
            .layout
            .as_ref()
            .unwrap()
            .vertex_buffer_descriptors
            .clone();

        let vertex_shader = shaders
            .get(&pipeline_descriptor.shader_stages.vertex)
            .unwrap();
        // log::info!("shader: {:?}", vertex_shader);

        let vertex_shader = self.compile_shader(vertex_shader);
        let fragment_shader = pipeline_descriptor
            .shader_stages
            .fragment
            .map(|shader_handle| self.compile_shader(shaders.get(&shader_handle).unwrap()));
        let program = link_program(
            &self.device.context,
            &vertex_shader,
            fragment_shader.as_ref().unwrap(), // TODO - allow for optional fragment shader
        )
        .unwrap();

        let gl = &self.device.context;

        for bind_group in layout.bind_groups.iter() {
            for binding in bind_group.bindings.iter() {
                let block_index =
                    crate::gl_call!(gl.get_uniform_block_index(&program, &binding.name));
                if block_index == 4294967295 {
                    log::warn!("invalid block index for {:?}, skipping", &binding.name);
                    continue;
                }
                let binding_point = self
                    .resources
                    .get_or_create_binding_point(bind_group.index, binding.index);
                crate::gl_call!(gl.uniform_block_binding(&program, block_index, binding_point));
                let min_data_size = gl
                    .get_active_uniform_block_parameter(
                        &program,
                        block_index,
                        Gl::UNIFORM_BLOCK_DATA_SIZE,
                    )
                    .unwrap();
                // log::info!(
                //     "uniform_block_binding: name: {:?}, block_index: {:?}, binding_point: {:?}, min_data_size: {:?}",
                //     binding.name,
                //     block_index,
                //     binding_point,
                //     min_data_size,
                // );
            }
        }

        // let buffer = gl.create_buffer().ok_or("failed to create_buffer").unwrap();
        // gl.bind_buffer(Gl::UNIFORM_BUFFER, Some(&buffer));
        // gl.buffer_data_with_u8_array(Gl::UNIFORM_BUFFER, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, ], Gl::DYNAMIC_DRAW);
        // gl.bind_buffer_base(Gl::UNIFORM_BUFFER, binding_point, Some(&buffer));

        let pipeline = WebGL2Pipeline {
            vertex_shader,
            fragment_shader,
            program,
            vertex_buffer_descriptors,
        };
        self.resources
            .pipelines
            .write()
            .insert(pipeline_handle, pipeline);
    }

    fn create_bind_group(
        &self,
        bind_group_descriptor_id: BindGroupDescriptorId,
        bind_group: &BindGroup,
    ) {
        assert!(self.bind_group_descriptor_exists(bind_group_descriptor_id));
        let layouts = self.resources.bind_group_layouts.read();
        let buffers = self.resources.buffers.read();
        let bind_group_layout = layouts.get(&bind_group_descriptor_id).unwrap();
        let gl = &self.device.context;
        let mut bind_groups = self.resources.bind_groups.write();
        if bind_groups.get(&bind_group.id).is_some() {
            return;
        }
        // log::info!(
        //     "create_bind_group for bind_group: {:#?}, layout: {:#?}",
        //     bind_group,
        //     bind_group_layout
        // );
        let bind_group_vec: Vec<_> = bind_group
            .indexed_bindings
            .iter()
            .filter(|entry| entry.entry.get_buffer().is_some()) // TODO
            .map(|entry| {
                let binding_point = self
                    .resources
                    .get_or_create_binding_point(bind_group_layout.index, entry.index);
                let (buffer, size) = match &entry.entry {
                    RenderResourceBinding::Buffer { buffer, range, .. } => (buffer, range.end),
                    _ => panic!("not supported yet"),
                };
                WebGL2BindGroup {
                    binding_point,
                    buffer: *buffer,
                    size,
                }
            })
            .collect();
        // log::info!("result: {:#?}", bind_group_vec,);

        bind_groups.insert(bind_group.id, bind_group_vec);
    }

    fn create_shader_module_from_source(&self, _shader_handle: Handle<Shader>, _shader: &Shader) {
        // log::info!(
        //     "create_shader_module_from_source: handle: {:?}",
        //     _shader_handle
        // );
    }

    fn remove_asset_resource_untyped(&self, handle: HandleUntyped, index: usize) {
        self.resources
            .asset_resources
            .write()
            .remove(&(handle, index));
    }

    fn clear_bind_groups(&self) {}

    fn get_buffer_info(&self, buffer: BufferId) -> Option<BufferInfo> {
        self.resources.buffer_info.read().get(&buffer).cloned()
    }

    fn bind_group_descriptor_exists(
        &self,
        bind_group_descriptor_id: BindGroupDescriptorId,
    ) -> bool {
        return self
            .resources
            .bind_group_layouts
            .read()
            .contains_key(&bind_group_descriptor_id);
    }
}
