mod normal;
mod orbit;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<Score>()
        .register_type::<Score>()
        .add_plugins(bevy_rapier2d::plugin::RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(bevy_rapier2d::prelude::RapierDebugRenderPlugin::default())
        .add_plugins(normal::NormalPlugin)
        .add_plugins(orbit::OrbitPlugin)
        .add_systems(Update, clean_up_lifetime)
        .register_type::<Paddle>();
    }
}

#[derive(Component)]
struct GameItem;

#[derive(Debug, Component, Reflect, Default)]
#[reflect(Component)]
pub struct Paddle {
    size: Vec2,
    speed: f32,
}

#[derive(Component)]
pub struct Ball;

#[derive(Debug, Resource, Default, Reflect)]
#[reflect(Resource)]
struct Score(u8, u8);

#[derive(Component)]
struct Goal;

fn clean_up_game(
    query: Query<Entity, With<GameItem>>,
    mut commands: Commands,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
struct Lifetime(f32);

fn clean_up_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut life) in &mut query {
        life.0 -= time.delta_seconds();
        if life.0 < 0. {
            commands.entity(entity).despawn_recursive();
        }
    }
}