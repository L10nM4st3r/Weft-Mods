use base_api::{CRAFTING, SMELTING};
use weft_sdk::*;

pub fn register(r: &mut Registry) {
    r.shaped(&CRAFTING, "base:cobblestone", 1, &["##", "##"], &[('#', "#pebble")]);
    r.shapeless(&CRAFTING, "base:stick", 1, &["#branch"], 0);
    r.shaped(&CRAFTING, "#hollow_log/{m}", 4, &[" # ", "# #", " # "], &[('#', "#hollow_log_of/{m}")]);
    r.shapeless(&SMELTING, "dense_rock", 1, &["#cobbled_dense_rock"], 200);
}
