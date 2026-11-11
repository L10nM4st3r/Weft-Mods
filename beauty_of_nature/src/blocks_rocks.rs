use base_api::tabs::{BUILDING, NATURAL, TOOLS};
use weft_sdk::*;

pub static DENSE_ROCK: Block = Block::named("dense_rock");
pub static COBBLED_DENSE_ROCK: Block = Block::named("cobbled_dense_rock");

pub static SUPER_DENSE_ROCK: Block = Block::named("super_dense_rock");
pub static ULTRA_DENSE_ROCK: Block = Block::named("ultra_dense_rock");

pub static UPPER_MANTLE_ROCK: Block = Block::named("upper_mantle_rock");
pub static LOWER_MANTLE_ROCK: Block = Block::named("lower_mantle_rock");

pub static UPPER_CORE_LIQUID: Block = Block::named("upper_core_liquid");
pub static CORE_ALLOY: Block = Block::named("core_alloy");

/// Ores: (ore, level to harvest it, drop, count).
pub const ORES: [(&str, u32, &str, u32); 5] = [
    ("coal", tool_tier::WOOD, "base:coal", 1),
    ("iron", tool_tier::STONE, "base:raw_iron", 1),
    ("gold", tool_tier::IRON, "base:raw_gold", 1),
    ("redstone", tool_tier::IRON, "base:redstone", 4),
    ("diamond", tool_tier::IRON, "base:diamond", 1),
];

// To add support for modded ores:
// - Get textures of all modded ores, use script to generate all ore block textures.
// - Add ore into the data above.
// - Also add ore generation data to worldgen.rs.

/// A rock/stone type.
fn add_rock<'a>(r: &'a mut Registry, handle: &'static Block, hardness: f32) -> BlockBuilder<'a> {
    r.block(handle)
        .sound_group("base:block.stone")
        .tag("stone")
        .tag("has_shapes")
        .tab(&BUILDING)
        .hardness(hardness)
        .mined_by("pickaxe", tool_tier::WOOD)
}

/// A ore block.
fn add_ore<'a>(r: &'a mut Registry, stone: &str, ore: &str, hardness: f32, tier: u32) -> BlockBuilder<'a> {
    r.block(Block::generated(format!("{stone}_{ore}_ore")))
        .sound_group("base:block.stone")
        .tag("stone")
        .tag("ore")
        .tag(&format!("ores/{ore}"))
        .tab(&NATURAL)
        .hardness(hardness)
        .mined_by("pickaxe", tier)
}

pub fn register(r: &mut Registry) {
    add_rock(r, &DENSE_ROCK, 4.5)
        .drops("cobbled_dense_rock", 1)
        .tag("smooth_stone")
        .tag("dense_rock_ore_replaceables");

    add_rock(r, &COBBLED_DENSE_ROCK, 4.5)
        .tag("cobbled_dense_rock")
        .tag("materials/stone");

    // Denser rocks are more difficlt to break, no cobbled version
    // Totally not bc I'm lazy
    add_rock(r, &SUPER_DENSE_ROCK, 7.5)
        .tag("smooth_stone")
        .tag("super_dense_rock_ore_replaceables");

    add_rock(r, &ULTRA_DENSE_ROCK, 13.5)
        .tag("smooth_stone")
        .tag("ultra_dense_rock_ore_replaceables");

    add_rock(r, &UPPER_MANTLE_ROCK, 32.5)
        .immovable()
        .tag("mantle_stone");

    add_rock(r, &LOWER_MANTLE_ROCK, 65.0)
        .immovable()
        .tag("mantle_stone");

    add_rock(r, &CORE_ALLOY, 500.0)
        .immovable()
        .texture("solid_core")
        .tag("core_alloy");

    r.liquid(&UPPER_CORE_LIQUID)
        .block(|b| {
            b.light(15)
                .drag(0.8)
                .immovable()
                .tag("core_liquid")
                .texture("upper_core_liquid")
                .set(
                    &tetanus_api::TETANUS_BLOCK_LIQUID,
                    tetanus_api::TetanusBlockLiquid {
                        temperature: 3000.0,
                        max_temperature: 3700.0,
                        min_temperature: 3000.0,
                        temperature_limits_forced: false,
                    },
                )
        })
        .flow_delay(120)
        .drop_off(2)
        .slope_find(2)
        .heat(3850)
        //.density(100000)
        .fog(0x991A00, 1.0)
        .no_overlay()
        .bucket(Item::generated("core_liquid_bucket".into()), Item::generated("base:bucket".into()), &TOOLS);
    r.liquid_reaction("#core_liquid", "#water")
        .source_turns_into("core_alloy")
        .flowing_turns_into("base:obsidian")
        .below_turns_into("upper_core_liquid");

    for (ore, level, drop, count) in ORES {
        add_ore(r, "dense_rock", ore, 7.0, level).drops(drop, count);
        add_ore(r, "super_dense_rock", ore, 10.0, level).drops(drop, count);
        add_ore(r, "ultra_dense_rock", ore, 15.0, level).drops(drop, count);
    }
}
