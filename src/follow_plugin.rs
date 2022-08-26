use bevy::app::Plugin;
use bevy::prelude::*;

use itertools::Itertools;

pub struct FollowPlugin;

#[derive(Component, Default)]
pub struct FollowComp {
    pub offset: (f32, Option<f32>, f32),
    pub label: String,
}

#[derive(Component)]
pub struct WatchComp;

#[derive(Component)]
pub struct FollowTarget(pub String);

pub struct FollowTargetMoveEvent {
    pub label: String,
    pub target_pos: Vec3,
}

impl Plugin for FollowPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FollowTargetMoveEvent>()
            .add_system(follow_comps);
    }
}

fn follow_comps(
    mut head_positions: Query<
        (&mut Transform, &FollowComp, Option<&WatchComp>),
        Without<FollowTarget>,
    >,
    mut ev_follow_target_move: EventReader<FollowTargetMoveEvent>,
) {
    for FollowTargetMoveEvent {
        label: lbl,
        target_pos: tgt,
    } in ev_follow_target_move
        .iter()
        .rev()
        .unique_by(|ftme| ftme.label.as_str())
    {
        for (
            mut transform,
            FollowComp {
                offset: (offsetx, offsety_op, offsetz),
                label,
            },
            watch_comp,
        ) in head_positions.iter_mut()
        {
            if lbl == label {
                let y = match offsety_op {
                    Some(offsety) => tgt.y + offsety,
                    None => transform.translation.y,
                };
                *transform = if watch_comp.is_some() {
                    Transform::from_xyz(tgt.x + offsetx, y, tgt.z + offsetz)
                        .looking_at(*tgt, Vec3::Y)
                } else {
                    Transform::from_xyz(tgt.x + offsetx, y, tgt.z + offsetz)
                }
            }
        }
    }
}
