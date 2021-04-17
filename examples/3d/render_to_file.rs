use bevy::{
    prelude::*,
    render::{
        camera::CameraProjection,
        render_graph::{
            base::{node::MAIN_PASS, BaseRenderGraphConfig},
            RenderGraph, TextureNode, TextureReadoutNode,
        },
        texture::{
            Extent3d, SamplerDescriptor, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsage,
        },
    },
    window::WindowId,
};

fn hello() {
    println!("hello!");
}

fn main() {
    let mut builder = App::build();
    builder.insert_resource(Msaa { samples: 1 });
    builder.add_plugins(MinimalPlugins);
    builder.add_plugin(bevy::log::LogPlugin::default());
    builder.add_plugin(bevy::transform::TransformPlugin::default());
    builder.add_plugin(bevy::diagnostic::DiagnosticsPlugin::default());
    builder.add_plugin(bevy::input::InputPlugin::default());
    builder.add_plugin(bevy::window::WindowPlugin::default());
    builder.add_plugin(bevy::asset::AssetPlugin::default());
    builder.add_plugin(bevy::scene::ScenePlugin::default());
    builder.add_plugin(bevy::gltf::GltfPlugin::default());

    // #[cfg(feature = "bevy_render")]
    builder.add_plugin(bevy::render::RenderPlugin {
        base_render_graph_config: Some(BaseRenderGraphConfig {
            add_2d_camera: false,
            add_3d_camera: true,
            add_main_depth_texture: false,
            add_main_pass: true,
            connect_main_pass_to_swapchain: false,
            connect_main_pass_to_main_depth_texture: false,
            create_swap_chain: false,
        }),
    });

    builder.add_plugin(bevy::pbr::PbrPlugin::default());

    // #[cfg(feature = "bevy_winit")]
    // builder.add_plugin(bevy::winit::WinitPlugin::default());

    #[cfg(feature = "bevy_wgpu")]
    builder.add_plugin(bevy::wgpu::WgpuPlugin::default());
    builder.add_system(hello.system());
    builder.add_startup_system(setup.system()).run();
}

pub const TEXTURE_NODE: &str = "texure_node";
pub const DEPTH_TEXTURE_NODE: &str = "depth_texure_node";

fn setup(
    mut commands: Commands,
    mut render_graph: ResMut<RenderGraph>,
    asset_server: Res<AssetServer>
) {
    commands.spawn_scene(asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"));
    commands
        .spawn_bundle(PointLightBundle {
            transform: Transform::from_xyz(3.0, 5.0, 3.0),
            ..Default::default()
        });

    let size = Extent3d::new(4096, 8192, 1);

    // camera
    let mut camera_bundle = PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        ..Default::default()
    };
    camera_bundle.camera.window = WindowId::new();
    let camera_projection = &mut camera_bundle.perspective_projection;
    camera_projection.update(size.width as f32, size.height as f32);
    camera_bundle.camera.projection_matrix = camera_projection.get_projection_matrix();
    camera_bundle.camera.depth_calculation = camera_projection.depth_calculation();

    commands.spawn_bundle(camera_bundle);

    render_graph.add_node(
        TEXTURE_NODE,
        TextureNode::new(
            TextureDescriptor {
                size: size.clone(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::COPY_SRC,
            },
            Some(SamplerDescriptor::default()),
            None,
        ),
    );
    render_graph.add_node(
        DEPTH_TEXTURE_NODE,
        TextureNode::new(
            TextureDescriptor {
                size: size.clone(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsage::OUTPUT_ATTACHMENT,
            },
            None,
            None,
        ),
    );

    render_graph
        .add_slot_edge(
            TEXTURE_NODE,
            TextureNode::TEXTURE,
            MAIN_PASS,
            "color_attachment",
        )
        .unwrap();
    render_graph
        .add_slot_edge(DEPTH_TEXTURE_NODE, TextureNode::TEXTURE, MAIN_PASS, "depth")
        .unwrap();

    // create a closure to save the file
    let file_saver = |data: &[u8], descriptor: TextureDescriptor| {
        let mut v = Vec::from(data);
        &v.chunks_exact_mut(4).for_each(|c| {
            let t = c[0];
            c[0] = c[2];
            c[2] = t;
        });

        match descriptor.format {
            TextureFormat::Bgra8UnormSrgb => {
                bevy::render::image::save_buffer(
                    "helmet.in_progress.jpg",
                    &v,
                    descriptor.size.width,
                    descriptor.size.height,
                    bevy::render::image::ColorType::Rgba8,
                )
                .unwrap();
                std::fs::rename("helmet.in_progress.jpg", "helmet.jpg").unwrap();
            }
            _ => {}
        }
    };

    // add a texture readout node
    render_graph.add_node(
        "save_to_file_readout",
        TextureReadoutNode::new(
            TextureDescriptor {
                size,
                format: Default::default(),
                ..Default::default()
            },
            file_saver,
        ),
    );
    // set the correct texture as the input to the readout node
    render_graph
        .add_slot_edge(
            TEXTURE_NODE,
            TextureNode::TEXTURE,
            "save_to_file_readout",
            TextureReadoutNode::IN_TEXTURE,
        )
        .unwrap();

    render_graph
        .add_node_edge(TEXTURE_NODE, "save_to_file_readout")
        .unwrap();
    render_graph
        .add_node_edge(DEPTH_TEXTURE_NODE, "save_to_file_readout")
        .unwrap();
    render_graph
        .add_node_edge("save_to_file_readout", MAIN_PASS)
        .unwrap();
}
