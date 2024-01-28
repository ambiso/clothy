//! Skinned mesh example with mesh and joints data loaded from a glTF file.
//! Example taken from <https://github.com/KhronosGroup/glTF-Tutorials/blob/master/gltfTutorial/gltfTutorial_019_SimpleSkin.md>

use std::f32::consts::*;
// use std::ops::Mul;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::render::mesh::shape::{Icosphere, UVSphere};
// use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
use bevy::render::mesh::{Mesh, PrimitiveTopology};
use bevy::render::view::NoFrustumCulling;
use bevy::utils::HashMap;
use bevy::{
    pbr::AmbientLight,
    prelude::*,
    render::mesh::{skinning::SkinnedMesh, Indices},
};
use bevy_xpbd_3d::prelude::*;
use noise::{NoiseFn, Perlin};
use plugins::camera::CameraTarget;
use plugins::score::{ScorePlugin, ScoreState, ScoreTarget};

mod plugins;

#[derive(Resource)]
struct BirbState {
    original_rots: Option<Vec<Quat>>,
    angles: Vec<f32>,
    angular_velocity: Vec<f32>,
    wing_joints: Option<Vec<Entity>>,
    up_force: f32,
}

impl BirbState {
    fn new() -> Self {
        Self {
            original_rots: None,
            angles: vec![0.0; 8],
            angular_velocity: vec![0.0; 8],
            wing_joints: None,
            up_force: 0.0,
        }
    }
}

#[derive(Component)]
struct Terrain;

#[derive(Resource)]
struct TerrainState {
    chunk_size: u32,
    view_radius: f32,
    loaded_chunks: HashMap<(i32, i32), Entity>, // Stores the coordinates of loaded chunks
}

impl TerrainState {
    pub fn new(chunk_size: u32, view_radius: f32) -> Self {
        TerrainState {
            chunk_size,
            view_radius,
            loaded_chunks: HashMap::new(),
        }
    }

    // Function to check if a chunk is loaded
    pub fn is_chunk_loaded(&self, x: i32, z: i32) -> bool {
        self.loaded_chunks.contains_key(&(x, z))
    }

    // Function to mark a chunk as loaded
    pub fn add_chunk(&mut self, x: i32, z: i32, entity: Entity) {
        self.loaded_chunks.insert((x, z), entity);
    }

    // Function to remove a chunk from the loaded set
    pub fn remove_chunk(&mut self, x: i32, z: i32) {
        self.loaded_chunks.remove(&(x, z));
    }
}

#[derive(Component)]
struct Collectible;

#[derive(Resource)]
struct GameState {
    waypoints_achieved_counter: u32,
}

#[derive(Component)]
struct Birb;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum AppState {
    MainMenu,
    #[default]
    InGame,
    Paused,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
        ))
        // .insert_resource(Gravity(Vec3::ZERO))
        .insert_resource(AmbientLight {
            brightness: 1.0,
            ..default()
        })
        .add_state::<AppState>()
        .insert_resource(BirbState::new())
        .insert_resource(TerrainState::new(128, 512.0 * CHUNK_SIZE_WORLD_SPACE_MUL))
        .add_plugins(ScorePlugin)
        .insert_resource(ScoreState {
            origin: BIRB_SPAWN.translation,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_terrain_system,
                move_terrain,
                respawn_birb_when_grounded,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(Update, (birb_inputs, birb_physics_update, joint_animation))
        .add_systems(Update, menu_stuff)
        .add_systems(Update, debug_keys)
        .add_plugins(plugins::camera::ControllerPlugin)
        // .edit_schedule(PostUpdate, |schedule| {
        //     schedule.set_build_settings(ScheduleBuildSettings {
        //         ambiguity_detection: LogLevel::Warn,
        //         ..default()
        //     });
        // })
        .run();
}

fn menu_stuff(
    mut physics_time: ResMut<Time<Physics>>,
    current_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    inputs: Res<Input<KeyCode>>,
) {
    if inputs.just_pressed(KeyCode::Escape) {
        match **current_state {
            AppState::MainMenu => todo!(),
            AppState::InGame => {
                next_state.set(AppState::Paused);
                physics_time.pause();
            }
            AppState::Paused => {
                next_state.set(AppState::InGame);
                physics_time.unpause();
            }
        }
    }
}

fn debug_keys(
    mut commands: Commands,
    mut birb_visiblity: Query<&mut Visibility, With<Birb>>,
    mut birb_physics: Query<Entity, With<Birb>>,
    inputs: Res<Input<KeyCode>>,
) {
    if inputs.just_pressed(KeyCode::F8) {
        for mut b in &mut birb_visiblity {
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
    if inputs.just_pressed(KeyCode::F9) {
        for b in &mut birb_physics {
            commands.entity(b).remove::<RigidBody>();
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
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/birb2.gltf#Scene0"),
            transform: BIRB_SPAWN,
            ..default()
        })
        .insert((
            RigidBody::Dynamic,
            LinearDamping(0.16),
            AngularDamping(1.6),
            Collider::ball(0.5),
            ExternalForce::default().with_persistence(false),
        ))
        .insert(CameraTarget)
        .insert(ScoreTarget)
        .insert(Birb);

    commands.spawn(DirectionalLightBundle {
        ..Default::default()
    });

    let perlin = Perlin::new(42);
    let scale = 10.0; // Scale for noise coordinates
                      // spawn collectibles
    for i in 0..100 {
        // Create an icosphere
        let uvsphere_mesh = Mesh::from(UVSphere {
            radius: 3.0,
            sectors: 14,
            stacks: 14,
        });

        // Use Perlin noise for position
        let position = Vec3::new(
            perlin.get([i as f64 * 0.1, 0.0, 0.0]) as f32 * scale,
            perlin.get([0.0, i as f64 * 0.1, 0.0]) as f32 * scale,
            perlin.get([0.0, 0.0, i as f64 * 0.1]) as f32 * scale,
        );

        // let position = Vec3::new(
        //     i as f32 * 10.0, 10.0, 10.0
        // );

        commands
            .spawn(PbrBundle {
                mesh: meshes.add(uvsphere_mesh),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_translation(position),
                ..Default::default()
            })
            .insert(Collectible);
    }

    // generate_terrain(&mut commands, &mut meshes, &mut materials);
}

const BIRB_SPAWN: Transform = Transform::from_xyz(0.0, 100.0, 0.0);

fn update_terrain_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut terrain_state: ResMut<TerrainState>,
    birb_query: Query<&GlobalTransform, With<Birb>>,
) {
    if let Some(player_transform) = birb_query.iter().next() {
        let player_pos = player_transform.compute_transform().translation;
        let chunk_size = terrain_state.chunk_size;
        let view_radius = terrain_state.view_radius;

        // Determine the range of chunks that should be loaded
        let min_chunk_x =
            ((player_pos.x - view_radius) / chunk_size as f32 / CHUNK_SIZE_WORLD_SPACE_MUL).floor()
                as i32;
        let max_chunk_x =
            ((player_pos.x + view_radius) / chunk_size as f32 / CHUNK_SIZE_WORLD_SPACE_MUL).ceil()
                as i32;
        let min_chunk_z =
            ((player_pos.z - view_radius) / chunk_size as f32 / CHUNK_SIZE_WORLD_SPACE_MUL).floor()
                as i32;
        let max_chunk_z =
            ((player_pos.z + view_radius) / chunk_size as f32 / CHUNK_SIZE_WORLD_SPACE_MUL).ceil()
                as i32;

        for x in min_chunk_x..=max_chunk_x {
            for z in min_chunk_z..=max_chunk_z {
                // Check if this chunk is already loaded
                if !terrain_state.is_chunk_loaded(x, z) {
                    // Generate this chunk
                    let chunk_world_x = x as f32 * chunk_size as f32 * CHUNK_SIZE_WORLD_SPACE_MUL;
                    let chunk_world_z = z as f32 * chunk_size as f32 * CHUNK_SIZE_WORLD_SPACE_MUL;

                    //dbg!(min_chunk_x, max_chunk_x, min_chunk_z, max_chunk_z, chunk_world_x, chunk_world_z);
                    //dbg!();

                    let chunk_entity = generate_terrain_chunk(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        chunk_world_x,
                        chunk_world_z,
                        chunk_size,
                    );

                    // Mark this chunk as loaded
                    terrain_state.add_chunk(x, z, chunk_entity);
                }
            }
        }

        // Unload distant chunks
        let mut chunks_to_unload = Vec::new();
        for (&(x, z), &entity) in terrain_state.loaded_chunks.iter() {
            if x < min_chunk_x || x > max_chunk_x || z < min_chunk_z || z > max_chunk_z {
                chunks_to_unload.push((x, z, entity));
            }
        }

        for (x, z, entity) in chunks_to_unload {
            commands.entity(entity).despawn();
            terrain_state.remove_chunk(x, z);
        }
    }
}

const CHUNK_SIZE_WORLD_SPACE_MUL: f32 = 12.0;

// Assuming generate_terrain_chunk is defined to generate a single chunk at specified coordinates.
fn generate_terrain_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_x: f32,
    chunk_z: f32,
    chunk_size: u32, // Assuming chunk_size is the number of vertices along one edge of the chunk
) -> Entity {
    let max_height = 15.0; // Maximum elevation of the terrain
    let perlin = Perlin::new(1337); // Perlin noise generator

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Generate terrain vertices
    for x in 0..=chunk_size {
        for z in 0..=chunk_size {
            let world_x = chunk_x + x as f32 * CHUNK_SIZE_WORLD_SPACE_MUL;
            let world_z = chunk_z + z as f32 * CHUNK_SIZE_WORLD_SPACE_MUL;
            let perlin_scale = 0.01;
            let p = [world_x as f64 * perlin_scale, world_z as f64 * perlin_scale];
            let height = perlin.get(p) as f32 * max_height;

            let delta = 0.001;
            let height_xm = perlin.get([p[0] - delta, p[1]]) as f32 * max_height;
            let height_zm = perlin.get([p[0], p[1] - delta]) as f32 * max_height;
            let height_xp = perlin.get([p[0] + delta, p[1]]) as f32 * max_height;
            let height_zp = perlin.get([p[0], p[1] + delta]) as f32 * max_height;

            let x_point = Vec3::new((p[0] + delta) as f32, height_xp, p[1] as f32)
                - Vec3::new((p[0] - delta) as f32, height_xm, p[1] as f32);
            let z_point = Vec3::new(p[0] as f32, height_zp, (p[1] + delta) as f32)
                - Vec3::new(p[0] as f32, height_zm, (p[1] - delta) as f32);
            let real_normal = x_point.cross(z_point).normalize();

            //let real_normal = Vec3::new(0.0, 1.0, 0.0);

            // let linie = [original_point, original_point + real_normal];
            // let normal_display_mesh = Mesh::new(PrimitiveTopology::LineList)
            //     .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, linie.to_vec());
            // let arrow_mesh_handle = meshes.add(normal_display_mesh);
            // let arrow_mat = materials.add(Color::RED.into());

            // commands
            //     .spawn(PbrBundle {
            //         transform: Transform::from_xyz(x as f32, height, z as f32),
            //         mesh: arrow_mesh_handle.clone(),
            //         material: arrow_mat.clone(),
            //         ..default()
            //     })
            //     .insert(Terrain);

            positions.push([world_x as f32, height, world_z as f32]);
            normals.push(real_normal);
            uvs.push([
                x as f32 / (chunk_size - 1) as f32,
                z as f32 / (chunk_size - 1) as f32,
            ]);
        }
    }

    // Generate indices for the mesh
    for x in 0..(chunk_size) {
        for z in 0..(chunk_size) {
            let start = x * (chunk_size + 1) + z;
            indices.extend(&[
                (start + chunk_size + 2) as u32,
                (start + chunk_size + 1) as u32,
                start as u32,
                (start + 1) as u32,
                (start + chunk_size + 2) as u32,
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
            // transform: Transform::from_xyz(chunk_x, 0.0, chunk_z),
            transform: Transform::from_xyz(0.0, -10.0, 0.0),
            mesh: meshes.add(mesh.clone()),
            material: materials.add(Color::GREEN.into()),
            ..default()
        })
        .insert((
            RigidBody::Static,
            Collider::convex_hull_from_mesh(&mesh).unwrap(),
        ))
        .insert(NoFrustumCulling)
        .insert(Terrain)
        .id()
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
    mut transform_query: Query<&mut Transform>,
    mut birb_state: ResMut<BirbState>,
    time: Res<Time>,
    // names: Query<&Name>,
) {
    // Iter skinned mesh entity
    for skinned_mesh_parent in &parent_query {
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
        if birb_state.wing_joints.is_none() {
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

            birb_state.wing_joints = Some(vec![
                left4, left3, left2, left1, right1, right2, right3, right4,
            ]);
        }
        if birb_state.original_rots.is_none() {
            let mut prev_rots = vec![];
            for entity in birb_state.wing_joints.as_ref().unwrap() {
                prev_rots.push(transform_query.get_mut(*entity).unwrap().rotation);
            }
            birb_state.original_rots = Some(prev_rots);
        }
        for ((entity, angle), orig_rot) in birb_state
            .wing_joints
            .as_ref()
            .unwrap()
            .iter()
            .zip(birb_state.angles.iter())
            .zip(birb_state.original_rots.as_ref().unwrap().iter())
        {
            let wing_joint_transform = &mut transform_query.get_mut(*entity).unwrap();
            let wind_force: Quat =
                calculate_turbulence_rotation(&time, wing_joint_transform.translation);
            let rot = &mut wing_joint_transform.rotation;
            *rot = wind_force * *orig_rot * Quat::from_rotation_x(*angle);
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
    .rev()
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

    birb_state.up_force = if keyboard_input.pressed(KeyCode::V) {
        1.0
    } else if keyboard_input.pressed(KeyCode::N) {
        -1.0
    } else {
        0.0
    };
}

const ANGULAR_ACCELERATION: f32 = 20.0;
const MIN_ANGLE: f32 = -0.15 * PI;
const MAX_ANGLE: f32 = 0.15 * PI;

fn birb_physics_update(
    time: Res<Time>,
    mut birb_state: ResMut<BirbState>,
    mut birb: Query<(&mut ExternalForce, &GlobalTransform), With<Birb>>,
    global_transforms: Query<&GlobalTransform>,
    app_state: Res<State<AppState>>,
) {
    let birb_state = &mut *birb_state;
    let paused = **app_state != AppState::InGame;
    if let Some(wing_joints) = birb_state.wing_joints.as_ref() {
        for ((angle, angular_vel), wing_joint) in birb_state
            .angles
            .iter_mut()
            .zip(birb_state.angular_velocity.iter_mut())
            .zip(wing_joints)
        {
            // let wing_joint_global_transform = global_transforms.get(*wing_joint).unwrap();
            //
            // let rot: &mut Quat = &mut transforms.get_mut(*entity).unwrap().rotation;
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
        let lengths = [1.14, 2.57, 2.24, 1.29];
        let area = [2.0, 1.0, 0.5, 0.25];
        let mut acc_vels = [0.0; 8];
        let mut acc_angles = [0.0; 8];
        for side in 0..2 {
            let mut acc_vel = 0.0;
            let mut acc_ang_vel = 0.0;
            let mut acc_angle = 0.0;
            for joint in 0..4 {
                let length = lengths[joint];
                let idx = side * 4 + if side == 0 { 3 - joint } else { joint };
                acc_ang_vel += birb_state.angular_velocity[idx];
                acc_vel += acc_ang_vel * length * area[joint];
                acc_vels[idx] = acc_vel;

                acc_angle += birb_state.angles[idx];
                acc_angles[idx] = acc_angle;
            }
        }
        for (i, ((wing_joint, accumulated_angular_vel), acc_angle)) in
            wing_joints.iter().zip(acc_vels).zip(acc_angles).enumerate()
        {
            let wing_joint_global_transform = global_transforms.get(*wing_joint).unwrap();
            let wind_force: Vec3 = calculate_wind_force(&time, wing_joint_global_transform) * 0.01;
            for (mut b, bt) in &mut birb {
                if !paused {
                    b.apply_force_at_point(
                        wind_force,
                        wing_joint_global_transform.translation(),
                        bt.translation(),
                    );
                    b.apply_force(
                        (bt.compute_transform().rotation * Vec3::new(0.0, 1.5, 1.0))
                            * if accumulated_angular_vel <= 0.0 {
                                1.0
                            } else {
                                5.0
                            }
                            * accumulated_angular_vel
                            * time.delta_seconds(),
                    );
                    b.apply_force_at_point(
                        (bt.compute_transform().rotation
                            * Quat::from_rotation_z(if i >= 4 { -1.0 } else { 1.0 } * acc_angle)
                            * Vec3::new(0.0, 1.5, -0.05))
                            * if accumulated_angular_vel <= 0.0 {
                                0.001
                            } else {
                                0.1
                            }
                            * accumulated_angular_vel
                            * time.delta_seconds(),
                        wing_joint_global_transform.translation(),
                        bt.translation(),
                    );
                }
            }
        }
    }

    // up & down

    if !paused {
        for (mut b, bt) in &mut birb {
            b.apply_force_at_point(
                // (wing_rot.rotation * Vec3::new(0.0, 0.0, -1.0))
                bt.compute_transform().rotation * Vec3::new(0.0, -0.35, 0.0) * birb_state.up_force,
                bt.translation() + bt.compute_transform().rotation * Vec3::new(0.0, 0.0, -1.0),
                bt.translation(),
            );
        }
    }
    // dbg!();
}

fn calculate_wind_force(time: &Res<Time>, bone: &GlobalTransform) -> Vec3 {
    let perlin = Perlin::new(1337);
    let time_factor = time.elapsed_seconds_f64();

    let position = bone.translation();

    // Use Perlin noise to generate wind force
    let wind_force_x = perlin.get([
        position.x as f64,
        position.x as f64,
        position.z as f64,
        time_factor,
    ]) as f32;
    let wind_force_y = perlin.get([
        position.x as f64 + 100.,
        position.x as f64,
        position.z as f64,
        time_factor,
    ]) as f32;
    let wind_force_z = perlin.get([
        position.x as f64 + 200.,
        position.x as f64,
        position.z as f64,
        time_factor,
    ]) as f32;
    Vec3::new(wind_force_x, wind_force_y, wind_force_z)
}

fn calculate_turbulence_rotation(time: &Res<Time>, wing_position: Vec3) -> Quat {
    let perlin = Perlin::new(1337);
    let time_factor = time.elapsed_seconds_f64();

    // Adjust these scales to control the intensity and frequency of the turbulence
    let scale = 3.0; // Scale for the noise to keep rotations subtle
    let time_scale = 2.0; // Scale for time to control the speed of changes

    let intensity = 0.02 + perlin.get([time_factor, 0.0]) as f32 * 0.12;

    // Generate Perlin noise values for each axis
    let rotation_x =
        perlin.get([wing_position.x as f64, time_factor * time_scale, 0.0]) as f32 * scale;
    let rotation_y = perlin.get([
        wing_position.y as f64 + 100.0, // Offset to ensure different noise value
        time_factor * time_scale,
        0.0,
    ]) as f32
        * scale;
    let rotation_z = perlin.get([
        wing_position.z as f64 + 200.0, // Further offset
        time_factor * time_scale,
        0.0,
    ]) as f32
        * scale;

    // Create quaternions for each axis and multiply them to combine the rotations
    (Quat::from_rotation_x(rotation_x)
        * Quat::from_rotation_y(rotation_y)
        * Quat::from_rotation_z(rotation_z))
    .lerp(Quat::IDENTITY, 1.0 - intensity)
}

fn respawn_birb_when_grounded(
    mut commands: Commands,
    mut collision_event_reader: EventReader<Collision>,
    mut birb: Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity), With<Birb>>,
    terrains: Query<&Terrain>,
    collectibles: Query<&Collectible>,
    mut score_state: ResMut<ScoreState>,
) {
    for Collision(a) in collision_event_reader.read() {
        info!("Collsision");
        if birb.get(a.entity1).is_ok() || birb.get(a.entity2).is_ok() {
            if terrains.get(a.entity1).is_ok() || terrains.get(a.entity2).is_ok() {
                for (mut bt, mut lv, mut av) in &mut birb {
                    bt.translation.y = BIRB_SPAWN.translation.y;
                    bt.rotation = BIRB_SPAWN.rotation;
                    score_state.origin = bt.translation;
                    lv.0 = Vec3::ZERO;
                    av.0 = Vec3::ZERO;
                }
            } else if collectibles.get(a.entity1).is_ok() || collectibles.get(a.entity2).is_ok() {
                let collectible_entity = if collectibles.get(a.entity1).is_ok() {
                    a.entity1
                } else {
                    a.entity2
                };

                // Despawn the collectible
                commands.entity(collectible_entity).despawn();

                // Increment the score
                // GameState += 1;
            }
        }
    }
}
