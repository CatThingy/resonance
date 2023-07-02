use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    enemy::{Enemy, EnemyHitbox, Hitstun},
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

#[derive(Component)]
pub struct NoEffect;

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
            stroke.options.line_width = 2.0 + 30.0 * (wave.radius / wave.max_radius).powi(2);
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
            let interference_size = match interference.kind {
                InterferenceKind::Destructive => 30.0,
                InterferenceKind::Positive => 15.0,
                InterferenceKind::Negative => 25.0,
            };
            cmd.spawn((
                SpatialBundle {
                    transform: Transform::from_translation(
                        (interference.position + interference.direction * 2.0).extend(0.01),
                    ),
                    ..default()
                },
                WaveInterference {
                    kind: interference.kind,
                    direction: interference.direction,
                    strength: interference.strength,
                },
                Lifespan::new(0.05),
                Collider::ball(2.0 + interference_size * (1.0 - interference.strength)),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
            ));
        }
    }

    fn enemy_interaction(
        q_wave: Query<(&Wave, &GlobalTransform)>,
        q_enemy: Query<(Entity, &GlobalTransform), (With<Enemy>, Without<NoEffect>)>,
        mut q_projectile: Query<
            (&GlobalTransform, &mut Velocity),
            (With<EnemyHitbox>, Without<Enemy>),
        >,
        mut ev_health: EventWriter<HealthChangeEvent>,
        time: Res<Time>,
    ) {
        for (wave, wave_transform) in &q_wave {
            let wave_origin = wave_transform.translation().truncate();
            match wave.kind {
                WaveKind::Positive => {
                    for (enemy_entity, enemy_transform) in &q_enemy {
                        let enemy_pos = enemy_transform.translation().truncate();
                        let offset = f32::abs(enemy_pos.distance(wave_origin) - wave.radius);
                        if offset < 2.0 + 30.0 * (wave.radius / wave.max_radius).powi(2) {
                            ev_health.send(HealthChangeEvent {
                                target: enemy_entity,
                                amount: -10.0 * time.delta_seconds(),
                            });
                        }
                    }
                }
                WaveKind::Negative => {
                    for (enemy_transform, mut vel) in &mut q_projectile {
                        let enemy_pos = enemy_transform.translation().truncate();
                        let offset = f32::abs(enemy_pos.distance(wave_origin) - wave.radius);
                        if offset < 2.0 + 30.0 * (wave.radius / wave.max_radius).powi(2) {
                            vel.linvel +=
                                (enemy_pos - wave_origin).normalize() * 10.0 * time.delta_seconds();
                        }
                    }
                }
            }
        }
    }

    fn positive_interference(
        q_interference: Query<&WaveInterference>,
        mut q_enemy: Query<(&mut Hitstun, &mut Velocity), (With<Enemy>, Without<NoEffect>)>,
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
                        amount: -10.0 * (0.5 + interference.strength / 2.0),
                    });
                }
                enemy_hitstun.set(0.25);
                enemy_velocity.linvel =
                    interference.direction * 50.0 * (0.5 + interference.strength / 2.0);
            }
        }
    }

    fn negative_interference(
        mut cmd: Commands,
        q_interference: Query<&WaveInterference>,
        q_enemy_projectile: Query<(), (With<EnemyHitbox>, Without<Enemy>, Without<NoEffect>)>,
        mut ev_collisions: EventReader<CollisionEvent>,
    ) {
        for collision in &mut ev_collisions {
            let proj_entity;
            let interference;
            match collision {
                CollisionEvent::Started(e1, e2, _) => {
                    if let Ok(_) = q_enemy_projectile.get(*e1) {
                        if let Ok(i) = q_interference.get(*e2) {
                            proj_entity = e1;
                            interference = i;
                        } else {
                            continue;
                        }
                    } else if let Ok(_) = q_enemy_projectile.get(*e2) {
                        if let Ok(i) = q_interference.get(*e1) {
                            proj_entity = e2;
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

            if interference.kind == InterferenceKind::Negative {
                cmd.entity(*proj_entity).despawn_recursive();
            }
        }
    }

    fn destructive_interference(
        mut cmd: Commands,
        q_interference: Query<&WaveInterference>,
        q_enemy: Query<Entity, With<Enemy>>,
        mut ev_collisions: EventReader<CollisionEvent>,
    ) {
        for collision in &mut ev_collisions {
            let add;
            let target;
            let interference;
            let entity1;
            let entity2;
            match collision {
                CollisionEvent::Started(e1, e2, _) => {
                    add = true;
                    entity1 = e1;
                    entity2 = e2;
                }
                CollisionEvent::Stopped(e1, e2, _) => {
                    add = false;
                    entity1 = e1;
                    entity2 = e2;
                }
            }

            if let Ok(_) = q_enemy.get(*entity1) {
                if let Ok(i) = q_interference.get(*entity2) {
                    target = entity1;
                    interference = i;
                } else {
                    continue;
                }
            } else if let Ok(_) = q_enemy.get(*entity2) {
                if let Ok(i) = q_interference.get(*entity1) {
                    target = entity2;
                    interference = i;
                } else {
                    continue;
                }
            } else {
                continue;
            }

            if interference.kind == InterferenceKind::Destructive {
                if add {
                    cmd.entity(*target).insert(NoEffect);
                } else {
                    cmd.entity(*target).remove::<NoEffect>();
                }
            }
        }
    }

    fn cleanup_no_effect(mut cmd: Commands, q_affected: Query<Entity, With<NoEffect>>) {
        for affected in &q_affected {
            cmd.entity(affected).remove::<NoEffect>();
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
            .add_system(Self::destructive_interference)
            .add_system(Self::positive_interference.after(Self::destructive_interference))
            .add_system(Self::negative_interference.after(Self::destructive_interference))
            .add_system(Self::enemy_interaction.after(Self::destructive_interference))
            .add_system(Self::cleanup_no_effect);
    }
}
