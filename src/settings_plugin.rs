use crate::game_plugin::HighScores;
use crate::sound_plugin::{EffectsVolume, MusicVolume};
use crate::tutorial_plugin::ShowTutorials;
use bevy::app::Plugin;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::Write;

pub struct SettingsPlugin;

pub struct SaveEvent;

#[derive(Serialize, Deserialize)]
enum SettingType {
    Float(f32),
    String(String),
    Bool(bool),
    Pair(isize, isize),
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SaveEvent>()
            .add_startup_system(setup_config)
            .add_system(save_config);
    }
}

fn setup_config(
    mut rmusic_volume: ResMut<MusicVolume>,
    mut rhigh_scores: ResMut<HighScores>,
    mut rshow_tutorials: ResMut<ShowTutorials>,
    mut reffects_volume: ResMut<EffectsVolume>,
) {
    if let Ok(data) = read_to_string("settings.json") {
        let settings: Result<HashMap<String, SettingType>, _> = serde_json::from_str(&data);
        if let Ok(settings) = settings {
            if let Some(SettingType::Float(music_volume)) = settings.get("music_volume") {
                rmusic_volume.0 = *music_volume;
            }
            if let Some(SettingType::Float(effects_volume)) = settings.get("effects_volume") {
                reffects_volume.0 = *effects_volume;
            }
            if let Some(SettingType::Bool(show_tutorials)) = settings.get("show_tutorials") {
                rshow_tutorials.0 = *show_tutorials;
            }
            if let Some(SettingType::Pair(ls, hs)) = settings.get("high_scores") {
                *rhigh_scores = HighScores(*ls, *hs)
            }
        }
    }
}

fn save_config(
    mut ev_save: EventReader<SaveEvent>,
    rmusic_volume: Res<MusicVolume>,
    reffects_volume: Res<EffectsVolume>,
    rshow_tutorials: Res<ShowTutorials>,
    rhigh_scores: Res<HighScores>,
) {
    let mut settings = HashMap::new();
    settings.insert("music_volume", SettingType::Float(rmusic_volume.0));
    settings.insert("effects_volume", SettingType::Float(reffects_volume.0));
    settings.insert("show_tutorials", SettingType::Bool(rshow_tutorials.0));
    settings.insert(
        "high_scores",
        SettingType::Pair(rhigh_scores.0, rhigh_scores.1),
    );
    for _ in ev_save.iter().last() {
        let new_file = File::create("settings.json");
        if let Ok(mut output) = new_file {
            let stringified = serde_json::to_string(&settings);
            if let Ok(string) = stringified {
                if let Err(err) = write!(output, "{}", string) {
                    println!("Error saving settings: {}", err)
                }
            } else {
                println!("Could not serialize: {:?}", stringified);
            }
        } else {
            println!("Could not create file: {:?}", new_file);
        }
    }
}
