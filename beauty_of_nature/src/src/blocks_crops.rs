use base_api::tabs::{FOOD, NATURAL};
use base_api::{FLAMMABLE, FOOD as FOOD_VALUE, Food};
use weft_sdk::*;

/// Each crop: a wild plant found in the world, the crop it is planted as on farmland, and the
/// food both give.
const CROPS: [&str; 3] = ["carrot", "potato", "onion"];

pub fn register(r: &mut Registry) {
    for name in CROPS {
        let (wild, crop) = (Block::generated(format!("wild_{name}")), Block::generated(format!("{name}_crop")));
        r.block(wild)
            .sound_group("base:block.grass")
            .plant()
            .hitbox([5.0, 0.0, 5.0], [11.0, 10.0, 11.0])
            .needs_below("#plant_soil")
            .set(&FLAMMABLE, 15)
            .tag("plant")
            .shares_cell_with("#water")
            .tag("wild_crop")
            .tab(&NATURAL);
        r.loot(wild).pool(Pool::new().entry(Entry::item(name).count(1, 3)));

        r.block(crop)
            .sound_group("base:block.grass")
            .prop("age", 0..=6)
            .states(move |p, s| s.texture(&format!("{name}_stage_{}", p.get("age"))))
            .plant()
            .shares_cell_with("#water")
            .needs_below("#crop_soil")
            .grows(205.0, 9)
            .set(&FLAMMABLE, 15)
            .no_item();
        // Unripe it gives back what planted it, ripe up to three more.
        r.loot(crop)
            .pool(Pool::new().entry(Entry::item(name)))
            .pool(
                Pool::new()
                    .when(Cond::state("age=6"))
                    .entry(Entry::item(name).count(1, 3)),
            );

        r.item(Item::generated(name.into()))
            .set(&FOOD_VALUE, Food { hunger: 3, saturation: 2.0 })
            .use_ticks(32)
            .on_use(base_api::eat)
            .places(crop)
            .tag(&format!("crops/{name}"))
            .tab(&FOOD);
    }
}
