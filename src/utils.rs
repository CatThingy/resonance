use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;

use crate::{GameState, MainCamera};

#[derive(Resource)]
pub struct MousePosition(pub Vec3);

#[derive(Component)]
pub struct Lifespan(Timer);

impl Lifespan {
    pub fn new(duration: f32) -> Self {
        Lifespan(Timer::from_seconds(duration, TimerMode::Once))
    }
}

pub struct PlaySound(pub String);

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

    fn music(audio: Res<Audio>, assets: Res<AssetServer>) {
        audio.play_with_settings(assets.load("interfere.ogg"), PlaybackSettings::LOOP);
    }

    fn play_sound(audio: Res<Audio>, assets: Res<AssetServer>, mut events: EventReader<PlaySound>) {
        for PlaySound(sound) in events.iter() {
            audio.play(assets.load(sound));
        }
    }
    fn pause_on_lost_focus(mut time: ResMut<Time>, q_window: Query<&Window, With<PrimaryWindow>>) {
        if q_window.single().focused {
            time.unpause();
        } else {
            time.pause();
        }
    }

    fn preload(mut preloaded: Local<Vec<HandleUntyped>>, assets: Res<AssetServer>) {
        *preloaded = vec![
            assets.load_untyped("ding.ogg"),
            assets.load_untyped("dong.ogg"),
            assets.load_untyped("interfere.ogg"),
            assets.load_untyped("layer.png"),
            assets.load_untyped("layer_shot.png"),
            assets.load_untyped("shooter.png"),
            assets.load_untyped("shooter_shot.png"),
            assets.load_untyped("title.png"),
            assets.load_untyped("play.png"),
        ];
    }
}

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MousePosition(Vec3::ZERO))
            .add_startup_system(Self::music)
            .add_startup_system(Self::preload)
            .add_event::<PlaySound>()
            .add_system(Self::play_sound)
            .add_system(Self::pause_on_lost_focus)
            .add_system(Self::update_mouse_position.run_if(in_state(GameState::InGame)))
            .add_system(Self::update_lifespan.run_if(in_state(GameState::InGame)))
            .add_system(Self::velocity_abuse.run_if(in_state(GameState::InGame)));
    }
}
