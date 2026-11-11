use base_api::FUEL;
use base_api::tabs::FUNCTIONAL;
pub use slow_start_api::{
    BRICK_STOVE_COOKED, BRICK_STOVE_RECIPE, BRICK_STOVE_STATE, BRICK_STOVE_TIME, BrickStoveCooked, BrickStoveCooking, FIRE_STARTER,
};
use weft_sdk::*;

pub static BRICK_STOVE: Block = Block::named("brick_stove");
pub static BRICK_STOVE_MENU: Menu = Menu::named("brick_stove");

pub fn register(r: &mut Registry) {
    r.entity_component(
        &BRICK_STOVE_STATE,
        BrickStoveCooking { burn: 0, cook: 0, cook_max: 0, cooking_item: 0, was_lit: false },
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&BRICK_STOVE_STATE, tick_furnaces);

    // Brick furnace
    r.block(&BRICK_STOVE)
        // Front towards the player; unlit and lit.
        .sound_group("base:block.stone")
        .texture("base:log_oak")
        .facing(Facings::HORIZONTAL, Toward::Player)
        .hardness(3.5)
        .mined_by("pickaxe", 0)
        .prop("lit", ["false", "true"])
        .states(|p, s: State| {
            let lit = p.is("lit", "true");

            let s = s
                .front(if lit { "brick_stove_front_lit" } else { "brick_stove_front" })
                .sides("brick_stove_side")
                .ends("brick_stove_top");
            if lit { s.light_rgb(13, 11, 8) } else { s }
        })
        .tab(&FUNCTIONAL)
        .entity(|e| e.attach(&BRICK_STOVE_STATE).inventory(3))
        .opens(&BRICK_STOVE_MENU);

    r.menu(&BRICK_STOVE_MENU)
        .group("input", Slots::single().holder())
        .group("fuel", Slots::single().accepts("#fuel #campfire_starter").holder())
        .group("output", Slots::single().output().holder())
        .recipes(&BRICK_STOVE_RECIPE)
        .player_inventory()
        .screen(|ui| {
            ui.panel("panel", S::column().pad(7.0).gap(4.0).stretch(), |ui| {
                ui.title();
                ui.row(S::row().gap(8.0), |ui| {
                    ui.spacer();
                    ui.column(S::column().gap(2.0), |ui| {
                        ui.group("input");
                        ui.progress_bind_dir("flame", S::default().size(14.0, 14.0), abi::ui_dir::UP, |c| {
                            c.menu_component(&BRICK_STOVE_STATE)
                                .map_or(0.0, |s| ratio(s.burn, 24000))
                        });
                        ui.group("fuel");
                    });
                    ui.recipe_arrow_bind(|c| {
                        c.menu_component(&BRICK_STOVE_STATE)
                            .map_or(0.0, |s| ratio(s.cook, s.cook_max))
                    });
                    ui.group("output");
                    ui.spacer();
                });
                ui.player_inventory(true);
            });
        });
}

/// `ticks` of one furnace at once (its area was unloaded, the time jumped): it smelts while
/// input, room and fuel last, lighting fuel as it goes, then the fire burns down and the
/// progress cools, as tick by tick.
fn catch_up(w: &World, id: u64, s: &mut BrickStoveCooking, ticks: u32) {
    let inv = w.inventory(id);
    let (input_slot, fuel_slot, output_slot) = (inv.group("input"), inv.group("fuel"), inv.group("output"));
    let (input, fuel, output) = (input_slot.get(0), fuel_slot.get(0), output_slot.get(0));
    let mut idle = ticks as u64;
    if let Some((out, time)) = (!input.is_empty())
        .then(|| recipe_process(&BRICK_STOVE_RECIPE, input.item))
        .flatten()
    {
        let q = abi::Query { entity: id, item: input.item, ..query() };
        s.cook_max = w
            .chain_compute(&BRICK_STOVE_TIME, time as f64, &q)
            .round()
            .max(1.0) as u32;
        let items = (input.count).min(output.room_for(out)) as u64;
        let per_fuel = FUEL.on_stack_item(&fuel).unwrap_or(0) as u64;
        let fuel_items = if per_fuel > 0 { fuel.count as u64 } else { 0 };
        let work = (items * s.cook_max as u64).saturating_sub(s.cook as u64);
        let worked = idle.min(work).min(s.burn as u64 + fuel_items * per_fuel);
        let lit = if worked > s.burn as u64 { (worked - s.burn as u64).div_ceil(per_fuel.max(1)) } else { 0 };
        if lit > 0 {
            let mut left = fuel;
            for _ in 0..lit {
                left = consume_one(left);
            }
            fuel_slot.set(0, left);
        }
        s.burn = (s.burn as u64 + lit * per_fuel - worked) as u32;
        let total = s.cook as u64 + worked;
        let done = (total / s.cook_max as u64) as u32;
        s.cook = (total % s.cook_max as u64) as u32;
        if done > 0 {
            input_slot.set(0, input.shrink(done));
            output_slot.set(0, output.merged(out, done));
            for _ in 0..done {
                w.emit(&BRICK_STOVE_COOKED, BrickStoveCooked { furnace: id, input: input.item, output: out });
            }
        }
        idle -= worked;
    }
    s.burn = (s.burn as u64).saturating_sub(idle) as u32;
    s.cook = (s.cook as u64).saturating_sub(idle * 2) as u32;
}

/// Every furnace of a region at once, through its state (a system: no host call to reach it).
fn tick_furnaces(b: &mut SystemBatch<BrickStoveCooking>) {
    let w = b.world();
    for (id, s, ticks) in b.iter_mut() {
        let Some(st) = w.entity(id) else { continue };
        if ticks > 1 {
            let was_lit = s.burn > 0;
            catch_up(&w, id, s, ticks);
            let pos = st.block_pos();
            if (s.burn > 0) != was_lit && BRICK_STOVE.is(w.block(pos)) {
                w.prop_set(pos, "lit", if s.burn > 0 { "true" } else { "false" });
            }
            continue;
        }
        let pos = st.block_pos();
        let inv = w.inventory(id);
        let (input_slot, fuel_slot, output_slot) = (inv.group("input"), inv.group("fuel"), inv.group("output"));
        let (input, fuel, output) = (input_slot.get(0), fuel_slot.get(0), output_slot.get(0));
        let was_lit = s.burn > 0;
        if s.burn > 0 {
            s.burn -= 1;
        }
        // Another item in the input starts over, before this tick counts for it.
        if input.item != s.cooking_item {
            s.cooking_item = input.item;
            s.cook = 0;
        }
        let recipe = (!input.is_empty())
            .then(|| recipe_process(&BRICK_STOVE_RECIPE, input.item))
            .flatten();
        let fits = recipe.is_some_and(|(out, _)| output.room_for(out) > 0);
        if s.burn > 0 {
            if let Some(ticks) = FUEL.on_stack_item(&fuel) {
                s.burn += ticks * fuel.count;
                fuel_slot.set(0, ItemStack { item: 0, count: 0, data: 0 });
            }
        } else if s.burn == 0 {
            // Must use fire starter to begin a fire, then use fuel to kindle it
            if let Some(ticks) = FIRE_STARTER.on_stack_item(&fuel) {
                s.burn += ticks;
                fuel_slot.set(0, consume_one(fuel));
            }
        }
        if s.burn > 0 && fits {
            let (out, time) = recipe.unwrap();
            let q = abi::Query { entity: id, item: input.item, ..query() };
            s.cook_max = w
                .chain_compute(&BRICK_STOVE_TIME, time as f64, &q)
                .round()
                .max(1.0) as u32;
            if s.burn > 18000 {
                s.cook += 1;
            }
            s.cook += 1;
            if s.cook >= s.cook_max {
                s.cook = 0;
                input_slot.set(0, input.shrink(1));
                output_slot.set(0, output.merged(out, 1));
                w.emit(&BRICK_STOVE_COOKED, BrickStoveCooked { furnace: id, input: input.item, output: out });
            }
        } else if s.burn > 0 {
            // Modern's: burning with nothing it can do starts over; out of fuel it cools down.
            s.cook = 0;
        } else {
            s.cook = s.cook.saturating_sub(2);
        }
        let lit = s.burn > 0;
        if lit != was_lit && BRICK_STOVE.is(w.block(pos)) {
            w.prop_set(pos, "lit", if lit { "true" } else { "false" });
        }
    }
}
