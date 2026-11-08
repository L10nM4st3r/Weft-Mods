mod blocks;
mod crafting;
mod items;

use weft_sdk::*;

declare_mod!(deps: ["base", "beauty_of_nature"], init: init);
static COW: EntityKind = EntityKind::named("base:cow");

fn init(r: &mut Registry) {
    blocks::register(r);
    crafting::register(r);
    items::register(r);

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
