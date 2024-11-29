mod components;
mod systems;
mod uVec3;

use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    sprite::MaterialMesh2dBundle,
    window::{
        WindowResized,
        PrimaryWindow,
    },
};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::{InputPlugin, InputSystem};
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::window::{PresentMode, WindowTheme};

const WHITE: Color = Color::linear_rgba(1., 1., 1., 1.);
const INVISIBLE: Color = Color::linear_rgba(0., 0., 0., 0.);
const CAMERA_SPEED: f32 = 500.0;
const ZOOM_MULTIPLIER: f32 = 5.;
const UPDATE_COUNT_LIMIT: i32 = 20;
const UNSTBLE_CHANGER_LIMIT: i32 = 3;


fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
                            primary_window: Some(Window {
                                title: "Game Of Life 2D".into(),
                                name: Some("gameoflife2d.frisk197".into()),
                                resolution: (1920., 1080.).into(),
                                present_mode: PresentMode::AutoNoVsync,
                                // Tells Wasm to resize the window according to the available canvas
                                fit_canvas_to_parent: true,
                                // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
                                prevent_default_event_handling: false,
                                window_theme: Some(WindowTheme::Dark),
                                enabled_buttons: bevy::window::EnabledButtons {
                                    maximize: true,
                                    ..Default::default()
                                },
                                // This will spawn an invisible window
                                // The window will be made visible in the make_visible() system after 3 frames.
                                // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
                                visible: true,
                                ..default()
                            }),
                            ..default()
                        }),
                      // LogDiagnosticsPlugin::default(),
                      // FrameTimeDiagnosticsPlugin,
        ))
        .add_systems(Startup, (systems::setup_camera, systems::setup_batching, systems::setup_simulation, systems::setup_refresh_timer))
        .add_systems(Update, (systems::camera_mouvement, systems::tile_placement.before(systems::display_tilemap), systems::display_tilemap.after(systems::hide_off_tiles), systems::run_simulation.after(systems::hide_off_tiles), systems::place_patterns.before(systems::display_tilemap), systems::toggle_vsync, systems::hide_off_tiles))
        .run();
}






