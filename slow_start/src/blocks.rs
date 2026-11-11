use base_api::tabs::{FUNCTIONAL, INGREDIENTS};
use base_api::{FLAMMABLE, FUEL};
use weft_sdk::*;

pub static PLANT_FIBRE: Item = Item::named("plant_fibre");
pub static KNIVES: Tag = Tag::named("tools/knife");
pub static PLACABLE_BRICK_CONVERTABLE: Tag = Tag::named("placable_brick_convertable");
pub static TALL_GRASS: Block = Block::named("base:tall_grass");

pub static CAMPFIRE_BASE: Block = Block::named("campfire_base");
pub static CAMPFIRE: Block = Block::named("campfire_complete");
pub static RAW_CLAY_SINGLE_BRICK: Block = Block::named("raw_clay_single_brick");
pub static SINGLE_BRICK: Block = Block::named("single_brick");

//pub static IRON_ORE: Block = Block::named("base:iron_ore");

pub fn register(r: &mut Registry) {
    r.edit(&TALL_GRASS)
        .on_break(|w: &World, b: &BlockChange| break_grass(w, b));

    r.block(&CAMPFIRE_BASE)
        .model_file("campfire_base")
        .sound_group("base:block.wood")
        .hardness(2.0)
        .mined_by("axe", tool_tier::HAND)
        .set(&FLAMMABLE, 5)
        .set(&FUEL, 300)
        .facing(Facings::HORIZONTAL, Toward::Player)
        .shares_cell_with("#water")
        .texture("base:log_oak")
        .on_use(upgrade_campfire_base)
        .outline([0.0, 0.0, 0.0], [16.0, 3.0, 16.0])
        .hitbox([0.0, 0.0, 0.0], [16.0, 3.0, 16.0])
        .tab(&FUNCTIONAL);

    do_brick_behaviour(
        r.block(&RAW_CLAY_SINGLE_BRICK)
            .tag("placable_brick_convertable")
            .shares_cell_with("#water")
            .texture("base:clay")
            .grows(120.0, 0)
            .item_texture("raw_clay_brick")
            .on_grow(on_brick_grown)
            //.sound_group("base:block.clay")
            .hardness(0.01),
    );

    do_brick_behaviour(
        r.block(&SINGLE_BRICK)
            .shares_cell_with("#water")
            .texture("single_brick")
            .sound_group("base:block.stone")
            .item_texture("brick")
            .hardness(0.01),
    );

    // r.edit_pattern::<kind::Block>("#ores/iron")
    //     .replaceable()
    //     .mined_by("pickaxe", tool_tier::WOOD);
}

fn upgrade_campfire_base(u: &mut Use) -> bool {
    let (w, pos, player) = (u.world(), u.pos(), u.player());
    let slot = w.selected_slot(player);
    let held = w.inventory(player).get(slot);

    let places = item_info(held.item).and_then(|i| i.block);
    if Some(block_of(w.block(pos))) == places {
        w.set_block(pos, CAMPFIRE.id());
        w.inventory(player).set(slot, consume_one(held));
        return true;
    }
    false
}

fn do_brick_behaviour(block: BlockBuilder) {
    block
        .tab(&INGREDIENTS)
        .prop("x", 1..=8)
        .prop("y", 1..=8)
        .prop("z", 1..=8)
        .cull_same()
        .states(|p: &Props, s| {
            let x = p.get("x").parse::<f32>().unwrap() * 2.0;
            let y = p.get("y").parse::<f32>().unwrap() * 2.0;
            let z = p.get("z").parse::<f32>().unwrap() * 2.0;
            s.model_box([x - 2.0, y - 2.0, z - 2.0], [x + 2.0, y + 2.0, z + 4.0])
        })
        .placement(|p, props| {
            let hit = p.hit();
            props.set("x", &(hit.x * 8.0).floor().to_string());
            props.set("y", &((hit.y * 8.0) + 1.0).floor().to_string());
            props.set("z", &(hit.z * 8.0).floor().to_string());
        });
}

fn on_brick_grown(b: &mut BlockEvent) {
    let (w, pos, state) = (b.world(), b.pos(), b.state());
    let parts = w.parts(pos);

    if w.weather() != weathers::CLEAR.id() && w.sky_light(pos) == 15 {
        w.spawn_particles(&Particle::named("core:poof"), Vec3::new(pos.x as f64 + 0.5, pos.y as f64 + 0.5, pos.z as f64 + 0.5))
            .count(5)
            .outward()
            .velocity([0.0 as f32, 0. - 3.0 as f32, 0.0 as f32])
            .spread(0.1)
            .speed(2.0)
            .color(8029081);
        parts.iter().for_each(|&s| {
            if PLACABLE_BRICK_CONVERTABLE.on_block(s) && s == state {
                w.part_replace(pos, s, 0);
            }
        });
        return;
    }

    let part_offset = state - RAW_CLAY_SINGLE_BRICK.id();

    parts.iter().for_each(|&s| {
        if PLACABLE_BRICK_CONVERTABLE.on_block(s) && s == state {
            w.part_replace(pos, s, Block::named("single_brick").id() + part_offset);
        }
    });
}

fn break_grass(w: &World, b: &BlockChange) {
    let (pos, player) = (b.pos, b.player);
    let (inv, held_slot) = (w.inventory(player), w.selected_slot(player));
    let held = inv.get(held_slot);

    if KNIVES.on_item(held.item) {
        let left = DURABILITY
            .on_stack(&held)
            .unwrap_or(0)
            .saturating_sub(held.wear());
        let extra = left.min(1); // Lose 1 durability
        let next = held.add_wear(extra);
        inv.set(held_slot, next);

        w.spawn_item(Vec3::new(pos.x as f64, pos.y as f64, pos.z as f64), ItemStack { item: PLANT_FIBRE.id(), count: 1, data: 0 });
    }
}
