use base_api::FUEL;
use base_api::tabs::FUNCTIONAL;
pub use slow_start_api::{CAMPFIRE_COOKED, CAMPFIRE_RECIPE, CAMPFIRE_STATE, CAMPFIRE_TIME, CampfireCooked, CampfireCooking, FIRE_STARTER};
use weft_sdk::{damage_causes::FIRE, *};

pub static CAMPFIRE: Block = Block::named("campfire_complete");
pub static CAMPFIRE_MENU: Menu = Menu::named("campfire");
static WATER_TAG: Tag = Tag::named("water");

pub fn register(r: &mut Registry) {
    r.entity_component(
        &CAMPFIRE_STATE,
        CampfireCooking { burn: 0, cook: 0, cook_max: 0, cooking_item: 0, was_lit: false, show_ash: false },
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&CAMPFIRE_STATE, tick_furnaces);

    // Campfire
    r.block(&CAMPFIRE)
        // Front towards the player; unlit and lit.
        .sound_group("base:block.wood")
        .texture("base:log_oak")
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("campfire_full_unlit")
        .outline([0.0, 0.0, 0.0], [16.0, 6.0, 16.0])
        .hitbox([0.0, 0.0, 0.0], [16.0, 6.0, 16.0])
        .hardness(2.0)
        .shares_cell_with("#water")
        .mined_by("axe", tool_tier::HAND)
        .prop("lit", ["false", "true"])
        .prop("show_ash", ["false", "true"])
        .drops("campfire_base", 2)
        .states(|p, s| {
            let s = if p.is("show_ash", "false") { s.model_texture("0", "slow_start:empty") } else { s };
            if p.is("lit", "true") { s.model_file("campfire_full_lit").light_rgb(13, 11, 8) } else { s }
        })
        .tab(&FUNCTIONAL)
        .entity(|e| e.attach(&CAMPFIRE_STATE).inventory(3))
        .opens(&CAMPFIRE_MENU);

    r.menu(&CAMPFIRE_MENU)
        .group("input", Slots::single().holder())
        .group("fuel", Slots::single().accepts("#fuel #campfire_starter").holder())
        .group("output", Slots::single().output().holder())
        .recipes(&CAMPFIRE_RECIPE)
        .player_inventory()
        .screen(|ui| {
            ui.panel("panel", S::column().pad(7.0).gap(4.0).stretch(), |ui| {
                ui.title();
                ui.row(S::row().gap(8.0), |ui| {
                    ui.spacer();
                    ui.column(S::column().gap(2.0), |ui| {
                        ui.group("input");
                        ui.progress_bind_dir("flame", S::default().size(14.0, 14.0), abi::ui_dir::UP, |c| {
                            c.menu_component(&CAMPFIRE_STATE)
                                .map_or(0.0, |s| ratio(s.burn, 24000))
                        });
                        ui.group("fuel");
                    });
                    ui.recipe_arrow_bind(|c| {
                        c.menu_component(&CAMPFIRE_STATE)
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
fn catch_up(w: &World, id: u64, s: &mut CampfireCooking, ticks: u32) {
    let inv = w.inventory(id);
    let (input_slot, fuel_slot, output_slot) = (inv.group("input"), inv.group("fuel"), inv.group("output"));
    let (input, fuel, output) = (input_slot.get(0), fuel_slot.get(0), output_slot.get(0));
    let mut idle = ticks as u64;
    if let Some((out, time)) = (!input.is_empty())
        .then(|| recipe_process(&CAMPFIRE_RECIPE, input.item))
        .flatten()
    {
        let q = abi::Query { entity: id, item: input.item, ..query() };
        s.cook_max = w
            .chain_compute(&CAMPFIRE_TIME, time as f64, &q)
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
                w.emit(&CAMPFIRE_COOKED, CampfireCooked { campfire: id, input: input.item, output: out });
            }
        }
        idle -= worked;
    }
    s.burn = (s.burn as u64).saturating_sub(idle) as u32;
    s.cook = (s.cook as u64).saturating_sub(idle * 2) as u32;
}

/// Every furnace of a region at once, through its state (a system: no host call to reach it).
fn tick_furnaces(b: &mut SystemBatch<CampfireCooking>) {
    let w = b.world();
    for (id, s, ticks) in b.iter_mut() {
        let Some(st) = w.entity(id) else { continue };
        if ticks > 1 {
            let was_lit = s.burn > 0;
            catch_up(&w, id, s, ticks);
            let pos = st.block_pos();
            if (s.burn > 0) != was_lit && CAMPFIRE.is(w.block(pos)) {
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
            if is_waterlogged(&w, pos) {
                s.burn = 0;
                s.cook = 0;
                w.prop_set(pos, "lit", "false");
                continue;
            }

            let entities: Vec<u64> = w.entities_near(Vec3::new(pos.x as f64 + 0.5, pos.y as f64 + 0.5, pos.z as f64 + 0.5), 1.0, None);
            // Set entities on fire when standing on top
            for entity in entities {
                if w.entity(entity).unwrap().block_pos() == pos {
                    w.entity_modify(entity).fire(100);
                    w.damage(entity, Damage::new(1.0, &FIRE));
                }
            }
            s.burn -= 1;
        }
        // Another item in the input starts over, before this tick counts for it.
        if input.item != s.cooking_item {
            s.cooking_item = input.item;
            s.cook = 0;
        }
        let recipe = (!input.is_empty())
            .then(|| recipe_process(&CAMPFIRE_RECIPE, input.item))
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
                if !s.show_ash {
                    s.show_ash = true;
                    w.prop_set(pos, "show_ash", "true");
                }
                fuel_slot.set(0, consume_one(fuel));
            }
        }
        if s.burn > 0 && fits {
            let (out, time) = recipe.unwrap();
            let q = abi::Query { entity: id, item: input.item, ..query() };
            s.cook_max = w
                .chain_compute(&CAMPFIRE_TIME, time as f64, &q)
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
                w.emit(&CAMPFIRE_COOKED, CampfireCooked { campfire: id, input: input.item, output: out });
            }
        } else if s.burn > 0 {
            // Modern's: burning with nothing it can do starts over; out of fuel it cools down.
            s.cook = 0;
        } else {
            s.cook = s.cook.saturating_sub(2);
        }
        let lit = s.burn > 0;
        if lit != was_lit && CAMPFIRE.is(w.block(pos)) {
            w.prop_set(pos, "lit", if lit { "true" } else { "false" });
        }
    }
}

fn is_waterlogged(w: &World, pos: BlockPos) -> bool {
    let parts = w.parts(pos);
    parts.len() > 1 && parts.iter().any(|&s| WATER_TAG.on_block(s))
}
