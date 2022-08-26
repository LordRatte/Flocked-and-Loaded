use crate::asset_plugin::{Objects, MUSIC_TRACKS};

use bevy::app::Plugin;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use rand::{self, Rng};

pub struct SoundPlugin;

pub struct MusicVolume(pub f32);
impl Default for MusicVolume {
    fn default() -> Self {
        MusicVolume(100.)
    }
}

pub struct EffectsVolume(pub f32);
impl Default for EffectsVolume {
    fn default() -> Self {
        EffectsVolume(100.)
    }
}

#[derive(Default)]
pub struct PlayMusic(pub bool);

struct BackgroundMusicHandle {
    instance: Option<Handle<AudioInstance>>,
}

pub enum Effect {
    BombZap,
    SheepBaa,
    LauncherBoom,
}

pub struct SoundEffectEvent {
    pub effect: Effect,
}

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BackgroundMusicHandle { instance: None })
            .init_resource::<PlayMusic>()
            .init_resource::<MusicVolume>()
            .init_resource::<EffectsVolume>()
            .add_event::<SoundEffectEvent>()
            .add_system(monitor_track)
            .add_system(play_effect)
            .add_system(manage_music);
    }
}

fn play_effect(
    audio: Res<Audio>,
    objects: Res<Objects>,
    mut ev_effect: EventReader<SoundEffectEvent>,
    effects_volume: Res<EffectsVolume>,
) {
    for SoundEffectEvent { effect } in ev_effect.iter() {
        match effect {
            Effect::BombZap => audio.play(objects.0[&"bomb_zap".to_string()].clone_weak().typed()),
            Effect::SheepBaa => {
                audio.play(objects.0[&"sheep_baa".to_string()].clone_weak().typed())
            }
            Effect::LauncherBoom => {
                audio.play(objects.0[&"launcher_boom".to_string()].clone_weak().typed())
            }
        }
        .with_volume((effects_volume.0 / 101.) as f64);
    }
}

fn track_no() -> u8 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1..=MUSIC_TRACKS)
}

fn manage_music(
    music_handle: Res<BackgroundMusicHandle>,
    music_volume: Res<MusicVolume>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    play_music: Res<PlayMusic>,
) {
    if let Some(instance) = music_handle
        .instance
        .as_ref()
        .and_then(|instance| audio_instances.get_mut(instance))
    {
        instance.set_volume(music_volume.0 as f64 / 200., AudioTween::default());
        if play_music.0 {
            instance.resume(AudioTween::default());
        } else {
            instance.pause(AudioTween::default());
        }
    }
}

fn monitor_track(
    objects: Res<Objects>,
    audio: Res<Audio>,
    mut music_handle: ResMut<BackgroundMusicHandle>,
) {
    match &music_handle.instance {
        None => {
            music_handle.instance = Some(
                audio
                    .play(
                        objects.0[&format!("track{}", track_no())]
                            .clone_weak()
                            .typed(),
                    )
                    .with_volume(0.)
                    .handle(),
            )
        }
        Some(instance) => match audio.state(&instance) {
            PlaybackState::Stopped => {
                music_handle.instance = Some(
                    audio
                        .play(
                            objects.0[&format!("track{}", track_no())]
                                .clone_weak()
                                .typed(),
                        )
                        .with_volume(0.)
                        .handle(),
                )
            }
            _ => (),
        },
    }
}
