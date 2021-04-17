use crate::{
    render_graph::{Node, ResourceSlotInfo, ResourceSlots},
    renderer::{
        BufferId, BufferInfo, BufferMapMode, BufferUsage, RenderContext, RenderResourceId,
        RenderResourceType,
    },
    texture::TextureDescriptor,
};
use bevy_ecs::world::World;
use std::borrow::Cow;

pub struct TextureReadoutNode {
    descriptor: TextureDescriptor,
    texture_buffer: Option<BufferId>,
    texture_buffer_size: usize,
    read: Box<dyn Fn(&[u8], TextureDescriptor) + Send + Sync>,
}

impl TextureReadoutNode {
    pub const IN_TEXTURE: &'static str = "texture";

    pub fn new(
        descriptor: TextureDescriptor,
        read: impl Fn(&[u8], TextureDescriptor) + 'static + Send + Sync,
    ) -> Self {
        Self {
            descriptor,
            texture_buffer: None,
            texture_buffer_size: 0,
            read: Box::new(read),
        }
    }
}

impl Node for TextureReadoutNode {
    fn input(&self) -> &[ResourceSlotInfo] {
        static INPUT: &[ResourceSlotInfo] = &[ResourceSlotInfo {
            name: Cow::Borrowed(TextureReadoutNode::IN_TEXTURE),
            resource_type: RenderResourceType::Texture,
        }];
        INPUT
    }

    fn update(
        &mut self,
        _world: &World,
        render_context: &mut dyn RenderContext,
        input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        if let Some(RenderResourceId::Texture(texture)) = input.get(0) {
            let render_resource_context = render_context.resources_mut();
            let width = self.descriptor.size.width as usize;
            let aligned_width = render_resource_context.get_aligned_texture_size(width);
            let format_size = self.descriptor.format.pixel_size();

            let texture_buffer = match self.texture_buffer {
                Some(buffer) => buffer,
                None => {
                    let buffer_size = self.descriptor.size.volume() * format_size;
                    let buffer = render_resource_context.create_buffer(BufferInfo {
                        size: buffer_size,
                        buffer_usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
                        mapped_at_creation: false,
                    });
                    self.texture_buffer = Some(buffer);
                    self.texture_buffer_size = buffer_size;
                    buffer
                }
            };

            render_context.copy_texture_to_buffer(
                texture,
                [0, 0, 0],
                0,
                texture_buffer,
                0,
                (format_size * aligned_width) as u32,
                self.descriptor.size,
            );

            let render_resource_context = render_context.resources_mut();
            let reader = &self.read;
            render_resource_context.map_buffer(texture_buffer, BufferMapMode::Read);
            render_resource_context.read_mapped_buffer(
                texture_buffer,
                0..self.texture_buffer_size as u64,
                &|data, _renderer| reader(data.as_ref(), self.descriptor),
            );
            render_resource_context.unmap_buffer(texture_buffer);
        }
    }
}
