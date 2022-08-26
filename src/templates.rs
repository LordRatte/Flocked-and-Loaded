use crate::asset_plugin::{Objects, TriggerLoopAnimEvent};
use crate::chunk_manager_plugin::Chunk;
use crate::follow_plugin::*;
use crate::item_plugin::{Item, ItemType};
use crate::player_manager_plugin::Inventory;
use crate::share::*;
use bevy::prelude::*;
use bevy_mod_picking::*;
use bevy_rapier3d::prelude::*;

pub fn make_player_lamp(commands: &mut Commands) -> Entity {
    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 15000.0,
                shadows_enabled: true,
                color: Color::YELLOW,
                radius: 50.,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..default()
        })
        .insert(FollowComp {
            offset: (0., Some(1.), 0.),
            label: "player".to_string(),
        })
        .insert(DynamicPos)
        .id()
}

pub fn make_main_camera(commands: &mut Commands) -> Entity {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 15.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(Camera)
        .insert(WatchComp)
        .insert(FollowComp {
            offset: (0., None, 7.),
            label: "player".to_owned(),
        })
        .insert(DynamicPos)
        .insert_bundle(PickingCameraBundle::default())
        .id()
}

pub fn make_main_player(
    commands: &mut Commands,
    objects: &Objects,
    ev_trigger_loop_anim: &mut EventWriter<TriggerLoopAnimEvent>,
) -> Entity {
    let ent = commands
        .spawn()
        .insert(OldLoc(6., 6.))
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert(ExternalForce {
            ..Default::default()
        })
        .insert(Damping {
            linear_damping: 1.,
            angular_damping: 1.0,
        })
        .insert(Velocity {
            linvel: Vec3::new(0.01, 0., 0.),
            ..default()
        })
        .insert(DynamicPos)
        .insert(Inventory { hand: None })
        .insert_bundle(PickableBundle::default())
        .insert_bundle(PickableBundle::default())
        .insert_bundle(PbrBundle {
            mesh: objects.0[&"sphere".to_string()].clone_weak().typed(),
            material: objects.0[&"invisible".to_string()].clone_weak().typed(),
            ..default()
        })
        .insert_bundle(SceneBundle {
            scene: objects.0[&"sheep".to_string()].clone_weak().typed(),
            transform: Transform::from_xyz(6., 2., 6.),
            ..default()
        })
        .id();
    ev_trigger_loop_anim.send(TriggerLoopAnimEvent(ent, "sheep_move".to_string()));
    ent
}

pub fn entities_for_tile(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    tile_settings: &TileSettings,
    chunk: &Chunk,
    pos: (Option<f32>, Option<f32>, Option<f32>),
    objects: &Res<Objects>,
) {
    let TileSettings {
        kind,
        copse,
        height,
        item,
    } = tile_settings;
    let ent = commands
        .spawn_bundle(PbrBundle {
            mesh: objects.0[&"cube".to_string()].clone_weak().typed(),
            material: materials.add(type_to_colour(&kind).into()),
            transform: Transform::from_xyz(
                pos.0.unwrap_or(0.),
                pos.1.unwrap_or(*height as f32),
                pos.2.unwrap_or(0.),
            ),
            ..default()
        })
        .insert(Terrain(chunk.0, chunk.1))
        .insert(DynamicPos)
        .id();

    if *height == 0 {
        commands.entity(ent).insert(Indestructible);
    }

    if *height > 0 {
        commands.entity(ent).insert(Collider::cuboid(0.5, 0.5, 0.5));
        commands
            .spawn_bundle(PbrBundle {
                mesh: objects.0[&"cube".to_string()].clone_weak().typed(),
                material: materials.add(type_to_colour(&TileType::Base).into()),
                transform: Transform::from_xyz(
                    pos.0.unwrap_or(0.),
                    pos.1.unwrap_or(0.),
                    pos.2.unwrap_or(0.),
                ),
                ..default()
            })
            .insert(DynamicPos)
            .insert(Indestructible)
            .insert(Terrain(chunk.0, chunk.1));
    }

    if *copse {
        commands
            .spawn_bundle(SceneBundle {
                scene: objects.0[&"tree".to_string()].clone_weak().typed(),
                transform: Transform::from_xyz(
                    pos.0.unwrap_or(0.),
                    *height as f32,
                    pos.2.unwrap_or(0.),
                ),
                ..default()
            })
            .insert(Collider::round_cylinder(4., 0.2, 0.2))
            .insert(Terrain(chunk.0, chunk.1))
            .insert(DynamicPos);
    }
    match *item {
        Some(kind) => {
            let new_item = make_item(commands, kind, objects);
            commands
                .entity(new_item)
                .insert(Transform::from_xyz(
                    pos.0.unwrap_or(0.)
                        + match kind {
                            ItemType::Launcher => 0.,
                            _ => 0.,
                        },
                    *height as f32
                        + match kind {
                            ItemType::Launcher => 0.5,
                            _ => 1.,
                        },
                    pos.2.unwrap_or(0.)
                        + match kind {
                            ItemType::Launcher => 0.,
                            _ => 0.,
                        },
                ))
                .insert(Terrain(chunk.0, chunk.1))
                .insert(DynamicPos);
        }
        _ => {}
    }
}

pub fn make_item(commands: &mut Commands, kind: ItemType, objects: &Objects) -> Entity {
    match kind {
        ItemType::Axe => commands
            .spawn_bundle(SceneBundle {
                scene: objects.0[&"axe".to_string()].clone_weak().typed(),
                ..default()
            })
            .insert(Item(ItemType::Axe))
            .id(),
        ItemType::Cage => commands
            .spawn_bundle(SceneBundle {
                scene: objects.0[&"cage".to_string()].clone_weak().typed(),
                ..default()
            })
            .insert(Item(ItemType::Cage))
            .insert(Collider::cuboid(0.5, 1., 0.5))
            .id(),
        ItemType::Launcher => commands
            .spawn_bundle(SceneBundle {
                scene: objects.0[&"launcher".to_string()].clone_weak().typed(),
                ..default()
            })
            .insert(Item(ItemType::Launcher))
            .id(),
        ItemType::Bomb => {
            let light = commands
                .spawn_bundle(PointLightBundle {
                    point_light: PointLight {
                        intensity: 1500.0,
                        shadows_enabled: false,
                        color: Color::RED,
                        radius: 50.,
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.6, 0.0),
                    ..default()
                })
                .id();
            commands
                .spawn_bundle(SceneBundle {
                    scene: objects.0[&"bomb".to_string()].clone_weak().typed(),
                    ..default()
                })
                .insert(Item(ItemType::Bomb))
                .insert(RigidBody::Dynamic)
                .insert(Collider::ball(0.5))
                .insert(Restitution::coefficient(0.9))
                .insert(ExternalForce {
                    ..Default::default()
                })
                .insert(Damping {
                    linear_damping: 0.,
                    angular_damping: 1.0,
                })
                .insert(Velocity { ..default() })
                .insert_children(0, &[light])
                .id()
        }
    }
}
