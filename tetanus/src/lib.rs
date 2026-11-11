//use base_api::{FOOD, HUNGER};
use base_api::{CRAFTING, FLATLANDS, OVERWORLD, tabs::TOOLS};
use tetanus_api::{
    CANTEEN, CANTEEN_OTHER, Canteen, DEFAULT_BIOME, TEMPERATURE, TETANUS_BIOME, TETANUS_BLOCK_LIQUID, TETANUS_DIMENSION, THIRST,
    Temperature, TetanusBiome, TetanusBlockLiquid, TetanusDimension, Thirst,
};
use weft_sdk::{comps::dimension::DAY_LENGTH, *};

declare_mod!(deps: ["base"], init: init);

static HUD_THIRST: HudElement = HudElement::named("thirst");
static THIRST_RATE: Attribute = Attribute::named("thirst_rate");
static THIRSTY: Condition = Condition::named("thirsty");
static COLD: Condition = Condition::named("cold");

static HUD_TEMPERATURE: HudElement = HudElement::named("temperature");
static DEBUG_PANEL: HudElement = HudElement::named("debug");
/// Speed of which temperature moves towards the environmental temperature.
static TEMPERATURE_RATE: Attribute = Attribute::named("temperature_rate");

pub static DEHYDRATE: DamageCause = DamageCause::named("tetanus:dehydrate");
pub static HEAT_EXHAUSTION: DamageCause = DamageCause::named("tetanus:heat_exhaustion");
pub static INCINERATED: DamageCause = DamageCause::named("tetanus:incinerated");
pub static MELTED: DamageCause = DamageCause::named("tetanus:melted");
pub static HYPOTHERMIA: DamageCause = DamageCause::named("tetanus:hypothermia");

pub static EMPTY_LEATHER_CANTEEN: Item = Item::named("empty_leather_canteen");
pub static FILLED_LEATHER_CANTEEN: Item = Item::named("filled_leather_canteen");

pub static THIRST_ENABLED: Config<bool> = Config::named("thirst_enabled");
pub static TEMPERATURE_ENABLED: Config<bool> = Config::named("temperature_enabled");
pub static TEMPERATURE_UPDATE_FREQUENCY: Config<u32> = Config::named("temperature_update_frequency");

pub static PLAINS: Biome = Biome::named("base:plains");
pub static FOREST: Biome = Biome::named("base:forest");
pub static BIRCH_FOREST: Biome = Biome::named("base:birch_forest");
pub static TAIGA: Biome = Biome::named("base:taiga");
pub static SNOWY: Biome = Biome::named("base:snowy_plains");
pub static DESERT: Biome = Biome::named("base:desert");
pub static MOUNTAINS: Biome = Biome::named("base:mountains");
pub static OCEAN: Biome = Biome::named("base:ocean");
pub static BEACH: Biome = Biome::named("base:beach");
pub static DEEP_CAVES: Biome = Biome::named("base:deep_caves");

// TODO: Make thirst damage deal damage every 20 ticks
// Make campfires deal damage as well as set mob on fire

fn init(r: &mut Registry) {
    // OVERWORLD
    // FLATLANDS
    r.config(&THIRST_ENABLED).bool(true).world();
    r.config(&TEMPERATURE_ENABLED).bool(true).world();
    r.config(&TEMPERATURE_UPDATE_FREQUENCY)
        .server()
        .int(60, 5, 100)
        .step(5.0);

    r.item(&EMPTY_LEATHER_CANTEEN)
        .tab(&TOOLS)
        .max_stack(1)
        .use_ticks(32)
        .set_text(&CANTEEN_OTHER, "tetanus:filled_leather_canteen")
        .on_use(tetanus_api::use_empty_canteen);

    r.item(&FILLED_LEATHER_CANTEEN)
        .no_tab()
        .max_stack(1)
        .durability(8)
        .use_ticks(32)
        .set(&CANTEEN, Canteen { thirst: 4, wet_throat: 3.0 })
        .set_text(&CANTEEN_OTHER, "tetanus:empty_leather_canteen")
        .on_use(tetanus_api::drink_canteen);

    r.shaped(&CRAFTING, "empty_leather_canteen", 1, &[" # ", "# #", " # "], &[('#', "#materials/leather")]);

    r.damage_cause(&DEHYDRATE)
        .bypasses_armor()
        .ignores_cooldown()
        .no_knockback()
        .death_message("death.tetanus.dehydrate");
    r.damage_cause(&HEAT_EXHAUSTION)
        .bypasses_armor()
        .ignores_cooldown()
        .no_knockback()
        .death_message("death.tetanus.heat_exhaustion");
    r.damage_cause(&INCINERATED)
        .bypasses_armor()
        .burning()
        .ignores_cooldown()
        .no_knockback()
        .death_message("death.tetanus.incinerated");
    r.damage_cause(&MELTED)
        .bypasses_armor()
        .ignores_cooldown()
        .no_knockback()
        .death_message("death.tetanus.melted");
    r.damage_cause(&HYPOTHERMIA)
        .bypasses_armor()
        .ignores_cooldown()
        .no_knockback()
        .death_message("death.tetanus.hypothermia");

    r.edit::<kind::Dimension>(&OVERWORLD).set(
        &TETANUS_DIMENSION,
        TetanusDimension {
            temperature: [15.0, 10.0],
            underground_temperature: 60.0,
            underground_height: [40.0, -512.0],
            deep_underground_temperature: 0.0,
            deep_underground_height: [f32::MIN, f32::MIN],
            elevated_temperature: -10.0,
            elevated_height: [80.0, 512.0],
        },
    );
    r.edit::<kind::Dimension>(&FLATLANDS).set(
        &TETANUS_DIMENSION,
        TetanusDimension {
            temperature: [15.0, 10.0],
            underground_temperature: 0.0,
            underground_height: [f32::MIN, f32::MIN],
            deep_underground_temperature: 0.0,
            deep_underground_height: [f32::MIN, f32::MIN],
            elevated_temperature: -10.0,
            elevated_height: [80.0, 512.0],
        },
    );

    r.edit_pattern::<kind::Block>("#water").set(
        &TETANUS_BLOCK_LIQUID,
        TetanusBlockLiquid { temperature: -10.0, max_temperature: f32::MAX, min_temperature: -f32::MAX, temperature_limits_forced: false },
    );

    r.edit_pattern::<kind::Block>("#lava").set(
        &TETANUS_BLOCK_LIQUID,
        TetanusBlockLiquid { temperature: 180.0, max_temperature: f32::MAX, min_temperature: 180.0, temperature_limits_forced: true },
    );

    r.edit::<kind::Biome>(&PLAINS)
        .set(&TETANUS_BIOME, TetanusBiome { temperature: [5.0; 2], underground: false, water_is_clean: true });
    r.edit::<kind::Biome>(&FOREST)
        .set(&TETANUS_BIOME, TetanusBiome { temperature: [4.0; 2], underground: false, water_is_clean: true });
    r.edit::<kind::Biome>(&BIRCH_FOREST)
        .set(&TETANUS_BIOME, TetanusBiome { temperature: [4.0; 2], underground: false, water_is_clean: true });
    r.edit::<kind::Biome>(&TAIGA)
        .set(&TETANUS_BIOME, TetanusBiome { temperature: [0.0; 2], underground: false, water_is_clean: true });
    r.edit::<kind::Biome>(&SNOWY)
        .set(&TETANUS_BIOME, TetanusBiome { temperature: [-30.0, -40.0], underground: false, water_is_clean: false });
    r.edit::<kind::Biome>(&DESERT)
        .set(&TETANUS_BIOME, TetanusBiome { temperature: [45.0, -30.0], underground: false, water_is_clean: true });
    r.edit::<kind::Biome>(&MOUNTAINS)
        .set(&TETANUS_BIOME, TetanusBiome { temperature: [-2.0; 2], underground: false, water_is_clean: false });
    r.edit::<kind::Biome>(&OCEAN)
        .set(&TETANUS_BIOME, TetanusBiome { temperature: [15.0, -3.0], underground: false, water_is_clean: false });
    r.edit::<kind::Biome>(&BEACH)
        .set(&TETANUS_BIOME, TetanusBiome { temperature: [5.0, -2.0], underground: false, water_is_clean: false });

    r.entity_component(
        &TEMPERATURE,
        Temperature {
            body_temperature: 20.0,
            environmental_temperature: 0.0,
            update_tick: 255,
            arrow_frame: 0,
            arrow_direction: 0,
            overlay_frame: 0,
        },
        component_flags::SAVE | component_flags::SYNC_OWNER,
    );
    r.entity_component(
        &THIRST,
        Thirst { thirst: 20.0, wet_throat: 5.0, thirst_exhaustion: 0.0, damage_tick: 0 },
        component_flags::SAVE | component_flags::SYNC_OWNER,
    );

    // Too hungry to sprint: modern's 3 drumsticks.
    r.condition(&THIRSTY, |c| {
        let w = c.world();
        w.is_some_and(|w| {
            w.component(c.entity(), &THIRST)
                .is_some_and(|h| h.thirst <= 6.0)
        })
    });
    r.condition(&COLD, |c| {
        let w = c.world();
        w.is_some_and(|w| {
            w.component(c.entity(), &TEMPERATURE)
                .is_some_and(|h| h.body_temperature <= -10.0)
        })
    });

    r.attribute(&THIRST_RATE, 1.0, 0.0, 10.0).rule();
    r.attribute(&TEMPERATURE_RATE, 1.0, 0.0, 10.0).rule();
    r.edit(&PLAYER)
        .attach(&THIRST)
        .attach(&TEMPERATURE)
        .modifier(Modifier::add(&attribute::CAN_SPRINT, -1.0).when(&THIRSTY))
        .modifier(Modifier::add(&attribute::MOVEMENT_SPEED, 0.8).when(&COLD));

    r.world_tick(tick_players);
    r.on_event(&actions::BREAK_BLOCK, |b| {
        let w = b.world();
        for e in b.iter() {
            tire(&w, e.player, 0.005);
        }
    });

    r.hud_element(&HUD_TEMPERATURE, hud_anchor::ABOVE_HOTBAR, -1, |ui: &mut Ui| {
        ui.text_bind_styled(S::row().size(32.0, 5.0).scale(0.75).center(), |ui: &ClientView, text_out: &mut TextOut| {
            if ui.outside_survival() || !TEMPERATURE_ENABLED.get() {
                text_out.text("");
                return;
            }

            //ui.component_of(ui.player(), &TEMPERATURE)
            let Some(temperature) = ui.component(&TEMPERATURE) else { return };

            let environmental_temperature: &str;
            let body_temperature: &str;

            if temperature.environmental_temperature < -10.0 {
                environmental_temperature = "freezing";
            } else if temperature.environmental_temperature < 10.0 {
                environmental_temperature = "cold";
            } else if temperature.environmental_temperature > 35.0 {
                environmental_temperature = "hot";
            } else if temperature.environmental_temperature > 28.0 {
                environmental_temperature = "warm";
            } else {
                environmental_temperature = "normal";
            }

            if temperature.body_temperature < -2.0 {
                body_temperature = "freezing";
            } else if temperature.body_temperature < 8.0 {
                body_temperature = "cold";
            } else if temperature.body_temperature > 36.0 {
                body_temperature = "hot";
            } else if temperature.body_temperature > 28.0 {
                body_temperature = "warm";
            } else {
                body_temperature = "normal";
            }

            if temperature.overlay_frame > 0 && (body_temperature == "hot" || body_temperature == "freezing") {
                text_out.set(
                    &format!(
                        "hud.tetanus.{}_{}_overlay_{}",
                        body_temperature,
                        environmental_temperature,
                        (temperature.overlay_frame / 10) + 1
                    ),
                    &[],
                );
            } else {
                text_out.set(&format!("hud.tetanus.{}_{}", body_temperature, environmental_temperature), &[]);
            }
        });
    });

    r.hud_element(&HUD_THIRST, hud_anchor::HOTBAR_RIGHT, 0, |ui: &mut Ui| {
        ui.icons("thirst", 10, |c| {
            if c.outside_survival() || !THIRST_ENABLED.get() { -1.0 } else { c.component(&THIRST).map_or(20.0, |h| h.thirst.ceil()) }
        })
        .from_right()
        // With no saturation left the food shakes, harder when it is nearly gone.
        .shaking_by(|c| match c.component(&THIRST) {
            Some(h) if h.wet_throat <= 0.0 => {
                if h.thirst <= 6.0 {
                    2.0
                } else {
                    0.5
                }
            }
            _ => 0.0,
        })
        .overlay(|c| c.component(&THIRST).map_or(0.0, |h| h.wet_throat.ceil()));
        // What the food in hand would fill, fading in and out.
        // .preview(1.6, |c| {
        //     let held = c
        //         .player_state()
        //         .map_or(ItemStack::EMPTY, |s| c.inventory_slot(s.selected));
        //     match (c.component(&THIRST), DRINK.on_item(held.item)) {
        //         (Some(h), Some(f)) => (h.thirst + f.thirst as f32).min(20.0),
        //         _ => 0.0,
        //     }
        // });
    });

    r.hud_element(&DEBUG_PANEL, hud_anchor::DEBUG_LEFT, 0, |ui| {
        ui.text_bind(|c, out| {
            if !TEMPERATURE_ENABLED.get() {
                out.text("");
                return;
            }
            let Some(temperature) = c.component(&TEMPERATURE) else { return };
            out.set(
                &format!("tetanus.debug_temperature"),
                &[
                    Arg::Float(temperature.body_temperature as f64, 1),
                    Arg::Float(temperature.environmental_temperature as f64, 1),
                    Arg::Int(temperature.overlay_frame as i64 / 10 + 1),
                ],
            );
        });
    });

    r.command("hydrate", "[thirst:int]", |cmd| {
        let food = cmd.int("thirst").unwrap_or(20).clamp(0, 20) as f32;
        if let Some(h) = cmd.world().component_mut(cmd.player(), &THIRST) {
            h.thirst = food;
            //h.wet_throat = h.wet_throat.min(food);
        }
        cmd.reply("command.tetanus.hydrate", &[Arg::Int(food as i64)]);
    });

    r.command("temperature", "[temperature:int]", |cmd| {
        let temperature = cmd.int("temperature").unwrap_or(20) as f32;
        if let Some(h) = cmd.world().component_mut(cmd.player(), &TEMPERATURE) {
            h.body_temperature = temperature;
        }
        cmd.reply("command.tetanus.temperature", &[Arg::Int(temperature as i64)]);
    });
}

fn get_local_temperature(w: &World, pos: BlockPos) -> f32 {
    let (biome, dimension) = (w.biome(pos), w.dimension());
    let biome_data_option = TETANUS_BIOME.on_biome(biome);

    let biome_data: TetanusBiome;
    if biome_data_option.is_none() {
        biome_data = DEFAULT_BIOME;
    } else {
        biome_data = biome_data_option.unwrap();
    }

    let Some(dimension_data) = TETANUS_DIMENSION.on_dimension(dimension) else { return 0.0 };
    let day_length = DAY_LENGTH.on_dimension(dimension).unwrap_or(24000) as u64;

    // Time of day temperature
    let time_of_day = wrap_u64(w.day_time(), 0, day_length);
    let is_day = time_of_day < (day_length as f32 * 0.55) as u64;

    let mut temperature = if is_day {
        dimension_data.temperature[0] + biome_data.temperature[0]
    } else {
        dimension_data.temperature[1] + biome_data.temperature[1]
    };

    // Elevation temperature
    temperature += linear(pos.y as f32, dimension_data.elevated_height, [0.0, dimension_data.elevated_temperature]);

    // Underground temperature
    temperature = linear(pos.y as f32, dimension_data.underground_height, [temperature, dimension_data.underground_temperature]);

    // Underground temperature
    temperature = linear(pos.y as f32, dimension_data.deep_underground_height, [temperature, dimension_data.deep_underground_temperature]);

    if pos.y <= dimension_data.underground_height[0] as i32 && biome_data.underground {
        temperature += biome_data.temperature[1];
    }

    // Block contributions to temperature
    let parts = w.parts(pos);
    parts.iter().for_each(|&s| {
        let has_comp = TETANUS_BLOCK_LIQUID.on_block(s);
        if !has_comp.is_none() {
            let comp = has_comp.unwrap();
            let max = if comp.temperature_limits_forced { comp.max_temperature.max(temperature) } else { comp.max_temperature };
            let min = if comp.temperature_limits_forced { comp.min_temperature.min(temperature) } else { comp.min_temperature };
            temperature = (temperature + comp.temperature).clamp(min, max);
        }
    });

    temperature
}

fn wrap_u64(value: u64, min: u64, max: u64) -> u64 {
    min + (value - min).rem_euclid(max - min)
}

fn linear(current_point: f32, points: [f32; 2], values: [f32; 2]) -> f32 {
    if points[1] == 0.0 || points[0] == 0.0 || (points[1] - points[0]) == 0.0 {
        return values[0];
    }
    if current_point < points[0] && points[0] < points[1] {
        return values[0];
    }
    if current_point > points[1] && points[0] < points[1] {
        return values[1];
    }

    // -200, -9000
    if points[0] > points[1] {
        if current_point > points[0] {
            return values[0];
        }
        if current_point < points[1] {
            return values[1];
        }
    }

    let t = (current_point - points[0]) / (points[1] - points[0]);
    values[0] + t * (values[1] - values[0])
}

fn tire(w: &World, player: u64, amount: f32) {
    if !THIRST_ENABLED.get() {
        return;
    }

    if let Some(h) = w
        .component_mut(player, &THIRST)
        .filter(|_| !w.outside_survival(player))
    {
        h.thirst_exhaustion += amount / 3.0;
    }
}

fn tick_players(w: &World) {
    for id in w.players() {
        let Some(st) = w.entity(id) else { continue };

        if TEMPERATURE_ENABLED.get() {
            tick_player_temperature(w, id, st);
        }

        if w.outside_survival(id) {
            continue;
        }

        if THIRST_ENABLED.get() {
            tick_player_thirst(w, id, st);
        }
    }
}

fn tick_player_thirst(w: &World, id: u64, st: EntityState) {
    //heal_or_starve(w, id, &mut st);
    // Jumping tires by a landing's worth of pulling yourself up.
    if st.jumped != 0 {
        tire(w, id, if st.sprinting != 0 { 0.2 } else { 0.05 });
    }
    tire(w, id, 0.005);

    let mut per_point = 4.0 / w.attribute(id, &THIRST_RATE) as f32;

    if TEMPERATURE_ENABLED.get() && !w.component_mut(id, &TEMPERATURE).is_none() {
        let Some(temperature) = w.component_mut(id, &TEMPERATURE) else { return };
        per_point -= ((temperature.body_temperature - 30.0) / 150.0 as f32).clamp(0.0, per_point - 0.01);
    }
    let Some(h) = w.component_mut(id, &THIRST) else { return };

    // Modern's: 0.1 a block sprinting, 0.01 swimming, 0 walking; jumps are counted from `Move`.
    h.thirst_exhaustion += st.moved
        * if st.sprinting != 0 {
            0.1
        } else if st.in_liquid != 0 {
            0.01
        } else {
            0.0
        };
    if h.thirst_exhaustion > per_point {
        h.thirst_exhaustion -= per_point;
        if h.wet_throat > 0.0 {
            h.wet_throat = (h.wet_throat - 1.0).max(0.0);
        } else {
            h.thirst = (h.thirst - 1.0).max(0.0);
        }
    }

    if h.thirst <= 0.0 {
        h.damage_tick += 1;
        if h.damage_tick >= 35 {
            h.damage_tick = 0;
            w.damage(id, Damage::new(1.0, &DEHYDRATE));
        }
    }
}

fn tick_player_temperature(w: &World, id: u64, st: EntityState) {
    let Some(temperature) = w.component_mut(id, &TEMPERATURE) else { return };
    temperature.update_tick += 1;

    if temperature.update_tick >= TEMPERATURE_UPDATE_FREQUENCY.get() as u16 {
        temperature.update_tick = 0;
        temperature.environmental_temperature =
            get_local_temperature(w, BlockPos { x: st.pos.x as i32, y: st.pos.y as i32, z: st.pos.z as i32 });
    }

    let temperature_rate = w.attribute(id, &TEMPERATURE_RATE) as f32;

    if temperature.body_temperature < temperature.environmental_temperature {
        let difference = temperature.environmental_temperature - temperature.body_temperature;
        let move_temperature = temperature_rate.max(difference * 0.4).min(difference) / 100.0;
        temperature.body_temperature += move_temperature;
    } else if temperature.body_temperature > temperature.environmental_temperature {
        let difference = temperature.body_temperature - temperature.environmental_temperature;
        let move_temperature = temperature_rate.max(difference * 0.4).min(difference) / 100.0;
        temperature.body_temperature -= move_temperature;
    }

    if temperature.body_temperature > 35.0 {
        temperature.overlay_frame += 1 + ((temperature.body_temperature - 35.0) / 25.0).clamp(0.0, 10.0) as u8;

        if temperature.overlay_frame > 39 {
            if temperature.body_temperature > 600.0 {
                w.damage(id, Damage::new(10.0, &MELTED));
            } else if temperature.body_temperature > 100.0 {
                w.damage(id, Damage::new(5.0, &INCINERATED));
            } else {
                w.damage(id, Damage::new(1.0, &HEAT_EXHAUSTION));
            }
            temperature.overlay_frame = 1;
        }
    } else if temperature.body_temperature < -8.0 {
        temperature.overlay_frame += 1 + ((-temperature.body_temperature + 8.0) / 10.0).clamp(0.0, 10.0) as u8;

        if temperature.overlay_frame > 39 {
            w.damage(id, Damage::new(1.0, &HYPOTHERMIA));
            temperature.overlay_frame = 1
        }
    } else {
        temperature.overlay_frame = 0;
    }
}
