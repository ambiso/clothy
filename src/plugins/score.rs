use bevy::prelude::*;

use crate::GameState;

#[derive(Component)]
pub struct ScoreText;

#[derive(Resource)]
pub struct ScoreState {
    pub distance: f32,
}

#[derive(Component)]
pub struct ScoreTarget {
    pub last_pos: Vec3,
}

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup_text)
            .add_systems(Update, text_update_system);
    }
}

fn setup_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Text with multiple sections
    let bold_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let medium_font = asset_server.load("fonts/FiraMono-Medium.ttf");
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: bold_font.clone(),
                    font_size: 60.0,
                    ..default()
                },
            ),
            TextSection::from_style(
                // "default_font" feature is unavailable, load a font to use instead.
                TextStyle {
                    font: medium_font.clone(),
                    font_size: 60.0,
                    color: Color::GOLD,
                },
            ),
            TextSection::new(
                "Distance: ",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: bold_font.clone(),
                    font_size: 60.0,
                    ..default()
                },
            ),
            TextSection::from_style(
                // "default_font" feature is unavailable, load a font to use instead.
                TextStyle {
                    font: medium_font.clone(),
                    font_size: 60.0,
                    color: Color::GOLD,
                },
            ),
            TextSection::new(
                "Collectibles: ",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: bold_font.clone(),
                    font_size: 60.0,
                    ..default()
                },
            ),
            TextSection::from_style(
                // "default_font" feature is unavailable, load a font to use instead.
                TextStyle {
                    font: medium_font.clone(),
                    font_size: 60.0,
                    color: Color::GOLD,
                },
            ),
        ]),
        ScoreText,
    ));
}

fn text_update_system(
    mut query: Query<&mut Text, With<ScoreText>>,
    mut target: Query<(&Transform, &mut ScoreTarget)>,
    mut state: ResMut<ScoreState>,
    gamestate: Res<GameState>,
) {
    for mut text in &mut query {
        for (tt, mut tst) in &mut target {
            state.distance += (tt.translation - tst.last_pos).length();
            tst.last_pos = tt.translation;
        }
        let coll = gamestate.waypoints_achieved_counter;
        let distance = state.distance;
        text.sections[1].value = format!("{:.2}\n", 2000.0 * coll as f32 + distance);
        text.sections[3].value = format!("{distance:.2}\n");
        text.sections[5].value = format!("{coll:.2}\n");
    }
}
