use base_api::tabs::{COMBAT, TOOLS};
use weft_sdk::*;

#[derive(Clone)]
pub struct ToolMaterial<'a> {
    material_name: &'a str,
    axe_enabled: bool,
    pickaxe_enabled: bool,
    hoe_enabled: bool,
    shovel_enabled: bool,
    sword_enabled: bool,
    knife_enabled: bool,
    level: u32,
    damage: f32,
    hoe_speed: f32,
    axe_speed: f32,
    axe_damage: f32,
    hoe_damage: f32,
    mining_speed: f32,
    durability: u32,
}

impl ToolMaterial<'static> {
    fn create_tool(&self, r: &mut Registry, kind: &str, damage: f32, speed: f32) {
        let b = r
            .item(Item::generated(format!("{}_{kind}", self.material_name)))
            .tool(kind, self.level)
            .mining_speed(self.mining_speed)
            .durability(self.durability)
            .attack_damage(damage)
            .attack_speed(speed)
            // Weapons wear by one per hit, tools (made for blocks) by two.
            .set(&WEAR_PER_HIT, if kind == "sword" { 1 } else { 2 })
            .tab(if kind == "sword" || kind == "knife" { &COMBAT } else { &TOOLS });
        if kind == "knife" {
            b.tag(&format!("knives/{}", self.material_name));
        } else {
            b.tag(&format!("{kind}s/{}", self.material_name));
        }
        if kind == "sword" || kind == "knife" {
            b.sweeps().no_creative_break();
        }
    }

    /// Allows this tool material to create a axe item.
    pub fn add_axe(mut self, axe_damage: f32, axe_speed: f32) -> ToolMaterial<'static> {
        self.axe_enabled = true;
        self.axe_damage = axe_damage;
        self.axe_speed = axe_speed;
        self
    }

    /// Allows this tool material to create a pickaxe item.
    pub fn add_pickaxe(mut self) -> ToolMaterial<'static> {
        self.pickaxe_enabled = true;
        self
    }

    /// Allows this tool material to create a hoe item. Note: Usually hoes deal 1 damage.
    pub fn add_hoe(mut self, hoe_damage: f32, hoe_speed: f32) -> ToolMaterial<'static> {
        self.hoe_enabled = true;
        self.hoe_speed = hoe_speed;
        self.hoe_damage = hoe_damage;
        self
    }

    /// Allows this tool material to create a shovel item.
    pub fn add_shovel(mut self) -> ToolMaterial<'static> {
        self.shovel_enabled = true;
        self
    }

    /// Allows this tool material to create all vanilla tools (does not include weapons).
    pub fn add_vanilla_tools(&self, hoe_damage: f32, hoe_speed: f32, axe_damage: f32, axe_speed: f32) -> ToolMaterial<'static> {
        let mut s = self.to_owned();
        s.shovel_enabled = true;
        s.hoe_enabled = true;
        s.hoe_speed = hoe_speed;
        s.hoe_damage = hoe_damage;
        s.pickaxe_enabled = true;
        s.axe_enabled = true;
        s.axe_damage = axe_damage;
        s.axe_speed = axe_speed;
        s
    }

    /// Allows this tool material to create all vanilla tools and weapons.
    pub fn add_all_vanilla(&self, hoe_damage: f32, hoe_speed: f32, axe_damage: f32, axe_speed: f32) -> ToolMaterial<'static> {
        self.add_vanilla_tools(hoe_damage, hoe_speed, axe_damage, axe_speed);
        self.to_owned().sword_enabled = true;
        self.to_owned()
    }

    /// Allows this tool material to create all vanilla and modded tools and weapons.
    pub fn add_all(&self, hoe_damage: f32, hoe_speed: f32, axe_damage: f32, axe_speed: f32) -> ToolMaterial<'static> {
        let mut s = self.to_owned();
        s.add_vanilla_tools(hoe_damage, hoe_speed, axe_damage, axe_speed);
        s.sword_enabled = true;
        s.knife_enabled = true;
        s
    }

    /// Allows this tool material to create a sword item.
    pub fn add_sword(mut self) -> ToolMaterial<'static> {
        self.sword_enabled = true;
        self
    }

    /// Allows this tool material to create a knife item.
    pub fn add_knife(mut self) -> ToolMaterial<'static> {
        self.knife_enabled = true;
        self
    }

    /// Registers every enabled item into the item registry.
    pub fn register(self, r: &mut Registry) {
        if self.axe_enabled {
            self.create_tool(r, "axe", self.axe_damage, self.axe_speed);
        }
        if self.pickaxe_enabled {
            self.create_tool(r, "pickaxe", 2.0 + self.damage, 1.0);
        }
        if self.hoe_enabled {
            self.create_tool(r, "hoe", self.hoe_damage, self.hoe_speed);
        }
        if self.shovel_enabled {
            self.create_tool(r, "shovel", 2.5 + self.damage, 1.0);
        }
        if self.sword_enabled {
            self.create_tool(r, "sword", 4.0 + self.damage, 1.6);
        }
        if self.knife_enabled {
            self.create_tool(r, "knife", 1.5 + self.damage, 2.0);
        }
    }
}

/// Creates a tool material helper. Use this when adding new tool material sets.
pub fn create_tool_material(material: &'static str, level: u32, damage: f32, mining_speed: f32, durability: u32) -> ToolMaterial<'static> {
    return ToolMaterial {
        material_name: material,
        axe_enabled: false,
        pickaxe_enabled: false,
        hoe_enabled: false,
        shovel_enabled: false,
        sword_enabled: false,
        knife_enabled: false,
        level: level,
        damage: damage,
        hoe_damage: 1.0,
        axe_damage: damage,
        axe_speed: 0.0,
        hoe_speed: 0.0,
        mining_speed: mining_speed,
        durability: durability,
    };
}
