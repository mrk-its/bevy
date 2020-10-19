use crate::{gl_call, renderer::*};
use bevy_asset::Handle;
use bevy_render::{
    pass::RenderPass,
    pipeline::{BindGroupDescriptorId, PipelineDescriptor},
    renderer::{BindGroupId, BufferId, RenderContext},
};
use std::ops::Range;
pub struct WebGL2RenderPass<'a> {
    pub render_context: &'a WebGL2RenderContext,
    pub pipeline_descriptor: Option<&'a PipelineDescriptor>,
    pub pipeline: Option<Handle<PipelineDescriptor>>,
}

impl<'a> WebGL2RenderPass<'a> {
    fn bind_buffer(&mut self, target: u32, buffer_id: BufferId) {
        let ctx = &self.render_context;
        let gl = &ctx.device.get_context();
        let buffers = &ctx.render_resource_context.resources.buffers.read();
        let buffer = buffers.get(&buffer_id).unwrap();
        gl_call!(gl.bind_buffer(target, Some(buffer)));
    }
}

impl<'a> RenderPass for WebGL2RenderPass<'a> {
    fn get_render_context(&self) -> &dyn RenderContext {
        self.render_context
    }

    fn set_vertex_buffer(&mut self, _start_slot: u32, buffer_id: BufferId, _offset: u64) {
        // TODO - start_slot and offset parameters
        let resources = &self.render_context.render_resource_context.resources;
        let pipelines = resources.pipelines.read();
        let pipeline_handle = self.pipeline.as_ref().unwrap();
        let pipeline = pipelines.get(&pipeline_handle).unwrap();

        let gl = &self.render_context.device.get_context();

        assert!(pipeline.vertex_buffer_descriptors.len() == 1);
        let vertex_buffer_descriptor = &pipeline.vertex_buffer_descriptors[0];

        gl.bind_vertex_array(Some(&pipeline.vao));

        self.bind_buffer(Gl::ARRAY_BUFFER, buffer_id);
        //self.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, buffer_id);

        for attr_descr in vertex_buffer_descriptor.attributes.iter() {
            // gl_call!(gl.enable_vertex_attrib_array(position_attribute_location as u32));
            gl_call!(gl.vertex_attrib_pointer_with_i32(
                attr_descr.attrib_location as u32,
                attr_descr.format.nr_of_components,
                attr_descr.format.format,
                attr_descr.format.normalized,
                vertex_buffer_descriptor.stride,
                attr_descr.offset,
            ));
        }
    }

    fn set_viewport(
        &mut self,
        _x: f32,
        _y: f32,
        _w: f32,
        _h: f32,
        _min_depth: f32,
        _max_depth: f32,
    ) {
        // log::info!(
        //     "render_pass: set_viewport {:?}",
        //     (x, y, w, h, min_depth, max_depth)
        // );
        panic!("not implemented");
        // self.render_pass
        //     .set_viewport(x, y, w, h, min_depth, max_depth);
    }

    fn set_stencil_reference(&mut self, _reference: u32) {
        // log::info!("render_pass: set_stencil_reference {:?}", reference);
        //self.render_pass.set_stencil_reference(reference);
    }

    fn set_index_buffer(&mut self, buffer_id: BufferId, _offset: u64) {
        // TODO - offset parameter
        // log::info!("render_pass: set_index_buffer, id: {:?}", buffer_id);
        self.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, buffer_id)
    }

    fn draw_indexed(&mut self, indices: Range<u32>, _base_vertex: i32, _instances: Range<u32>) {
        let ctx = &self.render_context;
        let gl = &ctx.device.get_context();
        gl_call!(gl.draw_elements_with_i32(Gl::TRIANGLES, indices.end as i32, Gl::UNSIGNED_INT, 0,));
        gl.bind_vertex_array(None);
    }

    fn draw(&mut self, _vertices: Range<u32>, _instances: Range<u32>) {
        // log::info!("render_pass: draw {:?}", (vertices, instances));
        // self.render_pass.draw(vertices, instances);
    }

    fn set_bind_group(
        &mut self,
        _index: u32,
        _bind_group_descriptor_id: BindGroupDescriptorId,
        bind_group_id: BindGroupId,
        dynamic_uniform_indices: Option<&[u32]>,
    ) {
        let resources = &self.render_context.render_resource_context.resources;
        let bind_groups = resources.bind_groups.read();
        let bind_group = bind_groups.get(&bind_group_id).unwrap();
        let buffers = resources.buffers.read();
        let textures = resources.textures.read();
        let gl = &self.render_context.device.get_context();
        for (i, binding) in bind_group.iter().enumerate() {
            let offset = dynamic_uniform_indices.map_or(0, |indices| indices[i]);
            match binding {
                crate::WebGL2RenderResourceBinding::Buffer {
                    binding_point,
                    buffer,
                    size,
                } => gl_call!(gl.bind_buffer_range_with_i32_and_i32(
                    Gl::UNIFORM_BUFFER,
                    *binding_point,
                    Some(buffers.get(&buffer).unwrap()),
                    offset as i32,
                    *size as i32,
                )),
                crate::WebGL2RenderResourceBinding::Texture {
                    texture,
                    texture_unit,
                } => {
                    // it seems it may not work
                    // (forcing texture_unit=1 do not work properly)
                    gl_call!(gl.active_texture(Gl::TEXTURE0 + texture_unit));
                    gl_call!(gl.bind_texture(Gl::TEXTURE_2D, Some(textures.get(texture).unwrap())))
                }
                crate::WebGL2RenderResourceBinding::Sampler(_) => {
                    // TODO
                }
            }
        }
    }

    fn set_pipeline(&mut self, pipeline_handle: &Handle<PipelineDescriptor>) {
        self.pipeline = Some(pipeline_handle.as_weak());

        let resources = &self.render_context.render_resource_context.resources;
        let programs = resources.programs.read();
        let pipelines = resources.pipelines.read();
        let pipeline = pipelines.get(&pipeline_handle).unwrap();
        let program = programs.get(&pipeline.shader_stages).unwrap();

        let ctx = self.render_context;
        let gl = &ctx.device.get_context();
        gl_call!(gl.use_program(Some(&program)));
    }
}
