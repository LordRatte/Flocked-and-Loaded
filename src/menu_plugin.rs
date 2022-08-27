use crate::game_plugin::{CurrentScore, HighScores, NewGameEvent, PauseEvent, Paused};
use crate::settings_plugin::SaveEvent;
use crate::sound_plugin::{EffectsVolume, MusicVolume, PlayMusic};
use crate::tutorial_plugin::{ShowTutorial, ShowTutorials, Tutorial};
use bevy::app::AppExit;
use bevy::app::Plugin;
use bevy::prelude::{App, EventWriter, Res, ResMut};
use bevy_egui::egui::*;
use bevy_egui::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Menu>()
            .add_system(menu)
            .add_system(hud)
            .add_system(pause_menu);
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Menu {
    Main,
    Options,
    Credits,
    Game,
    GameOver,
}

impl Default for Menu {
    fn default() -> Self {
        Menu::Main
    }
}

fn sized_text(text: &str, size: Option<f32>) -> RichText {
    RichText::new(text).size(size.unwrap_or(40.0))
}

fn hud(
    mut ev_pause: EventWriter<PauseEvent>,
    mut egui_context: ResMut<EguiContext>,
    mut play_music: ResMut<PlayMusic>,
    menu: ResMut<Menu>,
    current_score: Res<CurrentScore>,
) {
    if *menu == Menu::Game {
        TopBottomPanel::top("hud").show(egui_context.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                if ui.button(sized_text("⚙", None)).clicked() {
                    ev_pause.send(PauseEvent);
                }
                ui.label(sized_text(
                    format!("Score: {}", current_score.0).as_str(),
                    None,
                ));
            });
        });
        play_music.0 = true;
    } else {
        play_music.0 = false;
    }
}

fn menu(
    mut egui_context: ResMut<EguiContext>,
    mut ev_new_game: EventWriter<NewGameEvent>,
    mut menu: ResMut<Menu>,
    mut exit: EventWriter<AppExit>,
    high_scores: Res<HighScores>,
    mut music_volume: ResMut<MusicVolume>,
    mut effects_volume: ResMut<EffectsVolume>,
    mut show_tutorials: ResMut<ShowTutorials>,
    mut ev_save: EventWriter<SaveEvent>,
    current_score: Res<CurrentScore>,
) {
    let mut m = *menu;
    if &m != &Menu::Game {
        CentralPanel::default().show(egui_context.ctx_mut(), |ui| match m {
            Menu::Main => menu_main(ui, &mut m, &mut ev_new_game, &mut exit),
            Menu::Options => menu_options(
                ui,
                &mut m,
                &mut music_volume,
                &mut effects_volume,
                &mut show_tutorials,
                &mut ev_save,
            ),
            Menu::Credits => menu_credits(ui, &mut m),
            Menu::GameOver => menu_game_over(ui, &current_score),
            _ => (),
        });
        TopBottomPanel::bottom("scores").show(egui_context.ctx_mut(), |ui| {
            ui.label(sized_text(
                format!(
                    "High Score: {}\nLow Score: {}\nFixed-point Score: 0",
                    high_scores.1, high_scores.0
                )
                .as_str(),
                None,
            ))
        });
    }
    *menu = m;
}

fn menu_main(
    ui: &mut Ui,
    menu: &mut Menu,
    ev_new_game: &mut EventWriter<NewGameEvent>,
    exit: &mut EventWriter<AppExit>,
) {
    ui.vertical_centered(|ui| {
        ui.label(sized_text("Flocked and Loaded", Some(60.)));
        if ui
            .add_sized(
                [200.0, 100.0],
                egui::Button::new(sized_text("New Game", None)),
            )
            .clicked()
        {
            *menu = Menu::Game;
            ev_new_game.send(NewGameEvent);
        }
        if ui
            .add_sized(
                [200.0, 100.0],
                egui::Button::new(sized_text("Credits", None)),
            )
            .clicked()
        {
            *menu = Menu::Credits;
        };
        if ui
            .add_sized(
                [200.0, 100.0],
                egui::Button::new(sized_text("Options", None)),
            )
            .clicked()
        {
            *menu = Menu::Options;
        }
        if ui
            .add_sized([200.0, 100.0], egui::Button::new(sized_text("Exit", None)))
            .clicked()
        {
            exit.send(AppExit);
        }
    });
}

fn menu_options(
    ui: &mut Ui,
    menu: &mut Menu,
    music_volume: &mut ResMut<MusicVolume>,
    effects_volume: &mut ResMut<EffectsVolume>,
    show_tutorials: &mut ResMut<ShowTutorials>,
    ev_save: &mut EventWriter<SaveEvent>,
) {
    if ui.button(sized_text("⬅", None)).clicked() {
        *menu = Menu::Main;
    }
    ui.vertical(|ui| {
        settings_components(
            ui,
            &mut music_volume.0,
            &mut effects_volume.0,
            &mut show_tutorials.0,
            ev_save,
        )
    });
}

fn settings_components(
    ui: &mut Ui,
    music_volume: &mut f32,
    effects_volume: &mut f32,
    show_tutorials: &mut bool,
    ev_save: &mut EventWriter<SaveEvent>,
) {
    let old_music_volume = *music_volume;
    ui.add(egui::Slider::new(music_volume, 0.0..=100.0).text("Music Volume"));
    if &old_music_volume != music_volume {
        ev_save.send(SaveEvent);
    }

    let old_effects_volume = *effects_volume;
    ui.add(egui::Slider::new(effects_volume, 0.0..=100.0).text("Sound Effects Volume"));
    if &old_effects_volume != effects_volume {
        ev_save.send(SaveEvent);
    }

    let old_show_tutorials = *show_tutorials;
    ui.checkbox(show_tutorials, "Show Tutorials");
    if &old_show_tutorials != show_tutorials {
        ev_save.send(SaveEvent);
    }
}

fn pause_menu(
    paused: ResMut<Paused>,
    show_tutorial: Res<ShowTutorial>,
    mut egui_context: ResMut<EguiContext>,
    mut music_volume: ResMut<MusicVolume>,
    mut effects_volume: ResMut<EffectsVolume>,
    mut show_tutorials: ResMut<ShowTutorials>,
    mut ev_save: EventWriter<SaveEvent>,
) {
    if paused.0 {
        match show_tutorial.0 {
            None => {
                Window::new("Paused").show(egui_context.ctx_mut(), |mut ui| {
                    settings_components(
                        &mut ui,
                        &mut music_volume.0,
                        &mut effects_volume.0,
                        &mut show_tutorials.0,
                        &mut ev_save,
                    );
                });
            }
            Some(tutorial) => {
                let tip_name = match tutorial {
                    Tutorial::Player => "Player",
                    Tutorial::Cage => "Cage",
                    Tutorial::Launcher => "Launcher",
                    Tutorial::Minion => "Flock",
                };
                let content = match tutorial {
                    Tutorial::Player => "Control your player with the direction keys or “W”, “A”, “S” and “D”.\n\nFind crates with trapped sheep in and free them to combine into a bigger flock.\n\nTravel as far as you can.\n\nPause/Resume with “Esc” or the gear icon.",
                    Tutorial::Cage =>"Open a cage by standing near it and pressing “E” or by getting a vaporiser blow it open.",
                    Tutorial::Launcher=>"The Launcher will periodically fire vaporisers at you. When vaporisers turn blue, they are getting ready to go off.\n Try to find a way to clear a way with them when your path is blocked.",
                    Tutorial::Minion => "Your flock follows you as the leader. Click on a follower to make it the leader. You can also press “Q” to quick-switch"
                };
                Window::new(format!("Tip: {tip_name}")).show(egui_context.ctx_mut(), |ui| {
                    ui.label(sized_text(content, Some(20.)))
                });
            }
        }
    }
}

fn menu_credits(ui: &mut Ui, menu: &mut Menu) {
    if ui.button(sized_text("⬅", None)).clicked() {
        *menu = Menu::Main;
    }
    ui.label(sized_text("Attributions", None));
    ui.label(sized_text("Thank you to Isabella, Philip, Michael and everyone else who provided suggestions, advice and encouragement", Some(20.)));
    ui.label(sized_text("Code and Animations", None));
    ui.label(sized_text("LordRatte", Some(20.)));
    ui.label(sized_text("Music", None));
    ui.label(sized_text(
        "Royalty free music from https://pixabay.com",
        Some(20.),
    ));
    ui.label(sized_text(
        include_str!("../assets/music/licences.txt"),
        Some(20.),
    ));
}

fn menu_game_over(ui: &mut Ui, current_score: &Res<CurrentScore>) {
    ui.label(sized_text("Game Over", Some(60.)));
    ui.label(sized_text(
        format!("Ewe got a score of: {}", current_score.0).as_str(),
        None,
    ));
}
