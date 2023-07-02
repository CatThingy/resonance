use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;
use enemy::{Enemy, Hitstun, ShootingEnemy};
use health::{Health, HealthBar};

mod enemy;
mod health;
mod player;
mod utils;
mod wave;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(ShapePlugin)
        .add_plugin(enemy::Plugin)
        .add_plugin(health::Plugin)
        .add_plugin(player::Plugin)
        .add_plugin(utils::Plugin)
        .add_plugin(wave::Plugin);

    app.add_startup_system(init);

    app.add_system(debug_spawn_enemy);
    app.run();
}

#[derive(Component)]
pub struct MainCamera;

fn init(mut cmd: Commands) {
    let camera = Camera2dBundle::default();
    cmd.spawn((camera, MainCamera));
}

fn debug_spawn_enemy(
    mut cmd: Commands,
    keyboard: Res<Input<KeyCode>>,
    mouse: Res<utils::MousePosition>,
    assets: Res<AssetServer>,
) {
    if keyboard.just_pressed(KeyCode::E) {
        cmd.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::GREEN,
                    custom_size: Some(Vec2::splat(40.0)),
                    ..default()
                },
                transform: Transform::from_translation(mouse.0),
                ..default()
            },
            Enemy { speed: 30.0 },
            Collider::cuboid(20.0, 20.0),
            Health::new(30.0),
            Hitstun::new(0.0),
            Sensor,
            RigidBody::KinematicVelocityBased,
            Velocity::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::DARK_GREEN,
                        custom_size: Some(Vec2::new(40.0, 5.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 30.0, 0.1),
                    ..default()
                },
                HealthBar::new(40.0),
            ));
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::new(40.0, 5.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 30.0, 0.0),
                ..default()
            });
        });
    }

    if keyboard.just_pressed(KeyCode::R) {
        cmd.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::GREEN,
                    custom_size: Some(Vec2::splat(40.0)),
                    ..default()
                },
                transform: Transform::from_translation(mouse.0),
                ..default()
            },
            Enemy { speed: 30.0 },
            ShootingEnemy {
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                speed: 500.0,
                lifespan: 5.0,
                damage: 20.0,
                size: 8.0,
                texture: assets.load("shooter_shot.png"),
            },
            Collider::cuboid(20.0, 20.0),
            Health::new(30.0),
            Hitstun::new(0.0),
            Sensor,
            RigidBody::KinematicVelocityBased,
            Velocity::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::DARK_GREEN,
                        custom_size: Some(Vec2::new(40.0, 5.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 30.0, 0.1),
                    ..default()
                },
                HealthBar::new(40.0),
            ));
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::new(40.0, 5.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 30.0, 0.0),
                ..default()
            });
        });
    }
}
