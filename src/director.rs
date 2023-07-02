use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    enemy::{Enemy, EnemyHitbox, Hitstun, ShootingEnemy},
    health::{Health, HealthBar, HealthChangeEvent},
    player::Player,
    wave::{Wave, WaveInterference},
    GameState, MainCamera,
};

#[derive(Resource)]
pub struct Rounds(pub u32);

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

#[derive(Component)]
pub struct Root;

#[derive(Component)]
pub struct RoundCounter;

const NORMIE_COST: u32 = 1;
const NORMIE_DELAY: f32 = 1.0;
const NORMIE_REQUIRED_BUDGET: u32 = 0;

const LAYER_COST: u32 = 2;
const LAYER_DELAY: f32 = 2.0;
const LAYER_REQUIRED_BUDGET: u32 = 6;

const RANGER_COST: u32 = 5;
const RANGER_DELAY: f32 = 2.0;
const RANGER_REQUIRED_BUDGET: u32 = 10;

#[derive(SystemSet, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Plugin;

impl Plugin {
    fn init_ui(mut cmd: Commands, assets: Res<AssetServer>) {
        cmd.spawn(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .insert(Root)
        .with_children(|root| {
            root.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Round 0",
                        TextStyle {
                            font: assets.load("FiraSans-Light.ttf"),
                            font_size: 72.0,
                            color: Color::hex("bc53ff").unwrap(),
                        },
                    ),
                    ..default()
                },
                RoundCounter,
            ));
        });
    }

    fn tick_round_delay(
        q_player: Query<Entity, With<Player>>,
        mut ev_health: EventWriter<HealthChangeEvent>,
        q_enemy: Query<(), With<Enemy>>,
        budget: Res<Budget>,
        mut round_delay: ResMut<RoundDelay>,
        mut rounds: ResMut<Rounds>,
        mut spawn_status: ResMut<SpawnStatus>,
        time: Res<Time>,
    ) {
        if spawn_status.enabled == false && q_enemy.iter().size_hint().0 == 0 {
            round_delay.0.tick(time.delta());
            if let Ok(player) = q_player.get_single() {
                ev_health.send(HealthChangeEvent {
                    target: player,
                    amount: time.delta_seconds() * 15.0,
                });
            }
            if round_delay.0.just_finished() {
                rounds.0 += 1;
                spawn_status.enabled = true;
                spawn_status.budget = budget.0;
            }
        }
    }

    fn update_round_counter(
        rounds: Res<Rounds>,
        mut counter: Query<&mut Text, With<RoundCounter>>,
    ) {
        if rounds.is_changed() {
            for mut counter in &mut counter {
                counter.sections[0].value = format!("Round {}", rounds.0);
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

                if budget.0 > LAYER_REQUIRED_BUDGET && spawn_status.budget >= LAYER_COST {
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
                        spawn_layer(&mut cmd, perim_point.extend(0.0), &assets);
                        spawn_status.budget -= LAYER_COST;
                        spawn_status
                            .spawn_timer
                            .set_duration(Duration::from_secs_f32(LAYER_DELAY));
                    }
                    2 => {
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

    fn reset(
        mut cmd: Commands,
        q_cleanup: Query<
            Entity,
            Or<(
                With<Wave>,
                With<EnemyHitbox>,
                With<WaveInterference>,
                With<Root>,
            )>,
        >,
        mut budget: ResMut<Budget>,
        mut round_delay: ResMut<RoundDelay>,
        mut spawn_status: ResMut<SpawnStatus>,
    ) {
        budget.0 = 5;
        round_delay.0.reset();
        spawn_status.budget = 5;
        spawn_status.spawn_timer.reset();
        spawn_status.enabled = false;

        for entity in &q_cleanup {
            cmd.entity(entity).despawn_recursive();
        }
    }
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Budget(5))
            .insert_resource(Rounds(0))
            .insert_resource(RoundDelay(Timer::from_seconds(3.0, TimerMode::Repeating)))
            .insert_resource(SpawnStatus {
                budget: 5,
                spawn_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                enabled: false,
            })
            .add_system(Self::tick_round_delay.in_set(Self))
            .add_system(Self::update_round_counter.in_set(Self))
            .add_system(Self::spawn_enemy.in_set(Self))
            .add_system(Self::init_ui.in_schedule(OnEnter(GameState::InGame)))
            .add_system(Self::reset.in_schedule(OnExit(GameState::InGame)));
        app.configure_set(Self.run_if(in_state(GameState::InGame)));
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
        Enemy { speed: 80.0 },
        Collider::cuboid(20.0, 20.0),
        Health::new(30.0),
        Hitstun::new(0.0),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        Velocity::default(),
        EnemyHitbox {
            damage: 10.0,
            once: false,
        },
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
            transform: Transform::from_xyz(0.0, 30.0, 0.05),
            ..default()
        });
    });
}

fn spawn_ranger(cmd: &mut Commands, pos: Vec3, assets: &AssetServer) {
    cmd.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::AZURE,
                custom_size: Some(Vec2::splat(40.0)),
                ..default()
            },
            transform: Transform::from_translation(pos),
            texture: assets.load("shooter.png"),
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
        Collider::cuboid(20.0, 20.0),
        Health::new(10.0),
        Hitstun::new(0.0),
        RigidBody::KinematicVelocityBased,
        Velocity::default(),
        EnemyHitbox {
            damage: 0.1,
            once: false,
        },
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

fn spawn_layer(cmd: &mut Commands, pos: Vec3, assets: &AssetServer) {
    cmd.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::TEAL,
                custom_size: Some(Vec2::splat(40.0)),
                ..default()
            },
            transform: Transform::from_translation(pos),
            texture: assets.load("layer.png"),
            ..default()
        },
        Enemy { speed: 40.0 },
        ShootingEnemy {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            speed: 4.0,
            lifespan: 60.0,
            damage: 10.0,
            size: 8.0,
            texture: assets.load("layer_shot.png"),
        },
        Collider::cuboid(20.0, 20.0),
        Health::new(20.0),
        Hitstun::new(0.0),
        RigidBody::KinematicVelocityBased,
        Velocity::default(),
        EnemyHitbox {
            damage: 0.1,
            once: false,
        },
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
