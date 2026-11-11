pub use base_api::{CRAFTING, SMELTING, WOODCUTTING};
pub use slow_start_api::{AIR_DRYING, BRICK_STOVE_RECIPE, CAMPFIRE_RECIPE};
use weft_sdk::*;

pub fn register(r: &mut Registry) {
    r.recipe_kind(&AIR_DRYING).inputs(1, 1).outputs(1, 1);
    r.recipe_kind(&CAMPFIRE_RECIPE).inputs(1, 1).outputs(1, 1);
    r.recipe_kind(&BRICK_STOVE_RECIPE)
        .inputs(1, 1)
        .outputs(1, 1);

    // Disable vanilla logs -> planks and crafting table recipes
    r.remove(&Recipe::named("base:crafting/crafting_table"));
    r.remove(&Recipe::named("base:crafting/#planks/{m}"));
    r.remove(&Recipe::named("base:woodcutter"));
    r.remove(&Recipe::named("beauty_of_nature:crafting/cobblestone"));

    if r.has_mod("tetanus") {
        r.remove(&Recipe::named("tetanus:empty_leather_canteen"));
        r.shaped(&CRAFTING, "tetanus:empty_leather_canteen", 1, &["##", "##"], &[('#', "#materials/leather")]);
    }

    // Add new recipes
    r.shaped(&CRAFTING, "base:bricks", 1, &["##", "##"], &[('#', "single_brick")]);
    r.shaped(&CRAFTING, "base:woodcutter", 1, &["ii", "##"], &[('#', "#logs"), ('i', "#materials/iron")]);
    r.shaped(
        &CRAFTING,
        "base:crafting_table",
        1,
        &["il", "##"],
        &[('#', "#planks"), ('i', "#materials/iron"), ('l', "#materials/leather")],
    );
    r.shapeless(&CRAFTING, "raw_clay_single_brick", 2, &["base:clay_ball", "base:clay_ball", "base:clay_ball"], 0);
    r.shaped(&CRAFTING, "campfire_base", 1, &["ss", "ss"], &[('s', "#stick")]);
    r.shaped(&CRAFTING, "brick_stove", 1, &["ss", "cs"], &[('s', "base:bricks"), ('c', "campfire_base")]);
    //r.shaped(&CRAFTING, "campfire_complete", 1, &["s", "s"], &[('s', "campfire_base")]);
    r.shaped(&CRAFTING, "flint_knife", 1, &["s", "#"], &[('#', "#stick"), ('s', "base:flint")]);
    r.shaped(&CRAFTING, "flint_axe", 1, &["s#", "sp"], &[('#', "#stick"), ('s', "base:flint"), ('p', "#plant_string")]);
    r.shaped(&CRAFTING, "flint_pickaxe", 1, &["ss", "#p"], &[('#', "#stick"), ('s', "base:flint"), ('p', "#plant_string")]);
    r.shaped(&CRAFTING, "flint_pickaxe", 1, &["ss", "p#"], &[('#', "#stick"), ('s', "base:flint"), ('p', "#plant_string")]);

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

    //r.shapeless(&CAMPFIRE_RECIPE, "single_brick", 1, &["raw_clay_single_brick"], 600);
    r.shapeless(&CAMPFIRE_RECIPE, "#cooked_meat/{m}", 1, &["#raw_meat/{m}"], 600);
    r.shapeless(&SMELTING, "single_brick", 1, &["raw_clay_single_brick"], 200);

    r.shapeless(&BRICK_STOVE_RECIPE, "base:iron_ingot", 1, &["base:raw_iron"], 1200);
    r.shapeless(&BRICK_STOVE_RECIPE, "base:gold_ingot", 1, &["base:raw_gold"], 1200);
    r.shapeless(&BRICK_STOVE_RECIPE, "single_brick", 1, &["raw_clay_single_brick"], 500);

    r.shapeless(&WOODCUTTING, "#planks/{m}", 4, &["#logs/{m}"], 0);
}
