use base_api::tabs::FUNCTIONAL;
pub use slow_start_api::{AIR_DRYING, AIR_DRYING_EVENT, AIR_DRYING_STATE, AIR_DRYING_TIME, AirDried, AirDrying};
use weft_sdk::*;

pub static DRYING_RACK: Block = Block::named("drying_rack");
pub static DRYING_RACK_MENU: Menu = Menu::named("drying_rack");

pub fn register(r: &mut Registry) {
    r.entity_component(
        &AIR_DRYING_STATE,
        AirDrying { progress: 0, progress_max: 0, dry: 0, dry_max: 0, drying_item: 0, was_filled: false },
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&AIR_DRYING_STATE, tick_furnaces);

    // Drying Rack
    r.block(&DRYING_RACK)
        // Front towards the player; unlit and lit.
        .sound_group("base:block.wood")
        .texture("base:log_oak")
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("drying_rack")
        .hardness(2.0)
        .shares_cell_with("#water")
        .mined_by("axe", 0)
        .prop("filled", ["false", "true"])
        .states(|p, s| if p.is("filled", "true") { s.model_file("drying_rack_filled") } else { s })
        .tab(&FUNCTIONAL)
        .entity(|e| e.attach(&AIR_DRYING_STATE).inventory(2))
        .opens(&DRYING_RACK_MENU);

    r.menu(&DRYING_RACK_MENU)
        .group("input", Slots::single().holder())
        .group("output", Slots::single().output().holder())
        .recipes(&AIR_DRYING)
        .player_inventory()
        .screen(|ui| {
            ui.panel("panel", S::column().pad(7.0).gap(4.0).stretch(), |ui| {
                ui.title();
                ui.row(S::row().gap(8.0), |ui| {
                    ui.spacer();
                    ui.column(S::column().gap(2.0), |ui| {
                        ui.group("input");
                    });
                    ui.recipe_arrow_bind(|c| {
                        c.menu_component(&AIR_DRYING_STATE)
                            .map_or(0.0, |s| ratio(s.dry, s.dry_max))
                    });
                    ui.group("output");
                    ui.spacer();
                });
                ui.player_inventory(true);
            });
        });
}

/// `ticks` of one drying rack at once (its area was unloaded, the time jumped)
fn catch_up(w: &World, id: u64, s: &mut AirDrying, ticks: u32) {
    let inv = w.inventory(id);
    let (input_slot, output_slot) = (inv.group("input"), inv.group("output"));
    let (input, output) = (input_slot.get(0), output_slot.get(0));
    let mut idle = ticks as u64;
    if let Some((out, time)) = (!input.is_empty())
        .then(|| recipe_process(&AIR_DRYING, input.item))
        .flatten()
    {
        let q = abi::Query { entity: id, item: input.item, ..query() };
        s.dry_max = w
            .chain_compute(&AIR_DRYING_TIME, time as f64, &q)
            .round()
            .max(1.0) as u32;
        let items = (input.count).min(output.room_for(out)) as u64;
        let work = (items * s.dry_max as u64).saturating_sub(s.dry as u64);
        let worked = idle.min(work);
        let filled = if worked > s.progress as u64 { worked - s.progress as u64 } else { 0 };
        s.progress = (s.progress as u64 + filled - worked) as u32;
        let total = s.dry as u64 + worked;
        let done = (total / s.dry_max as u64) as u32;
        s.dry = (total % s.dry_max as u64) as u32;
        if done > 0 {
            input_slot.set(0, input.shrink(done));
            output_slot.set(0, output.merged(out, done));
            for _ in 0..done {
                w.emit(&AIR_DRYING_EVENT, AirDried { drying_rack: id, input: input.item, output: out });
            }
        }
        idle -= worked;
    }
    s.progress = (s.progress as u64).saturating_sub(idle) as u32;
    s.dry = (s.dry as u64).saturating_sub(idle * 2) as u32;
}

/// Every furnace of a region at once, through its state (a system: no host call to reach it).
fn tick_furnaces(b: &mut SystemBatch<AirDrying>) {
    let w = b.world();
    for (id, s, ticks) in b.iter_mut() {
        let Some(st) = w.entity(id) else { continue };
        if ticks > 1 {
            catch_up(&w, id, s, ticks);
            continue;
        }
        let pos = st.block_pos();
        let inv = w.inventory(id);
        let (input_slot, output_slot) = (inv.group("input"), inv.group("output"));
        let (input, output) = (input_slot.get(0), output_slot.get(0));

        if s.progress > 0 {
            s.progress -= 1;
        }
        // Another item in the input starts over, before this tick counts for it.
        if input.item != s.drying_item {
            s.drying_item = input.item;
            s.dry = 0;
        }
        let recipe = (!input.is_empty())
            .then(|| recipe_process(&AIR_DRYING, input.item))
            .flatten();
        let fits = recipe.is_some_and(|(out, _)| output.room_for(out) > 0);
        if s.progress == 0 && fits {
            s.progress = ticks;
            s.progress_max = ticks;
        }
        if s.progress > 0 && fits {
            let (out, time) = recipe.unwrap();
            let q = abi::Query { entity: id, item: input.item, ..query() };
            s.dry_max = w
                .chain_compute(&AIR_DRYING_TIME, time as f64, &q)
                .round()
                .max(1.0) as u32;
            s.dry += 1;
            if s.dry >= s.dry_max {
                s.dry = 0;
                input_slot.set(0, input.shrink(1));
                output_slot.set(0, output.merged(out, 1));
                w.emit(&AIR_DRYING_EVENT, AirDried { drying_rack: id, input: input.item, output: out });
            }
        } else if s.progress > 0 {
            // Modern's: burning with nothing it can do starts over; out of fuel it cools down.
            s.dry = 0;
        } else {
            s.dry = s.dry.saturating_sub(2);
        }

        let (input, output) = (input_slot.get(0), output_slot.get(0));
        let filled = input.item != 0 || output.item != 0;
        if filled != s.was_filled && DRYING_RACK.is(w.block(pos)) {
            s.was_filled = filled;
            w.prop_set(pos, "filled", if filled { "true" } else { "false" });
        }
    }
}
