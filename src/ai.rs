use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;

use crate::{Player, game::{Ball, Paddle}};

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_state::<Opponent>()
        .insert_resource(AiBrain::new_simple())
        .add_system(update_ai.in_base_set(CoreSet::Last));
    }
}

#[derive(Resource)]
pub struct AiBrain(Box<dyn PongAi>);

impl AiBrain {
    pub fn new_simple() -> AiBrain {
        AiBrain(Box::new(SimplePongAi{delta: 0.}))
    }

    pub fn new_goaly() -> AiBrain {
        AiBrain(Box::new(GoalyPongAi{delta: 0.}))
    }

    pub fn new_smart() -> AiBrain {
        AiBrain(Box::new(SmartPongAi{delta: 0.}))
    }
}

fn update_ai(
    world: &mut World,
) {
    world.resource_scope(|world, mut ai: Mut<AiBrain>| {
        ai.update(world);
    });
}

impl PongAi for AiBrain {
    fn get_delta(&self) -> f32 {
        self.0.get_delta()
    }
    fn update(&mut self, world: &mut World) {
        self.0.update(world)
    }
}

#[derive(Debug, Default, States, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Opponent {
    #[default]
    Human,
    Ai,
}

pub trait PongAi: 'static + Send + Sync {
    fn get_delta(&self) -> f32;
    fn update(&mut self, world: &mut World);
}

struct SimplePongAi {
    delta: f32,
}

impl PongAi for SimplePongAi {
    fn get_delta(&self) -> f32 {
        self.delta
    }
    fn update(&mut self, world: &mut World) {
        let mut query = world.query_filtered::<(&Player, &Transform), With<Paddle>>();
        let mut balls = world.query_filtered::<&Transform, With<Ball>>();
        for (player, pos) in query.iter(world) {
            if Player::PlayerTwo.ne(player) {continue;}
            let mut ball_pos = Vec3::NEG_X * 1000.;
            for ball in balls.iter(&world) {
                if ball.translation.x > ball_pos.x {ball_pos = ball.translation };
            }
            let error = ball_pos.y - pos.translation.y;
            if error > 10. {
                self.delta = 1.;
            } else if error < -10. {
                self.delta = -1.;
            } else if error.abs() < 1. {
                self.delta = 0.;
            };
        }
    }
}

struct GoalyPongAi {
    delta: f32,
}

impl PongAi for GoalyPongAi {
    fn get_delta(&self) -> f32 {
        self.delta
    }
    fn update(&mut self, world: &mut World) {
        let mut query = world.query_filtered::<(&Player, &Transform), With<Paddle>>();
        let mut balls = world.query_filtered::<&Transform, With<Ball>>();
        for (player, pos) in query.iter(world) {
            if Player::PlayerTwo.ne(player) {continue;}
            let mut ball_pos = Vec3::ZERO;
            for ball in balls.iter(&world) {
                if ball.translation.x > ball_pos.x {ball_pos = ball.translation };
            }
            let error = ball_pos.y - pos.translation.y;
            if error > 10. {
                self.delta = 1.;
            } else if error < -10. {
                self.delta = -1.;
            } else if error.abs() < 1. {
                self.delta = 0.;
            };
        }
    }
}

struct SmartPongAi {
    delta: f32,
}

impl PongAi for SmartPongAi {
    fn get_delta(&self) -> f32 {
        self.delta
    }
    fn update(&mut self, world: &mut World) {
        let mut query = world.query_filtered::<(&Player, &Transform), With<Paddle>>();
        let mut balls = world.query_filtered::<(&Transform, &Velocity), With<Ball>>();
        let window = world.query_filtered::<&Window, With<PrimaryWindow>>().single(&world);
        for (player, pos) in query.iter(world) {
            if Player::PlayerTwo.ne(player) {continue;}
            let mut ball_pos = Vec3::NEG_X * 1000.;
            let mut ball_speed = Vec2::ZERO;
            for (ball, speed) in balls.iter(&world) {
                if speed.linvel.x > 0. {
                    if ball.translation.x > ball_pos.x {
                        ball_pos = ball.translation;
                        ball_speed = speed.linvel;
                    };
                }
            }
            let x_dif = pos.translation.x - ball_pos.x;
            let y_steps = x_dif / ball_speed.x;
            let y_dif = y_steps * ball_speed.y;
            let y_dif = y_dif % window.height();
            let error = ball_pos.y + y_dif - pos.translation.y;
            if error > 10. {
                self.delta = 1.;
            } else if error < -10. {
                self.delta = -1.;
            } else if error.abs() < 1. {
                self.delta = 0.;
            };
        }
    }
}