use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

mod player;
mod utils;
mod wave;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(ShapePlugin)
        .add_plugin(player::Plugin)
        .add_plugin(utils::Plugin)
        .add_plugin(wave::Plugin);

    app.add_startup_system(init);
    app.run();
}

#[derive(Component)]
pub struct MainCamera;

fn init(mut cmd: Commands) {
    let camera = Camera2dBundle::default();
    cmd.spawn((camera, MainCamera));
}
