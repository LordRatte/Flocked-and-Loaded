use bevy::app::Plugin;
use bevy::prelude::*;
use std::collections::HashMap;

pub struct AssetPlugin;

#[derive(Default)]
pub struct Animations(pub HashMap<String, Handle<AnimationClip>>);

#[derive(Default)]
pub struct Objects(pub HashMap<String, HandleUntyped>);

pub struct TriggerLoopAnimEvent(pub Entity, pub String);

pub const MUSIC_TRACKS: u8 = 12;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Objects(HashMap::new()))
            .add_event::<TriggerLoopAnimEvent>()
            .add_startup_system(load_assets.label("assets"))
            .add_system(loop_anim_handler);
    }
}

fn load_assets(
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut objects: ResMut<Objects>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //Scenes
    let tree: Handle<Scene> = asset_server.load("tree.glb#Scene0");
    let cage: Handle<Scene> = asset_server.load("cage.glb#Scene0");
    let sheep: Handle<Scene> = asset_server.load("sheep.glb#Scene0");
    let launcher: Handle<Scene> = asset_server.load("launcher.glb#Scene0");
    let bomb: Handle<Scene> = asset_server.load("bomb.glb#Scene0");
    objects.0.insert("tree".to_string(), tree.clone_untyped());
    objects.0.insert("cage".to_string(), cage.clone_untyped());
    objects.0.insert("sheep".to_string(), sheep.clone_untyped());
    objects
        .0
        .insert("launcher".to_string(), launcher.clone_untyped());
    objects.0.insert("bomb".to_string(), bomb.clone_untyped());

    // Meshes
    let cube: Handle<Mesh> = meshes.add(Mesh::from(shape::Cube { size: 1. }));
    let sphere: Handle<Mesh> = meshes.add(Mesh::from(shape::Icosphere {
        radius: 2.,
        subdivisions: 1,
    }));
    objects.0.insert("cube".to_string(), cube.clone_untyped());
    objects
        .0
        .insert("sphere".to_string(), sphere.clone_untyped());

    // Animations
    let sheep_move: Handle<AnimationClip> = asset_server.load("sheep.glb#Animation0");
    objects
        .0
        .insert("sheep_move".to_string(), sheep_move.clone_untyped());

    // Materials
    let invisible: Handle<StandardMaterial> = materials.add(Color::rgba(0., 0., 0., 0.).into());
    objects
        .0
        .insert("invisible".to_string(), invisible.clone_untyped());

    //Audio
    for i in 1..=MUSIC_TRACKS {
        let a: Handle<StandardMaterial> =
            asset_server.load(format!("music/track{}.mp3", i).as_str());
        objects.0.insert(format!("track{}", i), a.clone_untyped());
    }
    let bomb_zap: Handle<StandardMaterial> = asset_server.load("bomb_zap.mp3");
    let sheep_baa: Handle<StandardMaterial> = asset_server.load("sheep_baa.mp3");
    let launcher_boom: Handle<StandardMaterial> = asset_server.load("launcher_boom.mp3");
    objects
        .0
        .insert("bomb_zap".to_string(), bomb_zap.clone_untyped());
    objects
        .0
        .insert("sheep_baa".to_string(), sheep_baa.clone_untyped());
    objects
        .0
        .insert("launcher_boom".to_string(), launcher_boom.clone_untyped());
}

pub fn play_decendent_animation(
    entity: Entity,
    children_query: &Query<&Children>,
    animation_player: &mut Query<&mut AnimationPlayer>,
    animation: &Handle<AnimationClip>,
    repeat: bool,
) {
    if let Ok(mut player) = animation_player.get_mut(entity) {
        if repeat {
            player.play(animation.clone()).repeat();
        } else {
            player.play(animation.clone());
        }
    }
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            play_decendent_animation(*child, children_query, animation_player, animation, repeat);
        }
    }
}

fn loop_anim_handler(
    mut animation_player: Query<&mut AnimationPlayer>,
    objects: ResMut<Objects>,
    children_query: Query<&Children>,
    mut ev_loop_anim_trigger: EventReader<TriggerLoopAnimEvent>,
) {
    for TriggerLoopAnimEvent(ent, animation) in ev_loop_anim_trigger.iter() {
        play_decendent_animation(
            *ent,
            &children_query,
            &mut animation_player,
            &objects.0[animation].clone_weak().typed(),
            true,
        );
    }
}
