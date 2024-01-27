//! Skinned mesh example with mesh and joints data loaded from a glTF file.
//! Example taken from <https://github.com/KhronosGroup/glTF-Tutorials/blob/master/gltfTutorial/gltfTutorial_019_SimpleSkin.md>

use std::f32::consts::*;

use bevy::render::mesh::{Mesh, PrimitiveTopology};
use bevy::render::view::NoFrustumCulling;
use bevy::{
    pbr::AmbientLight,
    prelude::*,
    render::mesh::{skinning::SkinnedMesh, Indices},
};
use noise::{NoiseFn, Perlin};

#[derive(Resource)]
struct BirbState {
    original_rots: Option<Vec<Quat>>,
    angles: Vec<f32>,
    angular_velocity: Vec<f32>,
    global_pos: Transform
}

impl BirbState {
    fn new() -> Self {
        Self {
            original_rots: None,
            angles: vec![0.0; 8],
            angular_velocity: vec![0.0; 8],
            global_pos: Transform::from_xyz(0.0, 0.0, 0.0)
        }
    }
}

#[derive(Component)]
struct Terrain;

#[derive(Component)]
struct Birb;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(AmbientLight {
            brightness: 1.0,
            ..default()
        })
        .insert_resource(BirbState::new())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                joint_animation,
                birb_inputs,
                move_terrain,
                birb_physics_update,
                birb_visibility_for_debug,
            ),
        )
        .run();
}

fn birb_visibility_for_debug(
    mut birb: Query<&mut Visibility, With<Birb>>,
    inputs: Res<Input<KeyCode>>,
) {
    if inputs.just_pressed(KeyCode::F8) {
        for mut b in &mut birb {
            match *b {
                Visibility::Visible | Visibility::Inherited => {
                    *b = Visibility::Hidden;
                }
                Visibility::Hidden => {
                    *b = Visibility::Visible;
                }
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 4.5, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Spawn the first scene in `models/SimpleSkin/SimpleSkin.gltf`
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/birb2.gltf#Scene0"),
        ..default()
    }).insert(Birb);

    generate_terrain(&mut commands, &mut meshes, &mut materials);
}

fn generate_terrain(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let size = 100; // Size of the terrain
    let max_height = 3.0; // Maximum elevation of the terrain
    let perlin = Perlin::new(1337); // Perlin noise generator

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Generate terrain vertices
    for x in 0..size {
        for z in 0..size {
            let perlin_scale = 0.1;
            let p = [x as f64 * perlin_scale, z as f64 * perlin_scale];
            let height = perlin.get(p) as f32 * max_height;
            let delta = 0.001;
            let height_xm = perlin.get([p[0] - delta, p[1]]) as f32 * max_height;
            let height_zm = perlin.get([p[0], p[1] - delta]) as f32 * max_height;
            let height_xp = perlin.get([p[0] + delta, p[1]]) as f32 * max_height;
            let height_zp = perlin.get([p[0], p[1] + delta]) as f32 * max_height;
            let original_point = Vec3::new(p[0] as f32, height, p[1] as f32);
            let x_point = Vec3::new((p[0] + delta) as f32, height_xp, p[1] as f32)
                - Vec3::new((p[0] - delta) as f32, height_xm, p[1] as f32);
            let z_point = Vec3::new(p[0] as f32, height_zp, (p[1] + delta) as f32)
                - Vec3::new(p[0] as f32, height_zm, (p[1] - delta) as f32);
            let real_normal = x_point.cross(z_point).normalize();
            // let real_normal = Vec3::new(0.0, 1.0, 0.0);

            let linie = [original_point, original_point + real_normal];
            let normal_display_mesh = Mesh::new(PrimitiveTopology::LineList)
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, linie.to_vec());
            let arrow_mesh_handle = meshes.add(normal_display_mesh);
            let arrow_mat = materials.add(Color::RED.into());

            commands
                .spawn(PbrBundle {
                    transform: Transform::from_xyz(x as f32, height, z as f32),
                    mesh: arrow_mesh_handle.clone(),
                    material: arrow_mat.clone(),
                    ..default()
                })
                .insert(Terrain);

            positions.push([x as f32, height, z as f32]);
            normals.push(real_normal);
            uvs.push([x as f32 / (size - 1) as f32, z as f32 / (size - 1) as f32]);
        }
    }

    // Generate indices for the mesh
    for x in 0..(size - 1) {
        for z in 0..(size - 1) {
            let start = x * size + z;
            indices.extend(&[
                (start + size + 1) as u32,
                (start + size) as u32,
                start as u32,
                (start + 1) as u32,
                (start + size + 1) as u32,
                start as u32,
            ]);
        }
    }

    // Create the mesh
    let mesh = Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_indices(Some(Indices::U32(indices)));

    // Spawn the terrain entity
    commands
        .spawn(PbrBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            mesh: meshes.add(mesh),
            material: materials.add(Color::GREEN.into()),
            ..default()
        })
        .insert(NoFrustumCulling)
        .insert(Terrain);
}

fn move_terrain(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Terrain>>,
) {
    // Speed of movement
    let speed = 2.0;

    for mut transform in query.iter_mut() {
        // Move the terrain based on keyboard input
        if keyboard_input.pressed(KeyCode::Down) {
            transform.translation.z += speed;
        }
        if keyboard_input.pressed(KeyCode::Up) {
            transform.translation.z -= speed;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.x -= speed;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.x += speed;
        }
        if keyboard_input.pressed(KeyCode::Numpad2) || keyboard_input.pressed(KeyCode::Key2) {
            transform.translation.y -= speed;
        }
        if keyboard_input.pressed(KeyCode::Numpad8) || keyboard_input.pressed(KeyCode::Key8) {
            transform.translation.y += speed;
        }
    }
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
    parent_query: Query<&Parent, With<SkinnedMesh>>,
    children_query: Query<&Children>,
    time: Res<Time>,
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
            let bone_pos = transform_query.get(*entity).unwrap();
            let wind_force: Vec3 = calculate_wind_force(&time, bone_pos);
            let rot: &mut Quat = &mut transform_query.get_mut(*entity).unwrap().rotation;
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

const ANGULAR_ACCELERATION: f32 = 20.0;
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

fn calculate_wind_force(
    time: &Res<Time>,
    bone: &Transform,
) -> Vec3 {
    let perlin = Perlin::new(1337);
    let time_factor = time.elapsed_seconds_f64();

    let position = bone.translation;

    // Use Perlin noise to generate wind force
    let wind_force_x = perlin.get([position.x as f64, time_factor]) as f32;
    let wind_force_y = perlin.get([position.y as f64, time_factor]) as f32;
    let wind_force_z = perlin.get([position.z as f64, time_factor]) as f32;
    Vec3::new(wind_force_x, wind_force_y, wind_force_z)
}
