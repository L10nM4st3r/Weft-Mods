//! What the overworld has more of: flowers, branches, pebbles and wild crops on the surface
//! (scatters, by biome), and dense rock deep down with ores of its own.

use base_api::OVERWORLD;
use weft_sdk::noise::{hash, unit};
use weft_sdk::terrain::Site;
use weft_sdk::*;

use crate::blocks::{BRANCH, STONE_PEBBLE};
use crate::blocks_rocks::{DENSE_ROCK, SUPER_DENSE_ROCK, ULTRA_DENSE_ROCK, LOWER_MANTLE_ROCK, UPPER_MANTLE_ROCK, UPPER_CORE_LIQUID, CORE_ALLOY, ORES};

/// Base's flowers by biome, which its decoration plants.
static FLOWERS: Component<BlockList> = Component::named("flowers");
/// Shares of the ground covered, by biome.
static BRANCH_COVER: Component<f32> = Component::named("branch_cover");
static PEBBLE_COVER: Component<f32> = Component::named("pebble_cover");
static CROP_COVER: Component<f32> = Component::named("crop_cover");
/// What grows wild where `CROP_COVER` says, by biome.
static CROPS: Component<BlockList> = Component::named("crops");
static STONE: Tag = Tag::named("stone_ore_replaceables");

static BRANCHES: Scatter = Scatter::named("branches");
static PEBBLES: Scatter = Scatter::named("pebbles");
static WILD_CROPS: Scatter = Scatter::named("wild_crops");

/// Where the crust ends and the mantle begins.
const CRUST_BOTTOM: i32 = -45000;
const SUPER_DENSE_ROCK_LAYER: i32 = -10000;
const ULTRA_DENSE_ROCK_LAYER: i32 = -40000;
const LOWER_MANTLE_START: i32 = -1_885_000;
const MANTLE_BOTTOM: i32 = -2_885_000;
const UPPER_CORE_BOTTOM: i32 = -5_155_000;
//const LOWER_CORE_BOTTOM: i32 = -6_371_000;

const FLOWERS_PLAIN: &str = "base:dandelion:8 base:rose:32 blue_orchid:1";
const FLOWERS_FOREST: &str = "base:dandelion:8 base:rose:32 brown_mushroom:1 red_mushroom:1";
const FLOWERS_BIRCH: &str = "base:dandelion:8 base:rose:32 brown_mushroom:6 red_mushroom:6";
const WILD: &str = "wild_carrot:1 wild_potato:1 wild_onion:1";
const DESERT: &str = "dead_bush:1 dry_grass:2";

pub fn register(r: &mut Registry) {
    // (biome, flowers, branches, pebbles, crops and how many)
    for (biome, flowers, branches, pebbles, (crops, cover)) in [
        ("plains", Some(FLOWERS_PLAIN), 0.01, 0.005, (WILD, 0.008)),
        ("forest", Some(FLOWERS_FOREST), 0.05, 0.02, (WILD, 0.0004)),
        ("birch_forest", Some(FLOWERS_BIRCH), 0.05, 0.02, (WILD, 0.0004)),
        ("taiga", Some(FLOWERS_FOREST), 0.05, 0.02, (WILD, 0.0004)),
        ("mountains", Some(FLOWERS_PLAIN), 0.0, 0.02, (WILD, 0.0)),
        ("desert", None, 0.0, 0.0, (DESERT, 0.005)),
    ] {
        let b = r
            .edit(Biome::generated(format!("base:{biome}")))
            .set(&BRANCH_COVER, branches)
            .set(&PEBBLE_COVER, pebbles)
            .set(&CROP_COVER, cover)
            .set_text(&CROPS, crops);
        if let Some(f) = flowers {
            b.set_text(&FLOWERS, f);
        }
    }
    let crop = |site: &Site| CROPS.on_biome(site.biome()).and_then(|l| l.pick(site.random()));
    let scatters: [(&'static Scatter, &'static Component<f32>, &str, fn(&Site) -> Option<u32>); 3] = [
        (&BRANCHES, &BRANCH_COVER, "", |_| Some(BRANCH.id())),
        (&PEBBLES, &PEBBLE_COVER, "", |_| Some(STONE_PEBBLE.id())),
        (&WILD_CROPS, &CROP_COVER, "#desert_plant_soil", crop),
    ];
    for (scatter, cover, ground, pick) in scatters {
        let s = r.scatter(scatter).cell(1).density(cover).dimension(&OVERWORLD);
        let s = if ground.is_empty() { s } else { s.ground(ground) };
        s.shape(move |site| {
            if let Some(what) = pick(site) {
                site.put(site.pos(), what);
            }
        });
    }

    r.decoration(rock_layers).dimension(&OVERWORLD);
    //r.decoration(surface_decorator).dimension(&OVERWORLD);

    // Base's ores, deeper and with a block of their own where a vein goes through dense rock.
    for ((ore, ..), (min_y, max_y, veins, size, exposed)) in ORES.iter().zip([
        (-128, 128, 6, 8, 0.0),
        (-128, 128, 4, 5, 0.0),
        (CRUST_BOTTOM, -16, 2, 7, 0.5),
        (CRUST_BOTTOM / 2, 6, 4, 8, 0.0),
        (CRUST_BOTTOM, 0, 3, 6, 0.5),
    ]) {
        let dense = Block::generated(format!("dense_rock_{ore}_ore"));
        let super_dense = Block::generated(format!("super_dense_rock_{ore}_ore"));
        let ultra_dense = Block::generated(format!("ultra_dense_rock_{ore}_ore"));

        r.edit(Block::generated(format!("base:{ore}_ore")).ore())
            .y(min_y, max_y)
            .veins(veins)
            .vein_size(size)
            .exposed(exposed)
            .replaces_with("#dense_rock_ore_replaceables", dense)
            .replaces_with("#super_dense_rock_ore_replaceables", super_dense)
            .replaces_with("#ultra_dense_rock_ore_replaceables", ultra_dense);
    }
}

/// Create multiple layers of different rocks.
fn rock_layers(g: &mut Gen) {
    let (seed, o) = (g.seed(), g.origin());
    if o.y > 10 {
        return;
    }
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                let wy = o.y + y as i32;

                if wy < UPPER_CORE_BOTTOM {
                    g.set(x, y, z, CORE_ALLOY.id());
                }
                else if wy < MANTLE_BOTTOM {
                    g.set(x, y, z, UPPER_CORE_LIQUID.id());
                }
                else if STONE.on_block(g.get(x, y, z)) {
                    let (wx, wz) = (o.x + x as i32, o.z + z as i32);
                    let line = || (unit(hash(seed ^ 0xF10, wx as i64, wy as i64, wz as i64)) * 10.0).round() as i32;
                    let this_line = line();

                    if wy < LOWER_MANTLE_START || wy < this_line + LOWER_MANTLE_START {
                        g.set(x, y, z, LOWER_MANTLE_ROCK.id());
                    }
                    else if wy < CRUST_BOTTOM || wy < this_line + CRUST_BOTTOM {
                        g.set(x, y, z, UPPER_MANTLE_ROCK.id());
                    }
                    else if wy < ULTRA_DENSE_ROCK_LAYER || wy < this_line + ULTRA_DENSE_ROCK_LAYER {
                        g.set(x, y, z, ULTRA_DENSE_ROCK.id());
                    }
                    else if wy < SUPER_DENSE_ROCK_LAYER || wy < this_line + SUPER_DENSE_ROCK_LAYER {
                        g.set(x, y, z, SUPER_DENSE_ROCK.id());
                    }
                    else if wy < 0 || wy < this_line {
                        g.set(x, y, z, DENSE_ROCK.id());
                    }
                }
            }
        }
    }
}





// pub fn surface_decorator(g: &mut Gen) {
//     //let Some(&cols) = g.columns() else { return };
//     let seed = g.seed();
//     let BlockPos { x: ox, y: oy, z: oz } = g.origin();

//     if oy < 60 || oy > 256 {
//         return;
//     }
//     for z in 0..16usize { for x in 0..16usize { for y in 0..16usize {
//         let (wx, wy, wz) = (ox + x as i32, oy + y as i32, oz + z as i32);
//         let h = hash(seed ^ 0xF10, wx as i64, wy as i64, wz as i64);
//         let roll = unit(h);

//         if g.get(x, y, z) == 0 {
//             continue;
//         }

//         if g.get(x, y, z) == BIRCH_LOG.id() {
//             for (new_x, new_z, facing) in [
//                 (x + 1, z, Dir::East),
//                 (x, z + 1, Dir::South),
//                 (x - 1, z, Dir::West),
//                 (x, z - 1, Dir::North)
//             ] {
//                 if g.get(new_x, y, new_z) == 0 {
//                     let (nwx, nwz) = (ox + new_x as i32, oz + new_z as i32);
//                     let new_h = hash(seed ^ 0xF10, nwx as i64, wy as i64, nwz as i64);
//                     let new_roll = unit(new_h);

//                     if new_roll < 0.1 {
//                         g.set(new_x, y, new_z, BROWN_SHELF_MUSHROOM.id())
//                         g.set_prop(new_x, y, new_z, "facing", facing)
//                     }
//                 }
//             }

//             continue;
//         }

//         let max_height = (roll * 10.0).round() as i32;

//         if wy < 0 + max_height {
//             g.set(x, y, z, DENSE_ROCK.id());
//         }

//     } } }
// }
