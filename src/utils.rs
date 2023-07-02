use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;

use crate::MainCamera;

#[derive(Resource)]
pub struct MousePosition(pub Vec3);

#[derive(Component)]
pub struct Lifespan(Timer);

impl Lifespan {
    pub fn new(duration: f32) -> Self {
        Lifespan(Timer::from_seconds(duration, TimerMode::Once))
    }
}

pub struct Plugin;

impl Plugin {
    fn update_mouse_position(
        q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
        windows: Query<&Window, With<PrimaryWindow>>,
        mut mouse_pos: ResMut<MousePosition>,
    ) {
        let (camera, camera_transform) = q_camera.single();
        let Some(cursor_pos) = windows.single().cursor_position() else {return } ;
        mouse_pos.0 = camera
            .viewport_to_world(camera_transform, cursor_pos)
            .unwrap()
            .origin;
        mouse_pos.0.z = 0.0;
    }

    fn update_lifespan(
        mut cmd: Commands,
        mut q_lifespan: Query<(Entity, &mut Lifespan)>,
        time: Res<Time>,
    ) {
        for (entity, mut lifespan) in &mut q_lifespan {
            lifespan.0.tick(time.delta());
            if lifespan.0.finished() {
                cmd.entity(entity).despawn_recursive();
            }
        }
    }
    fn velocity_abuse(
        mut vel_query: Query<(&mut Transform, &Velocity), Without<RigidBody>>,
        time: Res<Time>,
    ) {
        for (mut transform, vel) in &mut vel_query {
            transform.translation += time.delta_seconds() * vel.linvel.extend(0.0);
            transform.rotation *= Quat::from_rotation_z(time.delta_seconds() * vel.angvel);
        }
    }
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MousePosition(Vec3::ZERO))
            .add_system(Self::update_mouse_position)
            .add_system(Self::update_lifespan)
            .add_system(Self::velocity_abuse);
    }
}
