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

#[derive(Component)]
pub struct HealthBar {
    width: f32,
    fraction: f32,
}

impl HealthBar {
    pub fn new(width: f32) -> Self {
        HealthBar {
            width,
            fraction: 1.0,
        }
    }
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

    fn update_healthbar(
        q_health: Query<(&Health, &Children)>,
        mut q_healthbar: Query<(&mut HealthBar, &mut Sprite, &mut Transform)>,
    ) {
        for (health, children) in &q_health {
            for child in children.iter() {
                if let Ok((mut bar, mut sprite, mut transform)) = q_healthbar.get_mut(*child) {
                    bar.fraction = health.percentage();
                    sprite.custom_size = sprite
                        .custom_size
                        .map(|v| Vec2::new(bar.width * bar.fraction, v.y));
                    transform.translation.x = bar.width * (bar.fraction - 1.0) / 2.0;
                }
            }
        }
    }
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HealthChangeEvent>()
            .add_system(Self::update_health.in_base_set(CoreSet::PostUpdate))
            .add_system(Self::update_healthbar);
    }
}
