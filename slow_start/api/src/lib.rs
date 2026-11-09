//! What other mods use of the base game, as types: its events with their values, components,
//! value chains, recipe kinds, equipment slots, creative tabs and dimensions. A mod adds
//! `base_api` to its `Cargo.toml` and gets these checked when it compiles, without depending on
//! the game's code; the engine still compares the value layouts when the mods load.
//!
//! Names here are full (`base:..`): the statics compile into whichever mod uses them.

use weft_sdk::*;

pub static FIRE_STARTER: Component<u32> = Component::named("slow_start:fire_starter");

pub static AIR_DRYING: RecipeKind = RecipeKind::named("slow_start:air_drying");

value! {
    /// Fired when a drying rack finishes one item.
    pub struct AirDried { pub drying_rack: u64, pub input: u32, pub output: ItemStack }
}
pub static AIR_DRYING_EVENT: Event<AirDried> = Event::named("slow_start:air_dried_event");

value! {
    /// A drying rack's state, synced to whoever has it open.
    pub struct AirDrying { pub dry: u32, pub dry_max: u32, pub drying_item: u32, pub was_filled: bool }
}
pub static AIR_DRYING_STATE: Component<AirDrying> = Component::named("slow_start:air_dried_state");

/// A drying rack's drying time: base is the recipe's time, `entity` the drying rack, `item` the input.
pub static AIR_DRYING_TIME: Chain = Chain::named("slow_start:air_drying_time");

pub static CAMPFIRE_RECIPE: RecipeKind = RecipeKind::named("slow_start:campfire_recipe");

value! {
    /// Fired when a drying rack finishes one item.
    pub struct CampfireCooked { pub campfire: u64, pub input: u32, pub output: ItemStack }
}
pub static CAMPFIRE_COOKED: Event<CampfireCooked> = Event::named("slow_start:campfire_cooked");

value! {
    /// A drying rack's state, synced to whoever has it open.
    pub struct CampfireCooking { pub burn: u32, pub cook: u32, pub cook_max: u32, pub cooking_item: u32, pub was_lit: bool, pub show_ash: bool }
}
pub static CAMPFIRE_STATE: Component<CampfireCooking> = Component::named("slow_start:campfire_state");

/// A drying rack's drying time: base is the recipe's time, `entity` the drying rack, `item` the input.
pub static CAMPFIRE_TIME: Chain = Chain::named("slow_start:campfire_time");

pub static BRICK_STOVE_RECIPE: RecipeKind = RecipeKind::named("slow_start:brick_stove_recipe");

value! {
    /// Fired when a drying rack finishes one item.
    pub struct BrickStoveCooked { pub furnace: u64, pub input: u32, pub output: ItemStack }
}
pub static BRICK_STOVE_COOKED: Event<BrickStoveCooked> = Event::named("slow_start:brick_stove_cooked");

value! {
    /// A drying rack's state, synced to whoever has it open.
    pub struct BrickStoveCooking { pub burn: u32, pub cook: u32, pub cook_max: u32, pub cooking_item: u32, pub was_lit: bool }
}
pub static BRICK_STOVE_STATE: Component<BrickStoveCooking> = Component::named("slow_start:brick_stove_state");

/// A drying rack's drying time: base is the recipe's time, `entity` the drying rack, `item` the input.
pub static BRICK_STOVE_TIME: Chain = Chain::named("slow_start:brick_stove_time");
