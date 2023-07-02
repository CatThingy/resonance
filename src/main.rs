use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

mod director;
mod enemy;
mod health;
mod main_menu;
mod player;
mod utils;
mod wave;

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
}

fn main() {
    let mut app = App::new();

    app.insert_resource(RapierConfiguration {
        gravity: Vec2::ZERO,
        ..default()
    })
    .add_state::<GameState>()
    .add_plugins(DefaultPlugins)
    .add_plugin(WorldInspectorPlugin::default())
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugin(ShapePlugin)
    .add_plugin(director::Plugin)
    .add_plugin(enemy::Plugin)
    .add_plugin(main_menu::Plugin)
    .add_plugin(health::Plugin)
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
