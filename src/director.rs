use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    enemy::{Enemy, Hitstun, ShootingEnemy, RecentDamage, EnemyHitbox},
    health::{Health, HealthBar},
    MainCamera,
};

#[derive(Resource)]
pub struct Budget(u32);

#[derive(Resource)]
pub struct RoundDelay(Timer);

#[derive(Resource)]
pub struct SpawnStatus {
    budget: u32,
    spawn_timer: Timer,
    enabled: bool,
}

const NORMIE_COST: u32 = 1;
const NORMIE_DELAY: f32 = 1.0;
const NORMIE_REQUIRED_BUDGET: u32 = 0;

const RANGER_COST: u32 = 5;
const RANGER_DELAY: f32 = 2.0;
const RANGER_REQUIRED_BUDGET: u32 = 10;

pub struct Plugin;

impl Plugin {
    fn tick_round_delay(
        q_enemy: Query<(), With<Enemy>>,
        budget: Res<Budget>,
        mut round_delay: ResMut<RoundDelay>,
        mut spawn_status: ResMut<SpawnStatus>,
        time: Res<Time>,
    ) {
        if spawn_status.enabled == false && q_enemy.iter().size_hint().0 == 0 {
            round_delay.0.tick(time.delta());
            if round_delay.0.just_finished() {
                spawn_status.enabled = true;
                spawn_status.budget = budget.0;
            }
        }
    }

    fn spawn_enemy(
        mut cmd: Commands,
        q_camera: Query<&Camera, With<MainCamera>>,
        assets: Res<AssetServer>,
        time: Res<Time>,
        mut budget: ResMut<Budget>,
        mut spawn_status: ResMut<SpawnStatus>,
    ) {
        if spawn_status.enabled {
            spawn_status.spawn_timer.tick(time.delta());

            if spawn_status.spawn_timer.just_finished() {
                let viewport_size =
                    q_camera.single().logical_viewport_size().unwrap() + Vec2::splat(20.0);

                let mut rand = fastrand::f32() * (2.0 * viewport_size.x + 2.0 * viewport_size.y);

                let perim_point = 'a: {
                    if rand < viewport_size.x {
                        break 'a Vec2::new(rand, 0.0);
                    }
                    rand -= viewport_size.x;
                    if rand < viewport_size.y {
                        break 'a Vec2::new(viewport_size.x, rand);
                    }
                    rand -= viewport_size.y;
                    if rand < viewport_size.x {
                        break 'a Vec2::new(rand, viewport_size.y);
                    } else {
                        break 'a Vec2::new(0.0, rand - viewport_size.x);
                    }
                } - viewport_size / 2.0;

                let mut generated = 0;

                if budget.0 > NORMIE_REQUIRED_BUDGET && spawn_status.budget >= NORMIE_COST {
                    generated += 1;
                }

                if budget.0 > RANGER_REQUIRED_BUDGET && spawn_status.budget >= RANGER_COST {
                    generated += 1;
                }

                let selected_enemy = fastrand::u32(0..generated);

                match selected_enemy {
                    0 => {
                        spawn_normie(&mut cmd, perim_point.extend(0.0));
                        spawn_status.budget -= NORMIE_COST;
                        spawn_status
                            .spawn_timer
                            .set_duration(Duration::from_secs_f32(NORMIE_DELAY));
                    }
                    1 => {
                        spawn_ranger(&mut cmd, perim_point.extend(0.0), &assets);
                        spawn_status.budget -= RANGER_COST;
                        spawn_status
                            .spawn_timer
                            .set_duration(Duration::from_secs_f32(RANGER_DELAY));
                    }
                    _ => {}
                }
                if spawn_status.budget == 0 {
                    spawn_status.enabled = false;
                    budget.0 *= 7;
                    budget.0 /= 5;
                }
            }
        }
    }
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Budget(5))
            .insert_resource(RoundDelay(Timer::from_seconds(3.0, TimerMode::Repeating)))
            .insert_resource(SpawnStatus {
                budget: 5,
                spawn_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                enabled: false,
            })
            .add_system(Self::tick_round_delay)
            .add_system(Self::spawn_enemy);
    }
}

fn spawn_normie(cmd: &mut Commands, pos: Vec3) {
    cmd.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(Vec2::splat(40.0)),
                ..default()
            },
            transform: Transform::from_translation(pos),
            ..default()
        },
        Enemy { speed: 60.0 },
        Collider::cuboid(20.0, 20.0),
        RecentDamage(0.0),
        Health::new(30.0),
        Hitstun::new(0.0),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        Velocity::default(),
        EnemyHitbox {
            damage: 10.0,
            once: false,
        }
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

fn spawn_ranger(cmd: &mut Commands, pos: Vec3, assets: &AssetServer) {
    cmd.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN,
                custom_size: Some(Vec2::splat(40.0)),
                ..default()
            },
            transform: Transform::from_translation(pos),
            ..default()
        },
        Enemy { speed: 30.0 },
        ShootingEnemy {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            speed: 400.0,
            lifespan: 5.0,
            damage: 7.0,
            size: 8.0,
            texture: assets.load("shooter_shot.png"),
        },
        RecentDamage(0.0),
        Collider::cuboid(20.0, 20.0),
        Health::new(30.0),
        Hitstun::new(0.0),
        RigidBody::KinematicVelocityBased,
        Velocity::default(),
        EnemyHitbox {
            damage: 0.1,
            once: false,
        }
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
