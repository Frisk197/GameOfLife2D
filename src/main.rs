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

const WHITE: Color = Color::linear_rgba(1., 1., 1., 1.);
const INVISIBLE: Color = Color::linear_rgba(0., 0., 0., 0.);
const CAMERA_SPEED: f32 = 500.0;
const ZOOM_MULTIPLIER: f32 = 5.;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,
                      LogDiagnosticsPlugin::default(),
                      FrameTimeDiagnosticsPlugin,))
        .add_systems(Startup, (systems::setup_camera, systems::setup_mesh, systems::setup_simulation, systems::setup_refresh_timer))
        .add_systems(Update, (systems::camera_mouvement, systems::tile_placement, systems::display_tilemap, systems::run_simulation, systems::place_patterns, systems::toggle_vsync))
        .run();
}






