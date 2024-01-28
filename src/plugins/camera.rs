use bevy::{prelude::*, transform::TransformSystem};
use bevy_xpbd_3d::PhysicsSet;

pub struct ControllerPlugin;

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            camera_controller
                .after(PhysicsSet::Sync)
                .before(TransformSystem::TransformPropagate),
        );
    }
}

#[derive(Component)]
pub struct CameraTarget;

const CAMERA_DIST: f32 = 15.0;

fn camera_controller(
    mut transform: Query<&GlobalTransform, (With<CameraTarget>, Without<Camera>)>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<CameraTarget>)>,
) {
    // std::thread::sleep(Duration::from_secs_f64(0.05));
    for target in &mut transform {
        for mut cam in &mut camera {
            let bird_view = target.compute_transform().rotation * Vec3::new(0.0, 0.0, 1.0);
            let mut cam_transform = Transform::from_translation(
                target.translation()
                    - (bird_view - Vec3::new(0.0, 0.5, 0.0)).normalize() * CAMERA_DIST,
            );
            cam_transform.look_at(target.translation(), Vec3::Y);
            *cam = cam_transform;
        }
    }
}
