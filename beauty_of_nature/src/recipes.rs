use base_api::{CRAFTING, SMELTING};
use weft_sdk::*;

pub fn register(r: &mut Registry) {
    r.shaped(&CRAFTING, "base:cobblestone", 1, &["##", "##"], &[('#', "#stone_pebble")]);
    r.shaped(&CRAFTING, "base:raw_iron", 1, &["##", "##"], &[('#', "#iron_pebble")]);
    r.shapeless(&CRAFTING, "base:stick", 1, &["#branch"], 0);
    r.shapeless(&SMELTING, "dense_rock", 1, &["#cobbled_dense_rock"], 200);
}
