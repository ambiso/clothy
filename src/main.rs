//! Renders a 2D scene containing a single, moving sprite.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, move_sprites)
        .run();
}

#[derive(Component)]
enum Direction {
    Up,
    Down,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("icon.png"),
            transform: Transform::from_xyz(100., 0., 0.),
            ..default()
        },
        Direction::Up,
    ));
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn move_sprites(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut sprite: Query<(&mut Direction, &mut Transform)>,
) {
    for (mut _dir, mut transform) in sprite.iter_mut() {
        // match *dir {
        //     Direction::Up => transform.translation.y += ,
        //     Direction::Down => transform.translation.y -= 150. * time.delta_seconds(),
        // }
        let pos_change = 1500. * time.delta_seconds();

        if keyboard_input.pressed(KeyCode::Down) {
            info!("'Down' currently pressed");
            transform.translation.y -= pos_change;
        }
        if keyboard_input.pressed(KeyCode::Up) {
            info!("'Up' currently pressed");
            transform.translation.y += pos_change;
        }
        if keyboard_input.pressed(KeyCode::Left) {
            info!("'Left' currently pressed");
            transform.translation.x -= pos_change;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            info!("'Right' currently pressed");
            transform.translation.x += pos_change;
        }
    }
}
