use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

#[derive(Clone)]
pub enum WaveKind {
    Constructive,
    Destructive,
}

impl WaveKind {
    fn color(&self) -> Color {
        match self {
            WaveKind::Constructive => Color::RED,
            WaveKind::Destructive => Color::BLUE,
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
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::update_wave)
        .add_system(Self::update_delayed_wave);
    }
}
