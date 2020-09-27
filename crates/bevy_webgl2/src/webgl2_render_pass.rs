use crate::renderer::*;
use bevy_asset::Handle;
use bevy_render::{
    pass::RenderPass,
    pipeline::{BindGroupDescriptorId, PipelineDescriptor, VertexFormat},
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
        let gl = &ctx.device.context;
        let buffers = &ctx.render_resource_context.resources.buffers.read();
        let buffer = buffers.get(&buffer_id).unwrap();
        crate::gl_call!(gl.bind_buffer(target, Some(buffer)));
    }
}

impl<'a> RenderPass for WebGL2RenderPass<'a> {
    fn get_render_context(&self) -> &dyn RenderContext {
        self.render_context
    }

    fn set_vertex_buffer(&mut self, _start_slot: u32, buffer_id: BufferId, _offset: u64) {
        let pipelines = self
            .render_context
            .render_resource_context
            .resources
            .pipelines
            .read();
        let pipeline = pipelines.get(&self.pipeline.unwrap()).unwrap();

        let gl = &self.render_context.device.context;

        assert!(pipeline.vertex_buffer_descriptors.len() == 1);
        let vertex_buffer_descriptor = &pipeline.vertex_buffer_descriptors[0];
        // TODO - start_slot and offset parameters
        // log::info!(
        //     "render_pass: set_vertex_buffer, id: {:?}, vertex_buffer_descriptor: {:?}",
        //     buffer_id,
        //     vertex_buffer_descriptor
        // );
        self.bind_buffer(Gl::ARRAY_BUFFER, buffer_id);

        // TODO - use Vertex Array Objects!
        // https://www.khronos.org/opengl/wiki/Vertex_Specification#Vertex_Array_Object
        // https://gamedev.stackexchange.com/questions/8042/whats-the-purpose-of-opengls-vertex-array-objects

        for attr_descr in vertex_buffer_descriptor.attributes.iter() {
            let position_attribute_location =
                crate::gl_call!(gl.get_attrib_location(&pipeline.program, &*attr_descr.name));
            if position_attribute_location < 0 {
                continue;
            }
            let (_, nr_of_components) = attr_descr.format.get_sizes();
            // TODO - move to utils
            let (format, normalize) = match &attr_descr.format {
                VertexFormat::Uchar2 => (Gl::BYTE, false),
                VertexFormat::Uchar4 => (Gl::BYTE, false),
                VertexFormat::Char2 => (Gl::BYTE, false),
                VertexFormat::Char4 => (Gl::BYTE, false),
                VertexFormat::Uchar2Norm => (Gl::BYTE, true),
                VertexFormat::Uchar4Norm => (Gl::BYTE, true),
                VertexFormat::Char2Norm => (Gl::BYTE, true),
                VertexFormat::Char4Norm => (Gl::BYTE, true),
                VertexFormat::Ushort2 => (Gl::UNSIGNED_SHORT, false),
                VertexFormat::Ushort4 => (Gl::UNSIGNED_SHORT, false),
                VertexFormat::Short2 => (Gl::SHORT, false),
                VertexFormat::Short4 => (Gl::SHORT, false),
                VertexFormat::Ushort2Norm => (Gl::UNSIGNED_SHORT, true),
                VertexFormat::Ushort4Norm => (Gl::UNSIGNED_SHORT, true),
                VertexFormat::Short2Norm => (Gl::SHORT, true),
                VertexFormat::Short4Norm => (Gl::SHORT, true),
                VertexFormat::Half2 => (Gl::HALF_FLOAT, false),
                VertexFormat::Half4 => (Gl::HALF_FLOAT, false),
                VertexFormat::Float => (Gl::FLOAT, false),
                VertexFormat::Float2 => (Gl::FLOAT, false),
                VertexFormat::Float3 => (Gl::FLOAT, false),
                VertexFormat::Float4 => (Gl::FLOAT, false),
                VertexFormat::Uint => (Gl::UNSIGNED_INT, false),
                VertexFormat::Uint2 => (Gl::UNSIGNED_INT, false),
                VertexFormat::Uint3 => (Gl::UNSIGNED_INT, false),
                VertexFormat::Uint4 => (Gl::UNSIGNED_INT, false),
                VertexFormat::Int => (Gl::INT, false),
                VertexFormat::Int2 => (Gl::INT, false),
                VertexFormat::Int3 => (Gl::INT, false),
                VertexFormat::Int4 => (Gl::INT, false),
            };
            crate::gl_call!(gl.vertex_attrib_pointer_with_i32(
                position_attribute_location as u32,
                nr_of_components,
                format,
                normalize,
                vertex_buffer_descriptor.stride as i32,
                attr_descr.offset as i32,
            ));
            crate::gl_call!(gl.enable_vertex_attrib_array(position_attribute_location as u32));
        }
    }

    fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32, min_depth: f32, max_depth: f32) {
        // log::info!(
        //     "render_pass: set_viewport {:?}",
        //     (x, y, w, h, min_depth, max_depth)
        // );
        panic!("");
        // self.render_pass
        //     .set_viewport(x, y, w, h, min_depth, max_depth);
    }

    fn set_stencil_reference(&mut self, reference: u32) {
        // log::info!("render_pass: set_stencil_reference {:?}", reference);
        //self.render_pass.set_stencil_reference(reference);
    }

    fn set_index_buffer(&mut self, buffer_id: BufferId, _offset: u64) {
        // TODO - offset parameter
        // log::info!("render_pass: set_index_buffer, id: {:?}", buffer_id);
        self.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, buffer_id)
    }

    fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        // log::info!(
        //     "render_pass: draw_indexed {:?} {:?}, {:?}",
        //     indices,
        //     base_vertex,
        //     instances
        // );

        let ctx = &self.render_context;
        let gl = &ctx.device.context;
        crate::gl_call!(gl.draw_elements_with_i32(
            Gl::TRIANGLES,
            indices.end as i32,
            Gl::UNSIGNED_INT,
            0
        ));

        // self.render_pass
        //     .draw_indexed(indices, base_vertex, instances);
    }

    fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        // log::info!("render_pass: draw {:?}", (vertices, instances));
        // self.render_pass.draw(vertices, instances);
    }

    fn set_bind_group(
        &mut self,
        index: u32,
        bind_group_descriptor_id: BindGroupDescriptorId,
        bind_group_id: BindGroupId,
        dynamic_uniform_indices: Option<&[u32]>,
    ) {
        let resources = &self.render_context.render_resource_context.resources;
        let bind_groups = resources.bind_groups.read();
        let bind_group = bind_groups.get(&bind_group_id).unwrap();
        let buffers = resources.buffers.read();
        // log::info!(
        //     "render_pass: set_bind_group {:?} {:?}, {:?}, {:?} for {:#?}",
        //     index,
        //     bind_group_descriptor_id,
        //     bind_group_id,
        //     dynamic_uniform_indices,
        //     bind_group,
        // );
        let gl = &self.render_context.device.context;
        for (i, binding) in bind_group.iter().enumerate() {
            let offset = dynamic_uniform_indices.map_or(0, |indices| indices[i]);
            crate::gl_call!(gl.bind_buffer_range_with_i32_and_i32(
                Gl::UNIFORM_BUFFER,
                binding.binding_point,
                Some(buffers.get(&binding.buffer).unwrap()),
                offset as i32,
                binding.size as i32,
            ))
        }
    }

    fn set_pipeline(&mut self, pipeline_handle: Handle<PipelineDescriptor>) {
        self.pipeline = Some(pipeline_handle);
        let pipelines = self
            .render_context
            .render_resource_context
            .resources
            .pipelines
            .read();
        let pipeline = pipelines.get(&pipeline_handle).unwrap();
        // log::info!("render_pass: set_pipeline: {:?}", pipeline_handle);
        let ctx = &self.render_context;
        let gl = &ctx.device.context;

        crate::gl_call!(gl.use_program(Some(&pipeline.program)));
    }
}
