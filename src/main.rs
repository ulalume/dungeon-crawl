mod ldtk;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tweening::{lens::*, *};
use ldtk::Ldtk;
use std::{f32::consts::PI, time::Duration};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_plugin(WorldInspectorPlugin::default())
        .add_startup_system(setup)
        .add_system(setup_cat_animation)
        .add_system(update_player)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let level = serde_json::from_str::<Ldtk>(include_str!("../assets/level.ldtk"));
    if level.is_err() {
        return;
    }
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

    let get_transform = |direction: &str| match direction.to_lowercase().as_str() {
        "up" => Transform {
            translation: Vec3::NEG_Z * 0.5,
            rotation: Quat::from_rotation_y(0.0),
            ..default()
        },
        "right" => Transform {
            translation: Vec3::X * 0.5,
            rotation: Quat::from_rotation_y(-PI * 0.5),
            ..default()
        },
        "down" => Transform {
            translation: Vec3::Z * 0.5,
            rotation: Quat::from_rotation_y(PI),
            ..default()
        },
        "left" => Transform {
            translation: Vec3::NEG_X * 0.5,
            rotation: Quat::from_rotation_y(PI * 0.5),
            ..default()
        },
        _ => Transform::default(),
    };
    let spawn_wall = |commands: &mut Commands, direction: &str, x: f32, z: f32| {
        let mut transform = get_transform(direction);
        transform.translation += Vec3::new(x, 0.5, z);
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
    let spawn_cat =
        |commands: &mut Commands, direction: &str, message: Option<String>, x: f32, z: f32| {
            let mut transform = get_transform(direction);
            transform.translation += Vec3::new(x, 0.0, z);
            transform.rotate_y(PI);
            commands.spawn((
                Cat { message },
                SceneBundle {
                    scene: scene_cat.clone(),
                    transform,
                    ..default()
                },
            ));
        };

    // spawn tiles
    let mut camera_transform = Transform::from_translation(Vec3::new(0.0, 0.5, 0.0));
    let level = level.unwrap();
    if let Some(layer_instances) = level
        .levels
        .get(0)
        .and_then(|level| level.layer_instances.as_ref())
    {
        for layer_instance in layer_instances {
            let grid_size = (layer_instance.c_wid, layer_instance.c_wid);
            match layer_instance.identifier.as_str() {
                "Entities" => {
                    for entity in layer_instance.entity_instances.iter() {
                        let direction = entity
                            .field_instances
                            .iter()
                            .find(|field_instance| field_instance.identifier == "Direction")
                            .and_then(|field_instance| match field_instance.value.as_ref() {
                                Some(serde_json::Value::String(s)) => Some(s),
                                _ => None,
                            });
                        let x = entity.grid[0] as f32;
                        let z = entity.grid[1] as f32;
                        match entity.identifier.as_str() {
                            "PlayerStart" => {
                                if let Some(direction) = direction {
                                    camera_transform.rotation = get_transform(&direction).rotation;
                                }
                                camera_transform.translation += Vec3::new(x, 0.0, z);
                                println!("{:?}", camera_transform);
                            }
                            "Cat" => {
                                let message = entity
                                    .field_instances
                                    .iter()
                                    .find(|field_instance| field_instance.identifier == "Message")
                                    .and_then(|field_instance| {
                                        match field_instance.value.as_ref() {
                                            Some(serde_json::Value::String(s)) => {
                                                Some(s.to_owned())
                                            }
                                            _ => None,
                                        }
                                    });
                                spawn_cat(
                                    &mut commands,
                                    direction.map(|s| s.as_str()).unwrap_or("up"),
                                    message,
                                    x,
                                    z,
                                );
                            }
                            _ => {}
                        }
                    }
                }
                "Tiles" => {
                    let tileset = layer_instance
                        .tileset_def_uid
                        .and_then(|uid| {
                            level
                                .defs
                                .tilesets
                                .iter()
                                .find(|tileset| tileset.uid == uid)
                        })
                        .unwrap();
                    for tile in layer_instance.grid_tiles.iter() {
                        let x = (tile.px[0] / grid_size.0) as f32;
                        let z = (tile.px[1] / grid_size.1) as f32;
                        let directions = tileset
                            .custom_data
                            .iter()
                            .find(|d| d.tile_id == tile.t)
                            .map(|d| d.data.as_str())
                            .and_then(|s| {
                                if s.is_empty() {
                                    None
                                } else {
                                    Some(s.split(',').collect::<Vec<_>>())
                                }
                            })
                            .unwrap_or(vec![]);
                        for direction in directions {
                            spawn_wall(&mut commands, direction, x, z);
                        }
                        spawn_floor(&mut commands, x, z);
                    }
                }
                _ => {}
            }
        }
    }

    // spawn camera
    commands.spawn((
        Player,
        Camera3dBundle {
            transform: camera_transform,
            ..default()
        },
    ));
}

#[derive(Component)]
struct Player;

fn update_player(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    transforms: Query<(Entity, &Transform), With<Player>>,
) {
    if transforms.is_empty() {
        return;
    }
    let (entity, transform) = transforms.single();
    if keys.just_pressed(KeyCode::W) {
        commands
            .entity(entity)
            .insert(Animator::<Transform>::new(Tween::new(
                EaseFunction::QuadraticOut,
                Duration::from_millis(200),
                TransformPositionLens {
                    start: transform.translation,
                    end: transform.translation + transform.rotation * Vec3::NEG_Z,
                },
            )));
    }
    if keys.just_pressed(KeyCode::S) {
        commands
            .entity(entity)
            .insert(Animator::<Transform>::new(Tween::new(
                EaseFunction::QuadraticOut,
                Duration::from_millis(200),
                TransformPositionLens {
                    start: transform.translation,
                    end: transform.translation + transform.rotation * Vec3::Z,
                },
            )));
    }
    if keys.just_pressed(KeyCode::A) {
        commands
            .entity(entity)
            .insert(Animator::<Transform>::new(Tween::new(
                EaseFunction::QuadraticOut,
                Duration::from_millis(200),
                TransformRotationLens {
                    start: transform.rotation,
                    end: transform.rotation * Quat::from_rotation_y(PI * 0.5),
                },
            )));
    }
    if keys.just_pressed(KeyCode::D) {
        commands
            .entity(entity)
            .insert(Animator::<Transform>::new(Tween::new(
                EaseFunction::QuadraticOut,
                Duration::from_millis(200),
                TransformRotationLens {
                    start: transform.rotation,
                    end: transform.rotation * Quat::from_rotation_y(-PI * 0.5),
                },
            )));
    }
}

#[derive(Component)]
struct Cat {
    message: Option<String>,
}
#[derive(Resource)]
struct CatAnimation(Handle<AnimationClip>);

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
