//! What other mods use of the base game, as types: its events with their values, components,
//! value chains, recipe kinds, equipment slots, creative tabs and dimensions. A mod adds
//! `base_api` to its `Cargo.toml` and gets these checked when it compiles, without depending on
//! the game's code; the engine still compares the value layouts when the mods load.
//!
//! Names here are full (`base:..`): the statics compile into whichever mod uses them.

use base_api::{FOOD, HUNGER, eat as vanilla_eat};
use weft_sdk::*;

static ALWAYS_EATABLE: Tag = Tag::named("can_always_eat");
static WATER: Tag = Tag::named("water");

value! {
    /// A player's thirst.
    pub struct Thirst { pub thirst: f32, pub wet_throat: f32, pub thirst_exhaustion: f32, pub damage_tick: i8 }
}
pub static THIRST: Component<Thirst> = Component::named("tetanus:thirst");

value! {
    /// Read by the hunger system; other mods set it on their own food.
    pub struct Drink { pub thirst: u32, pub wet_throat: f32 }
}
// The common vocabulary the base game reads; any mod may set these on its own content.
pub static DRINK: Component<Drink> = Component::named("tetanus:drink");

value! {
    pub struct Canteen { pub thirst: u32, pub wet_throat: f32 }
}
// The common vocabulary the base game reads; any mod may set these on its own content.
pub static CANTEEN: Component<Canteen> = Component::named("tetanus:canteen");

pub static CANTEEN_OTHER: Component<ItemRef> = Component::named("tetanus:canteen_other");

/// Drinking for `.on_use`: drink and saturation from its `FOOD`, one item less. With `.use_ticks(32)`
/// it takes as long as the base game's food.
pub fn drink_item(u: &mut ItemUse) -> bool {
    let (w, stack) = (u.world(), u.stack());
    let Some(drink) = DRINK.on_stack(&stack) else {
        vanilla_eat(u);
        return false;
    };
    let Some(thirst) = w.component_mut(u.entity(), &THIRST) else { return false };

    // Player is not thirsty, and not can always eat
    if thirst.thirst >= 20.0 && !ALWAYS_EATABLE.on_item(stack.item) {
        // Player is not hungry and is holding food
        let Some(hunger) = w.component_mut(u.entity(), &HUNGER) else { return false };
        if hunger.food >= 20.0 && !FOOD.on_stack(&stack).is_none() {
            return false;
        }
    }
    thirst.thirst = (thirst.thirst + drink.thirst as f32).min(20.0);
    thirst.wet_throat = (thirst.wet_throat + drink.wet_throat).min(thirst.thirst);

    if !FOOD.on_stack(&stack).is_none() {
        let Some(hunger) = w.component_mut(u.entity(), &HUNGER) else {
            u.set_stack(consume_one(stack));
            return false;
        };
        let Some(food) = FOOD.on_stack(&stack) else {
            u.set_stack(consume_one(stack));
            return false;
        };
        hunger.food = (hunger.food + food.hunger as f32).min(20.0);
        hunger.saturation = (hunger.saturation + food.saturation).min(hunger.food);
    }

    u.set_stack(consume_one(stack));
    true
}

pub fn do_hydrate(h: &mut Thirst, thirst: u32, wet_throat: f32) {
    h.thirst = (h.thirst + thirst as f32).min(20.0);
    h.wet_throat = (h.wet_throat + wet_throat).min(h.thirst);
}

pub fn use_empty_canteen(u: &mut ItemUse) -> bool {
    let (w, stack) = (u.world(), u.stack());

    let look_at = w.looked_at(u.entity(), 4.0, Ray::blocks().liquids());

    if !look_at.is_none() && WATER.on_block(w.block(look_at.unwrap().pos)) {
        u.set_stack(ItemStack { item: CANTEEN_OTHER.on_stack(&stack).unwrap().0, count: 1, data: 0 });
        return true;
    };
    false
}

pub fn drink_canteen(u: &mut ItemUse) -> bool {
    let stack = u.stack();
    let Some(drink) = CANTEEN.on_stack(&stack) else { return false };
    let w = u.world();
    let Some(h) = w.component_mut(u.entity(), &THIRST) else { return false };

    let look_at = w.looked_at(u.entity(), 4.0, Ray::blocks().liquids());
    if !look_at.is_none() && WATER.on_block(w.block(look_at.unwrap().pos)) {
        let next = stack.with_wear(0);
        u.set_stack(next);
        //return true;
    };

    // if h.thirst >= 20.0 {
    //     return false;
    // }
    do_hydrate(h, drink.thirst, drink.wet_throat);

    if !w.outside_survival(u.entity()) {
        let left = DURABILITY
            .on_stack(&stack)
            .unwrap_or(0)
            .saturating_sub(stack.wear());

        if left <= 1 {
            u.set_stack(ItemStack { item: CANTEEN_OTHER.on_stack(&stack).unwrap().0, count: 1, data: 0 });
        } else {
            let extra = left.min(1); // Lose 1 durability
            let next = stack.add_wear(extra);
            u.set_stack(next);
        }
    }
    true
}

value! {
    /// A player's temperature component.
    pub struct Temperature {
        pub body_temperature: f32,
        pub environmental_temperature: f32,
        pub update_tick: u16,
        pub arrow_frame: u8,
        pub arrow_direction: u8,
        pub overlay_frame: u8
    }
}
pub static TEMPERATURE: Component<Temperature> = Component::named("tetanus:temperature");

value! {
    /// The dimension's base settings.
    pub struct TetanusDimension {
        /// The temperature for the day, and the night.
        pub temperature: [f32; 2],
        /// The height at which time and non-underground biomes' heat will be ignored and the maximum underground temperature is applied.
        pub underground_height: [f32; 2],
        /// The temperature at the bottom of the underground.
        pub underground_temperature: f32,
        /// The height at which time and non-underground biomes' heat will be ignored and the maximum underground temperature is applied. This can be used for planet cores or "hell".
        pub deep_underground_height: [f32; 2],
        /// The temperature at the bottom of the underground. This can be used for planet cores or "hell".
        pub deep_underground_temperature: f32,
        /// The height at which `elevated_temperature` begins to get added and then reaches the maximum.
        pub elevated_height: [f32; 2],
        /// The temperature added at y = `elevated_end_height`.
        pub elevated_temperature: f32,
    }
}
/// The dimension's base settings.
pub static TETANUS_DIMENSION: Component<TetanusDimension> = Component::named("tetanus:dimension_settings");

value! {
    /// The biome's base temperature.
    pub struct TetanusBiome {
        /// The temperature for the day, and the night.
        pub temperature: [f32; 2],
        /// Whether this biome's settings should apply at any depth below the ground.
        pub underground: bool,
        /// Whether this biome's water is clean to drink. This can be used for salt water, or swamp water, for example.
        pub water_is_clean: bool
    }
}
/// The biome's base temperature.
pub static TETANUS_BIOME: Component<TetanusBiome> = Component::named("tetanus:biome_settings");

pub static DEFAULT_BIOME: TetanusBiome = TetanusBiome { temperature: [0.0, 0.0], underground: false, water_is_clean: true };

value! {
    /// Tough as rust settings for a liquid block. These settings only apply to the player when standing inside the liquid.
    pub struct TetanusBlockLiquid {
        /// The temperature change while sitting in the liquid.
        pub temperature: f32,
        /// The max temperature you can be at while in this liquid.
        /// This will not cool you down if you are hotter, but will
        /// stop you exeeding this temperature if you weren't already
        /// and enter this liquid.
        pub max_temperature: f32,
        /// The minimum temperature you can be at while in this liquid.
        /// Works as the exact opposite as `max_temperature`.
        pub min_temperature: f32,
        /// If true, the temperature limits WILL work even if the player
        /// exeeds them from outside factors.
        /// This can be used for your magical cooling water, for example.
        pub temperature_limits_forced: bool
    }
}

/// Tough as rust settings for a liquid block. These settings only apply to the player when standing inside the liquid.
pub static TETANUS_BLOCK_LIQUID: Component<TetanusBlockLiquid> = Component::named("tetanus:block_liquid");
