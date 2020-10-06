use crate::renderer::{WebGL2RenderContext, WebGL2RenderResourceContext};
use bevy_app::prelude::*;
use bevy_ecs::{Resources, World};
use bevy_render::{
    render_graph::{
        DependentNodeStager, Edge, NodeId, RenderGraph, RenderGraphStager, ResourceSlots,
    },
    renderer::RenderResourceContext,
};
use bevy_window::{WindowCreated, WindowResized, Windows};
use std::{ops::Deref, sync::Arc};
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

type Gl = WebGl2RenderingContext;

use bevy_utils::HashMap;
use parking_lot::RwLock;

#[derive(Debug)]
pub struct Device {
    pub context: web_sys::WebGl2RenderingContext,
    //    pub context: web_sys::HtmlCanvasElement,
}
unsafe impl Send for Device {}
unsafe impl Sync for Device {}

pub struct WebGL2Renderer {
    pub device: Arc<crate::Device>,
}

impl std::default::Default for WebGL2Renderer {
    fn default() -> Self {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.query_selector("canvas").expect("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement =
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let context = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();
        crate::gl_call!(context.viewport(0, 0, 1280, 768));
        log::info!("UNIFORM_BLOCK_DATA_SIZE: {}", Gl::UNIFORM_BLOCK_DATA_SIZE);
        // return WebGL2Renderer {device: Arc::new(Device {context: canvas})};
        let device = Arc::new(Device { context });
        WebGL2Renderer { device }
    }
}
impl WebGL2Renderer {
    pub fn handle_window_created_events(&mut self, resources: &Resources) {
        // let mut render_resource_context = resources
        //     .get_mut::<Box<dyn RenderResourceContext>>()
        //     .unwrap();
        // let render_resource_context = render_resource_context
        //     .downcast_mut::<WebGL2RenderResourceContext>()
        //     .unwrap();
        // let windows = resources.get::<Windows>().unwrap();
        // let window_created_events = resources.get::<Events<WindowCreated>>().unwrap();
        // for window_created_event in self
        //     .window_created_event_reader
        //     .iter(&window_created_events)
        // {
        //     let window = windows
        //         .get(window_created_event.id)
        //         .expect("Received window created event for non-existent window");
        //     #[cfg(feature = "bevy_winit")]
        //     {
        //         let winit_windows = resources.get::<bevy_winit::WinitWindows>().unwrap();
        //         let winit_window = winit_windows.get_window(window.id).unwrap();
        //         let surface = unsafe { self.instance.create_surface(winit_window.deref()) };
        //         render_resource_context.set_window_surface(window.id, surface);
        //     }
        // }
    }

    pub fn run_graph(&mut self, world: &mut World, resources: &mut Resources) {
        let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();
        // stage nodes
        let mut stager = DependentNodeStager::loose_grouping();
        let stages = stager.get_stages(&render_graph).unwrap();
        let mut borrowed = stages.borrow(&mut render_graph);
        let mut render_resource_context = resources
            .get_mut::<Box<dyn RenderResourceContext>>()
            .unwrap();
        let render_resource_context = render_resource_context
            .downcast_mut::<WebGL2RenderResourceContext>()
            .unwrap();
        let node_outputs: Arc<RwLock<HashMap<NodeId, ResourceSlots>>> = Default::default();
        for stage in borrowed.iter_mut() {
            // TODO: sort jobs and slice by "amount of work" / weights
            // stage.jobs.sort_by_key(|j| j.node_states.len());

            let chunk_size = stage.jobs.len();
            for jobs_chunk in stage.jobs.chunks_mut(chunk_size) {
                let world = &*world;
                let render_resource_context = render_resource_context.clone();
                let node_outputs = node_outputs.clone();
                let mut render_context =
                    WebGL2RenderContext::new(self.device.clone(), render_resource_context);
                for job in jobs_chunk.iter_mut() {
                    for node_state in job.node_states.iter_mut() {
                        // bind inputs from connected node outputs
                        for (i, mut input_slot) in node_state.input_slots.iter_mut().enumerate() {
                            if let Edge::SlotEdge {
                                output_node,
                                output_index,
                                ..
                            } = node_state.edges.get_input_slot_edge(i).unwrap()
                            {
                                let node_outputs = node_outputs.read();
                                let outputs = if let Some(outputs) = node_outputs.get(output_node) {
                                    outputs
                                } else {
                                    panic!("node inputs not set")
                                };

                                let output_resource =
                                    outputs.get(*output_index).expect("output should be set");
                                input_slot.resource = Some(output_resource);
                            } else {
                                panic!("no edge connected to input")
                            }
                        }
                        log::debug!("node_state: {:?} update started", node_state.name);
                        node_state.node.update(
                            world,
                            resources,
                            &mut render_context,
                            &node_state.input_slots,
                            &mut node_state.output_slots,
                        );
                        node_outputs
                            .write()
                            .insert(node_state.id, node_state.output_slots.clone());
                        log::debug!("node_state: {:?} update finished", node_state.name);
                    }
                }
                //sender.send(render_context.finish()).unwrap();
            }
            // })
            // .unwrap();

            // let mut command_buffers = Vec::new();
            // for _i in 0..actual_thread_count {
            //     let command_buffer = receiver.recv().unwrap();
            //     if let Some(command_buffer) = command_buffer {
            //         command_buffers.push(command_buffer);
            //     }
            // }
        }
    }

    pub fn update(&mut self, world: &mut World, resources: &mut Resources) {
        self.handle_window_created_events(resources);
        self.run_graph(world, resources);
        let render_resource_context = resources.get::<Box<dyn RenderResourceContext>>().unwrap();
        render_resource_context.drop_all_swap_chain_textures();
        render_resource_context.clear_bind_groups();
    }
}