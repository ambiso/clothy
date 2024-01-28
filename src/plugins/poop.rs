use crate::{Birb, Layer};
use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

pub(crate) struct PoopPlugin;

impl Plugin for PoopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, poop_setup)
            .add_systems(Update, poop);
    }
}

#[derive(Resource)]
pub(crate) struct PoopState {
    // pub(crate) poop_asset: Handle<Scene>,
    poop_mesh: Handle<Mesh>,
    poop_material: Handle<StandardMaterial>,
    last_poop: f64,
}

pub(crate) fn poop_setup(
    // asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let poop_mesh = meshes.add(Mesh::from(shape::UVSphere {
        radius: 0.5,
        sectors: 16,
        stacks: 16,
    }));
    let poop_material = materials.add(Color::rgb_u8(140, 69, 18).into());
    commands.insert_resource(PoopState {
        // poop_asset: asset_server.load("models/poop.gltf#Scene0"),
        poop_mesh,
        poop_material,
        last_poop: -1.0,
    });
}

#[derive(Component)]
pub struct Poop;

pub(crate) fn poop(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    birb: Query<(&Transform, &LinearVelocity, &AngularVelocity), With<Birb>>,
    mut poop_state: ResMut<PoopState>,
    time: Res<Time>,
) {
    // cooldown
    if time.elapsed_seconds_f64() - poop_state.last_poop > 0.5 {
        for (bt, lv, av) in &birb {
            if input.just_pressed(KeyCode::Space) {
                commands
                    .spawn(PbrBundle {
                        mesh: poop_state.poop_mesh.clone(),
                        material: poop_state.poop_material.clone(),
                        // scene: poop_state.poop_asset.clone(),
                        transform: *bt,
                        ..default()
                    })
                    .insert(CollisionLayers::new(
                        [Layer::Poop],
                        [Layer::Enemy, Layer::Ground, Layer::Poop],
                    ))
                    .insert((
                        RigidBody::Dynamic,
                        Collider::ball(0.3),
                        lv.clone(),
                        av.clone(),
                    ))
                    .insert(Poop);
                poop_state.last_poop = time.elapsed_seconds_f64();
            }
        }
    }
}
