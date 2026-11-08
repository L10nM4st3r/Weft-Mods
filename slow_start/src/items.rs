use base_api::tabs::{FOOD, INGREDIENTS};
use base_api::{FOOD as FOOD_VALUE, Food};
use weft_sdk::*;

pub static ANIMAL_HIDE: Item = Item::named("animal_hide");
pub static PLANT_FIBRE: Item = Item::named("plant_fibre");
pub static PLANT_STRING: Item = Item::named("plant_string");
pub static JERKY: Item = Item::named("jerky");

pub fn register(r: &mut Registry) {
    r.item(&ANIMAL_HIDE).tag("animal_hide").tab(&INGREDIENTS);
    r.item(&PLANT_FIBRE).tag("plant_fibre").tab(&INGREDIENTS);
    r.item(&PLANT_STRING).tag("plant_string").tab(&INGREDIENTS);
    r.item(&JERKY)
        .tag("dried_meat")
        .tab(&FOOD)
        .set(&FOOD_VALUE, Food { hunger: 4, saturation: 6.0 })
        .use_ticks(32)
        .on_use(base_api::eat);
}
