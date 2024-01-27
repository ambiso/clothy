use bevy::prelude::*;

pub struct ControllerPlugin;

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_controller);
    }
}


#[derive(Component)]
pub struct CameraTarget;

const CAMERA_DIST: f32 = 15.0;

fn camera_controller(
    mut target: Query<&Transform, (With<CameraTarget>, Without<Camera>)>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<CameraTarget>)>,
) {
    for target in &mut target {
        for mut cam in &mut camera {
            let bird_view = target.rotation * Vec3::new(0.0, 0.0, 1.0);
            let mut cam_transform =
                Transform::from_translation(target.translation - (bird_view + Vec3::new(0.0, 1.0, 0.0)).normalize() * CAMERA_DIST);
            cam_transform.look_at(target.translation, Vec3::Y);
            *cam = cam_transform;
        }
    }
}
