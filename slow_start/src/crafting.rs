pub use base_api::CRAFTING;
pub use slow_start_api::AIR_DRYING;
use weft_sdk::*;

pub fn register(r: &mut Registry) {
    r.recipe_kind(&AIR_DRYING).inputs(1, 1).outputs(1, 1);

    // Disable vanilla logs -> planks and crafting table recipes
    r.remove(&Recipe::named("base:crafting/crafting_table"));
    r.remove(&Recipe::named("base:crafting/#planks/{m}"));
    r.remove(&Recipe::named("beauty_of_nature:crafting/cobblestone"));

    // Add new recipes
    r.shaped(&CRAFTING, "drying_rack", 1, &["ss", "##"], &[('#', "#logs"), ('s', "#stick")]);
    r.shaped(&CRAFTING, "base:cobblestone", 1, &["###", "# #", "###"], &[('#', "#stone_pebble")]);
    r.shapeless(&CRAFTING, "plant_string", 1, &["#plant_fibre", "#plant_fibre", "#plant_fibre"], 0);
    r.shapeless(&CRAFTING, "base:flint", 1, &["base:gravel", "base:gravel", "base:gravel"], 5);

    r.shapeless(&AIR_DRYING, "jerky", 1, &["base:porkchop"], 1000);
    r.shapeless(&AIR_DRYING, "jerky", 1, &["base:chicken"], 1000);
    r.shapeless(&AIR_DRYING, "jerky", 1, &["base:beef"], 1000);
    r.shapeless(&AIR_DRYING, "jerky", 1, &["base:mutton"], 1000);
    r.shapeless(&AIR_DRYING, "animal_hide", 1, &["base:rotten_flesh"], 1000);
    r.shapeless(&AIR_DRYING, "base:leather", 1, &["animal_hide"], 2000);
}
