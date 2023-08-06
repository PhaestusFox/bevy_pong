use bevy::{prelude::*, input::{keyboard::KeyboardInput, ButtonState}};
use belly::prelude::*;
use crate::{GameState, PlayerKeyBinds};

pub struct MenuPlugins;

impl PluginGroup for MenuPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        bevy::app::PluginGroupBuilder::start::<MenuPlugins>()
        .add(MenuCore)
        .add(MainMenuPlugin)
        .add(SettingsPlugin)
    }
}

pub struct MenuCore;

impl Plugin for MenuCore {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_belly);
    }
}

fn setup_belly(mut commands: Commands) {
    commands.add(StyleSheet::load("color-picker.ess"));
}

struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
        .add_systems(OnExit(GameState::MainMenu), close_menu);
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_state::<SetBinding>()
        .add_systems(Update, set_key_binding.run_if(not(in_state(SetBinding::None))))
        .add_systems(OnEnter(GameState::SettingsMenu), spawn_settings_menu)
        .add_systems(OnExit(GameState::SettingsMenu), close_menu)
        .add_systems(Update, name_state::<SetBinding>.run_if(state_changed::<SetBinding>()));
    }
}

fn spawn_main_menu(
    mut commands: Commands
) {
    commands.add(eml! {
        <div c:menu with>
            <button on:press=run!(|c| {
                c.commands().add(|world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::OpponentSelect);
                })
            })>
            <label c:content value="Play Normal"/>
            </button>

            <button on:press=run!(|c| {
                c.commands().add(|world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::PlayingOrbit);
                })
            })>
            <label c:content value="Play Orbit"/>
            </button>

            <button on:press=run!(|c| {
                c.commands().add(|world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::SettingsMenu);
                })
            })>
            <label c:content value="Settings"/>
            </button>
        </div>
    });
}

pub fn close_menu(
    mut elements: Elements,
) {
    elements.select(".menu").remove();
}

#[derive(Default, States, Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum SetBinding {
    #[default]
    None,
    P1Up,
    P1Down,
    P2Up,
    P2Down,
}

fn spawn_settings_menu(
    mut commands: Commands
) {
    commands.add(eml! {
        <div c:menu>
            <div c:even>
                <label value="Player One: "/>
                <button on:press=run!(|c| {
                    c.commands().add(|world: &mut World| {
                        world.resource_mut::<NextState<SetBinding>>().set(SetBinding::P1Up);
                    })
                })><label bind:value=from!(PlayerKeyBinds:player1.move_up|fmt.c("Up: {c:?}"))/></button>
                <button on:press=run!(|c| {
                    c.commands().add(|world: &mut World| {
                        world.resource_mut::<NextState<SetBinding>>().set(SetBinding::P1Down);
                    })
                })><label bind:value=from!(PlayerKeyBinds:player1.move_down|fmt.c("Down: {c:?}"))/></button>
            </div>
            <div c:even>
                <label value="Player Two: "/>
                <button on:press=run!(|c| {
                    c.commands().add(|world: &mut World| {
                        world.resource_mut::<NextState<SetBinding>>().set(SetBinding::P2Up);
                    })
                })><label bind:value=from!(PlayerKeyBinds:player2.move_up|fmt.c("Up: {c:?}"))/></button>
                <button on:press=run!(|c| {
                    c.commands().add(|world: &mut World| {
                        world.resource_mut::<NextState<SetBinding>>().set(SetBinding::P2Down);
                    })
                })><label bind:value=from!(PlayerKeyBinds:player2.move_down|fmt.c("Down: {c:?}"))/></button>
            </div>
        </div>
    });
}


const BAND_KEYS: [KeyCode; 2] = [KeyCode::Escape, KeyCode::Return];

fn set_key_binding(
    state: Res<State<SetBinding>>,
    mut events: EventReader<KeyboardInput>,
    mut next: ResMut<NextState<SetBinding>>,
    mut pkv: ResMut<bevy_pkv::PkvStore>,
    mut bindings: ResMut<PlayerKeyBinds>,
) {
    let mut new_key = None;
    for event in events.iter() {
        if event.state != ButtonState::Pressed {continue;};
        if let Some(key_bind) = event.key_code {
            if BAND_KEYS.contains(&key_bind) {continue;}
            new_key = Some(key_bind);
        }
    }
    if let Some(new) = new_key {
        match state.get() {
            SetBinding::None => unreachable!(),
            SetBinding::P1Up => bindings.set(crate::Player::PlayerOne, true, new),
            SetBinding::P1Down => bindings.set(crate::Player::PlayerOne, false, new),
            SetBinding::P2Up => bindings.set(crate::Player::PlayerTwo, true, new),
            SetBinding::P2Down => bindings.set(crate::Player::PlayerTwo, false, new),
        }
        next.set(SetBinding::None);
        if let Err(e) = pkv.set("KeyBind", &*bindings) {error!("{e}")};
        info!("Settings: {:#?}", bindings);
    }
}

fn name_state<T: States>(state: Res<State<T>>) {
    info!("You are in state: {:?}", state.get());
}