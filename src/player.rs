use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::wave::{DelayedWave, Wave, WaveBundle, WaveKind};

const PLAYER_SPEED: f32 = 200.0;

#[derive(Component)]
struct Player;

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
        ));
    }

    fn player_movement(
        mut q_player: Query<&mut Transform, With<Player>>,
        keys: Res<Input<KeyCode>>,
        time: Res<Time>,
        mut input_direction: Local<Vec2>,
    ) {
        let Ok(mut player) = q_player.get_single_mut() else { return };

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

        player.translation +=
            input_direction.normalize_or_zero().extend(0.0) * PLAYER_SPEED * time.delta_seconds();
    }
    fn spawn_wave(
        mut cmd: Commands,
        q_player: Query<&GlobalTransform, With<Player>>,
        mouse_buttons: Res<Input<MouseButton>>,
    ) {
        let Ok(player) = q_player.get_single() else { return };

        if mouse_buttons.just_pressed(MouseButton::Left) {
            cmd.spawn((
                WaveBundle {
                    wave: Wave {
                        kind: WaveKind::Constructive,
                        radius: 0.0,
                        speed: 100.0,
                    },
                    shape_bundle: ShapeBundle {
                        path: GeometryBuilder::build_as(&shapes::Circle {
                            radius: 0.0,
                            center: Vec2::ZERO,
                        }),
                        transform: player.compute_transform(),

                        ..default()
                    },
                },
                Stroke::new(Color::RED, 2.0),
            ));
            cmd.spawn(DelayedWave::new(
                Wave {
                    kind: WaveKind::Destructive,
                    radius: 0.0,
                    speed: 100.0,
                },
                player.compute_transform(),
                0.5,
            ));
        }
    }
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(Self::spawn_player)
            .add_system(Self::player_movement)
            .add_system(Self::spawn_wave);
    }
}
