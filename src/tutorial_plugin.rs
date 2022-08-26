use crate::game_plugin::{GameTime, PauseEvent};
use crate::item_plugin::{Item, ItemType};
use crate::player_manager_plugin::{Minion, Player};
use bevy::app::Plugin;
use bevy::prelude::*;
use std::time::Duration;

pub struct TutorialPlugin;

#[derive(Clone, Copy)]
pub enum Tutorial {
    Player,
    Cage,
    Launcher,
    Minion,
}

#[derive(Default)]
pub struct ShowTutorial(pub Option<Tutorial>);

#[derive(Default)]
struct PlayerTutorial(bool);

pub struct ShowTutorials(pub bool);

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShowTutorial>()
            .insert_resource(ShowTutorials(true))
            .init_resource::<PlayerTutorial>()
            .add_system(watch_for_novelty);
    }
}

fn watch_for_novelty(
    player: Query<&Player>,
    minion: Query<&Minion>,
    items: Query<&Item>,
    time: Res<GameTime>,
    mut last_tut: Local<Duration>,
    mut player_tutorial: Local<bool>,
    mut minion_tutorial: Local<bool>,
    mut launcher_tutorial: Local<bool>,
    mut cage_tutorial: Local<bool>,
    mut ev_pause: EventWriter<PauseEvent>,
    mut show_tutorial: ResMut<ShowTutorial>,
    show_tutorials: Res<ShowTutorials>,
) {
    if show_tutorials.0 && (time.0.elapsed() - *last_tut) >= Duration::from_secs(4) {
        let did_tut = if !*player_tutorial {
            for _ in player.iter().last() {
                *player_tutorial = true;
                show_tutorial.0 = Some(Tutorial::Player);
                ev_pause.send(PauseEvent);
            }
            true
        } else if !*launcher_tutorial {
            for _ in items
                .iter()
                .filter(|item| match item.0 {
                    ItemType::Launcher => true,
                    _ => false,
                })
                .last()
            {
                *launcher_tutorial = true;
                show_tutorial.0 = Some(Tutorial::Launcher);
                ev_pause.send(PauseEvent);
            }
            true
        } else if !*cage_tutorial {
            for _ in items
                .iter()
                .filter(|item| match item.0 {
                    ItemType::Cage => true,
                    _ => false,
                })
                .last()
            {
                *cage_tutorial = true;
                show_tutorial.0 = Some(Tutorial::Cage);
                ev_pause.send(PauseEvent);
            }
            true
        } else if !*minion_tutorial {
            for _ in minion.iter().last() {
                *minion_tutorial = true;
                show_tutorial.0 = Some(Tutorial::Minion);
                ev_pause.send(PauseEvent);
            }
            true
        } else {
            false
        };
        if did_tut {
            *last_tut = time.0.elapsed();
        }
    }
}
