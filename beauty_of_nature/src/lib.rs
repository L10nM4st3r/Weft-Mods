mod blocks;
mod blocks_crops;
mod blocks_rocks;
mod recipes;
mod worldgen;

use weft_sdk::*;

declare_mod!(deps: ["base"], init: init);

fn init(r: &mut Registry) {
    blocks::register(r);
    blocks_rocks::register(r);
    blocks_crops::register(r);
    worldgen::register(r);
    recipes::register(r);
}
