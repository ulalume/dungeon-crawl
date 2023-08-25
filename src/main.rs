mod cat;
mod dungeon;
mod ldtk;
mod player;
mod position;
mod saving;
use bevy::{
    prelude::*,
    window::{Window, WindowMode},
};
use bevy_tweening::*;
use cat::*;
use dungeon::{spawn_dungeon, Dungeon, DungeonLevel};
use player::*;
use position::Position;
use saving::*;

fn main() {
    let primary_window = Some(Window {
        mode: WindowMode::BorderlessFullscreen, // hack for macOS 14
        resolution: (640.0, 480.0).into(),
        resizable: true,
        title: "Dungeon".to_string(),
        ..default()
    });

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window,
                    ..default()
                }),
        )
        .add_plugins(TweeningPlugin)
        .add_event::<MessageEvent>()
        .add_event::<DespawnDungeonEvent>()
        .add_event::<SpawnDungeonEvent>()
        .insert_resource(Msaa::Off)
        .init_resource::<Dungeon>()
        .init_resource::<UiFont>()
        .init_resource::<CatAnimation>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                despawn_dungeon,
                spawn_dungeon,
                spawn_cats,
                spawn_player,
                setup_cats_animation,
                (
                    update_player,
                    update_message,
                    update_button_style,
                    interact_reset_button,
                    interact_save_button,
                    interact_load_button,
                ),
            )
                .chain(),
        )
        .run();
}

#[derive(Event)]
pub struct DespawnDungeonEvent;

#[derive(Event)]
pub struct SpawnDungeonEvent(Option<Position>);

#[derive(Event)]
pub struct MessageEvent(String);

#[derive(Component)]
struct MessageText;

#[derive(Component)]
struct SaveButton;

#[derive(Component)]
struct LoadButton;

#[derive(Component)]
struct ResetButton;

#[derive(Resource)]
struct UiFont(Handle<Font>);
impl FromWorld for UiFont {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        UiFont(asset_server.load("k8x12.ttf"))
    }
}

fn setup(
    mut commands: Commands,
    ui_font: Res<UiFont>,
    mut spawn_event: EventWriter<SpawnDungeonEvent>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(34.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexEnd,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            ResetButton,
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(48.0),
                                    height: Val::Px(24.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                },
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Reset",
                                TextStyle {
                                    font: ui_font.0.clone(),
                                    font_size: 12.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                    parent
                        .spawn((
                            LoadButton,
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(48.0),
                                    height: Val::Px(24.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                },
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Load",
                                TextStyle {
                                    font: ui_font.0.clone(),
                                    font_size: 12.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                    parent
                        .spawn((
                            SaveButton,
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(48.0),
                                    height: Val::Px(24.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                },
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Save",
                                TextStyle {
                                    font: ui_font.0.clone(),
                                    font_size: 12.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                });
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(100.0),
                        height: Val::Px(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        MessageText,
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font: ui_font.0.clone(),
                                font_size: 24.0,
                                color: Color::WHITE,
                            },
                        ),
                    ));
                });
        });
    commands.insert_resource(DungeonLevel(0));
    spawn_event.send(SpawnDungeonEvent(None));
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

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn update_button_style(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => *color = PRESSED_BUTTON.into(),
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}

fn interact_save_button(
    position_query: Query<&Position, With<Player>>,
    level: Res<DungeonLevel>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SaveButton>)>,
) {
    if position_query.is_empty() {
        return;
    }
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let player_position = position_query.single();
        save_game(player_position.clone(), DungeonLevel(level.0));
    }
}

fn interact_load_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<LoadButton>)>,
    mut despawn_events: EventWriter<DespawnDungeonEvent>,
    mut spawn_events: EventWriter<SpawnDungeonEvent>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let loaded_game_data = load_game();

        despawn_events.send(DespawnDungeonEvent);
        let (dungeon_level, position) = match loaded_game_data {
            Some((dungeon_level, position)) => (dungeon_level, Some(position)),
            None => (DungeonLevel(0), None),
        };
        commands.insert_resource(dungeon_level);
        spawn_events.send(SpawnDungeonEvent(position));
    }
}

fn interact_reset_button(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ResetButton>)>,
    mut despawn_events: EventWriter<DespawnDungeonEvent>,
    mut spawn_events: EventWriter<SpawnDungeonEvent>,
) {
    for interaction in &interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        commands.insert_resource(DungeonLevel(0));
        despawn_events.send(DespawnDungeonEvent);
        spawn_events.send(SpawnDungeonEvent(None));
    }
}

fn despawn_dungeon(
    mut commands: Commands,
    query: Query<(Entity, &Transform, Without<Node>)>,
    mut reset_events: EventReader<DespawnDungeonEvent>,
) {
    if reset_events.is_empty() {
        return;
    }
    for _ in reset_events.iter() {}

    for (entity, _, _) in query.iter() {
        commands.entity(entity).despawn();
    }
}
