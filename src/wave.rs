use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::utils::Lifespan;

#[derive(Clone, Copy)]
pub enum WaveKind {
    Positive,
    Negative,
}

#[derive(Clone, Copy)]
pub enum InterferenceKind {
    Destructive,
    Positive,
    Negative,
}

pub struct WaveInterference {
    kind: InterferenceKind,
    position: Vec2,
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
    fn update_wave(mut q_wave: Query<(&mut Wave, &mut Path)>, time: Res<Time>) {
        for (mut wave, mut path) in &mut q_wave {
            wave.radius += wave.speed * time.delta_seconds();
            *path = GeometryBuilder::build_as(&shapes::Circle {
                radius: wave.radius,
                center: Vec2::ZERO,
            });
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
        mut ev_interference: EventWriter<WaveInterference>,
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

            if intersect_offset.length() <= 5.0 {
                ev_interference.send(WaveInterference {
                    kind: interference_kind,
                    position: center,
                });
            } else {
                ev_interference.send(WaveInterference {
                    kind: interference_kind,
                    position: center + intersect_offset,
                });
                ev_interference.send(WaveInterference {
                    kind: interference_kind,
                    position: center - intersect_offset,
                });
            }
        }
    }

    fn intefere(mut cmd: Commands, mut ev_inteference: EventReader<WaveInterference>) {
        for interference in &mut ev_inteference {
            cmd.spawn((SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::ONE * 5.0),
                    ..default()
                },
                transform: Transform::from_translation(interference.position.extend(0.1)),
                ..default()
            }, Lifespan::new(0.1)));
        }
    }
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WaveInterference>()
            .add_system(Self::update_wave)
            .add_system(Self::update_delayed_wave)
            .add_system(Self::detect_interference)
            .add_system(Self::intefere.after(Self::detect_interference));
    }
}
