use crate::dungeon::{Dungeon, DungeonLevel, EntityType};
use crate::position::{get_transform, Direction, Position};
use crate::MessageEvent;
use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use bevy_tweening::{lens::*, *};

#[derive(Component)]
pub struct Player;

pub fn setup_player(
    mut commands: Commands,
    dungeon: Res<Dungeon>,
    dungeon_level: Res<DungeonLevel>,
) {
    let level = dungeon.levels.get(dungeon_level.0).unwrap();
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
            _ => (),
        };
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
    let tracks1 = Tracks::new(vec![
        Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_millis(50),
            TransformRotationLens {
                start: transform.rotation,
                end: between_transform.rotation,
            },
        ),
        Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_millis(50),
            TransformPositionLens {
                start: transform.translation,
                end: between_transform.translation,
            },
        ),
    ]);
    let tracks2 = Tracks::new(vec![
        Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_millis(100),
            TransformRotationLens {
                start: between_transform.rotation,
                end: now_transform.rotation,
            },
        ),
        Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_millis(100),
            TransformPositionLens {
                start: between_transform.translation,
                end: now_transform.translation,
            },
        ),
    ]);
    Animator::<Transform>::new(Sequence::new(vec![tracks1, tracks2]))
}

fn get_move_animator(transform: &Transform, new_position: &Position) -> Animator<Transform> {
    let new_transform = get_player_transform(
        &new_position.direction,
        new_position.x as f32,
        new_position.z as f32,
    );
    Animator::<Transform>::new(Tracks::new(vec![
        Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_millis(200),
            TransformRotationLens {
                start: transform.rotation,
                end: new_transform.rotation,
            },
        ),
        Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_millis(200),
            TransformPositionLens {
                start: transform.translation,
                end: new_transform.translation,
            },
        ),
    ]))
}

pub fn update_player(
    keys: Res<Input<KeyCode>>,
    dungeon: Res<Dungeon>,
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Position), With<Player>>,
    mut message_events: EventWriter<MessageEvent>,
    dungeon_level: Res<DungeonLevel>,
) {
    if query.is_empty() {
        return;
    }
    let level = dungeon.levels.get(dungeon_level.0).unwrap();

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

pub fn get_player_transform(direction: &Direction, x: f32, z: f32) -> Transform {
    let mut transform = get_transform(direction, x, z);
    transform.translation += match direction {
        Direction::Up => Vec3::Z,
        Direction::Right => Vec3::NEG_X,
        Direction::Down => Vec3::NEG_Z,
        Direction::Left => Vec3::X,
    } * 0.4
        + Vec3::new(0.0, 0.4, 0.0);
    transform
}
