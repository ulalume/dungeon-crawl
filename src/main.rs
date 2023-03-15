mod ldtk;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tweening::{lens::*, *};
use ldtk::Ldtk;
use std::fmt;
use std::str::FromStr;
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

fn get_transform(direction: &Direction, x: f32, z: f32) -> Transform {
    Transform {
        translation: Vec3::new(x, 0.0, z),
        rotation: match direction {
            Direction::Up => Quat::from_rotation_y(0.0),
            Direction::Right => Quat::from_rotation_y(-PI * 0.5),
            Direction::Down => Quat::from_rotation_y(PI),
            Direction::Left => Quat::from_rotation_y(PI * 0.5),
        },
        ..default()
    }
}
fn get_cat_transform(direction: &Direction, x: f32, z: f32) -> Transform {
    let mut transform = get_transform(direction, x, z);
    transform.rotate_y(PI);
    transform
}
fn get_wall_transform(direction: &Direction, x: f32, z: f32) -> Transform {
    let mut transform = get_transform(direction, x, z);
    transform.translation += match direction {
        Direction::Up => Vec3::NEG_Z,
        Direction::Right => Vec3::X,
        Direction::Down => Vec3::Z,
        Direction::Left => Vec3::NEG_X,
    } * 0.5
        + Vec3::new(0.0, 0.5, 0.0);
    transform
}
fn get_player_transform(direction: &Direction, x: f32, z: f32) -> Transform {
    let mut transform = get_transform(direction, x, z);
    transform.translation += match direction {
        Direction::Up => Vec3::Z,
        Direction::Right => Vec3::NEG_X,
        Direction::Down => Vec3::NEG_Z,
        Direction::Left => Vec3::X,
    } * 0.4
        + Vec3::new(0.0, 0.5, 0.0);
    transform
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

    let spawn_wall = |commands: &mut Commands, direction: &str, x: f32, z: f32| {
        let transform = get_wall_transform(&direction.parse::<Direction>().unwrap(), x, z);
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
    let spawn_cat = |commands: &mut Commands,
                     direction: &Direction,
                     message: Option<String>,
                     x: f32,
                     z: f32| {
        commands.spawn((
            Cat { message },
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
                                let direction = direction
                                    .map(|s| s.as_str())
                                    .unwrap_or("right")
                                    .parse::<Direction>()
                                    .unwrap();
                                camera_transform = get_player_transform(&direction, x, z);
                                player_position.x = x as i32;
                                player_position.z = z as i32;
                                player_position.direction = direction;
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
                                    &direction
                                        .map(|s| s.as_str())
                                        .unwrap_or("right")
                                        .parse::<Direction>()
                                        .unwrap(),
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
}

enum Direction {
    Right,
    Up,
    Left,
    Down,
}
impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Direction::Right => "right",
                Direction::Up => "up",
                Direction::Left => "left",
                Direction::Down => "down",
            }
        )
    }
}
impl FromStr for Direction {
    type Err = ();
    fn from_str(input: &str) -> Result<Direction, Self::Err> {
        return match input.to_lowercase().as_str() {
            "right" => Ok(Direction::Right),
            "up" => Ok(Direction::Up),
            "left" => Ok(Direction::Left),
            "down" => Ok(Direction::Down),
            _ => Err(()),
        };
    }
}
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Position {
    direction: Direction,
    x: i32,
    z: i32,
}

impl Position {
    fn go_forward(&mut self) {
        match self.direction {
            Direction::Right => self.x += 1,
            Direction::Up => self.z -= 1,
            Direction::Left => self.x -= 1,
            Direction::Down => self.z += 1,
        };
    }
    fn go_backward(&mut self) {
        match self.direction {
            Direction::Right => self.x -= 1,
            Direction::Up => self.z += 1,
            Direction::Left => self.x += 1,
            Direction::Down => self.z -= 1,
        };
    }
    fn rotate_right(&mut self) {
        self.direction = match self.direction {
            Direction::Right => Direction::Down,
            Direction::Up => Direction::Right,
            Direction::Left => Direction::Up,
            Direction::Down => Direction::Left,
        };
    }
    fn rotate_left(&mut self) {
        self.direction = match self.direction {
            Direction::Right => Direction::Up,
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
        };
    }
}

fn update_player(
    keys: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Position), With<Player>>,
) {
    if query.is_empty() {
        return;
    }
    let (entity, transform, mut position) = query.single_mut();
    let mut changed = false;
    if keys.just_pressed(KeyCode::W) {
        position.go_forward();
        changed = true
    }
    if keys.just_pressed(KeyCode::S) {
        position.go_backward();
        changed = true
    }
    if keys.just_pressed(KeyCode::A) {
        position.rotate_left();
        changed = true
    }
    if keys.just_pressed(KeyCode::D) {
        position.rotate_right();
        changed = true
    }

    if !changed {
        return;
    }

    let new_transform =
        get_player_transform(&position.direction, position.x as f32, position.z as f32);

    commands
        .entity(entity)
        .insert(Animator::<Transform>::new(Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_millis(200),
            MyTransformLens {
                start: (transform.translation, transform.rotation),
                end: (new_transform.translation, new_transform.rotation),
            },
        )));
}

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
