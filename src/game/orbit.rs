use bevy::prelude::*;
use belly::prelude::*;
use bevy_kira_audio::{Audio, AudioControl, AudioSource};
use crate::{GameState, PlayerKeyBinds};
use super::*;
pub struct OrbitPlugin;

impl Plugin for OrbitPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<GameEvent>()
        .configure_set(Update, Playing.run_if(in_state(GameState::PlayingOrbit)))
        .add_systems(OnEnter(GameState::PlayingOrbit), spawn_orbit_world)
        .add_systems(OnExit(GameState::PlayingOrbit), clean_up_game)
        .add_systems(Update, move_paddle.in_set(Playing))
        .add_systems(Update, (process_collision_event, process_events).in_set(Playing))
        .add_systems(OnEnter(GameState::PlayingOrbit), (particle, setup_screen_shake))
        .init_resource::<Sounds>()
        .add_systems(Update, screen_shake.in_set(Playing));
    }
}

#[derive(SystemSet, Hash, Debug, Clone, Copy, PartialEq, Eq)]
struct Playing;

#[derive(Component)]
struct OrbitPaddle(f32);

const PADDLE_OFFSET: f32 = 250.;

#[derive(Component)]
struct InPlay(Entity);

const BALLSPEED: f32 = 100.;
const PADDLESIZE: Vec2 = Vec2::new(25., 100.);
const BALLSIZE: f32 = 50.;

fn spawn_orbit_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {

    let id = commands.spawn(
        (GameItem, SpatialBundle::default(),
        Collider::ball(PADDLE_OFFSET + BALLSIZE),
        Sensor
    )).id();

    commands.spawn((GameItem, SpriteBundle {
        sprite: Sprite { color: Color::WHITE, custom_size: Some(PADDLESIZE), ..Default::default()},
        transform: Transform::from_translation(Vec3::new(PADDLE_OFFSET, 0., 0.)),
        texture: asset_server.load("moon.png"),
        ..Default::default()
    },
    OrbitPaddle(0.),
    Paddle{size: PADDLESIZE, speed: 0.05},
    Collider::cuboid(PADDLESIZE.x / 2., PADDLESIZE.y / 2.),
    Restitution{coefficient: 1.05, ..Default::default()},
    Friction{coefficient: 0., ..Default::default()},
    ));

    commands.spawn((GameItem, SpriteBundle {
        sprite: Sprite { color: Color::WHITE, custom_size: Some(Vec2::splat(BALLSIZE)), ..Default::default()},
        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
        texture: asset_server.load("earth.png"),
        ..Default::default()
    },
    Ball,
    RigidBody::Dynamic,
    Collider::ball(BALLSIZE / 2.),
    Velocity::linear(Vec2::new(rand::random(), rand::random()).normalize() * BALLSPEED),
    Restitution{coefficient: 1.0, ..Default::default()},
    Damping{linear_damping: 0., ..Default::default()},
    LockedAxes::ROTATION_LOCKED,
    GravityScale(0.),
    Friction{coefficient: 0., ..Default::default()},
    InPlay(id),
    ActiveEvents::COLLISION_EVENTS,
    ));

    commands.add(eml! {
        <div c:center>
        <label c:orbit c:score bind:value=from!(Score:0| fmt.c("{c}"))/>
        </div>
    })
}

fn move_paddle(
    mut paddle: Query<(&mut Transform, &mut OrbitPaddle, &Paddle)>,
    input: Res<Input<KeyCode>>,
    key_binding: Res<PlayerKeyBinds>,
) {
    let direction = if input.pressed(key_binding.player1.move_up) {
        1.
    } else if input.pressed(key_binding.player1.move_down) {
        0.01
    } else {
        0.1
    };

    for (mut pos,mut orbit, paddle) in &mut paddle {
        orbit.0 += direction * paddle.speed;
        let x = orbit.0.cos();
        let y = orbit.0.sin();
        pos.translation = Vec3::new(x * PADDLE_OFFSET, y * PADDLE_OFFSET, 0.);
        pos.rotate_z(direction * paddle.speed);
    }
}

fn process_collision_event(
    query: Query<&InPlay>,
    paddles: Query<(), With<Paddle>>,
    mut events: EventWriter<GameEvent>,
    mut events2: EventReader<CollisionEvent>,
    mut score: ResMut<Score>,
) {
    let ball = query.single();
    for event in events2.iter() {
        match event {
            CollisionEvent::Started(other, _, _) => {
                if paddles.contains(*other) {
                    events.send(GameEvent::SpawnParticles(*other));
                    score.0 += 1;
                }
            },
            CollisionEvent::Stopped(other, _, _) => {
                if *other == ball.0 {
                    score.0 = 0;
                    events.send(GameEvent::ResetBall);
                }
            },
        }
    }
}

fn process_events(
    mut commands: Commands,
    effect: Res<ParticleEffectHandle>,
    mut balls: Query<(&mut Transform, &mut Velocity), With<Ball>>,
    mut events: EventReader<GameEvent>,
    paddles: Query<&Transform, Without<Ball>>,
    sounds: Res<Sounds>,
    audio: Res<Audio>,
    mut query: Query<&mut ScreenShake>,
) {
    for event in events.iter() {
        match event {
            GameEvent::ResetBall => for mut ball in &mut balls {ball.0.translation = Vec3::ZERO; ball.1.linvel = ball.1.linvel.normalize() * BALLSPEED;},
            GameEvent::SpawnParticles(entity) => {
                let ball = balls.single().0;
                let Ok(paddle) = paddles.get(*entity) else {error!("Transform on {:?} not found", entity); continue;};
                let mut paddle = paddle.looking_at(Vec3::ZERO, Vec3::Z);
                paddle.rotate_local_z(-1.5708);
                let targer = ball.looking_at(paddle.translation, Vec3::Z).forward();
                for mut shake in &mut query {
                    shake.0 += targer.truncate();
                }
                let pos = ball.translation + (targer * BALLSIZE / 2.);
                        // Spawn an instance of the particle effect, and override its Z layer to
                        // be above the reference white square previously spawned.
                        commands
                        .spawn((ParticleEffectBundle {
                            // Assign the Z layer so it appears in the egui inspector and can be modified at runtime
                            effect: ParticleEffect::new(effect.0.clone()).with_z_layer_2d(Some(0.1)),
                            transform: Transform::from_translation(pos),
                            ..default()
                        },
                        Name::new("effect:moon"),
                        Lifetime(4.),
                        ));
                        // Spawn an instance of the particle effect, and override its Z layer to
                        // be above the reference white square previously spawned.
                        commands
                        .spawn((ParticleEffectBundle {
                            // Assign the Z layer so it appears in the egui inspector and can be modified at runtime
                            effect: ParticleEffect::new(effect.1.clone()).with_z_layer_2d(Some(0.1)),
                            transform: Transform::from_translation(pos).with_rotation(-paddle.rotation),
                            ..default()
                        },
                        Name::new("effect:earth"),
                        Lifetime(4.),
                        ));
                let pan = (((pos.x / PADDLE_OFFSET) as f64 / 2.) + 0.5).clamp(0., 1.);
                dbg!(pan);
                audio.play(sounds.0.clone()).with_panning(pan);
                audio.play(sounds.1.clone()).with_panning(pan);

            }
        }
    }
}

#[derive(Event)]
enum GameEvent {
    ResetBall,
    SpawnParticles(Entity),
}

use bevy_hanabi::prelude::*;

fn particle(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    // Create a color gradient for the particles
    let mut gradient_moon = Gradient::new();
    gradient_moon.add_key(0.0, Vec4::new(0.5, 0.5, 0.5, 1.0));
    gradient_moon.add_key(1.0, Vec4::new(0.5, 0.5, 0.5, 0.0));

    let writer = ExprWriter::new();

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let lifetime = writer.lit(4.).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        radius: writer.lit(10.).expr(),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocityCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        speed: writer.lit(100.).expr(),
    };

    // Create a new effect asset spawning 30 particles per second from a circle
    // and slowly fading from blue-ish to transparent over their lifetime.
    // By default the asset spawns the particles at Z=0.
    let spawner = Spawner::rate(60.0.into());
    let effect_moon = effects.add(
        EffectAsset::new(4096, spawner, writer.finish())
            .with_name("2d")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec2::splat(10.)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier { gradient: gradient_moon }),
    );

    // Create a color gradient for the particles
    let mut gradient_earth = Gradient::new();
    gradient_earth.add_key(0.0, Vec4::new(0.1, 0.1, 1.0, 1.0));
    gradient_earth.add_key(1.0, Vec4::new(0.1, 0.1, 1.0, 0.0));

    let writer = ExprWriter::new();

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let lifetime = writer.lit(4.).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let init_pos = SetPositionCone3dModifier {
        height: writer.lit(1.).expr(),
        base_radius: writer.lit(0.).expr(),
        top_radius: writer.lit(5.).expr(),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocityCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        speed: writer.lit(100.).expr(),
    };

    // Create a new effect asset spawning 30 particles per second from a circle
    // and slowly fading from blue-ish to transparent over their lifetime.
    // By default the asset spawns the particles at Z=0.
    let spawner = Spawner::rate(60.0.into());
    let effect_earth = effects.add(
        EffectAsset::new(4096, spawner, writer.finish())
            .with_name("2d")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .render(SizeOverLifetimeModifier {
                gradient: Gradient::constant(Vec2::splat(10.)),
                screen_space_size: false,
            })
            .render(ColorOverLifetimeModifier { gradient: gradient_earth }),
    );

    commands.insert_resource(ParticleEffectHandle(effect_moon, effect_earth));
}

#[derive(Resource)]
struct ParticleEffectHandle(Handle<EffectAsset>, Handle<EffectAsset>);

#[derive(Resource)]
struct Sounds(Handle<AudioSource>, Handle<AudioSource>);

impl FromWorld for Sounds {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Sounds(asset_server.load("hit_1.wav"), asset_server.load("hit_2.wav"))
    }
}

#[derive(Component)]
struct ScreenShake(Vec2);

fn setup_screen_shake(
    mut commands: Commands,
    query: Query<Entity, With<Camera>>,
) {
    for entity in &query {
        commands.entity(entity).insert((
            ScreenShake(Vec2::splat(1.)),
            Velocity::linear(Vec2::splat(0.)),
            RigidBody::Dynamic,
            ExternalImpulse::default(),
            GravityScale(0.),
        ));
    }
}

fn screen_shake(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut ExternalImpulse, &mut ScreenShake)>
) {
    for (mut transform, mut velocity,mut shake) in &mut query {
        transform.translation = transform.translation.lerp(Vec3::ZERO, time.delta_seconds());
        velocity.impulse = -shake.0 * 1000.;
        shake.0 *= -0.5;
    }
}