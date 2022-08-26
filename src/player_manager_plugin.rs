
use crate::chunk_manager_plugin::{Chunk, ChunkChangeEvent, FBLOCK_SIZE, RENDER_DISTANCE};
use crate::follow_plugin::FollowTarget;
use crate::follow_plugin::FollowTargetMoveEvent;
use crate::item_plugin::{EquipGiveEvent, EquipTakeEvent, ItemType};
use crate::share::OldLoc;


use bevy::app::Plugin;
use bevy::prelude::*;
use bevy_mod_picking::events::PickingEvent;
use bevy_rapier3d::prelude::ExternalForce;
use bevy_rapier3d::prelude::Velocity;

pub struct PlayerManagerPlugin;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Minion;

#[derive(Component)]
pub struct Inventory {
    pub hand: Option<ItemType>,
}

impl Plugin for PlayerManagerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_movement)
            .add_system(player_location_manager)
            .add_system(equip_player)
            .add_system(minion_ai)
            .add_system(change_controlled)
            .add_system(on_player_death)
            .add_system(minion_location_manager);
    }
}

fn minion_location_manager(
    mut minions: Query<(Entity, &mut Transform), (&Minion, Without<Player>)>,
    mut commands: Commands,
    player: Query<&Transform, &Player>,
) {
    for (_, mut transform) in minions.iter_mut() {
        if transform.translation.y < 0.5 {
            transform.translation.y = 1.0;
        }
    }
    for orig in player.iter() {
        for (ent, trans) in minions.iter() {
            if orig.translation.distance(trans.translation) > FBLOCK_SIZE * (RENDER_DISTANCE as f32)
            {
                commands.entity(ent).despawn_recursive();
            }
        }
    }
}

fn player_location_manager(
    mut player_positions: Query<(&mut Transform, &mut OldLoc, Option<&FollowTarget>), &Player>,
    mut chunk: ResMut<Chunk>,
    mut ev_chunk_change: EventWriter<ChunkChangeEvent>,
    mut ev_follow_target_move: EventWriter<FollowTargetMoveEvent>,
) {
    for (mut transform, mut oldloc, follow_target) in player_positions.iter_mut() {
        if transform.translation.y < 0.5 {
            transform.translation.y = 1.0;
        }

        // Hash the position moved
        if transform.translation.x != oldloc.0 || transform.translation.z != oldloc.1 {
            // Let any followers know that we have moved
            if let Some(FollowTarget(label)) = follow_target {
                ev_follow_target_move.send(FollowTargetMoveEvent {
                    label: label.to_string(),
                    target_pos: transform.translation,
                });
            }

            // Test to see if a chunk has changed
            let (cx, cz) = (transform.translation.x, transform.translation.z);
            let (ox, oz) = (oldloc.0, oldloc.1);
            let (dx, dz) = (
                (((cx / FBLOCK_SIZE).floor()) - ((ox / FBLOCK_SIZE).floor())) as isize,
                (((cz / FBLOCK_SIZE).floor()) - ((oz / FBLOCK_SIZE).floor())) as isize,
            );

            if dx != 0 || dz != 0 {
                let newchunk = Chunk(
                    chunk.0.saturating_add_signed(dx),
                    chunk.1.saturating_add_signed(dz),
                );
                ev_chunk_change.send(ChunkChangeEvent {
                    oldchunk: chunk.clone(),
                    newchunk: newchunk.clone(),
                });
                *chunk = newchunk;
            }
            *oldloc = OldLoc(transform.translation.x, transform.translation.z);
        }
    }
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut head_positions: Query<(&mut ExternalForce, &mut Velocity, &Transform), &Player>,
    mut ev_equip: EventWriter<EquipTakeEvent>,
    mut ev_pick: EventWriter<PickingEvent>,
    rapier_config: Res<bevy_rapier3d::plugin::RapierConfiguration>,
    minions: Query<Entity, &Minion>,
) {
    if rapier_config.physics_pipeline_active {
        for (mut ef, mut vel, transform) in head_positions.iter_mut() {
            if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
                ef.force.x = -5.0;
            } else if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
                ef.force.x = 5.0;
            } else {
                ef.force.x = 0.0;
            }

            if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
                ef.force.z = 5.0;
            } else if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
                ef.force.z = -5.0;
            } else {
                ef.force.z = 0.0;
            }

            #[cfg(build = "debug")]
            {
                if keyboard_input.pressed(KeyCode::Space) {
                    vel.linvel.y = 5.0;
                }
            }

            if keyboard_input.just_pressed(KeyCode::E) {
                ev_equip.send(EquipTakeEvent {
                    pos: transform.translation,
                    reach: 3.,
                });
            }

            if keyboard_input.just_pressed(KeyCode::Q) {
                for ent in minions.iter().next() {
                    ev_pick.send(PickingEvent::Clicked(ent));
                }
            }
        }
    }
}

fn equip_player(mut ev_equip: EventReader<EquipGiveEvent>) {
    for EquipGiveEvent { item } in ev_equip.iter().last() {
        match item {
            _ => (),
        }
    }
}

fn minion_ai(
    player_positions: Query<&Transform, &Player>,
    mut minions: Query<(&Transform, &mut ExternalForce), &Minion>,
) {
    for player_pos in player_positions.iter().last() {
        for (minion_pos, mut minion_force) in minions.iter_mut() {
            if minion_pos.translation.distance(player_pos.translation) > 3. {
                let direction = player_pos.translation - minion_pos.translation;
                if direction.length() > 0.1 {
                    minion_force.force = direction.normalize() * 2.5;
                }
            }
        }
    }
}

fn on_player_death(
    player: Query<&OldLoc, &Player>,
    minions: Query<Entity, &Minion>,
    mut ev_selection: EventWriter<PickingEvent>,
    mut last_loc: Local<Option<OldLoc>>,
    mut commands: Commands,
) {
    if player.is_empty() && last_loc.is_some() {
        if let Some(ent) = minions.iter().last() {
            commands
                .entity(ent)
                .insert(last_loc.expect("Last Loc should be set from the start"));
            ev_selection.send(PickingEvent::Clicked(ent));
        } else {
            todo!("Game over");
        }
    } else {
        for loc in player.iter() {
            *last_loc = Some(*loc);
        }
    }
}

fn change_controlled(
    mut ev_selection: EventReader<PickingEvent>,
    mut player: Query<(Entity, &Transform), &Player>,
    mut commands: Commands,
) {
    for ev in ev_selection
        .iter()
        .filter(|v| match v {
            PickingEvent::Clicked(_) => true,
            _ => false,
        })
        .last()
    {
        match ev {
            PickingEvent::Clicked(ent) => {
                let mut oldloc = None;
                for (ent, trans) in player.iter_mut() {
                    commands
                        .entity(ent)
                        .remove::<Player>()
                        .remove::<FollowTarget>()
                        .insert(Minion);
                    oldloc = Some(OldLoc(trans.translation.x, trans.translation.z));
                }

                commands
                    .entity(*ent)
                    .remove::<Minion>()
                    .insert(Player)
                    .insert(FollowTarget("player".to_string()));
                if let Some(oldloc) = oldloc {
                    commands.entity(*ent).insert(oldloc);
                }
            }
            _ => (),
        }
    }
}
