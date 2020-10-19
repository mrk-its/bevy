use super::{compile_shader, gl_vertex_format, link_program, reflect_layout, Gl, WebGlShader};
use crate::{
    gl_call, Device, GlVertexBufferDescripror, WebGL2Pipeline, WebGL2RenderResourceBinding,
    WebGL2Resources,
};
use bevy_asset::{Assets, Handle, HandleUntyped};
use bevy_render::{
    pipeline::{
        BindGroupDescriptor, BindGroupDescriptorId, BindType, BindingDescriptor, DynamicBinding,
        PipelineDescriptor, PipelineLayout, VertexBufferDescriptors,
    },
    renderer::{
        BindGroup, BufferId, BufferInfo, BufferUsage, RenderResourceBinding, RenderResourceContext,
        RenderResourceId, SamplerId, TextureId,
    },
    shader::{Shader, ShaderSource, ShaderStage, ShaderStages},
    texture::{SamplerDescriptor, TextureDescriptor},
};
use bevy_utils::HashMap;
use bevy_window::Window;
use parking_lot::RwLock;
use std::{ops::Range, sync::Arc};

#[derive(Clone)]
pub struct WebGL2RenderResourceContext {
    pub device: Arc<Device>,
    pub resources: WebGL2Resources,
    pub pipeline_descriptors: Arc<RwLock<HashMap<Handle<PipelineDescriptor>, PipelineDescriptor>>>,
    initialized: bool,
}

unsafe impl Send for WebGL2RenderResourceContext {}
unsafe impl Sync for WebGL2RenderResourceContext {}

impl WebGL2RenderResourceContext {
    pub fn new(device: Arc<crate::Device>) -> Self {
        WebGL2RenderResourceContext {
            device,
            resources: WebGL2Resources::default(),
            pipeline_descriptors: Default::default(),
            initialized: false,
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
            ShaderSource::Glsl(source) => {
                compile_shader(&self.device.get_context(), shader_type, source).unwrap()
            }
            _ => {
                panic!("unsupported shader format");
            }
        }
    }

    pub fn initialize(&mut self, winit_window: &winit::window::Window) {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowExtWebSys;

            let size = winit_window.inner_size();
            let gl = winit_window
                .canvas()
                .get_context("webgl2")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::WebGl2RenderingContext>()
                .unwrap();

            // let ret = gl
            //     .get_framebuffer_attachment_parameter(
            //         Gl::FRAMEBUFFER,
            //         Gl::BACK,
            //         Gl::FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING,
            //     )
            //     .unwrap()
            //     .as_f64()
            //     .unwrap() as u32;

            // log::info!(
            //     "FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING linear: {:?}, srgb: {:?}",
            //     ret == Gl::LINEAR,
            //     ret == Gl::SRGB
            // );

            gl.viewport(0, 0, size.width as i32, size.height as i32);
            gl.enable(Gl::BLEND);
            gl.enable(Gl::CULL_FACE);
            gl.enable(Gl::DEPTH_TEST);
            gl.blend_func(Gl::ONE, Gl::ONE_MINUS_SRC_ALPHA);

            self.device.set_context(gl);
            self.initialized = true;
        }
    }
}

impl RenderResourceContext for WebGL2RenderResourceContext {
    fn is_ready(&self) -> bool {
        self.initialized
    }
    fn flush(&self) {
        let gl = &self.device.get_context();
        gl_call!(gl.flush());
    }

    fn reflect_pipeline_layout(
        &self,
        shaders: &Assets<Shader>,
        shader_stages: &ShaderStages,
        _enforce_bevy_conventions: bool,
        vertex_buffer_descriptors: Option<&VertexBufferDescriptors>,
        dynamic_bindings: &[DynamicBinding],
    ) -> PipelineLayout {
        log::info!("reflecting shader layoyut!");
        let gl_shaders: Vec<WebGlShader> = shader_stages
            .iter()
            .map(|handle| self.compile_shader(shaders.get(&handle).unwrap()))
            .collect();

        let program =
            link_program(&*self.device.get_context(), &gl_shaders).expect("WebGL program");

        log::info!("program compiled!");

        let gl = &self.device.get_context();

        let mut layout = reflect_layout(&*gl, &program);
        log::info!("reflected layoyt: {:#?}", layout);
        self.resources
            .programs
            .write()
            .insert(shader_stages.clone(), program);

        if !dynamic_bindings.is_empty() {
            // set binding uniforms to dynamic if render resource bindings use dynamic
            for bind_group in layout.bind_groups.iter_mut() {
                let mut binding_changed = false;
                for binding in bind_group.bindings.iter_mut() {
                    if dynamic_bindings
                        .iter()
                        .any(|dynamic_binding| dynamic_binding.name == binding.name)
                    {
                        if let BindType::Uniform {
                            ref mut dynamic, ..
                        } = binding.bind_type
                        {
                            *dynamic = true;
                            binding_changed = true;
                        }
                    }
                }

                if binding_changed {
                    bind_group.update_id();
                }
            }
        }
        if let Some(vertex_buffer_descriptors) = vertex_buffer_descriptors {
            layout.sync_vertex_buffer_descriptors(vertex_buffer_descriptors);
        }
        layout
    }

    fn render_pipeline_exists(&self, pipeline_handle: &Handle<PipelineDescriptor>) -> bool {
        self.is_ready()
            && self
                .pipeline_descriptors
                .read()
                .contains_key(&pipeline_handle)
    }

    fn get_aligned_texture_size(&self, data_size: usize) -> usize {
        data_size
    }

    fn get_aligned_uniform_size(&self, size: usize, uniform_name: Option<&str>) -> usize {
        if let Some(name) = uniform_name {
            let pipeline_descriptors = self.pipeline_descriptors.read();
            // TODO: should we iterate over all pipeline descriptors?
            // PERF: direct create name -> BindingDescriptor hashmap
            for (_, descr) in pipeline_descriptors.iter() {
                if let Some(layout) = &descr.layout {
                    let binding = layout
                        .bind_groups
                        .iter()
                        .flat_map(|c| c.bindings.iter())
                        .find(|binding| binding.name == name);
                    if let Some(BindingDescriptor {
                        bind_type: BindType::Uniform { property, .. },
                        ..
                    }) = binding
                    {
                        return size.max(16).max(property.get_size() as usize);
                    }
                }
            }
        }
        size.max(16)
    }

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
        let texture_id = TextureId::new();
        self.add_texture_descriptor(texture_id, texture_descriptor);
        let gl = &self.device.get_context();
        let texture = crate::gl_call!(gl.create_texture()).unwrap();
        self.resources.textures.write().insert(texture_id, texture);
        texture_id
    }

    fn create_buffer(&self, buffer_info: BufferInfo) -> BufferId {
        let id = BufferId::new();
        let gl = &self.device.get_context();
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
        let gl = &self.device.get_context();

        let mut data = vec![0; (range.end - range.start) as usize];

        write(&mut data, self);

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
        let gl = &self.device.get_context();
        let buffer = gl_call!(gl.create_buffer())
            .ok_or("failed to create_buffer")
            .unwrap();
        let (target, usage) =
            if buffer_info.buffer_usage & BufferUsage::VERTEX == BufferUsage::VERTEX {
                (Gl::ARRAY_BUFFER, Gl::DYNAMIC_DRAW)
            } else if buffer_info.buffer_usage & BufferUsage::INDEX == BufferUsage::INDEX {
                (Gl::ELEMENT_ARRAY_BUFFER, Gl::DYNAMIC_DRAW)
            } else {
                (Gl::PIXEL_UNPACK_BUFFER, Gl::STREAM_DRAW)
            };
        crate::gl_call!(gl.bind_buffer(target, Some(&buffer)));
        crate::gl_call!(gl.buffer_data_with_u8_array(target, &data, usage));

        self.resources.buffers.write().insert(id, buffer);
        self.add_buffer_info(id, buffer_info);
        id
    }

    fn create_shader_module(&self, _shader_handle: &Handle<Shader>, _shaders: &Assets<Shader>) {}

    fn remove_buffer(&self, buffer: BufferId) {
        let gl = &self.device.get_context();
        let mut buffers = self.resources.buffers.write();
        let mut buffer_infos = self.resources.buffer_info.write();
        crate::gl_call!(gl.delete_buffer(Some(&buffers.remove(&buffer).unwrap())));
        buffer_infos.remove(&buffer);
    }

    fn remove_texture(&self, texture: TextureId) {
        let gl = &self.device.get_context();
        let mut texture_descriptors = self.resources.texture_descriptors.write();
        let mut textures = self.resources.textures.write();
        let gl_texture = textures.remove(&texture).unwrap();
        gl.delete_texture(Some(&gl_texture));
        texture_descriptors.remove(&texture);
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
        source_pipeline_handle: Handle<PipelineDescriptor>,
        pipeline_handle: Handle<PipelineDescriptor>,
        pipeline_descriptor: &PipelineDescriptor,
        _shaders: &Assets<Shader>,
    ) {
        // log::info!(
        //     "create render pipeline: source_handle: {:?} handle: {:?}, descriptor: {:#?}",
        //     source_pipeline_handle,
        //     pipeline_handle,
        //     pipeline_descriptor
        // );
        let layout = pipeline_descriptor.get_layout().unwrap();
        self.pipeline_descriptors
            .write()
            .insert(source_pipeline_handle, pipeline_descriptor.clone());
        for bind_group_descriptor in layout.bind_groups.iter() {
            self.create_bind_group_layout(&bind_group_descriptor);
        }
        let vertex_buffer_descriptors = pipeline_descriptor
            .layout
            .as_ref()
            .unwrap()
            .vertex_buffer_descriptors
            .clone();
        let gl = &self.device.get_context();

        let programs = self.resources.programs.read();
        let program = programs.get(&pipeline_descriptor.shader_stages).unwrap();
        log::info!("found compiled program: {:?}", program);
        gl.use_program(Some(&program));
        log::info!("start binding");
        for bind_group in layout.bind_groups.iter() {
            for binding in bind_group.bindings.iter() {
                let block_index =
                    gl_call!(gl.get_uniform_block_index(&program, &binding.name));
                log::info!("trying to bind {:?}", binding.name);
                if (block_index as i32) < 0 {
                    log::info!("invalid block index for {:?}, skipping", &binding.name);
                    if let Some(uniform_location) = gl.get_uniform_location(&program, &binding.name)
                    {
                        log::info!("found uniform location: {:?}", uniform_location);
                        if let BindType::SampledTexture { .. } = binding.bind_type {
                            let texture_unit = self
                                .resources
                                .get_or_create_texture_unit(bind_group.index, binding.index);
                            log::info!("here");
                            gl_call!(gl.uniform1i(Some(&uniform_location), texture_unit as i32));
                            log::info!(
                                "found texture uniform {:?}, binding to unit {:?}",
                                binding.name,
                                texture_unit
                            );
                        } else {
                            panic!("use non-block uniforms expected only for textures");
                        }
                    } else {
                        log::info!("can't bind {:?}", binding.name);
                    }
                    continue;
                }
                let binding_point = self
                    .resources
                    .get_or_create_binding_point(bind_group.index, binding.index);
                crate::gl_call!(gl.uniform_block_binding(&program, block_index, binding_point));
                let _min_data_size = gl_call!(gl.get_active_uniform_block_parameter(
                    &program,
                    block_index,
                    Gl::UNIFORM_BLOCK_DATA_SIZE,
                ))
                .unwrap();
                log::info!(
                    "uniform_block_binding: name: {:?}, block_index: {:?}, binding_point: {:?}, min_data_size: {:?}",
                    binding.name,
                    block_index,
                    binding_point,
                    _min_data_size,
                );
            }
        }
        log::info!("done binding");

        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(&vao));

        let vertex_buffer_descriptors = vertex_buffer_descriptors
            .iter()
            .map(|vertex_buffer_descriptor| {
                GlVertexBufferDescripror::from(gl, program, vertex_buffer_descriptor)
            })
            .collect();
        gl.bind_vertex_array(None);

        // let buffer = gl.create_buffer().ok_or("failed to create_buffer").unwrap();
        // gl.bind_buffer(Gl::UNIFORM_BUFFER, Some(&buffer));
        // gl.buffer_data_with_u8_array(Gl::UNIFORM_BUFFER, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, ], Gl::DYNAMIC_DRAW);
        // gl.bind_buffer_base(Gl::UNIFORM_BUFFER, binding_point, Some(&buffer));

        let pipeline = WebGL2Pipeline {
            shader_stages: pipeline_descriptor.shader_stages.clone(),
            vao,
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
        let bind_group_layout = layouts.get(&bind_group_descriptor_id).unwrap();
        let _gl = &self.device.get_context();
        let mut bind_groups = self.resources.bind_groups.write();
        if bind_groups.get(&bind_group.id).is_some() {
            return;
        }
        let bind_group_vec: Vec<_> = bind_group
            .indexed_bindings
            .iter()
            .filter(|entry| {
                entry.entry.get_buffer().is_some() || entry.entry.get_texture().is_some()
            }) // TODO
            .map(|entry| match &entry.entry {
                RenderResourceBinding::Buffer { buffer, range, .. } => {
                    let binding_point = self
                        .resources
                        .get_or_create_binding_point(bind_group_layout.index, entry.index);
                    WebGL2RenderResourceBinding::Buffer {
                        binding_point,
                        buffer: *buffer,
                        size: range.end - range.start,
                    }
                }
                RenderResourceBinding::Texture(texture) => {
                    let texture_unit = self
                        .resources
                        .get_or_create_texture_unit(bind_group_layout.index, entry.index);
                    WebGL2RenderResourceBinding::Texture {
                        texture: *texture,
                        texture_unit,
                    }
                }
                RenderResourceBinding::Sampler(sampler) => {
                    WebGL2RenderResourceBinding::Sampler(*sampler)
                }
            })
            .collect();
        bind_groups.insert(bind_group.id, bind_group_vec);
    }

    fn create_shader_module_from_source(&self, _shader_handle: &Handle<Shader>, _shader: &Shader) {}

    fn remove_asset_resource_untyped(&self, handle: HandleUntyped, index: usize) {
        self.resources
            .asset_resources
            .write()
            .remove(&(handle, index));
    }

    fn clear_bind_groups(&self) {
        self.resources.bind_groups.write().clear();
    }

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
