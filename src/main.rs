//! Skinned mesh example with mesh and joints data loaded from a glTF file.
//! Example taken from <https://github.com/KhronosGroup/glTF-Tutorials/blob/master/gltfTutorial/gltfTutorial_019_SimpleSkin.md>

use std::f32::consts::*;

use bevy::{pbr::AmbientLight, prelude::*, render::mesh::skinning::SkinnedMesh};

#[derive(Resource)]
struct BirbState {
    original_rots: Option<Vec<Quat>>,
    angles: Vec<f32>,
    angular_velocity: Vec<f32>,
}

impl BirbState {
    fn new() -> Self {
        Self {
            original_rots: None,
            angles: vec![0.0; 8],
            angular_velocity: vec![0.0; 8],
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(AmbientLight {
            brightness: 1.0,
            ..default()
        })
        .insert_resource(BirbState::new())
        .add_systems(Startup, setup)
        .add_systems(Update, (joint_animation, birb_inputs, birb_physics_update))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Create a camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 4.5, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Spawn the first scene in `models/SimpleSkin/SimpleSkin.gltf`
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/birb2.gltf#Scene0"),
        ..default()
    });
}

/// The scene hierarchy currently looks somewhat like this:
///
/// ```ignore
/// <Parent entity>
///   + Mesh node (without `PbrBundle` or `SkinnedMesh` component)
///     + Skinned mesh entity (with `PbrBundle` and `SkinnedMesh` component, created by glTF loader)
///     + First joint
///       + Second joint
/// ```
///
/// In this example, we want to get and animate the second joint.
/// It is similar to the animation defined in `models/SimpleSkin/SimpleSkin.gltf`.
fn joint_animation(
    time: Res<Time>,
    parent_query: Query<&Parent, With<SkinnedMesh>>,
    children_query: Query<&Children>,
    mut transform_query: Query<&mut Transform>,
    mut birb_state: ResMut<BirbState>,
    // names: Query<&Name>,
) {
    let mut ran = false;
    // Iter skinned mesh entity
    for skinned_mesh_parent in &parent_query {
        ran = true;
        // dbg!(&skinned_mesh_parent);
        // Mesh node is the parent of the skinned mesh entity.
        let mesh_node_entity = skinned_mesh_parent.get();
        // println!(
        //     "Parent: {}",
        //     names
        //         .get(mesh_node_entity)
        //         .map(|x| x.as_str())
        //         .unwrap_or("No Name")
        // );
        // for desc in children_query.iter_descendants(mesh_node_entity) {
        // dbg!(&desc);
        // println!(
        //     "{desc:?} {}",
        //     names.get(desc).map(|x| x.as_str()).unwrap_or("No Name")
        // );
        //     dbg!(&world.inspect_entity(desc));
        // }

        // Get `Children` in the mesh node.
        let mesh_node_children = children_query.get(mesh_node_entity).unwrap();

        let center_bone = mesh_node_children[1];
        let center_bone_children = children_query.get(center_bone).unwrap();

        let left1 = center_bone_children[1];
        let right1 = center_bone_children[0];

        let left2 = children_query.get(left1).unwrap()[0];
        let right2 = children_query.get(right1).unwrap()[0];

        let left3 = children_query.get(left2).unwrap()[0];
        let right3 = children_query.get(right2).unwrap()[0];

        let left4 = children_query.get(left3).unwrap()[0];
        let right4 = children_query.get(right3).unwrap()[0];

        if birb_state.original_rots.is_none() {
            let mut prev_rots = vec![];
            for entity in [left4, left3, left2, left1, right1, right2, right3, right4] {
                prev_rots.push(transform_query.get_mut(entity).unwrap().rotation);
            }
            birb_state.original_rots = Some(prev_rots);
        }
        for ((entity, angle), orig_rot) in
            [left4, left3, left2, left1, right1, right2, right3, right4]
                .iter()
                .zip(birb_state.angles.iter())
                .zip(birb_state.original_rots.as_ref().unwrap().iter())
        {
            let rot = &mut transform_query.get_mut(*entity).unwrap().rotation;
            *rot = *orig_rot * Quat::from_rotation_x(*angle);
        }
    }
}

fn birb_inputs(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut birb_state: ResMut<BirbState>,
    // world: &World,
    // names: Query<&Name>,
) {
    for (key, angular_vel) in [
        KeyCode::A,
        KeyCode::S,
        KeyCode::D,
        KeyCode::F,
        KeyCode::J,
        KeyCode::K,
        KeyCode::L,
        KeyCode::Semicolon,
    ]
    .iter()
    .zip(birb_state.angular_velocity.iter_mut())
    {
        *angular_vel += ANGULAR_ACCELERATION
            * time.delta_seconds()
            * if keyboard_input.pressed(*key) {
                1.0
            } else {
                -1.0
            };
    }
}

const ANGULAR_ACCELERATION: f32 = 3.0;
const MIN_ANGLE: f32 = -0.15 * PI;
const MAX_ANGLE: f32 = 0.15 * PI;

fn birb_physics_update(time: Res<Time>, mut birb_state: ResMut<BirbState>) {
    let birb_state = &mut *birb_state;
    for (angle, angular_vel) in birb_state
        .angles
        .iter_mut()
        .zip(birb_state.angular_velocity.iter_mut())
    {
        let mut new_angle = *angle + *angular_vel * time.delta_seconds();
        if new_angle < MIN_ANGLE {
            new_angle = MIN_ANGLE;
            *angular_vel = 0.0;
        }
        if new_angle > MAX_ANGLE {
            new_angle = MAX_ANGLE;
            *angular_vel = 0.0;
        }
        *angle = new_angle;
    }
}
