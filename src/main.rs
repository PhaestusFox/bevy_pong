use bevy::prelude::*;
use serde::{Serialize, Deserialize};

mod menu;

// mod game;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(AssetPlugin {
        watch_for_changes: true,
        ..Default::default()
    }))
    .add_plugin(belly::prelude::BellyPlugin)
    .add_plugin(bevy_editor_pls::EditorPlugin)
    .add_state::<GameState>()
    .add_plugins(menu::MenuPlugins)
    .insert_resource(bevy_pkv::PkvStore::new("PhaestusFox", "Pong"))
    .add_system(spawn_cam.on_startup())
    .init_resource::<PlayerKeyBinds>()
    .add_system(back_to_main_menu)
    .run()
}

fn spawn_cam(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Default, States, Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum GameState {
    #[default]
    MainMenu,
    SettingsMenu,
    Playing,
}

#[derive(Resource, Serialize, Deserialize, Debug)]
struct PlayerKeyBinds {
    player1: KeyBindings,
    player2: KeyBindings,
}

impl PlayerKeyBinds {
    fn set(&mut self, player_one: bool, up: bool, to: KeyCode) {
        info!("Start: {:#?}", self);
        // get old
        let old = match (player_one, up) {
            (true, true) => self.player1.move_up,
            (true, false) => self.player1.move_down,
            (false, true) => self.player2.move_up,
            (false, false) => self.player2.move_down,
        };
        // set new
        match (player_one, up) {
            (true, true) => self.player1.move_up = to,
            (true, false) => self.player1.move_down = to,
            (false, true) => self.player2.move_up = to,
            (false, false) => self.player2.move_down = to,
        };

        info!("Mid: {:#?}", self);

        //replace dub with old
        for val in [true, false].iter().cloned().zip([true, false]) {
            if (player_one, up) == val {continue;}
            let current = match val {
                (true, true) => self.player1.move_up,
                (true, false) => self.player1.move_down,
                (false, true) => self.player2.move_up,
                (false, false) => self.player2.move_down,
            };
            if current == to {
                match val {
                    (true, true) => self.player1.move_up = old,
                    (true, false) => self.player1.move_down = old,
                    (false, true) => self.player2.move_up = old,
                    (false, false) => self.player2.move_down = old,
                };
            }
        }
        info!("End: {:#?}", self);
    }
}

impl FromWorld for PlayerKeyBinds {
    fn from_world(world: &mut World) -> Self {
        let pkv = world.resource::<bevy_pkv::PkvStore>();
        if let Ok(bindings) = pkv.get("KeyBind") {
            bindings
        } else {
            PlayerKeyBinds {
                player1: KeyBindings { move_up: KeyCode::W, move_down: KeyCode::S },
                player2: KeyBindings { move_up: KeyCode::Up, move_down: KeyCode::Down},
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct KeyBindings {
    move_up: KeyCode,
    move_down: KeyCode,
}

fn back_to_main_menu(
    mut next: ResMut<NextState<GameState>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        next.set(GameState::MainMenu);
    }
}