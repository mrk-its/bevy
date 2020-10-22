use super::{Gl, WebGL2RenderResourceContext};

use crate::WebGL2RenderPass;
use bevy_render::{
    pass::{LoadOp, PassDescriptor, RenderPass},
    renderer::{BufferId, RenderContext, RenderResourceBindings, RenderResourceContext, TextureId},
    texture::Extent3d,
};
use std::sync::Arc;

pub struct WebGL2RenderContext {
    pub device: Arc<crate::Device>,
    pub render_resource_context: WebGL2RenderResourceContext,
}

impl WebGL2RenderContext {
    pub fn new(device: Arc<crate::Device>, resources: WebGL2RenderResourceContext) -> Self {
        WebGL2RenderContext {
            device,
            render_resource_context: resources,
        }
    }

    /// Consume this context, finalize the current CommandEncoder (if it exists), and take the current WebGL2Resources.
    /// This is intended to be called from a worker thread right before synchronizing with the main thread.
    pub fn finish(&mut self) {}
}

impl RenderContext for WebGL2RenderContext {
    fn copy_buffer_to_buffer(
        &mut self,
        source_buffer: BufferId,
        source_offset: u64,
        destination_buffer: BufferId,
        destination_offset: u64,
        size: u64,
    ) {
        let gl = &self.device.get_context();
        let resources = &self.render_resource_context.resources;
        let buffers = resources.buffers.read();

        let src = buffers.get(&source_buffer).unwrap();
        let dst = buffers.get(&destination_buffer).unwrap();
        crate::gl_call!(gl.bind_buffer(Gl::COPY_READ_BUFFER, Some(&src)));
        crate::gl_call!(gl.bind_buffer(Gl::COPY_WRITE_BUFFER, Some(&dst)));
        crate::gl_call!(gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
            Gl::COPY_READ_BUFFER,
            Gl::COPY_WRITE_BUFFER,
            source_offset as i32,
            destination_offset as i32,
            size as i32,
        ));
    }

    fn copy_buffer_to_texture(
        &mut self,
        source_buffer: BufferId,
        source_offset: u64,
        _source_bytes_per_row: u32,
        destination_texture: TextureId,
        _destination_origin: [u32; 3],
        _destination_mip_level: u32,
        size: Extent3d,
    ) {
        let gl = &self.device.get_context();
        let resources = &self.render_resource_context.resources;
        let textures = resources.textures.read();
        let texture = textures.get(&destination_texture).unwrap();
        let buffers = resources.buffers.read();
        let buffer = buffers.get(&source_buffer).unwrap();

        // TODO
        // let tex_internal_format = match &texture_descriptor.format {
        //     TextureFormat::Rgba8UnormSrgb => Gl::RGBA8_SNORM,
        //     TextureFormat::Rgba8Snorm => Gl::RGBA8_SNORM,
        //     _ => Gl::RGBA,
        // };

        crate::gl_call!(gl.bind_buffer(Gl::PIXEL_UNPACK_BUFFER, Some(&buffer)));
        crate::gl_call!(gl.bind_texture(Gl::TEXTURE_2D, Some(&texture)));

        crate::gl_call!(
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_f64(
                Gl::TEXTURE_2D,
                0,                       //destination_mip_level as i32,
                Gl::SRGB8_ALPHA8 as i32, // TODO
                size.width as i32,
                size.height as i32,
                0,
                Gl::RGBA,
                Gl::UNSIGNED_BYTE,
                source_offset as f64,
            )
        )
        .expect("tex image");
        crate::gl_call!(gl.generate_mipmap(Gl::TEXTURE_2D));

        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::NEAREST as i32);
        gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::NEAREST as i32);
        // crate::gl_call!(gl.tex_parameteri(
        //     Gl::TEXTURE_2D,
        //     Gl::TEXTURE_WRAP_S,
        //     Gl::CLAMP_TO_EDGE as i32
        // ));
        // crate::gl_call!(gl.tex_parameteri(
        //     Gl::TEXTURE_2D,
        //     Gl::TEXTURE_WRAP_T,
        //     Gl::CLAMP_TO_EDGE as i32
        // ));

        // crate::gl_call!(gl.tex_parameteri(
        //     Gl::TEXTURE_2D,
        //     Gl::TEXTURE_MAG_FILTER,
        //     Gl::NEAREST as i32,
        // ));

        // crate::gl_call!(gl.tex_parameteri(
        //     Gl::TEXTURE_2D,
        //     Gl::TEXTURE_MIN_FILTER,
        //     Gl::NEAREST as i32,
        // ));
    }

    fn resources(&self) -> &dyn RenderResourceContext {
        &self.render_resource_context
    }

    fn resources_mut(&mut self) -> &mut dyn RenderResourceContext {
        &mut self.render_resource_context
    }

    fn begin_pass(
        &mut self,
        pass_descriptor: &PassDescriptor,
        _render_resource_bindings: &RenderResourceBindings,
        run_pass: &mut dyn Fn(&mut dyn RenderPass),
    ) {
        if let LoadOp::Clear(c) = pass_descriptor.color_attachments[0].ops.load {
            let gl = &self.device.get_context();
            crate::gl_call!(gl.clear_color(c.r(), c.g(), c.b(), c.a()));
            crate::gl_call!(gl.clear(Gl::COLOR_BUFFER_BIT | Gl::DEPTH_BUFFER_BIT));
        }
        let mut render_pass = WebGL2RenderPass {
            render_context: self,
            pipeline_descriptor: None,
            pipeline: None,
        };
        run_pass(&mut render_pass);
    }
}
