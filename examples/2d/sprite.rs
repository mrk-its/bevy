extern crate console_error_panic_hook;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_render::pass::ClearColor;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    #[cfg(target_arch = "wasm32")]
    console_log::init_with_level(log::Level::Info).expect("cannot initialize console_log");

    App::build()
        .add_resource(WindowDescriptor {
            width: 1280,
            height: 720,
            #[cfg(target_arch = "wasm32")]
            canvas: Some("#bevy-canvas".to_string()),
            ..Default::default()
        })
        //        .add_resource(ClearColor(Color::rgba(0.0, 0.0, 0.3, 1.0)))
        .add_default_plugins()
        .add_startup_system(setup.system())
        .add_system(main_system.system())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("assets/branding/icon.png").unwrap();
    commands.spawn(Camera2dComponents::default());
    for i in -1..=1 {
        let i = i as f32;
        commands.spawn(SpriteComponents {
            material: materials.add(texture_handle.into()),
            transform: Transform::from_translation(Vec3::new(i * 100.0, i * 50 as f32, 100.0 - i)),
            ..Default::default()
        });
    }
}

#[derive(Default)]
struct State {
    pub cnt: u32,
}

fn main_system(
    mut state: Local<State>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    mut query: Query<(&Sprite, &GlobalTransform)>,
) {
    log::info!(
        "############################### frame #{} #################################",
        state.cnt
    );
    // for (sprite, transform) in &mut query.iter() {
    //     log::info!("sprite: {:?}, transform: {:?}", sprite, transform);
    // }
    state.cnt += 1;
    if state.cnt >= 10 {
        app_exit_events.send(AppExit);
    }
}
