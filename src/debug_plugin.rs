use bevy::app::Plugin;
use bevy::prelude::*;

use crate::game_plugin::NewGameEvent;
use crate::menu_plugin::Menu;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_mod_picking::{DebugCursorPickingPlugin};
use bevy_rapier3d::prelude::RapierDebugRenderPlugin;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    #[cfg(build = "debug")]
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin::new())
            .add_plugin(InspectableRapierPlugin)
            .add_plugin(RapierDebugRenderPlugin::default())
            .add_startup_system(new_game_debug)
            .add_plugin(DebugCursorPickingPlugin);
    }

    #[cfg(not(build = "debug"))]
    fn build(&self, app: &mut App) {}
}

fn new_game_debug(mut ev_new_game: EventWriter<NewGameEvent>, mut menu: ResMut<Menu>) {
    *menu = Menu::Game;
    ev_new_game.send(NewGameEvent);
}
