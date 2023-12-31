use belly::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow, sprite::Anchor};
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::{GameState, Player, PlayerKeyBinds, ai::{Opponent, AiBrain, PongAi}, KeyBindings};

use super::*;

pub struct NormalPlugin;

impl Plugin for NormalPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<GameEvent>()
        .add_systems(OnEnter(GameState::PlayingNormal), (spawn_game_scene, spawn_score))
        .add_systems(OnExit(GameState::PlayingNormal), clean_up_game)
        .add_systems(Update, move_paddle.in_set(Playing))
        .add_systems(Update, spawn_ball.in_set(Playing))
        .add_systems(Update, score_point.in_set(Playing))
        .configure_set(Update, Playing.run_if(in_state(GameState::PlayingNormal)))
        .add_plugins(OpponentMenuPlugin);
    }
}

#[derive(SystemSet, Hash, Debug, Clone, Copy, PartialEq, Eq)]
struct Playing;

fn spawn_game_scene(mut commands: Commands, window: Query<&Window, With<PrimaryWindow>>,
    mut events: EventWriter<GameEvent>,
) {
    let window = window.single();
    let x_pos = window.width()/2. * 0.8;
    let paddle_size = Vec2::new(40., window.height() * 0.2);
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
    Collider::cuboid(20., window.height() * 0.1),
    Player::PlayerOne,
    Paddle {size: paddle_size, speed: 250.},
    Restitution{coefficient: 1.05, ..Default::default()}
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
    Collider::cuboid(20., window.height() * 0.1),
    Player::PlayerTwo,
    Paddle{size: paddle_size, speed: 250.},
    Restitution{coefficient: 1.05, ..Default::default()}
    ));

    commands.spawn((SpriteBundle {
        transform: Transform::from_translation(Vec3::new(0., window.height() / 2., 0.)),
        ..Default::default()
    },
    Name::new("Top Wall"),
    GameItem,
    RigidBody::Fixed,
    Collider::cuboid(window.width() * 0.5, 5.),
    Restitution{coefficient: 1.0, ..Default::default()},
    Friction{coefficient: 0.0, ..Default::default()}
    ));

    commands.spawn((SpriteBundle {
        transform: Transform::from_translation(Vec3::new(0., -window.height() / 2., 0.)),
        ..Default::default()
    },
    Name::new("Bottom Wall"),
    GameItem,
    RigidBody::Fixed,
    Collider::cuboid(window.width() * 0.5, 5.),
    Restitution{coefficient: 1.0, ..Default::default()},
    Friction{coefficient: 0., ..Default::default()}
    ));

    commands.spawn((SpriteBundle {
        transform: Transform::from_translation(Vec3::new(-window.width() / 2., 0., 0.)),
        ..Default::default()
    },
    Name::new("Left Wall"),
    GameItem,
    RigidBody::Fixed,
    Sensor,
    Collider::cuboid(5., window.height() * 0.5),
    Player::PlayerOne,
    Goal,
    ));

    commands.spawn((SpriteBundle {
        transform: Transform::from_translation(Vec3::new(window.width() / 2., 0., 0.)),
        ..Default::default()
    },
    Name::new("Right Wall"),
    GameItem,
    RigidBody::Fixed,
    Sensor,
    Collider::cuboid( 5., window.height() * 0.5),
    Player::PlayerTwo,
    Goal,
    ));

    events.send(GameEvent::SpawnBallRandom);
}

fn move_paddle(
    mut query: Query<(&mut Transform, &Player, &Paddle)>,
    settings: Res<PlayerKeyBinds>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    opponent: Res<State<Opponent>>,
    ai: Res<AiBrain>,
) {
    let window = window.single();
    let map_size = Vec2::new(window.width()/2., window.height()/2.);
    for (mut transform, player, paddle) in &mut query {
        let mut delta = match player {
            Player::PlayerOne => get_human_delta(settings.get(Player::PlayerOne), &input),
            Player::PlayerTwo => match opponent.get() {
                Opponent::Human => get_human_delta(settings.get(Player::PlayerTwo), &input),
                Opponent::Ai => ai.get_delta(),
            }
        };
        delta *= paddle.speed * time.delta_seconds();
        transform.translation.y += delta;
        transform.translation.y = transform.translation.y.clamp(-map_size.y + (paddle.size.y / 2.), map_size.y - (paddle.size.y / 2.));
    }
}

fn get_human_delta(keys: KeyBindings, input: &Input<KeyCode>) -> f32 {
    let mut delta = 0.0;
    if input.pressed(keys.move_up) {delta += 1.;}
    if input.pressed(keys.move_down) {delta -= 1.;}
    delta
}

fn spawn_ball(
    mut commands: Commands,
    asset_sever: Res<AssetServer>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut events: EventReader<GameEvent>,
) {
    let window = window.single();
    let ball_size = window.height() * 0.05;
    for event in events.iter() {
        match event {
            GameEvent::SpawnBallRandom => spawn_ball_in(&mut commands, Vec2::new(if rand::thread_rng().gen_bool(0.5) {200.} else {-200.}, rand::thread_rng().gen_range(-25.0..25.0)), ball_size, &asset_sever),
            GameEvent::SpawnBallPlayer(player) => spawn_ball_in(&mut commands, Vec2::new(if Player::PlayerOne.eq(player) {200.} else {-200.}, rand::thread_rng().gen_range(-25.0..25.0)), ball_size, &asset_sever),
        };
    }
}

fn spawn_ball_in(commands: &mut Commands, start: Vec2, ball_size: f32, asset_sever: &AssetServer) {
    commands.spawn((
        GameItem,
        SpriteBundle{
            sprite: Sprite {
            custom_size: Some(Vec2::splat(ball_size)),
            ..Default::default()
        },
        texture: asset_sever.load("bevy.png"),
        ..Default::default()
    },
    Ball,
    RigidBody::Dynamic,
    Collider::ball(ball_size / 2.),
    Velocity::linear(start),
    Restitution{coefficient: 1.0, ..Default::default()},
    Damping{linear_damping: 0., ..Default::default()},
    LockedAxes::ROTATION_LOCKED,
    GravityScale(0.),
    Friction{coefficient: 0., ..Default::default()},
    ));
}

#[derive(Event)]
enum GameEvent {
    SpawnBallRandom,
    SpawnBallPlayer(Player),
}

fn score_point(
    mut score: ResMut<Score>,
    query: Query<(Entity, &Player), With<Goal>>,
    balls: Query<Entity, With<Ball>>,
    physics_world: Res<RapierContext>,
    mut commands: Commands,
    mut events: EventWriter<GameEvent>,
) {
    for ball in &balls {
        for (goal, player) in &query {
            if let Some(true) = physics_world.intersection_pair(ball, goal) {
                match player {
                    Player::PlayerOne => score.1 += 1,
                    Player::PlayerTwo => score.0 += 1,
                }
                println!("Score = {:?}", score);
                commands.entity(ball).despawn();
                events.send(GameEvent::SpawnBallPlayer(*player));
            }
        }
    }
}

fn spawn_score(
    mut commands: Commands,
) {
    commands.add(eml! {
        <div c:scoreboard with:GameItem>
            <div c:score>
                <label bind:value=from!(Score:0|fmt.c("{c}"))/>
            </div>
            <div c:break/>
            <div c:score>
                <label bind:value=from!(Score:1|fmt.c("{c}"))/>
            </div>
        </div>
    })
}


pub struct OpponentMenuPlugin;

impl Plugin for OpponentMenuPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(GameState::OpponentSelect), spawn_opponent_menu)
        .add_systems(OnExit(GameState::OpponentSelect), crate::menu::close_menu);
    }
}

fn spawn_opponent_menu(
    mut commands: Commands,
) {
    commands.add(eml!{
        <div c:menu>
        <button on:press=run!(|c| {
            c.commands().add(|world: &mut World| {
                world.resource_mut::<NextState<Opponent>>().set(Opponent::Human);
                world.resource_mut::<NextState<GameState>>().set(GameState::PlayingNormal);
            })
        })>
        <label c:content value="Player2"/>
        </button>
        <button on:press=run!(|c| {
            c.commands().add(|world: &mut World| {
                world.insert_resource(AiBrain::new_simple());
                world.resource_mut::<NextState<Opponent>>().set(Opponent::Ai);
                world.resource_mut::<NextState<GameState>>().set(GameState::PlayingNormal);
            })
        })>
        <label c:content value="Simple Ai"/>
        </button>
        <button on:press=run!(|c| {
            c.commands().add(|world: &mut World| {
                world.insert_resource(AiBrain::new_goaly());
                world.resource_mut::<NextState<Opponent>>().set(Opponent::Ai);
                world.resource_mut::<NextState<GameState>>().set(GameState::PlayingNormal);
            })
        })>
        <label c:content value="Goaly Ai"/>
        </button>
        <button on:press=run!(|c| {
            c.commands().add(|world: &mut World| {
                world.insert_resource(AiBrain::new_smart());
                world.resource_mut::<NextState<Opponent>>().set(Opponent::Ai);
                world.resource_mut::<NextState<GameState>>().set(GameState::PlayingNormal);
            })
        })>
        <label c:content value="Smart Ai"/>
        </button>
    </div>
    });
}
