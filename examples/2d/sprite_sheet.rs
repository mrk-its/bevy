extern crate console_error_panic_hook;
use bevy::app::AppExit;
use bevy::prelude::*;

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
        .add_default_plugins()
        .add_startup_system(setup.system())
        .add_system(animate_sprite_system.system())
        .run();
}

fn animate_sprite_system(
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (timer, mut sprite, texture_atlas_handle) in &mut query.iter() {
        if timer.finished {
            let texture_atlas = texture_atlases.get(&texture_atlas_handle).unwrap();
            sprite.index = ((sprite.index as usize + 1) % 7) as u32;
            if sprite.index == 0 {
                // app_exit_events.send(AppExit);
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<Texture>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server
        .load("assets/textures/rpg/chars/gabe/gabe-idle-run-512.png")
        // .load("assets/textures/rpg/chars/gabe/gabe-idle-run.png")
        .unwrap();
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(512.0, 64.0), 8, 1);
    //let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(168.0, 24.0), 7, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn(Camera2dComponents::default())
        .spawn(SpriteSheetComponents {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(2.0),
            // transform: Transform::from_scale(2.0),
            ..Default::default()
        })
        .with(Timer::from_seconds(0.1, true));
}
