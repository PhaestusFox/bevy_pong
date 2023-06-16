use bevy::{prelude::*, window::PrimaryWindow, sprite::Anchor};
use bevy_rapier2d::prelude::*;

use crate::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugin(bevy_rapier2d::plugin::RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(bevy_rapier2d::prelude::RapierDebugRenderPlugin::default())
        .add_system(spawn_game_scene.in_schedule(OnEnter(GameState::Playing)))
        .add_system(clean_up_game.in_schedule(OnExit(GameState::Playing)));
    }
}

#[derive(Component)]
struct GameItem;

fn spawn_game_scene(mut commands: Commands, window: Query<&Window, With<PrimaryWindow>>) {
    let window = window.single();
    let x_pos = window.width()/2. * 0.8;
    commands.spawn((SpriteBundle {
        transform: Transform::from_translation(Vec3::new(-x_pos, 0., 0.)),
        sprite: Sprite {
            color: Color::WHITE,
            anchor: Anchor::Center,
            custom_size: Some(Vec2::new(window.width() * 0.05, window.height() * 0.2)),
            ..Default::default()
        },
        ..Default::default()
    },
    Name::new("Left Paddle"),
    GameItem,
    RigidBody::KinematicPositionBased,
    Collider::cuboid(window.width() * 0.025, window.height() * 0.1),
    ));
    commands.spawn((SpriteBundle {
        transform: Transform::from_translation(Vec3::new(x_pos, 0., 0.)),
        sprite: Sprite {
            color: Color::WHITE,
            anchor: Anchor::Center,
            custom_size: Some(Vec2::new(window.width() * 0.05, window.height() * 0.2)),
            ..Default::default()
        },
        ..Default::default()
    },
    Name::new("Right Paddle"),
    GameItem,
    RigidBody::KinematicPositionBased,
    Collider::cuboid(window.width() * 0.025, window.height() * 0.1),
    ));
}

fn clean_up_game(
    mut commands: Commands,
    items: Query<Entity, With<GameItem>>,
) {
    for entity in &items {
        commands.entity(entity).despawn_recursive();
    }
}