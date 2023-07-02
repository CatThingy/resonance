use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    health::HealthChangeEvent,
    player::{AvgPlayerVel, Player},
    utils::Lifespan,
};

#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
}

#[derive(Component)]
pub struct Hitstun(Timer);

#[derive(Component)]
pub struct ShootingEnemy {
    pub timer: Timer,
    pub speed: f32,
    pub lifespan: f32,
    pub damage: f32,
    pub size: f32,
    pub texture: Handle<Image>,
}

#[derive(Component)]
pub struct EnemyHitbox {
    pub damage: f32,
    pub once: bool,
}

impl Hitstun {
    pub fn new(mut time: f32) -> Self {
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

    fn enemy_shoot(
        mut cmd: Commands,
        q_player: Query<&GlobalTransform, With<Player>>,
        mut q_shooter: Query<(&GlobalTransform, &mut ShootingEnemy, &Hitstun), With<Enemy>>,
        time: Res<Time>,
        player_vel: Res<AvgPlayerVel>,
    ) {
        let Ok(player_transform) = q_player.get_single() else { return };
        let player_pos = player_transform.translation().truncate();

        for (transform, mut shooter, hitstun) in &mut q_shooter {
            if hitstun.is_set() {
                continue;
            }

            shooter.timer.tick(time.delta());

            if shooter.timer.finished() {
                shooter.timer.reset();

                let player_dist = transform.translation().truncate().distance(player_pos);
                let travel_time = player_dist / shooter.speed;
                let target_pos = player_pos + player_vel.0 * travel_time;
                let target_dir =
                    (target_pos - transform.translation().truncate()).normalize_or_zero();

                cmd.spawn((
                    SpriteBundle {
                        texture: shooter.texture.clone(),
                        transform: transform.compute_transform().with_rotation(
                            Quat::from_rotation_z(-target_dir.angle_between(Vec2::NEG_Y)),
                        ),
                        ..default()
                    },
                    Collider::ball(shooter.size),
                    Sensor,
                    RigidBody::KinematicVelocityBased,
                    ActiveEvents::COLLISION_EVENTS,
                    ActiveCollisionTypes::KINEMATIC_STATIC
                        | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                    Velocity {
                        angvel: 0.0,
                        linvel: shooter.speed * target_dir,
                    },
                    Lifespan::new(shooter.lifespan),
                    EnemyHitbox {
                        damage: shooter.damage,
                        once: true,
                    },
                ));
            }
        }
    }

    fn enemy_damage(
        mut cmd: Commands,
        q_hitbox: Query<&EnemyHitbox>,
        q_player: Query<&Player>,
        mut ev_collisions: EventReader<CollisionEvent>,
        mut ev_health: EventWriter<HealthChangeEvent>,
    ) {
        for collision in &mut ev_collisions {
            let player_entity;
            let hitbox;
            let hitbox_entity;
            match collision {
                CollisionEvent::Started(e1, e2, _) => {
                    if let Ok(h) = q_hitbox.get(*e1) {
                        if let Ok(_) = q_player.get(*e2) {
                            hitbox = h;
                            hitbox_entity = e1;
                            player_entity = e2;
                        } else {
                            continue;
                        }
                    } else if let Ok(h) = q_hitbox.get(*e2) {
                        if let Ok(_) = q_player.get(*e1) {
                            hitbox = h;
                            hitbox_entity = e2;
                            player_entity = e1;
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                _ => continue,
            }

            ev_health.send(HealthChangeEvent {
                target: *player_entity,
                amount: -hitbox.damage,
            });
            if hitbox.once {
                cmd.entity(*hitbox_entity).despawn_recursive();
            }
        }
    }

}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::enemy_movement)
            .add_system(Self::enemy_shoot)
            .add_system(Self::enemy_damage);
    }
}
