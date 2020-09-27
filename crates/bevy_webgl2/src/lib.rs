// cfg_if::cfg_if! {
//     if #[cfg(target_arch = "wasm32")] {
pub mod renderer;
mod webgl2_render_pass;
mod webgl2_renderer;
mod webgl2_resources;
//mod webgl2_type_converter;

pub use webgl2_render_pass::*;
pub use webgl2_renderer::*;
//pub use webgl2_resources::*;
pub use webgl2_resources::*;

use bevy_app::prelude::*;
use bevy_ecs::{IntoQuerySystem, IntoThreadLocalSystem, Resources, World};
use bevy_render::renderer::{free_shared_buffers_system, RenderResourceContext, SharedBuffers};
use renderer::WebGL2RenderResourceContext;

#[derive(Default)]
pub struct WebGL2Plugin;

impl Plugin for WebGL2Plugin {
    fn build(&self, app: &mut AppBuilder) {
        let render_system = webgl2_render_system(app.resources_mut());
        app.add_system_to_stage(
            bevy_render::stage::RENDER,
            render_system.thread_local_system(),
        );
        // .add_system_to_stage(
        //     bevy_render::stage::POST_RENDER,
        //     free_shared_buffers_system.system(),
        // );
        // let resource_context = WebGL2RenderResourceContext::default();
        // app.add_resource::<Box<dyn RenderResourceContext>>(Box::new(resource_context));

        // app.add_resource(SharedBuffers::new(Box::new(WebGL2RenderResourceContext::default())));
    }
}

pub fn webgl2_render_system(resources: &mut Resources) -> impl FnMut(&mut World, &mut Resources) {
    let mut webgl2_renderer = WebGL2Renderer::default();
    let resource_context = WebGL2RenderResourceContext::new(webgl2_renderer.device.clone());
    resources.insert::<Box<dyn RenderResourceContext>>(Box::new(resource_context.clone()));
    resources.insert(SharedBuffers::new(Box::new(resource_context)));
    move |world, resources| {
        webgl2_renderer.update(world, resources);
    }
}
#[macro_export]
macro_rules! gl_call {
            ($device:ident . $func:ident ( $( $i:expr, )*) ) => {
                {
                    // log::info!("gl call: {}{:?}", stringify!($func), ($( $i ),*));
                    $device . $func( $( $i ),* )
                }
            };
            ($device:ident . $func:ident ( $( $i:expr), *) ) => {
                {
                    // log::info!("gl call: {}{:?}", stringify!($func), ($( $i ),*));
                    $device . $func( $( $i ),*, )
                }
            };
        }

//     }
// }
