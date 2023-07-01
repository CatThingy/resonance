use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::player::Player;

#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
}

#[derive(Component)]
pub struct Hitstun(Timer);

impl Hitstun {
    pub fn new(mut time: f32)-> Self {
        if time <= 0.0 {
            time = f32::EPSILON;
        }
    
        Hitstun(Timer::from_seconds(time, TimerMode::Once))

    }

    pub fn set(&mut self, time: f32) {
        self.0.set_duration(Duration::from_secs_f32(time));
        self.0.reset();
    }

    pub fn is_set(&self) -> bool {
        return !self.0.finished();
    }
}

pub struct Plugin;

impl Plugin {
    fn enemy_movement(
        q_player: Query<&GlobalTransform, With<Player>>,
        mut q_enemy: Query<(&mut Velocity, &GlobalTransform, &Enemy, &mut Hitstun)>,
        time: Res<Time>,
    ) {
        let Ok(player_transform) = q_player.get_single() else { return };
        let player_pos = player_transform.translation().truncate();

        for (mut enemy_vel, enemy_global_transform, enemy, mut hitstun) in &mut q_enemy {
            if hitstun.is_set() {
                hitstun.0.tick(time.delta());
                continue;
            }
            let enemy_pos = enemy_global_transform.translation().truncate();
            let direction = player_pos - enemy_pos;

            enemy_vel.linvel = direction.normalize_or_zero() * enemy.speed;
        }
    }
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::enemy_movement);
    }
}
