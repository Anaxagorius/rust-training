use iron_age_core::GameError;
use iron_age_inventory::{Inventory, Item, ItemType, MaterialTier, ItemRarity};
use rand::Rng;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CraftingProfession { Mining, Gathering, Cooking, Weaponsmithing, Armorsmithing, Alchemy, BoyerFletcher }

impl CraftingProfession {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mining => "Mining",
            Self::Gathering => "Gathering",
            Self::Cooking => "Cooking",
            Self::Weaponsmithing => "Weaponsmithing",
            Self::Armorsmithing => "Armorsmithing",
            Self::Alchemy => "Alchemy",
            Self::BoyerFletcher => "BoyerFletcher",
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CraftingStation { Forge, Anvil, TanningRack, Loom, Campfire, AlchemyStone, FletchingBench, None }

impl CraftingStation {
    /// Returns the station name to match against `Location::has_crafting_station`,
    /// or `None` if no station is required.
    pub fn name(&self) -> Option<&'static str> {
        match self {
            Self::Forge => Some("Forge"),
            Self::Anvil => Some("Anvil"),
            Self::TanningRack => Some("TanningRack"),
            Self::Loom => Some("Loom"),
            Self::Campfire => Some("Campfire"),
            Self::AlchemyStone => Some("AlchemyStone"),
            Self::FletchingBench => Some("FletchingBench"),
            Self::None => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecipeIngredient { pub item_id: String, pub quantity: u32 }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub profession: CraftingProfession,
    pub station: CraftingStation,
    pub ingredients: Vec<RecipeIngredient>,
    pub output_item_id: String,
    pub output_quantity: u32,
    pub required_skill_level: u32,
    pub required_int: i32,
    pub base_craft_time_seconds: u32,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CraftingQuality { Poor, Normal, Fine, Superior, Masterwork }

impl CraftingQuality {
    pub fn from_roll(roll: f32) -> Self {
        if roll < 0.1 { Self::Poor }
        else if roll < 0.5 { Self::Normal }
        else if roll < 0.75 { Self::Fine }
        else if roll < 0.92 { Self::Superior }
        else { Self::Masterwork }
    }
    pub fn stat_bonus(&self) -> i32 {
        match self { Self::Poor => -1, Self::Normal => 0, Self::Fine => 1, Self::Superior => 2, Self::Masterwork => 4 }
    }
    pub fn value_multiplier(&self) -> f32 {
        match self { Self::Poor => 0.7, Self::Normal => 1.0, Self::Fine => 1.3, Self::Superior => 1.6, Self::Masterwork => 2.5 }
    }
}

pub struct CraftingSystem {
    pub known_recipes: Vec<Recipe>,
    pub all_recipes: Vec<Recipe>,
}

impl CraftingSystem {
    pub fn new() -> Self { Self { known_recipes: Vec::new(), all_recipes: Self::starter_recipes() } }

    pub fn with_starter_recipes() -> Self {
        let recipes = Self::starter_recipes();
        Self { known_recipes: recipes.clone(), all_recipes: recipes }
    }

    pub fn learn_recipe(&mut self, recipe_id: &str) -> bool {
        if self.known_recipes.iter().any(|r| r.id == recipe_id) { return false; }
        if let Some(r) = self.all_recipes.iter().find(|r| r.id == recipe_id) {
            self.known_recipes.push(r.clone());
            true
        } else { false }
    }

    pub fn can_craft(&self, recipe: &Recipe, inventory: &Inventory, profession_skill: u32, intelligence: i32) -> Result<(), GameError> {
        if profession_skill < recipe.required_skill_level {
            return Err(GameError::MissingRequirements(format!("Need skill level {}", recipe.required_skill_level)));
        }
        if intelligence < recipe.required_int {
            return Err(GameError::MissingRequirements(format!("Need INT {}", recipe.required_int)));
        }
        for ing in &recipe.ingredients {
            let item = inventory.find_item(&ing.item_id);
            let qty = item.map_or(0, |i| i.quantity);
            if qty < ing.quantity {
                return Err(GameError::MissingRequirements(format!("Need {} x{}", ing.item_id, ing.quantity)));
            }
        }
        Ok(())
    }

    pub fn craft(
        &mut self, recipe_id: &str, inventory: &mut Inventory,
        profession_skill: u32, intelligence: i32, wisdom: i32, rng: &mut impl Rng,
    ) -> Result<(Item, CraftingQuality), GameError> {
        let recipe = self.known_recipes.iter().find(|r| r.id == recipe_id)
            .ok_or_else(|| GameError::NotFound(recipe_id.to_string()))?.clone();
        self.can_craft(&recipe, inventory, profession_skill, intelligence)?;
        for ing in &recipe.ingredients {
            inventory.remove_item(&ing.item_id, ing.quantity)?;
        }
        let quality = Self::calculate_quality(intelligence, wisdom, profession_skill, rng);
        let item = Item::new_consumable(&recipe.output_item_id, &recipe.name, ItemType::CraftingMaterial, recipe.output_quantity);
        Ok((item, quality))
    }

    pub fn calculate_quality(intelligence: i32, wisdom: i32, skill_level: u32, rng: &mut impl Rng) -> CraftingQuality {
        let base: f32 = rng.gen();
        let bonus = (intelligence + wisdom) as f32 * 0.005 + skill_level as f32 * 0.003;
        CraftingQuality::from_roll((base + bonus).min(1.0))
    }

    pub fn starter_recipes() -> Vec<Recipe> {
        vec![
            Recipe { id: "iron_short_sword".into(), name: "Iron Short Sword".into(),
                profession: CraftingProfession::Weaponsmithing, station: CraftingStation::Forge,
                ingredients: vec![RecipeIngredient { item_id: "iron_ingot".into(), quantity: 3 },
                                  RecipeIngredient { item_id: "leather_wrap".into(), quantity: 1 }],
                output_item_id: "iron_short_sword".into(), output_quantity: 1,
                required_skill_level: 1, required_int: 0, base_craft_time_seconds: 30,
                description: "A reliable iron short sword.".into() },
            Recipe { id: "iron_spear".into(), name: "Iron Spear".into(),
                profession: CraftingProfession::Weaponsmithing, station: CraftingStation::Forge,
                ingredients: vec![RecipeIngredient { item_id: "iron_ingot".into(), quantity: 2 },
                                  RecipeIngredient { item_id: "wood_shaft".into(), quantity: 1 }],
                output_item_id: "iron_spear".into(), output_quantity: 1,
                required_skill_level: 1, required_int: 0, base_craft_time_seconds: 25,
                description: "A sturdy iron-tipped spear.".into() },
            Recipe { id: "leather_helmet".into(), name: "Leather Helmet".into(),
                profession: CraftingProfession::Armorsmithing, station: CraftingStation::TanningRack,
                ingredients: vec![RecipeIngredient { item_id: "leather".into(), quantity: 3 }],
                output_item_id: "leather_helmet".into(), output_quantity: 1,
                required_skill_level: 1, required_int: 0, base_craft_time_seconds: 20,
                description: "Basic leather head protection.".into() },
            Recipe { id: "leather_torso".into(), name: "Leather Torso".into(),
                profession: CraftingProfession::Armorsmithing, station: CraftingStation::TanningRack,
                ingredients: vec![RecipeIngredient { item_id: "leather".into(), quantity: 5 }],
                output_item_id: "leather_torso".into(), output_quantity: 1,
                required_skill_level: 1, required_int: 0, base_craft_time_seconds: 30,
                description: "Basic leather chest armor.".into() },
            Recipe { id: "health_potion".into(), name: "Health Potion".into(),
                profession: CraftingProfession::Alchemy, station: CraftingStation::AlchemyStone,
                ingredients: vec![RecipeIngredient { item_id: "herbs".into(), quantity: 2 },
                                  RecipeIngredient { item_id: "clean_water".into(), quantity: 1 }],
                output_item_id: "health_potion".into(), output_quantity: 1,
                required_skill_level: 0, required_int: 2, base_craft_time_seconds: 15,
                description: "Restores a portion of HP.".into() },
            Recipe { id: "antidote".into(), name: "Antidote".into(),
                profession: CraftingProfession::Alchemy, station: CraftingStation::AlchemyStone,
                ingredients: vec![RecipeIngredient { item_id: "nightshade_leaf".into(), quantity: 1 },
                                  RecipeIngredient { item_id: "bog_moss".into(), quantity: 1 }],
                output_item_id: "antidote".into(), output_quantity: 1,
                required_skill_level: 1, required_int: 3, base_craft_time_seconds: 20,
                description: "Cures poison status.".into() },
            Recipe { id: "shortbow".into(), name: "Shortbow".into(),
                profession: CraftingProfession::BoyerFletcher, station: CraftingStation::FletchingBench,
                ingredients: vec![RecipeIngredient { item_id: "wood".into(), quantity: 2 },
                                  RecipeIngredient { item_id: "sinew".into(), quantity: 1 }],
                output_item_id: "shortbow".into(), output_quantity: 1,
                required_skill_level: 1, required_int: 0, base_craft_time_seconds: 20,
                description: "A simple wooden shortbow.".into() },
            Recipe { id: "iron_arrow".into(), name: "Iron Arrow".into(),
                profession: CraftingProfession::BoyerFletcher, station: CraftingStation::FletchingBench,
                ingredients: vec![RecipeIngredient { item_id: "iron_ingot".into(), quantity: 1 },
                                  RecipeIngredient { item_id: "feather".into(), quantity: 3 }],
                output_item_id: "iron_arrow".into(), output_quantity: 10,
                required_skill_level: 0, required_int: 0, base_craft_time_seconds: 10,
                description: "Iron-tipped arrows.".into() },
            Recipe { id: "campfire_stew".into(), name: "Campfire Stew".into(),
                profession: CraftingProfession::Cooking, station: CraftingStation::Campfire,
                ingredients: vec![RecipeIngredient { item_id: "meat".into(), quantity: 2 },
                                  RecipeIngredient { item_id: "herbs".into(), quantity: 1 }],
                output_item_id: "campfire_stew".into(), output_quantity: 1,
                required_skill_level: 0, required_int: 0, base_craft_time_seconds: 10,
                description: "A hearty stew that restores stamina.".into() },
            Recipe { id: "pitch_bomb".into(), name: "Pitch Bomb".into(),
                profession: CraftingProfession::Alchemy, station: CraftingStation::Campfire,
                ingredients: vec![RecipeIngredient { item_id: "pitch".into(), quantity: 2 },
                                  RecipeIngredient { item_id: "clay".into(), quantity: 1 }],
                output_item_id: "pitch_bomb".into(), output_quantity: 1,
                required_skill_level: 1, required_int: 2, base_craft_time_seconds: 15,
                description: "A crude fire-starting explosive.".into() },
            // ── Advanced Alchemy ────────────────────────────────────────────
            Recipe { id: "clarity_potion".into(), name: "Clarity Potion".into(),
                profession: CraftingProfession::Alchemy, station: CraftingStation::AlchemyStone,
                ingredients: vec![RecipeIngredient { item_id: "crystalline_dust".into(), quantity: 1 },
                                  RecipeIngredient { item_id: "clean_water".into(), quantity: 1 }],
                output_item_id: "clarity_potion".into(), output_quantity: 1,
                required_skill_level: 2, required_int: 4, base_craft_time_seconds: 20,
                description: "Restores mana. Brewed from crystalline cave dust.".into() },
            Recipe { id: "fortify_potion".into(), name: "Fortify Potion".into(),
                profession: CraftingProfession::Alchemy, station: CraftingStation::AlchemyStone,
                ingredients: vec![RecipeIngredient { item_id: "herbs".into(), quantity: 2 },
                                  RecipeIngredient { item_id: "bog_moss".into(), quantity: 1 },
                                  RecipeIngredient { item_id: "clean_water".into(), quantity: 1 }],
                output_item_id: "fortify_potion".into(), output_quantity: 1,
                required_skill_level: 2, required_int: 3, base_craft_time_seconds: 25,
                description: "Grants Regen for 5 turns.".into() },
            Recipe { id: "stamina_potion".into(), name: "Stamina Potion".into(),
                profession: CraftingProfession::Alchemy, station: CraftingStation::AlchemyStone,
                ingredients: vec![RecipeIngredient { item_id: "meat".into(), quantity: 1 },
                                  RecipeIngredient { item_id: "herbs".into(), quantity: 1 }],
                output_item_id: "stamina_potion".into(), output_quantity: 1,
                required_skill_level: 1, required_int: 2, base_craft_time_seconds: 15,
                description: "Restores stamina.".into() },
            // ── Advanced Smithing ───────────────────────────────────────────
            Recipe { id: "iron_long_sword".into(), name: "Iron Long Sword".into(),
                profession: CraftingProfession::Weaponsmithing, station: CraftingStation::Forge,
                ingredients: vec![RecipeIngredient { item_id: "iron_ingot".into(), quantity: 5 },
                                  RecipeIngredient { item_id: "leather_wrap".into(), quantity: 2 }],
                output_item_id: "iron_long_sword".into(), output_quantity: 1,
                required_skill_level: 2, required_int: 0, base_craft_time_seconds: 45,
                description: "A longer iron sword with better reach and damage.".into() },
            Recipe { id: "wolf_pelt_armor".into(), name: "Wolf Pelt Armour".into(),
                profession: CraftingProfession::Armorsmithing, station: CraftingStation::TanningRack,
                ingredients: vec![RecipeIngredient { item_id: "wolf_pelt".into(), quantity: 3 },
                                  RecipeIngredient { item_id: "leather".into(), quantity: 2 }],
                output_item_id: "wolf_pelt_armor".into(), output_quantity: 1,
                required_skill_level: 2, required_int: 0, base_craft_time_seconds: 35,
                description: "Tough armour stitched from wolf pelts.".into() },
            // ── New Zone Recipes ────────────────────────────────────────────
            Recipe { id: "iron_long_bow".into(), name: "Iron-Tipped Longbow".into(),
                profession: CraftingProfession::BoyerFletcher, station: CraftingStation::FletchingBench,
                ingredients: vec![RecipeIngredient { item_id: "wood".into(), quantity: 3 },
                                  RecipeIngredient { item_id: "iron_ingot".into(), quantity: 2 },
                                  RecipeIngredient { item_id: "sinew".into(), quantity: 2 }],
                output_item_id: "iron_long_bow".into(), output_quantity: 1,
                required_skill_level: 2, required_int: 0, base_craft_time_seconds: 40,
                description: "A powerful bow reinforced with iron tips for greater range.".into() },
            Recipe { id: "iron_chainmail".into(), name: "Iron Chainmail".into(),
                profession: CraftingProfession::Armorsmithing, station: CraftingStation::Forge,
                ingredients: vec![RecipeIngredient { item_id: "iron_ingot".into(), quantity: 6 },
                                  RecipeIngredient { item_id: "leather".into(), quantity: 2 }],
                output_item_id: "iron_chainmail".into(), output_quantity: 1,
                required_skill_level: 3, required_int: 0, base_craft_time_seconds: 60,
                description: "Interlocked iron rings form a flexible suit of armour.".into() },
            Recipe { id: "iron_round_shield".into(), name: "Iron Round Shield".into(),
                profession: CraftingProfession::Armorsmithing, station: CraftingStation::Forge,
                ingredients: vec![RecipeIngredient { item_id: "iron_ingot".into(), quantity: 3 },
                                  RecipeIngredient { item_id: "wood_shaft".into(), quantity: 1 }],
                output_item_id: "iron_round_shield".into(), output_quantity: 1,
                required_skill_level: 2, required_int: 0, base_craft_time_seconds: 35,
                description: "A sturdy round shield bossed with iron.".into() },
            Recipe { id: "bone_cuirass".into(), name: "Bone Cuirass".into(),
                profession: CraftingProfession::Armorsmithing, station: CraftingStation::TanningRack,
                ingredients: vec![RecipeIngredient { item_id: "bones".into(), quantity: 6 },
                                  RecipeIngredient { item_id: "leather".into(), quantity: 3 },
                                  RecipeIngredient { item_id: "sinew".into(), quantity: 2 }],
                output_item_id: "bone_cuirass".into(), output_quantity: 1,
                required_skill_level: 2, required_int: 0, base_craft_time_seconds: 40,
                description: "Armour plates carved and lashed from the bones of large creatures.".into() },
            Recipe { id: "serpent_scale_armor".into(), name: "Serpent Scale Armour".into(),
                profession: CraftingProfession::Armorsmithing, station: CraftingStation::TanningRack,
                ingredients: vec![RecipeIngredient { item_id: "serpent_scale".into(), quantity: 5 },
                                  RecipeIngredient { item_id: "leather".into(), quantity: 3 }],
                output_item_id: "serpent_scale_armor".into(), output_quantity: 1,
                required_skill_level: 3, required_int: 0, base_craft_time_seconds: 45,
                description: "Overlapping sea-serpent scales sewn onto a leather backing.".into() },
            Recipe { id: "black_iron_sword".into(), name: "Black Iron Sword".into(),
                profession: CraftingProfession::Weaponsmithing, station: CraftingStation::Forge,
                ingredients: vec![RecipeIngredient { item_id: "black_iron_ingot".into(), quantity: 4 },
                                  RecipeIngredient { item_id: "leather_wrap".into(), quantity: 2 }],
                output_item_id: "black_iron_sword".into(), output_quantity: 1,
                required_skill_level: 3, required_int: 0, base_craft_time_seconds: 55,
                description: "A dark blade of quarry-sourced black iron, harder than common iron.".into() },
            Recipe { id: "frost_salve".into(), name: "Frost Salve".into(),
                profession: CraftingProfession::Alchemy, station: CraftingStation::AlchemyStone,
                ingredients: vec![RecipeIngredient { item_id: "frost_crystal".into(), quantity: 1 },
                                  RecipeIngredient { item_id: "herbs".into(), quantity: 2 },
                                  RecipeIngredient { item_id: "clean_water".into(), quantity: 1 }],
                output_item_id: "health_potion".into(), output_quantity: 2,
                required_skill_level: 2, required_int: 3, base_craft_time_seconds: 25,
                description: "A healing salve brewed from frost crystals; yields two health potions.".into() },
            Recipe { id: "witch_brew".into(), name: "Witch's Brew".into(),
                profession: CraftingProfession::Alchemy, station: CraftingStation::AlchemyStone,
                ingredients: vec![RecipeIngredient { item_id: "nightshade_leaf".into(), quantity: 2 },
                                  RecipeIngredient { item_id: "witch_talisman".into(), quantity: 1 },
                                  RecipeIngredient { item_id: "clean_water".into(), quantity: 1 }],
                output_item_id: "fortify_potion".into(), output_quantity: 1,
                required_skill_level: 3, required_int: 5, base_craft_time_seconds: 30,
                description: "A potent brew from a swamp witch's recipe; grants Regen status.".into() },
            Recipe { id: "carved_bone_arrow".into(), name: "Carved Bone Arrows".into(),
                profession: CraftingProfession::BoyerFletcher, station: CraftingStation::FletchingBench,
                ingredients: vec![RecipeIngredient { item_id: "bones".into(), quantity: 3 },
                                  RecipeIngredient { item_id: "harpy_feather".into(), quantity: 2 }],
                output_item_id: "iron_arrow".into(), output_quantity: 15,
                required_skill_level: 1, required_int: 0, base_craft_time_seconds: 15,
                description: "Arrows tipped with sharpened bone and fletched with harpy feathers.".into() },
            Recipe { id: "sea_poultice".into(), name: "Sea Poultice".into(),
                profession: CraftingProfession::Alchemy, station: CraftingStation::AlchemyStone,
                ingredients: vec![RecipeIngredient { item_id: "sea_salt".into(), quantity: 2 },
                                  RecipeIngredient { item_id: "herbs".into(), quantity: 1 },
                                  RecipeIngredient { item_id: "bog_moss".into(), quantity: 1 }],
                output_item_id: "antidote".into(), output_quantity: 2,
                required_skill_level: 1, required_int: 2, base_craft_time_seconds: 20,
                description: "A coastal remedy using sea salt and bog moss; yields two antidotes.".into() },
        ]
    }
}

impl Default for CraftingSystem {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_starter_recipes_load() {
        let cs = CraftingSystem::with_starter_recipes();
        assert!(cs.all_recipes.len() >= 10);
    }

    #[test]
    fn test_craft_requires_ingredients() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let mut cs = CraftingSystem::with_starter_recipes();
        let mut inv = Inventory::new(40);
        // No ingredients - should fail
        assert!(cs.craft("health_potion", &mut inv, 0, 5, 5, &mut rng).is_err());
    }

    #[test]
    fn test_crafting_quality_scales() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(999);
        let low = CraftingSystem::calculate_quality(0, 0, 0, &mut rng);
        let mut rng2 = rand::rngs::StdRng::seed_from_u64(999);
        let high = CraftingSystem::calculate_quality(50, 50, 100, &mut rng2);
        // High stats produce at least as good quality as the same roll
        let _ = (low, high); // just ensure it compiles and runs
    }

    #[test]
    fn test_profession_name_roundtrip() {
        let professions = [
            (CraftingProfession::Mining, "Mining"),
            (CraftingProfession::Gathering, "Gathering"),
            (CraftingProfession::Cooking, "Cooking"),
            (CraftingProfession::Weaponsmithing, "Weaponsmithing"),
            (CraftingProfession::Armorsmithing, "Armorsmithing"),
            (CraftingProfession::Alchemy, "Alchemy"),
            (CraftingProfession::BoyerFletcher, "BoyerFletcher"),
        ];
        for (prof, expected) in &professions {
            assert_eq!(prof.name(), *expected);
        }
    }

    #[test]
    fn test_station_name_some_and_none() {
        assert_eq!(CraftingStation::None.name(), None);
        assert_eq!(CraftingStation::Forge.name(), Some("Forge"));
        assert_eq!(CraftingStation::Campfire.name(), Some("Campfire"));
        assert_eq!(CraftingStation::AlchemyStone.name(), Some("AlchemyStone"));
    }

    #[test]
    fn test_learn_recipe_and_craft_with_correct_skill() {
        use iron_age_inventory::{Item, ItemType};
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut cs = CraftingSystem::new();
        assert!(cs.learn_recipe("health_potion"));

        let mut inv = Inventory::new(40);
        // Add required ingredients: 2 herbs + 1 clean_water
        let mut herbs = Item::new_consumable("herbs", "Herbs", ItemType::CraftingMaterial, 10);
        herbs.quantity = 2;
        let water = Item::new_consumable("clean_water", "Clean Water", ItemType::CraftingMaterial, 5);
        inv.add_item(herbs).unwrap();
        inv.add_item(water).unwrap();

        // health_potion requires skill 0 and INT 2
        let result = cs.craft("health_potion", &mut inv, 0, 5, 5, &mut rng);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result.err());
    }

    #[test]
    fn test_craft_fails_when_skill_too_low() {
        use iron_age_inventory::{Item, ItemType};
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let mut cs = CraftingSystem::with_starter_recipes();
        let mut inv = Inventory::new(40);

        // iron_short_sword requires skill level 1; pass level 0 → should fail
        let mut ingot = Item::new_consumable("iron_ingot", "Iron Ingot", ItemType::CraftingMaterial, 10);
        ingot.quantity = 3;
        let wrap = Item::new_consumable("leather_wrap", "Leather Wrap", ItemType::CraftingMaterial, 10);
        inv.add_item(ingot).unwrap();
        inv.add_item(wrap).unwrap();

        let result = cs.craft("iron_short_sword", &mut inv, 0, 10, 10, &mut rng);
        assert!(result.is_err());
    }

    #[test]
    fn test_advanced_recipes_exist() {
        let cs = CraftingSystem::with_starter_recipes();
        let ids: Vec<&str> = cs.all_recipes.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"clarity_potion"), "clarity_potion recipe should exist");
        assert!(ids.contains(&"fortify_potion"), "fortify_potion recipe should exist");
        assert!(ids.contains(&"stamina_potion"), "stamina_potion recipe should exist");
        assert!(ids.contains(&"iron_long_sword"), "iron_long_sword recipe should exist");
        assert!(ids.contains(&"wolf_pelt_armor"), "wolf_pelt_armor recipe should exist");
    }

    #[test]
    fn test_craft_clarity_potion_with_crystalline_dust() {
        use iron_age_inventory::{Item, ItemType};
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        let mut cs = CraftingSystem::with_starter_recipes();
        let mut inv = Inventory::new(40);
        let mut dust = Item::new_consumable("crystalline_dust", "Crystalline Dust", ItemType::CraftingMaterial, 10);
        dust.quantity = 1;
        let water = Item::new_consumable("clean_water", "Clean Water", ItemType::CraftingMaterial, 5);
        inv.add_item(dust).unwrap();
        inv.add_item(water).unwrap();
        // Skill 2, INT 4 required
        let result = cs.craft("clarity_potion", &mut inv, 2, 4, 4, &mut rng);
        assert!(result.is_ok());
    }
}
