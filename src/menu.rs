use bevy::{prelude::*, input::{keyboard::KeyboardInput, ButtonState}};
use belly::prelude::*;
use crate::{GameState, PlayerKeyBinds, KeyBindings};

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
        app.add_system(setup_belly.on_startup());
    }
}

fn setup_belly(mut commands: Commands) {
    commands.add(StyleSheet::load("color-picker.ess"));
}

struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_main_menu.in_schedule(OnEnter(GameState::MainMenu)))
        .add_system(close_main_menu.in_schedule(OnExit(GameState::MainMenu)));
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_state::<SetBinding>()
        .add_system(set_key_binding.run_if(not(in_state(SetBinding::None))))
        .add_system(spawn_settings_menu.in_schedule(OnEnter(GameState::SettingsMenu)))
        .add_system(close_settings_menu.in_schedule(OnExit(GameState::SettingsMenu)))
        .add_system(name_state::<SetBinding>.run_if(state_changed::<SetBinding>()));
    }
}

#[derive(Component)]
enum MenuRoot {
    Main,
    Setting,
}

fn spawn_main_menu(
    mut commands: Commands
) {
    let root = MenuRoot::Main;
    commands.add(eml! {
        <div c:menu with=root>
            <button on:press=run!(|c| {
                c.commands().add(|world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::Playing);
                })
            })>
            <label c:content value="Play"/>
            </button>
            <button on:press=run!(|c| {
                c.commands().add(|world: &mut World| {
                    world.resource_mut::<NextState<GameState>>().set(GameState::SettingsMenu);
                })
            })>
            <label c:content value="Settings"/>
            </button>
        </div> // end top
    });
}

fn close_main_menu(
    items: Query<(Entity, &MenuRoot)>,
    mut commands: Commands,
) {
    for (entity, root) in &items {
        if let MenuRoot::Main = root {
            commands.entity(entity).despawn_recursive()
        }
    }
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
    let root = MenuRoot::Setting;
    commands.add(eml! {
        <div c:menu with=root>
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

fn close_settings_menu(
    items: Query<(Entity, &MenuRoot)>,
    mut commands: Commands,
) {
    for (entity, root) in &items {
        if let MenuRoot::Setting = root {
            commands.entity(entity).despawn_recursive()
        }
    }
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
        match state.0 {
            SetBinding::None => unreachable!(),
            SetBinding::P1Up => bindings.set(true, true, new),
            SetBinding::P1Down => bindings.set(true, false, new),
            SetBinding::P2Up => bindings.set(false, true, new),
            SetBinding::P2Down => bindings.set(false, false, new),
        }
        next.set(SetBinding::None);
        if let Err(e) = pkv.set("KeyBind", &*bindings) {error!("{e}")};
        info!("Settings: {:#?}", bindings);
    }
}

fn name_state<T: States>(state: Res<State<T>>) {
    info!("You are in state: {:?}", state.0);
}