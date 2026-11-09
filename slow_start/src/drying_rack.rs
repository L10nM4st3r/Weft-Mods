use base_api::tabs::FUNCTIONAL;
pub use slow_start_api::{AIR_DRYING, AIR_DRYING_EVENT, AIR_DRYING_STATE, AIR_DRYING_TIME, AirDried, AirDrying};
use weft_sdk::*;

pub static DRYING_RACK: Block = Block::named("drying_rack");
pub static DRYING_RACK_MENU: Menu = Menu::named("drying_rack");

pub fn register(r: &mut Registry) {
    r.entity_component(
        &AIR_DRYING_STATE,
        AirDrying { dry: 0, dry_max: 0, drying_item: 0, was_filled: false },
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
        .entity(|e| e.attach(&AIR_DRYING_STATE).inventory(1))
        .opens(&DRYING_RACK_MENU);

    r.menu(&DRYING_RACK_MENU)
        .group("input", Slots::single().holder().limit(1))
        .recipes(&AIR_DRYING)
        .player_inventory()
        .screen(|ui| {
            ui.panel("panel", S::column().pad(7.0).gap(4.0).stretch(), |ui| {
                ui.title();
                ui.row(S::row().gap(8.0), |ui| {
                    ui.spacer();
                    ui.recipe_arrow_bind(|c| {
                        c.menu_component(&AIR_DRYING_STATE)
                            .map_or(0.0, |s| ratio(s.dry, s.dry_max))
                    });
                    ui.column(S::column().gap(2.0), |ui| {
                        ui.group("input");
                    });
                    ui.spacer();
                });
                ui.player_inventory(true);
            });
        });
}

/// `ticks` of one drying rack at once (its area was unloaded, the time jumped)
fn catch_up(w: &World, id: u64, s: &mut AirDrying, ticks: u32) {
    let inv = w.inventory(id);
    let input_slot = inv.group("input");
    let input = input_slot.get(0);
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
        let work = (s.dry_max as u64).saturating_sub(s.dry as u64);
        let worked = idle.min(work);
        let total = s.dry as u64 + worked;
        s.dry = (total % s.dry_max as u64) as u32;
        if total >= s.dry_max as u64 {
            input_slot.set(0, out);
            w.emit(&AIR_DRYING_EVENT, AirDried { drying_rack: id, input: input.item, output: out });
        }
        idle -= worked;
    }
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
        let input_slot = inv.group("input");
        let input = input_slot.get(0);
        let filled = input.item != 0;
        if filled != s.was_filled && DRYING_RACK.is(w.block(pos)) {
            s.was_filled = filled;
            w.prop_set(pos, "filled", if filled { "true" } else { "false" });
        }

        // Another item in the input starts over, before this tick counts for it.
        if input.item != s.drying_item {
            s.drying_item = input.item;
            s.dry = 0;
        }
        let recipe = (!input.is_empty())
            .then(|| recipe_process(&AIR_DRYING, input.item))
            .flatten();

        if recipe == None {
            continue;
        }

        let (out, time) = recipe.unwrap();
        if out.item != 0 {
            let q = abi::Query { entity: id, item: input.item, ..query() };
            s.dry_max = w
                .chain_compute(&AIR_DRYING_TIME, time as f64, &q)
                .round()
                .max(1.0) as u32;
            s.dry += 1;
            if s.dry >= s.dry_max {
                s.dry = 0;
                input_slot.set(0, out);
                w.emit(&AIR_DRYING_EVENT, AirDried { drying_rack: id, input: input.item, output: out });
            }
        } else {
            s.dry = 0;
        }
    }
}
