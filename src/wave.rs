use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    enemy::{Enemy, Hitstun},
    health::HealthChangeEvent,
    utils::Lifespan,
};

#[derive(Clone, Copy)]
pub enum WaveKind {
    Positive,
    Negative,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InterferenceKind {
    Destructive,
    Positive,
    Negative,
}

#[derive(Component)]
pub struct WaveInterference {
    kind: InterferenceKind,
    direction: Vec2,
    strength: f32,
}

pub struct WaveInterferenceEvent {
    kind: InterferenceKind,
    position: Vec2,
    direction: Vec2,
    strength: f32,
}

impl WaveKind {
    fn color(&self) -> Color {
        match self {
            WaveKind::Positive => Color::RED,
            WaveKind::Negative => Color::BLUE,
        }
    }
}

#[derive(Component, Clone)]
pub struct Wave {
    pub kind: WaveKind,
    pub radius: f32,
    pub max_radius: f32,
    pub speed: f32,
}

#[derive(Bundle)]
pub struct WaveBundle {
    pub wave: Wave,
    pub shape_bundle: ShapeBundle,
}

#[derive(Component)]
pub struct DelayedWave {
    pub wave: Wave,
    delay_timer: Timer,
    transform: Transform,
}

impl DelayedWave {
    pub fn new(wave: Wave, transform: Transform, delay: f32) -> Self {
        DelayedWave {
            wave,
            delay_timer: Timer::from_seconds(delay, TimerMode::Once),
            transform,
        }
    }
}

pub struct Plugin;

impl Plugin {
    fn update_wave(
        mut cmd: Commands,
        mut q_wave: Query<(Entity, &mut Wave, &mut Path, &mut Stroke)>,
        time: Res<Time>,
    ) {
        for (entity, mut wave, mut path, mut stroke) in &mut q_wave {
            wave.radius += wave.speed * time.delta_seconds();
            if wave.radius >= wave.max_radius {
                cmd.entity(entity).despawn_recursive();
                continue;
            }
            *path = GeometryBuilder::build_as(&shapes::Circle {
                radius: wave.radius,
                center: Vec2::ZERO,
            });

            stroke.options.line_width = 30.0 * (wave.radius / wave.max_radius).powi(2);
            stroke
                .color
                .set_a(1.0 - (wave.radius / wave.max_radius).powi(2));
        }
    }
    fn update_delayed_wave(
        mut cmd: Commands,
        mut q_delayed_wave: Query<(Entity, &mut DelayedWave)>,
        time: Res<Time>,
    ) {
        for (entity, mut delayed_wave) in &mut q_delayed_wave {
            delayed_wave.delay_timer.tick(time.delta());

            if delayed_wave.delay_timer.finished() {
                cmd.spawn((
                    WaveBundle {
                        wave: delayed_wave.wave.clone(),
                        shape_bundle: ShapeBundle {
                            transform: delayed_wave.transform.clone(),
                            ..default()
                        },
                    },
                    Stroke::new(delayed_wave.wave.kind.color(), 2.0),
                ));
                cmd.entity(entity).despawn_recursive();
            }
        }
    }

    fn detect_interference(
        mut ev_interference: EventWriter<WaveInterferenceEvent>,
        q_wave: Query<(&Wave, &GlobalTransform)>,
    ) {
        for [(wave1, transform1), (wave2, transform2)] in q_wave.iter_combinations() {
            let pos1 = transform1.translation().truncate();
            let pos2 = transform2.translation().truncate();
            let distance = pos1.distance(pos2);
            if distance == 0.0
                || distance > wave1.radius + wave2.radius
                || distance < f32::abs(wave1.radius - wave2.radius)
            {
                continue;
            }

            let a =
                (wave1.radius.powi(2) - wave2.radius.powi(2) + distance.powi(2)) / (2.0 * distance);
            let h = f32::sqrt(wave1.radius.powi(2) - a.powi(2));

            let center = pos1 + a * (pos2 - pos1) / distance;

            let intersect_offset = Vec2::new(
                h * (pos2.y - pos1.y) / distance,
                -h * (pos2.x - pos1.x) / distance,
            );

            let interference_kind = match (&wave1.kind, &wave2.kind) {
                (WaveKind::Positive, WaveKind::Positive) => InterferenceKind::Positive,
                (WaveKind::Positive, WaveKind::Negative)
                | (WaveKind::Negative, WaveKind::Positive) => InterferenceKind::Destructive,
                (WaveKind::Negative, WaveKind::Negative) => InterferenceKind::Negative,
            };

            let prev_radius1 = wave1.radius - 0.1;
            let prev_radius2 = wave2.radius - 0.1;

            let prev_a =
                (prev_radius1.powi(2) - prev_radius2.powi(2) + distance.powi(2)) / (2.0 * distance);
            let prev_h = f32::sqrt(prev_radius1.powi(2) - prev_a.powi(2));
            let prev_center = pos1 + prev_a * (pos2 - pos1) / distance;

            let prev_intersect_offset = Vec2::new(
                prev_h * (pos2.y - pos1.y) / distance,
                -prev_h * (pos2.x - pos1.x) / distance,
            );

            if intersect_offset.length() <= 5.0 {
                ev_interference.send(WaveInterferenceEvent {
                    kind: interference_kind,
                    position: center,
                    direction: center - prev_center,
                    strength: 1.0
                        - f32::min(
                            (wave1.radius / wave1.max_radius).powi(2),
                            (wave2.radius / wave2.max_radius).powi(2),
                        ),
                });
            } else {
                ev_interference.send(WaveInterferenceEvent {
                    kind: interference_kind,
                    position: center + intersect_offset,
                    direction: ((center + intersect_offset)
                        - (prev_center + prev_intersect_offset))
                        .normalize(),

                    strength: 1.0
                        - f32::min(
                            (wave1.radius / wave1.max_radius).powi(2),
                            (wave2.radius / wave2.max_radius).powi(2),
                        ),
                });
                ev_interference.send(WaveInterferenceEvent {
                    kind: interference_kind,
                    position: center - intersect_offset,
                    direction: ((center - intersect_offset)
                        - (prev_center - prev_intersect_offset))
                        .normalize(),
                    strength: 1.0
                        - f32::min(
                            (wave1.radius / wave1.max_radius).powi(2),
                            (wave2.radius / wave2.max_radius).powi(2),
                        ),
                });
            }
        }
    }

    fn interfere(mut cmd: Commands, mut ev_inteference: EventReader<WaveInterferenceEvent>) {
        for interference in &mut ev_inteference {
            cmd.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(5.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(interference.position.extend(0.1)),
                    ..default()
                },
                WaveInterference {
                    kind: interference.kind,
                    direction: interference.direction,
                    strength: interference.strength,
                },
                Lifespan::new(0.1),
                Collider::ball(30.0 * (1.0 - interference.strength)),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                ActiveCollisionTypes::KINEMATIC_STATIC,
            ));
        }
    }

    fn handle_positive_interference(
        q_interference: Query<&WaveInterference>,
        mut q_enemy: Query<(&mut Hitstun, &mut Velocity), With<Enemy>>,
        mut ev_collisions: EventReader<CollisionEvent>,
        mut ev_health: EventWriter<HealthChangeEvent>,
    ) {
        for collision in &mut ev_collisions {
            let mut enemy_hitstun;
            let mut enemy_velocity;
            let enemy_entity;
            let interference;
            match collision {
                CollisionEvent::Started(e1, e2, _) => {
                    if let Ok((h, v)) = q_enemy.get_mut(*e1) {
                        if let Ok(i) = q_interference.get(*e2) {
                            enemy_entity = e1;
                            enemy_hitstun = h;
                            enemy_velocity = v;
                            interference = i;
                        } else {
                            continue;
                        }
                    } else if let Ok((h, v)) = q_enemy.get_mut(*e2) {
                        if let Ok(i) = q_interference.get(*e1) {
                            enemy_entity = e2;
                            enemy_hitstun = h;
                            enemy_velocity = v;
                            interference = i;
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                _ => continue,
            }

            if interference.kind == InterferenceKind::Positive {
                if !enemy_hitstun.is_set() {
                    ev_health.send(HealthChangeEvent {
                        target: *enemy_entity,
                        amount: -10.0 * interference.strength,
                    });
                }
                enemy_hitstun.set(0.25);
                enemy_velocity.linvel = interference.direction * 50.0 * interference.strength;
            }
        }
    }
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WaveInterferenceEvent>()
            .add_system(Self::update_wave)
            .add_system(Self::update_delayed_wave)
            .add_system(Self::detect_interference)
            .add_system(Self::interfere.after(Self::detect_interference))
            .add_system(Self::handle_positive_interference);
    }
}
