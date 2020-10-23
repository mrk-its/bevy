use crate::{
    render_graph::{Node, ResourceSlots},
    renderer::{RenderContext, SharedBuffers},
};
use bevy_ecs::{Resources, World};

#[derive(Debug, Default)]
pub struct SharedBuffersNode;

impl Node for SharedBuffersNode {
    fn update(
        &mut self,
        _world: &World,
        resources: &Resources,
        render_context: &mut dyn RenderContext,
        _input: &ResourceSlots,
        _output: &mut ResourceSlots,
    ) {
        let shared_buffers = resources.get::<Option<SharedBuffers>>().unwrap();
        if shared_buffers.is_none() {
            return;
        }
        let mut command_queue = shared_buffers.as_ref().unwrap().reset_command_queue();
        command_queue.execute(render_context);
    }
}
