mod cat;
mod dungeon;
mod ldtk;
mod player;
mod position;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tweening::*;
use dungeon::{setup_dungeon, Dungeon, DungeonLevel};

use cat::*;
use player::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_plugin(WorldInspectorPlugin::default())
        .add_event::<MessageEvent>()
        .init_resource::<Dungeon>()
        .insert_resource(DungeonLevel(0))
        .add_startup_system(setup_message)
        .add_startup_system(setup_dungeon)
        .add_startup_system(setup_player)
        .add_startup_system(setup_cats)
        .add_system(setup_cats_animation)
        .add_system(update_player)
        .add_system(update_message)
        .run();
}

pub struct MessageEvent(String);

#[derive(Component)]
struct MessageText;

fn setup_message(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((
        MessageText,
        TextBundle::from_section(
            "",
            TextStyle {
                font: asset_server.load("k8x12.ttf"),
                font_size: 36.0,
                color: Color::WHITE,
            },
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(10.0),
                left: Val::Px(20.0),
                ..default()
            },
            ..default()
        }),
    ));
}

fn update_message(
    mut message_events: EventReader<MessageEvent>,
    mut query: Query<&mut Text, With<MessageText>>,
) {
    if message_events.is_empty() {
        return;
    }
    let mut text = query.single_mut();
    for ev in message_events.iter() {
        text.sections[0].value = ev.0.clone()
    }
}
