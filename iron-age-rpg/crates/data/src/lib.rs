use iron_age_core::{Stats, DamageType, ElementalResistances, StatusEffect};
use iron_age_inventory::{Item, ItemType, MaterialTier, ItemRarity, EquipSlot};
use iron_age_character::Character;
use iron_age_combat::Combatant;
use rand::Rng;

// ── Enemy Templates ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnemyTemplate {
    pub id: String,
    pub name: String,
    pub level: u32,
    pub base_hp: i32,
    pub stats: Stats,
    pub armor: i32,
    pub weapon_damage: i32,
    pub damage_type: DamageType,
    pub resistances: ElementalResistances,
    pub xp_reward: u64,
    pub gold_min: u32,
    pub gold_max: u32,
    pub loot_item_ids: Vec<String>,
    pub description: String,
    pub abilities: Vec<String>,
    pub on_death_status: Option<StatusEffect>,
}

impl EnemyTemplate {
    pub fn spawn(&self) -> Combatant {
        let mut character = Character::new(self.name.clone());
        character.level = self.level;
        character.stats = self.stats.clone();
        character.max_hp = self.base_hp;
        character.hp = self.base_hp;
        let max_stamina = 20 + self.stats.constitution * 2;
        character.max_stamina = max_stamina;
        character.stamina = max_stamina;
        Combatant::new(character, false)
    }
}

/// Return all enemy templates defined in the game.
pub fn all_enemy_templates() -> Vec<EnemyTemplate> {
    vec![
        // ── Tier 1: Weak Enemies ──────────────────────────────────────────────
        EnemyTemplate {
            id: "goblin_scout".to_string(),
            name: "Goblin Scout".to_string(),
            level: 1,
            base_hp: 25,
            stats: Stats::new(4, 3, 2, 3, 6, 2),
            armor: 1,
            weapon_damage: 5,
            damage_type: DamageType::Physical,
            resistances: ElementalResistances::none(),
            xp_reward: 20,
            gold_min: 1, gold_max: 5,
            loot_item_ids: vec!["crude_knife".to_string()],
            description: "A wiry goblin that relies on speed over strength.".to_string(),
            abilities: vec![],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "wolf".to_string(),
            name: "Wolf".to_string(),
            level: 2,
            base_hp: 35,
            stats: Stats::new(6, 2, 3, 5, 8, 1),
            armor: 0,
            weapon_damage: 7,
            damage_type: DamageType::Physical,
            resistances: ElementalResistances::none(),
            xp_reward: 30,
            gold_min: 0, gold_max: 2,
            loot_item_ids: vec!["wolf_pelt".to_string()],
            description: "A pack hunter from the Ashwood Forest.".to_string(),
            abilities: vec!["Bite".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "forest_spider".to_string(),
            name: "Forest Spider".to_string(),
            level: 2,
            base_hp: 28,
            stats: Stats::new(3, 2, 2, 4, 9, 1),
            armor: 1,
            weapon_damage: 6,
            damage_type: DamageType::Poison,
            resistances: ElementalResistances::none(),
            xp_reward: 25,
            gold_min: 0, gold_max: 1,
            loot_item_ids: vec!["spider_silk".to_string()],
            description: "A venomous spider lurking in the dark forest.".to_string(),
            abilities: vec!["Venom Bite".to_string()],
            on_death_status: Some(StatusEffect::Poison { damage_per_turn: 2, turns_remaining: 2 }),
        },
        EnemyTemplate {
            id: "bog_crawler".to_string(),
            name: "Bog Crawler".to_string(),
            level: 3,
            base_hp: 45,
            stats: Stats::new(7, 2, 2, 8, 3, 1),
            armor: 3,
            weapon_damage: 8,
            damage_type: DamageType::Nature,
            resistances: {
                let mut r = ElementalResistances::none();
                r.nature = 25;
                r.fire = -10;
                r
            },
            xp_reward: 45,
            gold_min: 0, gold_max: 3,
            loot_item_ids: vec!["bog_moss".to_string(), "chitinous_shell".to_string()],
            description: "An armoured crustacean that hunts in the bog shallows.".to_string(),
            abilities: vec!["Shell Slam".to_string()],
            on_death_status: None,
        },

        // ── Tier 2: Mid-Tier Enemies ──────────────────────────────────────────
        EnemyTemplate {
            id: "goblin_warrior".to_string(),
            name: "Goblin Warrior".to_string(),
            level: 3,
            base_hp: 55,
            stats: Stats::new(7, 4, 3, 6, 5, 2),
            armor: 4,
            weapon_damage: 11,
            damage_type: DamageType::Physical,
            resistances: ElementalResistances::none(),
            xp_reward: 60,
            gold_min: 3, gold_max: 10,
            loot_item_ids: vec!["crude_sword".to_string(), "iron_ingot".to_string()],
            description: "A battle-hardened goblin armed with a crude iron blade.".to_string(),
            abilities: vec!["Reckless Strike".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "goblin_archer".to_string(),
            name: "Goblin Archer".to_string(),
            level: 3,
            base_hp: 40,
            stats: Stats::new(4, 5, 4, 4, 9, 3),
            armor: 2,
            weapon_damage: 9,
            damage_type: DamageType::Physical,
            resistances: ElementalResistances::none(),
            xp_reward: 55,
            gold_min: 2, gold_max: 8,
            loot_item_ids: vec!["crude_bow".to_string()],
            description: "A goblin with a knack for ranged harassment.".to_string(),
            abilities: vec!["Pinning Shot".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "dire_wolf".to_string(),
            name: "Dire Wolf".to_string(),
            level: 4,
            base_hp: 70,
            stats: Stats::new(10, 3, 4, 8, 9, 2),
            armor: 1,
            weapon_damage: 14,
            damage_type: DamageType::Physical,
            resistances: ElementalResistances::none(),
            xp_reward: 90,
            gold_min: 0, gold_max: 5,
            loot_item_ids: vec!["wolf_pelt".to_string(), "dire_wolf_fang".to_string()],
            description: "A massive wolf, twice the size of a normal one.".to_string(),
            abilities: vec!["Savage Bite".to_string(), "Howl".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "swamp_witch".to_string(),
            name: "Swamp Witch".to_string(),
            level: 4,
            base_hp: 50,
            stats: Stats::new(3, 12, 10, 4, 5, 6),
            armor: 0,
            weapon_damage: 12,
            damage_type: DamageType::Poison,
            resistances: {
                let mut r = ElementalResistances::none();
                r.poison = 50;
                r.nature = 30;
                r
            },
            xp_reward: 100,
            gold_min: 5, gold_max: 15,
            loot_item_ids: vec!["nightshade_leaf".to_string(), "bog_moss".to_string(), "witch_talisman".to_string()],
            description: "A twisted hag who commands the bog's dark magic.".to_string(),
            abilities: vec!["Poison Hex".to_string(), "Wither".to_string()],
            on_death_status: None,
        },

        // ── Tier 3: Elite Enemies ──────────────────────────────────────────────
        EnemyTemplate {
            id: "goblin_shaman".to_string(),
            name: "Goblin Shaman".to_string(),
            level: 5,
            base_hp: 65,
            stats: Stats::new(5, 10, 9, 5, 6, 5),
            armor: 2,
            weapon_damage: 14,
            damage_type: DamageType::Nature,
            resistances: {
                let mut r = ElementalResistances::none();
                r.nature = 20;
                r
            },
            xp_reward: 150,
            gold_min: 10, gold_max: 25,
            loot_item_ids: vec!["iron_key".to_string(), "shaman_staff".to_string()],
            description: "A goblin spellcaster who commands the tribe's rituals. \
                          Carries the iron key to the keep tower.".to_string(),
            abilities: vec!["Curse".to_string(), "Nature Bolt".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "dire_wolf_alpha".to_string(),
            name: "Dire Wolf Alpha".to_string(),
            level: 6,
            base_hp: 120,
            stats: Stats::new(14, 4, 5, 12, 10, 5),
            armor: 3,
            weapon_damage: 20,
            damage_type: DamageType::Physical,
            resistances: ElementalResistances::none(),
            xp_reward: 250,
            gold_min: 0, gold_max: 10,
            loot_item_ids: vec!["alpha_pelt".to_string(), "dire_wolf_fang".to_string()],
            description: "The scarred leader of the Ashwood wolf pack. \
                          Battle-hardened and ferocious.".to_string(),
            abilities: vec!["Alpha Howl".to_string(), "Savage Mauling".to_string(), "Pack Call".to_string()],
            on_death_status: None,
        },

        // ── Boss ──────────────────────────────────────────────────────────────
        EnemyTemplate {
            id: "goblin_warlord".to_string(),
            name: "Grukk the Warlord".to_string(),
            level: 7,
            base_hp: 160,
            stats: Stats::new(15, 6, 5, 14, 7, 8),
            armor: 8,
            weapon_damage: 22,
            damage_type: DamageType::Physical,
            resistances: {
                let mut r = ElementalResistances::none();
                r.physical = 10;
                r
            },
            xp_reward: 500,
            gold_min: 30, gold_max: 60,
            loot_item_ids: vec!["warlord_battle_axe".to_string(), "iron_long_sword".to_string()],
            description: "The brutal goblin warlord who has claimed Ironmere Keep \
                          as his own. He wears pilfered iron armour and commands \
                          the respect of every goblin in the region.".to_string(),
            abilities: vec!["Cleave".to_string(), "Battle Roar".to_string(), "Iron Skin".to_string()],
            on_death_status: None,
        },
    ]
}

/// Find a template by id.
pub fn find_template(id: &str) -> Option<EnemyTemplate> {
    all_enemy_templates().into_iter().find(|t| t.id == id)
}

/// Roll loot for a defeated enemy. Returns (gold, item_ids_dropped).
/// Each loot_item_id has a 50% drop chance.
pub fn roll_loot(template_id: &str, rng: &mut impl Rng) -> (u32, Vec<String>) {
    let Some(t) = find_template(template_id) else {
        return (0, vec![]);
    };
    let gold = if t.gold_max > t.gold_min {
        rng.gen_range(t.gold_min..=t.gold_max)
    } else {
        t.gold_min
    };
    let items = t.loot_item_ids.iter()
        .filter(|_| rng.gen_bool(0.5))
        .cloned()
        .collect();
    (gold, items)
}

/// Spawn a random enemy appropriate for a given difficulty tier.
pub fn spawn_random_enemy(difficulty: u32, rng: &mut impl Rng) -> Option<Combatant> {
    let pool: Vec<EnemyTemplate> = all_enemy_templates()
        .into_iter()
        .filter(|t| {
            let tier = (t.level.saturating_sub(1)) / 2;
            tier <= difficulty
        })
        .collect();
    if pool.is_empty() { return None; }
    let idx = rng.gen_range(0..pool.len());
    Some(pool[idx].spawn())
}

// ── Item Catalog ──────────────────────────────────────────────────────────────

/// Return the complete catalog of items that exist in the world.
pub fn all_items() -> Vec<Item> {
    vec![
        // Weapons
        Item::new_weapon("crude_knife", "Crude Knife", ItemType::ShortSword, MaterialTier::Stone, ItemRarity::Common, 4),
        Item::new_weapon("crude_sword", "Crude Sword", ItemType::ShortSword, MaterialTier::Stone, ItemRarity::Common, 6),
        Item::new_weapon("crude_bow", "Crude Bow", ItemType::Shortbow, MaterialTier::Wood, ItemRarity::Common, 5),
        Item::new_weapon("iron_short_sword", "Iron Short Sword", ItemType::ShortSword, MaterialTier::Iron, ItemRarity::Common, 10),
        Item::new_weapon("iron_spear", "Iron Spear", ItemType::IronSpear, MaterialTier::Iron, ItemRarity::Common, 9),
        Item::new_weapon("iron_long_sword", "Iron Long Sword", ItemType::LongSword, MaterialTier::Iron, ItemRarity::Uncommon, 14),
        Item::new_weapon("shortbow", "Shortbow", ItemType::Shortbow, MaterialTier::Wood, ItemRarity::Common, 7),
        Item::new_weapon("shaman_staff", "Shaman's Staff", ItemType::Spear, MaterialTier::Hardwood, ItemRarity::Uncommon, 11),
        Item::new_weapon("warlord_battle_axe", "Warlord's Battle Axe", ItemType::BattleAxe, MaterialTier::Iron, ItemRarity::Rare, 18),

        // Armor
        Item::new_armor("leather_helmet", "Leather Helmet", EquipSlot::Helmet, MaterialTier::Wood, ItemRarity::Common, 3),
        Item::new_armor("leather_torso", "Leather Chest", EquipSlot::Torso, MaterialTier::Wood, ItemRarity::Common, 5),
        Item::new_armor("wolf_pelt_armor", "Wolf Pelt Armour", EquipSlot::Torso, MaterialTier::Wood, ItemRarity::Uncommon, 7),
        Item::new_armor("alpha_pelt", "Alpha Wolf Pelt", EquipSlot::Cape, MaterialTier::Hardwood, ItemRarity::Rare, 6),
        Item::new_armor("chitinous_shell", "Chitinous Shell Pauldrons", EquipSlot::Shoulders, MaterialTier::Stone, ItemRarity::Common, 4),

        // Consumables
        Item::new_consumable("health_potion", "Health Potion", ItemType::HealthPotion, 5),
        Item::new_consumable("stamina_potion", "Stamina Potion", ItemType::StaminaPotion, 5),
        Item::new_consumable("antidote", "Antidote", ItemType::AntidotePotion, 5),
        Item::new_consumable("campfire_stew", "Campfire Stew", ItemType::HealthPotion, 3),
        Item::new_consumable("pitch_bomb", "Pitch Bomb", ItemType::PitchBomb, 3),

        // Crafting materials
        Item::new_consumable("iron_ingot", "Iron Ingot", ItemType::CraftingMaterial, 10),
        Item::new_consumable("leather", "Leather", ItemType::CraftingMaterial, 10),
        Item::new_consumable("leather_wrap", "Leather Wrap", ItemType::CraftingMaterial, 10),
        Item::new_consumable("wood_shaft", "Wood Shaft", ItemType::CraftingMaterial, 10),
        Item::new_consumable("herbs", "Herbs", ItemType::CraftingMaterial, 10),
        Item::new_consumable("clean_water", "Clean Water", ItemType::CraftingMaterial, 5),
        Item::new_consumable("nightshade_leaf", "Nightshade Leaf", ItemType::CraftingMaterial, 10),
        Item::new_consumable("bog_moss", "Bog Moss", ItemType::CraftingMaterial, 10),
        Item::new_consumable("wood", "Wood", ItemType::CraftingMaterial, 10),
        Item::new_consumable("sinew", "Sinew", ItemType::CraftingMaterial, 10),
        Item::new_consumable("meat", "Meat", ItemType::CraftingMaterial, 10),
        Item::new_consumable("feather", "Feather", ItemType::CraftingMaterial, 20),
        Item::new_consumable("pitch", "Pitch", ItemType::CraftingMaterial, 5),
        Item::new_consumable("clay", "Clay", ItemType::CraftingMaterial, 5),

        // Loot/materials
        Item::new_consumable("wolf_pelt", "Wolf Pelt", ItemType::CraftingMaterial, 5),
        Item::new_consumable("dire_wolf_fang", "Dire Wolf Fang", ItemType::CraftingMaterial, 3),
        Item::new_consumable("spider_silk", "Spider Silk", ItemType::CraftingMaterial, 10),
        Item::new_consumable("witch_talisman", "Witch Talisman", ItemType::CraftingMaterial, 1),

        // Quest items
        Item::new_consumable("iron_key", "Iron Key", ItemType::CraftingMaterial, 1),
    ]
}

/// Find a catalog item by id.
pub fn find_item(id: &str) -> Option<Item> {
    all_items().into_iter().find(|i| i.id == id)
}

/// Build a starter inventory set for a new player.
pub fn starter_items() -> Vec<Item> {
    vec![
        {
            let mut i = find_item("crude_knife").unwrap();
            i.equip_slot = Some(EquipSlot::MainHand);
            i
        },
        find_item("health_potion").unwrap(),
        find_item("health_potion").unwrap(),
        find_item("herbs").unwrap(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_all_enemy_templates_have_ids() {
        let templates = all_enemy_templates();
        assert!(!templates.is_empty());
        for t in &templates {
            assert!(!t.id.is_empty());
            assert!(!t.name.is_empty());
            assert!(t.base_hp > 0);
        }
    }

    #[test]
    fn test_find_template() {
        let t = find_template("goblin_warlord");
        assert!(t.is_some());
        let t = t.unwrap();
        assert_eq!(t.name, "Grukk the Warlord");
    }

    #[test]
    fn test_spawn_enemy_creates_combatant() {
        let t = find_template("wolf").unwrap();
        let combatant = t.spawn();
        assert_eq!(combatant.character.name, "Wolf");
        assert!(!combatant.is_player);
        assert!(combatant.character.hp > 0);
    }

    #[test]
    fn test_spawn_random_enemy() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let result = spawn_random_enemy(2, &mut rng);
        assert!(result.is_some());
    }

    #[test]
    fn test_all_items_have_ids() {
        let items = all_items();
        assert!(!items.is_empty());
        for item in &items {
            assert!(!item.id.is_empty());
        }
    }

    #[test]
    fn test_starter_items() {
        let items = starter_items();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_goblin_shaman_drops_iron_key() {
        let t = find_template("goblin_shaman").unwrap();
        assert!(t.loot_item_ids.contains(&"iron_key".to_string()));
    }

    #[test]
    fn test_boss_has_highest_level() {
        let templates = all_enemy_templates();
        let warlord = find_template("goblin_warlord").unwrap();
        let max_level = templates.iter().map(|t| t.level).max().unwrap();
        assert_eq!(warlord.level, max_level);
    }

    #[test]
    fn test_roll_loot_returns_gold_in_range() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let template = find_template("goblin_scout").unwrap();
        for _ in 0..20 {
            let (gold, _) = roll_loot("goblin_scout", &mut rng);
            assert!(gold >= template.gold_min && gold <= template.gold_max,
                "gold {} out of range [{}, {}]", gold, template.gold_min, template.gold_max);
        }
    }

    #[test]
    fn test_roll_loot_unknown_template_returns_zero() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let (gold, items) = roll_loot("nonexistent_enemy", &mut rng);
        assert_eq!(gold, 0);
        assert!(items.is_empty());
    }
}
