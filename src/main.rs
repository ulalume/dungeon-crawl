mod dungeon;
mod ldtk;
mod pos;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tweening::{lens::*, *};
use dungeon::{Dungeon, EntityType, Level};
use ldtk::Ldtk;
use pos::{get_cat_transform, get_player_transform, get_wall_transform, Direction, Position};
use std::{f32::consts::PI, time::Duration};

struct MessageEvent(String);

struct MyTransformLens {
    start: (Vec3, Quat),
    end: (Vec3, Quat),
}
impl Lens<Transform> for MyTransformLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let (start_vec, start_rot) = self.start;
        let (end_vec, end_rot) = self.end;
        target.translation = start_vec + (end_vec - start_vec) * ratio;
        target.rotation = start_rot.slerp(end_rot, ratio);
    }
}

#[derive(Component)]
struct MessageText;

#[derive(Component)]
struct Cat;

#[derive(Resource)]
struct CatAnimation(Handle<AnimationClip>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_plugin(WorldInspectorPlugin::default())
        .add_event::<MessageEvent>()
        .add_startup_system(setup)
        .add_system(setup_cat_animation)
        .add_system(update_player)
        .add_system(update_message)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let lgtk = serde_json::from_str::<Ldtk>(include_str!("../assets/level.ldtk"))
        .expect("Failed to open level.ldtk");
    let dungeon = Dungeon::from(&lgtk);
    let level = dungeon.levels.get(0).unwrap();

    // light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::Hsla {
                hue: 0.0,
                saturation: 0.2,
                lightness: 1.0,
                alpha: 1.0,
            },
            illuminance: 350.0,
            ..default()
        },
        transform: Transform::from_xyz(50.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // texture, material
    let wall_texture = asset_server.load("wall.png");
    let material_floor = materials.add(StandardMaterial {
        base_color: Color::GRAY,
        ..default()
    });
    let material_wall = materials.add(StandardMaterial {
        base_color_texture: Some(wall_texture),
        ..default()
    });

    // mesh, scene, animation
    let mesh_wall = meshes.add(shape::Quad::default().into());
    let scene_cat = asset_server.load("cat.glb#Scene0");
    commands.insert_resource(CatAnimation(asset_server.load("cat.glb#Animation0")));

    let spawn_wall = |commands: &mut Commands, direction: &Direction, x: f32, z: f32| {
        let transform = get_wall_transform(direction, x, z);
        commands.spawn(MaterialMeshBundle {
            mesh: mesh_wall.clone(),
            material: material_wall.clone(),
            transform,
            ..default()
        });
    };
    let spawn_floor = |commands: &mut Commands, x: f32, z: f32| {
        commands.spawn(MaterialMeshBundle {
            mesh: mesh_wall.clone(),
            material: material_floor.clone(),
            transform: Transform {
                translation: Vec3::new(x, 0.0, z),
                rotation: Quat::from_rotation_x(-PI * 0.5),
                ..default()
            },
            ..default()
        });
    };
    let spawn_cat = |commands: &mut Commands, direction: &Direction, x: f32, z: f32| {
        commands.spawn((
            Cat,
            SceneBundle {
                scene: scene_cat.clone(),
                transform: get_cat_transform(direction, x, z),
                ..default()
            },
        ));
    };

    // spawn tiles
    let mut camera_transform = Transform::from_translation(Vec3::new(0.0, 0.5, 0.0));
    let mut player_position = Position {
        direction: Direction::Left,
        x: 0,
        z: 0,
    };

    for entity in level.entities.iter() {
        match entity.entity_type {
            EntityType::PlayerStart => {
                camera_transform =
                    get_player_transform(&entity.direction, entity.x as f32, entity.z as f32);
                player_position.x = entity.x;
                player_position.z = entity.z;
                player_position.direction = entity.direction.clone();
            }
            EntityType::Cat => {
                spawn_cat(
                    &mut commands,
                    &entity.direction,
                    entity.x as f32,
                    entity.z as f32,
                );
            }
        };
    }
    for tile in level.tiles.iter() {
        for direction in tile.walls.iter() {
            spawn_wall(&mut commands, direction, tile.x as f32, tile.z as f32);
        }
        spawn_floor(&mut commands, tile.x as f32, tile.z as f32);
    }

    commands.spawn((
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
        MessageText,
    ));

    // spawn camera
    commands.spawn((
        Player,
        player_position,
        Camera3dBundle {
            transform: camera_transform,
            projection: Projection::Perspective(PerspectiveProjection {
                fov: PI / 2.5,
                ..default()
            }),
            ..default()
        },
    ));

    commands.insert_resource((*level).clone());
}

#[derive(Component)]
struct Player;

fn get_cannot_move_animator(
    transform: &Transform,
    now_position: &Position,
    new_position: &Position,
) -> Animator<Transform> {
    let now_transform = get_player_transform(
        &now_position.direction,
        now_position.x as f32,
        now_position.z as f32,
    );
    let between_transform = get_player_transform(
        &new_position.direction,
        now_position.x as f32 + (new_position.x as f32 - now_position.x as f32) * 0.1,
        now_position.z as f32 + (new_position.z as f32 - now_position.z as f32) * 0.1,
    );

    let tween1 = Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(50),
        MyTransformLens {
            start: (transform.translation, transform.rotation),
            end: (between_transform.translation, between_transform.rotation),
        },
    );
    let tween2 = Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(100),
        MyTransformLens {
            start: (between_transform.translation, between_transform.rotation),
            end: (now_transform.translation, now_transform.rotation),
        },
    );
    Animator::<Transform>::new(tween1.then(tween2))
}

fn get_move_animator(transform: &Transform, new_position: &Position) -> Animator<Transform> {
    let new_transform = get_player_transform(
        &new_position.direction,
        new_position.x as f32,
        new_position.z as f32,
    );

    Animator::<Transform>::new(Tween::new(
        EaseFunction::QuadraticOut,
        Duration::from_millis(200),
        MyTransformLens {
            start: (transform.translation, transform.rotation),
            end: (new_transform.translation, new_transform.rotation),
        },
    ))
}
fn update_player(
    keys: Res<Input<KeyCode>>,
    level: Res<Level>,
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Position), With<Player>>,
    mut message_events: EventWriter<MessageEvent>,
) {
    if query.is_empty() {
        return;
    }
    let (entity, transform, mut position) = query.single_mut();
    let tile = level.get_tile(position.x, position.z);

    let (tweened, wall_position): (bool, Option<Position>) = if keys.just_pressed(KeyCode::W) {
        if tile.is_some() && !tile.unwrap().has_wall(&position.direction) {
            position.go_forward();
            (true, None)
        } else {
            let mut wall = position.clone();
            wall.go_forward();
            (true, Some(wall))
        }
    } else if keys.just_pressed(KeyCode::S) {
        if tile.is_some() && !tile.unwrap().has_wall(&position.direction.reverse()) {
            position.go_backward();
            (true, None)
        } else {
            let mut wall = position.clone();
            wall.go_backward();
            (true, Some(wall))
        }
    } else if keys.just_pressed(KeyCode::A) {
        position.rotate_left();
        (true, None)
    } else if keys.just_pressed(KeyCode::D) {
        position.rotate_right();
        (true, None)
    } else {
        (false, None)
    };

    if !tweened {
        return;
    };

    if let Some(wall_position) = wall_position {
        commands.entity(entity).insert(get_cannot_move_animator(
            transform,
            &position,
            &wall_position,
        ));
        return;
    }

    commands
        .entity(entity)
        .insert(get_move_animator(transform, &position));

    match level.get_entity(position.x, position.z) {
        Some(event_entity) => {
            if event_entity.message.is_some() {
                message_events.send(MessageEvent(event_entity.message.clone().unwrap()))
            }
        }
        _ => message_events.send(MessageEvent("".to_owned())),
    };
}

fn check_descendant<T: Component>(
    components: &Query<Entity, With<T>>,
    parents: &Query<&Parent>,
    entity: Entity,
) -> bool {
    if components.contains(entity.clone()) {
        return true;
    }
    if let Ok(parent) = parents.get(entity) {
        return check_descendant(components, parents, parent.get());
    }
    false
}
fn setup_cat_animation(
    animation: Res<CatAnimation>,
    mut animation_players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    cats: Query<Entity, With<Cat>>,
    parents: Query<&Parent>,
) {
    for (_, mut animation_player) in animation_players
        .iter_mut()
        .filter(|(entity, _)| check_descendant(&cats, &parents, entity.to_owned()))
    {
        animation_player.play(animation.0.clone_weak()).repeat();
    }
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
