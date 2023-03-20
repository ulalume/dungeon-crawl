mod cat;
mod dungeon;
mod ldtk;
mod player;
mod position;

use bevy::{prelude::*, window::Window};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tweening::*;
use dungeon::{setup_dungeon, Dungeon, DungeonLevel};

use cat::*;
use player::*;

fn main() {
    let window = Window {
        resolution: (640.0, 480.0).into(),
        resizable: true,
        title: "Dungeon".to_string(),
        ..default()
    };
    let primary_window = Some(window);
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window,
                    ..default()
                }),
        )
        .add_plugin(TweeningPlugin)
        .add_plugin(WorldInspectorPlugin::default())
        .add_event::<MessageEvent>()
        .insert_resource(Msaa::Off)
        .init_resource::<Dungeon>()
        .insert_resource(DungeonLevel(0))
        .add_startup_system(setup_message)
        .add_startup_system(setup_dungeon)
        .add_startup_system(setup_player)
        .add_startup_system(setup_cats)
        .add_system(setup_cats_animation)
        .add_system(update_player)
        .add_system(update_message)
        .add_system(update_button)
        .run();
}

pub struct MessageEvent(String);

#[derive(Component)]
struct MessageText;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

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

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Button",
                        TextStyle {
                            font: asset_server.load("k8x12.ttf"),
                            font_size: 36.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
        });
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

fn update_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Press".to_string();
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Button".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}
