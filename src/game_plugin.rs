use crate::asset_plugin::{Objects, TriggerLoopAnimEvent};
use crate::chunk_manager_plugin::{Chunk, ChunkChangeEvent, CHUNK_MID, FBLOCK_SIZE};
use crate::follow_plugin::FollowTarget;
use crate::menu_plugin::Menu;
use crate::player_manager_plugin::Minion;
use crate::player_manager_plugin::Player;
use crate::settings_plugin::SaveEvent;
use crate::templates;
use crate::tutorial_plugin::ShowTutorial;
use bevy::app::Plugin;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_rapier3d::prelude::*;

pub struct GamePlugin;

pub struct NewGameEvent;

pub struct Paused(pub bool);

pub struct PauseEvent;

pub struct CurrentScore(pub isize);
pub struct HighScores(pub isize, pub isize);

#[derive(Debug)]
pub struct GameTime(pub Stopwatch);

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NewGameEvent>()
            .add_event::<PauseEvent>()
            .insert_resource(Paused(false))
            .insert_resource(CurrentScore(0))
            .insert_resource(HighScores(0, 0))
            .insert_resource(GameTime(Stopwatch::new()))
            .add_system(init_game)
            .add_system(controls)
            .add_system(paused_check)
            .add_system(score_manager)
            .add_system(tick);
    }
}

fn tick(time: Res<Time>, mut stopwatch: ResMut<GameTime>) {
    stopwatch.0.tick(time.delta());
}

fn score_manager(
    mut ev_chunk_change: EventReader<ChunkChangeEvent>,
    mut scores: ResMut<CurrentScore>,
    mut high_scores: ResMut<HighScores>,
    mut ev_save: EventWriter<SaveEvent>,
) {
    if let Some(chunk) = ev_chunk_change
        .iter()
        .map(|ChunkChangeEvent { newchunk, .. }| newchunk.0)
        .max()
    {
        let signed_pos = if chunk >= CHUNK_MID.0 {
            (chunk - CHUNK_MID.0) as isize
        } else {
            -((CHUNK_MID.0 - chunk) as isize)
        };
        scores.0 = if (signed_pos < scores.0) || (signed_pos > scores.0) || scores.0 == 0 {
            let is_new_high_sore = if signed_pos < high_scores.0 && signed_pos < 0 {
                high_scores.0 = signed_pos;
                true
            } else if signed_pos > high_scores.1 && signed_pos > 0 {
                high_scores.1 = signed_pos;
                true
            } else {
                false
            };
            if is_new_high_sore && (high_scores.0 % 5 == 0 || high_scores.1 % 5 == 0) {
                ev_save.send(SaveEvent);
            }
            signed_pos
        } else {
            scores.0
        }
    }
}

fn init_game(
    mut commands: Commands,
    objects: Res<Objects>,
    mut chunk: ResMut<Chunk>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_chunk_change: EventWriter<ChunkChangeEvent>,
    mut ev_new_game: EventReader<NewGameEvent>,
    mut ev_trigger_loop_anim: EventWriter<TriggerLoopAnimEvent>,
) {
    for _ in ev_new_game.iter().last() {
        // light
        commands.insert_resource(AmbientLight {
            brightness: 1.,
            ..default()
        });
        commands.spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: false,
                radius: 100.,
                range: 200.,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 10.0, 0.0),
            ..default()
        });

        // camera
        templates::make_main_camera(&mut commands);

        // player
        let mut player = {
            let ent =
                templates::make_main_player(&mut commands, &objects, &mut ev_trigger_loop_anim);
            commands.entity(ent)
        };
        player
            .insert(Player)
            .insert(FollowTarget("player".to_owned()))
            .insert(Transform::from_xyz(FBLOCK_SIZE / 2., 10., FBLOCK_SIZE / 2.));
        //templates::make_player_lamp(&mut commands);

        #[cfg(build = "debug")]
        {
            // Test minions
            for _ in 0..5 {
                let ent =
                    templates::make_main_player(&mut commands, &objects, &mut ev_trigger_loop_anim);
                let mut ent = commands.entity(ent);
                ent.insert(Transform::from_xyz(5., 5., 5.)).insert(Minion);
            }
            // Anchor block
            commands.spawn_bundle(PbrBundle {
                mesh: objects.0[&"cube".to_string()].clone_weak().typed(),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 6., 0.0),
                ..default()
            });
        }

        // floor colliders
        commands
            .spawn()
            .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)))
            .insert(Collider::cuboid(
                FBLOCK_SIZE * 10.0,
                0.5,
                FBLOCK_SIZE * 10.0,
            ));
        // far collider
        commands
            .spawn()
            .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 0.0, -0.5)))
            .insert(Collider::cuboid(FBLOCK_SIZE * 10.0, 50., 0.5));
        // near collider
        commands
            .spawn()
            .insert_bundle(TransformBundle::from(Transform::from_xyz(
                0.0,
                0.0,
                FBLOCK_SIZE - 0.5,
            )))
            .insert(Collider::cuboid(FBLOCK_SIZE * 10.0, 50., 0.5));
        // ceiling collider
        commands
            .spawn()
            .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 50., 0.0)))
            .insert(Collider::cuboid(
                FBLOCK_SIZE * 10.0,
                0.5,
                FBLOCK_SIZE * 10.0,
            ));

        *chunk = CHUNK_MID.clone();
        ev_chunk_change.send(ChunkChangeEvent {
            oldchunk: CHUNK_MID.clone(),
            newchunk: CHUNK_MID.clone(),
        });
    }
}

fn paused_check(
    mut ev_pause: EventReader<PauseEvent>,
    mut rapier_config: ResMut<bevy_rapier3d::plugin::RapierConfiguration>,
    mut paused: ResMut<Paused>,
    mut stopwatch: ResMut<GameTime>,
    mut show_tutorial: ResMut<ShowTutorial>,
) {
    for _ in ev_pause.iter().last() {
        rapier_config.physics_pipeline_active = paused.0;
        paused.0 = !paused.0;
        if paused.0 {
            stopwatch.0.pause();
        } else {
            show_tutorial.0 = None;
            stopwatch.0.unpause();
        }
    }
}

fn controls(
    keyboard_input: Res<Input<KeyCode>>,
    mut ev_pause: EventWriter<PauseEvent>,
    menu: Res<Menu>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) && *menu == Menu::Game {
        ev_pause.send(PauseEvent);
    }
}
