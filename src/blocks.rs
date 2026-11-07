use base_api::tabs::NATURAL;
use base_api::{FLAMMABLE, FUEL};
use weft_sdk::*;

pub static BRANCH: Block = Block::named("branch");
pub static STONE_PEBBLE: Block = Block::named("stone_pebble");

pub static BLUE_ORCHID: Block = Block::named("blue_orchid");
pub static CACTUS_FLOWER: Block = Block::named("cactus_flower");
pub static DEAD_BUSH: Block = Block::named("dead_bush");
pub static DRY_GRASS: Block = Block::named("dry_grass");
pub static RED_MUSHROOM: Block = Block::named("red_mushroom");
pub static BROWN_MUSHROOM: Block = Block::named("brown_mushroom");
pub static BROWN_SHELF_MUSHROOM: Block = Block::named("brown_shelf_mushroom");

static CACTUS: Block = Block::named("base:cactus");
static FARMLAND: Block = Block::named("base:farmland");
static HOES: Tag = Tag::named("tools/hoe");

/// Small, like a flower or a mushroom.
const SMALL: ([f32; 3], [f32; 3]) = ([5.0, 0.0, 5.0], [11.0, 10.0, 11.0]);
/// Bushy, like a dead bush.
const BUSHY: ([f32; 3], [f32; 3]) = ([2.0, 0.0, 2.0], [14.0, 12.0, 14.0]);

/// A plant on what `soil` takes (a block matcher; empty: anything solid).
fn plant<'a>(r: &'a mut Registry, handle: &'static Block, soil: &str, (min, max): ([f32; 3], [f32; 3])) -> BlockBuilder<'a> {
    r.block(handle)
        .sound_group("base:block.grass")
        .tag("plant")
        .plant()
        .holds_liquid()
        .hitbox(min, max)
        .needs_below(soil)
        .tab(&NATURAL)
}

pub fn register(r: &mut Registry) {
    // Lying on the ground, turned the way it was placed. Picked up as itself; burns and crafts
    // like a stick.
    r.block(&BRANCH)
        .sound_group("base:block.wood")
        .texture("fallen_branch")
        .on_sides(Facings::only(&[Dir::Down]))
        .facing(Facings::HORIZONTAL, Toward::Face)
        .hardness(0.0)
        .not_solid()
        .replaceable()
        .holds_liquid()
        .set(&FLAMMABLE, 15)
        .set(&FUEL, 100)
        .tag("branch")
        .tag("stick")
        .item_texture("fallen_branch")
        .tab(&NATURAL);

    r.block(&STONE_PEBBLE)
        .sound_group("base:block.stone")
        .texture("base:stone")
        .model_file("pebble")
        .hardness(0.1)
        .mined_by("pickaxe", tool_tier::HAND)
        .not_solid()
        .replaceable()
        .holds_liquid()
        .on_sides(Facings::only(&[Dir::Down]))
        .facing(Facings::HORIZONTAL, Toward::Face)
        .tag("pebble")
        .tab(&NATURAL);

    plant(r, &BLUE_ORCHID, "#plant_soil", SMALL)
        .texture("flower_blue_orchid")
        .tag("flower");
    plant(r, &CACTUS_FLOWER, "", BUSHY);
    plant(r, &DEAD_BUSH, "#desert_plant_soil", BUSHY);
    r.loot(&DEAD_BUSH)
        .pool(Pool::new().entry(Entry::item("base:stick").count(0, 2)));
    plant(r, &DRY_GRASS, "#desert_plant_soil", BUSHY).drops_nothing();
    plant(r, &RED_MUSHROOM, "", SMALL);
    plant(r, &BROWN_MUSHROOM, "", SMALL);

    // On the side of a trunk, sticking out of it: written on a trunk to the south.
    r.block(&BROWN_SHELF_MUSHROOM)
        .sound_group("base:block.wood")
        .hardness(0.2)
        .not_solid()
        .holds_liquid()
        .tag("shelf_mushroom")
        .facing(Facings::HORIZONTAL, Toward::Face)
        .model_box([3.0, 1.0, 11.0], [15.0, 3.0, 16.0])
        .model_box([1.0, 5.0, 11.0], [13.0, 7.0, 16.0])
        .model_box([2.0, 9.0, 11.0], [14.0, 11.0, 16.0])
        .needs(Dir::South, "")
        .tab(&NATURAL);

    // A hollow log of every log, any mod's.
    r.shapes_of("hollow_log", "#logs", "_hollow")
        .sound_group("base:block.wood")
        .model_file("hollow_log")
        .axis()
        .holds_liquid()
        .signal_blocker()
        .outline([0.0; 3], [16.0; 3])
        .tag("hollow_log")
        .tab(&NATURAL);

    // Cactus grows a flower once it is three tall.
    r.edit(&CACTUS)
        .holds_liquid()
        .on_grow(|e| grow_column(e, &CACTUS, 3, &CACTUS_FLOWER));

    // A hoe turns grass and dirt, any mod's tagged so, into farmland.
    r.tag("tillable", &["base:grass", "base:dirt"]);
    r.edit_pattern::<kind::Block>("#tillable").on_use(till);

    r.tag("desert_plant_soil", &["#plant_soil", "#cactus_soil"]);
}

fn till(u: &mut Use) -> bool {
    let (w, pos, player) = (u.world(), u.pos(), u.player());
    let held = w.inventory(player).get(w.selected_slot(player));
    if !HOES.on_item(held.item) || w.block([pos.x, pos.y + 1, pos.z]) != 0 {
        return false;
    }
    w.set_block(pos, FARMLAND.id());
    true
}

/// One more of `me` on top, `flower` instead once the column is `max` tall.
fn grow_column(e: &mut BlockEvent, me: &'static Block, max: i32, flower: &'static Block) {
    let (w, p) = (e.world(), e.pos());
    let above = [p.x, p.y + 1, p.z];
    if w.block(above) != 0 {
        return;
    }
    let mut height = 1;
    while height < max && me.is(w.block([p.x, p.y - height, p.z])) {
        height += 1;
    }
    w.set_block(above, if height < max { me.id() } else { flower.id() });
}
