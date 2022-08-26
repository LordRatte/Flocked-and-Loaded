use crate::asset_plugin::Objects;
use crate::item_plugin::ItemType;
use crate::share::*;
use crate::templates::entities_for_tile;
use bevy::app::Plugin;
use bevy::math::DVec2;
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use itertools::iproduct;
use noise::{NoiseFn, Perlin, Seedable};

use rand::Rng;
use rand::SeedableRng;
use std::collections::{HashMap, HashSet};

pub const CHUNK_MID: Chunk = Chunk((u32::MAX / 2) as usize, (u32::MAX / 2) as usize);

pub const MAX_ELEV: usize = 5;

pub const BLOCK_SIZE: usize = 20;
pub const FBLOCK_SIZE: f32 = BLOCK_SIZE as f32;

pub const RENDER_DISTANCE: isize = 2;

#[derive(Inspectable, Default, Hash, Eq, PartialEq, Clone, Debug)]
pub struct Chunk(pub usize, pub usize);

pub struct ChunkChangeEvent {
    pub oldchunk: Chunk,
    pub newchunk: Chunk,
}
pub struct SpawnBlockEvent {
    pub chunk: Chunk,
    pub chunk_offset: (isize, isize),
}

#[derive(Default)]
struct LoadedChunks(HashSet<Chunk>);

pub struct Seed(u32);

struct WorldGrid(HashMap<Chunk, [[TileSettings; BLOCK_SIZE]; BLOCK_SIZE]>);

impl FromWorld for WorldGrid {
    fn from_world(world: &mut World) -> Self {
        let mut chunks = HashMap::new();
        let seed = world
            .get_resource::<Seed>()
            .expect("Could not generate seed");
        let origin = propogate_block(CHUNK_MID.0, CHUNK_MID.1, Some(seed.0));
        chunks.insert(CHUNK_MID, origin);

        WorldGrid(chunks)
    }
}

pub struct ChunkManagerPlugin;

impl Plugin for ChunkManagerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource::<Seed>(Seed({
            let mut rng = rand::thread_rng();
            rng.gen_range(0..u32::MAX)
        }))
        .init_resource::<WorldGrid>()
        .add_event::<ChunkChangeEvent>()
        .add_event::<SpawnBlockEvent>()
        .init_resource::<Chunk>()
        .init_resource::<LoadedChunks>()
        .add_system(handle_chunk_change)
        .add_system(handle_spawn_block)
        .add_system(cull_far_entities);
    }
}

fn handle_chunk_change(
    mut ev_chunk_change: EventReader<ChunkChangeEvent>,
    mut ev_spawn_block: EventWriter<SpawnBlockEvent>,
    mut transforms: Query<(&mut Transform, Option<&mut OldLoc>, Option<&DynamicPos>)>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    if let Some(ChunkChangeEvent { oldchunk, newchunk }) = ev_chunk_change.iter().last() {
        // Get shift ammount in chunks
        let (shx, shz) = {
            let (ishx, ishz) = (
                if oldchunk.0 > newchunk.0 { 1 } else { -1 }
                    * ((oldchunk.0).abs_diff(newchunk.0) as isize),
                if oldchunk.1 > newchunk.1 { 1 } else { -1 }
                    * ((oldchunk.1).abs_diff(newchunk.1) as isize),
            );

            ((ishx as f32) * FBLOCK_SIZE, (ishz as f32) * FBLOCK_SIZE)
        };

        // Shift the current entities by the floating origin
        // and while we're at it, update the terrain tag (which stores the chunk
        // of each item).
        transforms.par_for_each_mut(1000, |(mut trans, oldloc, perma_pos)| {
            if let Some(mut oldloc) = oldloc {
                oldloc.0 += shx;
                oldloc.1 += shz;
            }
            if perma_pos.is_some() {
                trans.translation.x += shx;
                trans.translation.z += shz;
            }
        });

        // Make sure that surrounding chunks are generated
        // and spawn them
        for (cx, cz) in iproduct!(-(RENDER_DISTANCE - 1)..=(RENDER_DISTANCE - 1), -0..=0) {
            let chunk = Chunk(
                newchunk.0.saturating_add_signed(cx),
                newchunk.1.saturating_add_signed(cz),
            );
            if loaded_chunks.0.insert(chunk.clone()) {
                ev_spawn_block.send(SpawnBlockEvent {
                    chunk,
                    chunk_offset: (cx, cz),
                });
            }
        }
    }
}

fn handle_spawn_block(
    mut ev_spawn_block: EventReader<SpawnBlockEvent>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    objects: Res<Objects>,
    mut world_grid: ResMut<WorldGrid>,
    seed: Res<Seed>,
) {
    for SpawnBlockEvent {
        chunk,
        chunk_offset: (cx, cz),
    } in ev_spawn_block.iter()
    {
        let block = world_grid
            .0
            .entry(chunk.clone())
            .or_insert_with(|| propogate_block(chunk.0, chunk.1, Some(seed.0)));

        for (x, col) in block.iter().enumerate() {
            for (z, t) in col.iter().enumerate() {
                let (x, z) = (x as f32, z as f32);
                let (x, z) = (
                    x + (FBLOCK_SIZE) * (*cx as f32),
                    z + (FBLOCK_SIZE) * (*cz as f32),
                );
                // Crate the entities that should go here
                entities_for_tile(
                    &mut commands,
                    &mut materials,
                    t,
                    chunk,
                    (Some(x), None, Some(z)),
                    &objects,
                );
            }
        }
    }
}

fn cull_far_entities(
    mut ev_chunk_change: EventReader<ChunkChangeEvent>,
    terrain: Query<(Entity, &Terrain)>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    mut commands: Commands,
) {
    if let Some(ChunkChangeEvent {
        oldchunk: _,
        newchunk,
    }) = ev_chunk_change.iter().last()
    {
        for (en, Terrain(x, z)) in terrain.iter() {
            if DVec2::new(*x as f64, *z as f64)
                .distance(DVec2::new(newchunk.0 as f64, newchunk.1 as f64))
                > (RENDER_DISTANCE as f64)
            {
                loaded_chunks.0.remove(&Chunk(*x, *z));
                commands.entity(en).despawn_recursive();
            }
        }
    }
}

fn propogate_block(
    chunk_x: usize,
    chunk_z: usize,
    seed: Option<u32>,
) -> [[TileSettings; BLOCK_SIZE]; BLOCK_SIZE] {
    let seed = seed.unwrap_or(0);
    let perlin_elev = Perlin::new().set_seed(seed);
    let perlin_trees = Perlin::new().set_seed(seed.wrapping_add(100));
    let perlin_items = Perlin::new().set_seed(seed.wrapping_add(200));

    let (chunk_x, chunk_z) = (chunk_x as f64, chunk_z as f64);

    let mut block = [[TileSettings { ..default() }; BLOCK_SIZE]; BLOCK_SIZE];
    let mut has_cage = false;
    let mut launcher_count = 2;

    // Visit blocks in a pseudo random order
    let mut rng = rand::rngs::StdRng::seed_from_u64(
        ((chunk_x as u64) * (2_u64.pow(32)) + (chunk_z as u64)).wrapping_add(seed as u64),
    );

    let mut rows: Vec<usize> = (0..BLOCK_SIZE).collect();
    while !rows.is_empty() {
        let x = rows.swap_remove(rng.gen::<usize>() % rows.len());
        let mut cols: Vec<usize> = (0..BLOCK_SIZE).collect();
        while !cols.is_empty() {
            let z = cols.swap_remove(rng.gen::<usize>() % cols.len());
            let perlin_x = (chunk_x * (BLOCK_SIZE as f64) + x as f64) / 10.; //(BLOCK_SIZE as f64);
            let perlin_z = (chunk_z * (BLOCK_SIZE as f64) + z as f64) / 10.; //(BLOCK_SIZE as f64);
            let elev = perlin_elev.get([perlin_x, perlin_z]);
            let trees = perlin_trees.get([perlin_x, perlin_z]);

            let items = perlin_items.get([perlin_x, perlin_z]);

            if elev > 0.125 {
                let norm = (elev - 0.125) / (1.0 - 0.125);
                block[x as usize][z as usize].kind = TileType::B;
                block[x as usize][z as usize].height =
                    ((norm * ((MAX_ELEV - 1) as f64)) as usize) + 1;
            }

            if 0 != launcher_count && items > 0. {
                launcher_count -= 1;
                block[x as usize][z as usize].item = Some(ItemType::Launcher);
            } else if !has_cage && items > 0.9 {
                has_cage = true;
                block[x as usize][z as usize].item = Some(ItemType::Cage);
            } else if trees > 0.7 {
                block[x as usize][z as usize].copse = true;
            }
        }
    }
    block
}
