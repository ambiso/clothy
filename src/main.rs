//! Skinned mesh example with mesh and joints data loaded from a glTF file.
//! Example taken from <https://github.com/KhronosGroup/glTF-Tutorials/blob/master/gltfTutorial/gltfTutorial_019_SimpleSkin.md>

use std::f32::consts::*;

use bevy::{pbr::AmbientLight, prelude::*, render::mesh::skinning::SkinnedMesh};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(AmbientLight {
            brightness: 1.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, joint_animation)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Create a camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
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
    keyboard_input: Res<Input<KeyCode>>,
    // world: &World,
    names: Query<&Name>,
) {
    let mut ran = false;
    // Iter skinned mesh entity
    for skinned_mesh_parent in &parent_query {
        ran = true;
        // dbg!(&skinned_mesh_parent);
        // Mesh node is the parent of the skinned mesh entity.
        let mesh_node_entity = skinned_mesh_parent.get();
        println!("Parent: {}", names.get(mesh_node_entity).map(|x| x.as_str()).unwrap_or("No Name"));
        for desc in children_query.iter_descendants(mesh_node_entity) {
        // dbg!(&desc);
        println!("{desc:?} {}", names.get(desc).map(|x| x.as_str()).unwrap_or("No Name"));
        //     dbg!(&world.inspect_entity(desc));
        }

        // Get `Children` in the mesh node.
        let mesh_node_children = children_query.get(mesh_node_entity).unwrap();
        let center_bone = mesh_node_children[1];
        let center_bone_children = children_query.get(center_bone).unwrap();

        let left1 = center_bone_children[0];
        let right1 = center_bone_children[1];

        let left2 = children_query.get(left1).unwrap()[0];
        let right2 = children_query.get(right1).unwrap()[0];

        let left3 = children_query.get(left2).unwrap()[0];
        let right3 = children_query.get(right2).unwrap()[0];

        let left4 = children_query.get(left3).unwrap()[0];
        let right4 = children_query.get(right3).unwrap()[0];

        // for child in  {
        //     println!("{}", names.get(*child).map(|x| x.as_str()).unwrap_or("No Name"));
        // }
        // let mesh_node_children = children_query.get(mesh_node_entity).unwrap();
        // dbg!(&mesh_node_children);
        // for child in mesh_node_children {
        //     let more_children = children_query.get(*child);
        //     dbg!(&more_children);
        // }

        // First joint is the second child of the mesh node.
        // let first_joint_entity = mesh_node_children[1];
        // dbg!(&first_joint_entity);
        // dbg!(&world.inspect_entity(first_joint_entity));
        // Get `Children` in the first joint.
        // let first_joint_children = children_query.get(first_joint_entity).unwrap();

        // Second joint is the first child of the first joint.
        // let second_joint_entity = first_joint_children[0];
        // dbg!(&second_joint_entity);

        // Get `Transform` in the second joint.
        // let mut second_joint_transform = transform_query.get_mut(second_joint_entity).unwrap();
        let mut transform = transform_query.get_mut(right2).unwrap();

        if keyboard_input.pressed(KeyCode::A) {
            dbg!(&transform);
            transform.rotation = Quat::from_rotation_z(TAU * time.elapsed_seconds().sin());
        }
    }
    if ran {
        // std::process::exit(0);
    }
}
