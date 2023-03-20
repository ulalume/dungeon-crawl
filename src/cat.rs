use crate::dungeon::{Dungeon, DungeonLevel, EntityType};
use crate::position::{get_transform, Direction};
use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Component)]
pub struct Cat;

#[derive(Resource)]
pub struct CatAnimation(pub Handle<AnimationClip>);

pub fn setup_cats(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    dungeon: Res<Dungeon>,
    dungeon_level: Res<DungeonLevel>,
) {
    let level = dungeon.levels.get(dungeon_level.0).unwrap();
    let scene_cat = asset_server.load("cat.glb#Scene0");
    commands.insert_resource(CatAnimation(asset_server.load("cat.glb#Animation0")));

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

    for entity in level.entities.iter() {
        match entity.entity_type {
            EntityType::Cat => {
                spawn_cat(
                    &mut commands,
                    &entity.direction,
                    entity.x as f32,
                    entity.z as f32,
                );
            }
            _ => (),
        };
    }
}

pub fn check_descendant<T: Component>(
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

pub fn setup_cats_animation(
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

pub fn get_cat_transform(direction: &Direction, x: f32, z: f32) -> Transform {
    let mut transform = get_transform(direction, x, z);
    transform.rotate_y(PI);
    transform
}
