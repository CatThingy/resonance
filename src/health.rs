use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Health { current: max, max }
    }

    pub fn percentage(&self) -> f32 {
        self.current / self.max
    }
}

pub struct HealthChangeEvent {
    pub target: Entity,
    pub amount: f32,
}

pub struct Plugin;

impl Plugin {
    fn update_health(
        mut cmd: Commands,
        mut q_health: Query<(Entity, &mut Health)>,
        mut event_reader: EventReader<HealthChangeEvent>,
    ) {
        for event in event_reader.iter() {
            if let Ok((entity, mut health)) = q_health.get_mut(event.target) {
                health.current += event.amount;

                if health.current > health.max {
                    health.current = health.max;
                } else if health.current < 0.0 {
                    cmd.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HealthChangeEvent>()
        .add_system(Self::update_health);
    }
}
