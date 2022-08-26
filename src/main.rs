#![feature(mixed_integer_ops)]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_kira_audio::prelude::*;
use bevy_mod_picking::*;
use bevy_rapier3d::prelude::*;

mod share;

mod templates;

mod debug_plugin;
use debug_plugin::DebugPlugin;

mod chunk_manager_plugin;
use chunk_manager_plugin::ChunkManagerPlugin;

mod player_manager_plugin;
use player_manager_plugin::PlayerManagerPlugin;

mod follow_plugin;
use follow_plugin::FollowPlugin;

mod asset_plugin;
use asset_plugin::AssetPlugin;

mod item_plugin;
use item_plugin::ItemPlugin;

mod game_plugin;
use game_plugin::GamePlugin;

mod menu_plugin;
use menu_plugin::MenuPlugin;

mod sound_plugin;
use sound_plugin::SoundPlugin;

mod settings_plugin;
use settings_plugin::SettingsPlugin;

mod tutorial_plugin;
use tutorial_plugin::TutorialPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(FollowPlugin)
        .add_plugin(ChunkManagerPlugin)
        .add_plugin(PlayerManagerPlugin)
        .add_plugin(ItemPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(AssetPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(DebugPlugin)
        .add_plugin(GamePlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(AudioPlugin)
        .add_plugin(SoundPlugin)
        .add_plugin(SettingsPlugin)
        .add_plugin(TutorialPlugin)
        .run();
}
