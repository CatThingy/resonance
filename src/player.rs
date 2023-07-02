use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    health::{Health, HealthBar},
    wave::{DelayedWave, Wave, WaveBundle, WaveKind},
};

const PLAYER_SPEED: f32 = 200.0;

#[derive(Resource)]
pub struct AvgPlayerVel(pub Vec2);

#[derive(Component)]
pub struct Player;

pub struct Plugin;

impl Plugin {
    fn spawn_player(mut cmd: Commands) {
        cmd.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(40.0, 40.0)),
                    ..default()
                },
                ..default()
            },
            Player,
            Collider::cuboid(20.0, 20.0),
            ActiveEvents::COLLISION_EVENTS,
            LockedAxes::ROTATION_LOCKED,
            RigidBody::Dynamic,
            Velocity::default(),
            Health::new(100.0),
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

    fn player_movement(
        mut q_player: Query<&mut Velocity, With<Player>>,
        keys: Res<Input<KeyCode>>,
        mut input_direction: Local<Vec2>,
        mut avg_vel: ResMut<AvgPlayerVel>,
        time: Res<Time>,
    ) {
        let Ok(mut player_vel) = q_player.get_single_mut() else { return };

        // let mut input_direction = Vec2::ZERO;
        if keys.just_pressed(KeyCode::A) || keys.just_pressed(KeyCode::Left) {
            input_direction.x = -1.0;
        }
        if keys.just_pressed(KeyCode::D) || keys.just_pressed(KeyCode::Right) {
            input_direction.x = 1.0;
        }
        if keys.just_pressed(KeyCode::W) || keys.just_pressed(KeyCode::Up) {
            input_direction.y = 1.0;
        }
        if keys.just_pressed(KeyCode::S) || keys.just_pressed(KeyCode::Down) {
            input_direction.y = -1.0;
        }

        if keys.just_released(KeyCode::A) || keys.just_released(KeyCode::Left) {
            if keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right) {
                input_direction.x = 1.0;
            } else {
                input_direction.x = 0.0;
            }
        }
        if keys.just_released(KeyCode::D) || keys.just_released(KeyCode::Right) {
            if keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left) {
                input_direction.x = -1.0;
            } else {
                input_direction.x = 0.0;
            }
        }
        if keys.just_released(KeyCode::W) || keys.just_released(KeyCode::Up) {
            if keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down) {
                input_direction.y = -1.0;
            } else {
                input_direction.y = 0.0;
            }
        }
        if keys.just_released(KeyCode::S) || keys.just_released(KeyCode::Down) {
            if keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up) {
                input_direction.y = 1.0;
            } else {
                input_direction.y = 0.0;
            }
        }

        player_vel.linvel = input_direction.normalize_or_zero() * PLAYER_SPEED;

        let alpha = 0.5 * time.delta_seconds();

        let prev = avg_vel.0;
        avg_vel.0 += alpha * (player_vel.linvel - prev);
    }

    fn spawn_wave(
        mut cmd: Commands,
        q_player: Query<&GlobalTransform, With<Player>>,
        mouse_buttons: Res<Input<MouseButton>>,
    ) {
        let Ok(player_transform) = q_player.get_single() else { return };

        let wave_transform = player_transform.compute_transform();

        if mouse_buttons.just_pressed(MouseButton::Left) {
            cmd.spawn((
                WaveBundle {
                    wave: Wave {
                        kind: WaveKind::Positive,
                        radius: 0.0,
                        speed: 100.0,
                        max_radius: 400.0,
                    },
                    shape_bundle: ShapeBundle {
                        path: GeometryBuilder::build_as(&shapes::Circle {
                            radius: 0.0,
                            center: Vec2::ZERO,
                        }),
                        transform: wave_transform,

                        ..default()
                    },
                },
                Stroke::new(Color::RED, 2.0),
            ));
            cmd.spawn(DelayedWave::new(
                Wave {
                    kind: WaveKind::Negative,
                    radius: 0.0,
                    max_radius: 400.0,
                    speed: 100.0,
                },
                wave_transform,
                0.5,
            ));
        }
    }
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(Self::spawn_player)
            .insert_resource(AvgPlayerVel(Vec2::ZERO))
            .add_system(Self::player_movement)
            .add_system(Self::spawn_wave);
    }
}
