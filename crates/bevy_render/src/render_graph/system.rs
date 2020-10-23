use super::RenderGraph;
use crate::renderer::RenderResourceContext;
use bevy_ecs::{Resources, World};

pub fn render_graph_schedule_executor_system(world: &mut World, resources: &mut Resources) {
    if resources
        .get::<Option<Box<dyn RenderResourceContext>>>().unwrap().is_none()
    {
        return;
    }
    // run render graph systems
    let (mut system_schedule, commands) = {
        let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();
        (render_graph.take_schedule(), render_graph.take_commands())
    };

    commands.apply(world, resources);
    if let Some(schedule) = system_schedule.as_mut() {
        schedule.run(world, resources);
    }
    let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();
    if let Some(schedule) = system_schedule.take() {
        render_graph.set_schedule(schedule);
    }
}
