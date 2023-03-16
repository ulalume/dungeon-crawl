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
            Cat {},
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

fn update_player(
    keys: Res<Input<KeyCode>>,
    level: Res<Level>,
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Position), With<Player>>,
) {
    if query.is_empty() {
        return;
    }
    let (entity, transform, mut position) = query.single_mut();
    let mut changed = false;
    let tile = level.get_tile(position.x, position.z);

    if keys.just_pressed(KeyCode::W) {
        if tile.is_some() && !tile.unwrap().has_wall(&position.direction) {
            position.go_forward();
            changed = true
        }
    }
    if keys.just_pressed(KeyCode::S) {
        if tile.is_some() && !tile.unwrap().has_wall(&position.direction.reverse()) {
            position.go_backward();
            changed = true
        }
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
struct Cat {}
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
