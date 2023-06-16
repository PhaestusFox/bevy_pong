use bevy::{prelude::*, window::PrimaryWindow, sprite::Anchor};
use bevy_rapier2d::prelude::*;

use crate::{GameState, Player, PlayerKeyBinds};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugin(bevy_rapier2d::plugin::RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(bevy_rapier2d::prelude::RapierDebugRenderPlugin::default())
        .add_system(spawn_game_scene.in_schedule(OnEnter(GameState::Playing)))
        .add_system(clean_up_game.in_schedule(OnExit(GameState::Playing)))
        .add_system(move_paddle.in_set(OnUpdate(GameState::Playing)));
    }
}

#[derive(Component)]
struct GameItem;

#[derive(Debug, Component)]
struct Paddle {
    size: Vec2,
    speed: f32,
}

fn spawn_game_scene(mut commands: Commands, window: Query<&Window, With<PrimaryWindow>>) {
    let window = window.single();
    let x_pos = window.width()/2. * 0.8;
    let paddle_size = Vec2::new(window.width() * 0.05, window.height() * 0.2);
    commands.spawn((SpriteBundle {
        transform: Transform::from_translation(Vec3::new(-x_pos, 0., 0.)),
        sprite: Sprite {
            color: Color::WHITE,
            anchor: Anchor::Center,
            custom_size: Some(paddle_size),
            ..Default::default()
        },
        ..Default::default()
    },
    Name::new("Left Paddle"),
    GameItem,
    RigidBody::KinematicPositionBased,
    Collider::cuboid(window.width() * 0.025, window.height() * 0.1),
    Player::PlayerOne,
    Paddle {size: paddle_size, speed: 250.}
    ));
    commands.spawn((SpriteBundle {
        transform: Transform::from_translation(Vec3::new(x_pos, 0., 0.)),
        sprite: Sprite {
            color: Color::WHITE,
            anchor: Anchor::Center,
            custom_size: Some(paddle_size),
            ..Default::default()
        },
        ..Default::default()
    },
    Name::new("Right Paddle"),
    GameItem,
    RigidBody::KinematicPositionBased,
    Collider::cuboid(window.width() * 0.025, window.height() * 0.1),
    Player::PlayerTwo,
    Paddle{size: paddle_size, speed: 250.},
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

fn move_paddle(
    mut query: Query<(&mut Transform, &Player, &Paddle)>,
    settings: Res<PlayerKeyBinds>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>
) {
    let window = window.single();
    let map_size = Vec2::new(window.width()/2., window.height()/2.);
    for (mut transform, player, paddle) in &mut query {
        let keys = settings.get(*player);
        let mut delta = 0.0;
        if input.pressed(keys.move_up) {delta += 1.;}
        if input.pressed(keys.move_down) {delta -= 1.;}
        delta *= paddle.speed * time.delta_seconds();
        transform.translation.y += delta;
        transform.translation.y = transform.translation.y.clamp(-map_size.y + (paddle.size.y / 2.), map_size.y - (paddle.size.y / 2.));
    }
}