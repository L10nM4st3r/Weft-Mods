mod blocks;
mod brick_stove;
mod campfire;
mod crafting;
mod drying_rack;
mod items;

use weft_sdk::*;

declare_mod!(deps: ["base", "beauty_of_nature"], init: init);
static COW: EntityKind = EntityKind::named("base:cow");

fn init(r: &mut Registry) {
    blocks::register(r);
    crafting::register(r);
    drying_rack::register(r);
    campfire::register(r);
    items::register(r);
    brick_stove::register(r);

    r.edit(&COW.loot())
        .clear()
        .pool(
            Pool::new().entry(
                Entry::item("base:beef")
                    .count(1, 3)
                    .smelted(&base_api::SMELTING),
            ),
        )
        .pool(Pool::new().items("animal_hide", 0, 2));
}
