use bevy::{prelude::*, window::PrimaryWindow};

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
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MousePosition(Vec3::ZERO))
            .add_system(Self::update_mouse_position)
            .add_system(Self::update_lifespan);
    }
}
