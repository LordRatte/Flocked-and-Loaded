use crate::asset_plugin::{Objects, TriggerLoopAnimEvent};
use crate::game_plugin::GameTime;
use crate::player_manager_plugin::{Minion, Player};
use crate::share::{DynamicPos, Indestructible, Terrain};
use crate::sound_plugin::{Effect, SoundEffectEvent};
use crate::templates;
use bevy::app::Plugin;
use bevy::prelude::*;
use bevy_rapier3d::prelude::ExternalImpulse;
use std::time::Duration;

pub struct ItemPlugin;

#[derive(Component)]
pub struct Item(pub ItemType);

pub struct EquipTakeEvent {
    pub pos: Vec3,
    pub reach: f32,
}
pub struct EquipGiveEvent {
    pub item: ItemType,
}

#[derive(Debug, Clone, Copy)]
pub enum ItemType {
    Cage,
    Launcher,
    Bomb,
}

#[derive(Component, Debug)]
struct Cooldown(Duration);

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EquipTakeEvent>()
            .add_event::<EquipGiveEvent>()
            .add_system(laucher_ai.label("laucher_ai"))
            .add_system(bomb_ai.after("laucher_ai"))
            .add_system(equip_manager);
    }
}

fn equip_manager(
    mut ev_equip_take: EventReader<EquipTakeEvent>,
    items: Query<(Entity, &Transform, &Item)>,
    mut commands: Commands,
    objects: Res<Objects>,
    mut ev_trigger_loop_anim: EventWriter<TriggerLoopAnimEvent>,
    mut ev_effect: EventWriter<SoundEffectEvent>,
) {
    for EquipTakeEvent { pos, reach } in ev_equip_take.iter().last() {
        for (ent, transform, Item(kind)) in items.iter() {
            if transform.translation.distance(*pos) < *reach {
                match *kind {
                    ItemType::Cage => {
                        cage_despawn_helper(
                            &mut commands,
                            &mut ev_trigger_loop_anim,
                            &mut ev_effect,
                            &objects,
                            ent,
                            *transform,
                        );
                    }
                    _ => (),
                }
            }
        }
    }
}

fn cage_despawn_helper(
    commands: &mut Commands,
    ev_trigger_loop_anim: &mut EventWriter<TriggerLoopAnimEvent>,
    ev_effect: &mut EventWriter<SoundEffectEvent>,
    objects: &Res<Objects>,
    ent: Entity,
    transform: Transform,
) {
    ev_effect.send(SoundEffectEvent {
        effect: Effect::SheepBaa,
    });
    commands.entity(ent).despawn_recursive();
    let mut entity = {
        let id = templates::make_main_player(commands, objects, ev_trigger_loop_anim);
        commands.entity(id)
    };
    entity.insert(transform).insert(Minion);
}

fn laucher_ai(
    items: Query<(Entity, &Item, &Transform, Option<&Cooldown>)>,
    targets: Query<&Transform, Or<(&Player, &Minion)>>,
    mut commands: Commands,
    objects: Res<Objects>,
    time: Res<GameTime>,
    mut ev_effect: EventWriter<SoundEffectEvent>,
) {
    for (ent, _, trans, cooldown) in items.iter().filter(|(_, it, _, _)| match it.0 {
        ItemType::Launcher => true,
        _ => false,
    }) {
        let delta: Duration = cooldown
            .map(|v| time.0.elapsed() - v.0)
            .unwrap_or_else(|| time.0.elapsed());
        if delta >= Duration::from_secs_f32(10.) {
            for target_pos in targets
                .iter()
                .filter(|pos| pos.translation.distance(trans.translation) <= 6.)
            {
                ev_effect.send(SoundEffectEvent {
                    effect: Effect::LauncherBoom,
                });
                commands.entity(ent).insert(Cooldown(time.0.elapsed()));
                let bomb = templates::make_item(&mut commands, ItemType::Bomb, &objects);
                commands
                    .entity(bomb)
                    .insert(Transform::from_xyz(
                        trans.translation.x,
                        trans.translation.y + 1.,
                        trans.translation.z,
                    ))
                    .insert(DynamicPos)
                    .insert(ExternalImpulse {
                        impulse: (target_pos.translation - trans.translation).normalize_or_zero()
                            * 7.,
                        ..default()
                    })
                    .insert(Cooldown(time.0.elapsed()));
                break;
            }
        }
    }
}

fn bomb_ai(
    bombs: Query<(Entity, &Item, &Transform, &Cooldown, &Children)>,
    destruct: Query<
        (Entity, &Transform, Option<&Item>),
        (Or<(&Terrain, &Player, &Minion)>, Without<Indestructible>),
    >,
    time: Res<GameTime>,
    objects: Res<Objects>,
    mut commands: Commands,
    mut light: Query<&mut PointLight>,
    mut ev_effect: EventWriter<SoundEffectEvent>,
    mut ev_trigger_loop_anim: EventWriter<TriggerLoopAnimEvent>,
) {
    for (ent, _, trans, cooldown, children) in bombs.iter().filter(|(_, it, _, _, _)| match it.0 {
        ItemType::Bomb => true,
        _ => false,
    }) {
        let delta = time.0.elapsed() - cooldown.0;
        if delta >= Duration::from_secs_f32(7.) {
            for child in children.iter() {
                if let Ok(mut light) = light.get_mut(*child) {
                    light.color = Color::BLUE;
                }
            }
        }
        if delta >= Duration::from_secs_f32(10.) {
            ev_effect.send(SoundEffectEvent {
                effect: Effect::BombZap,
            });
            for (dent, dtrans, item) in destruct.iter() {
                if dtrans.translation.distance(trans.translation) <= 3. {
                    if item
                        .map(|item| match item.0 {
                            ItemType::Cage => true,
                            _ => false,
                        })
                        .unwrap_or(false)
                    {
                        cage_despawn_helper(
                            &mut commands,
                            &mut ev_trigger_loop_anim,
                            &mut ev_effect,
                            &objects,
                            dent,
                            *dtrans,
                        );
                    } else {
                        commands.entity(dent).despawn_recursive();
                    }
                }
            }
            commands.entity(ent).despawn_recursive();
        }
    }
}
