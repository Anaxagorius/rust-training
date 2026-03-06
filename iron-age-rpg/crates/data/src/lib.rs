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

        // ── Valley & Cave Enemies ─────────────────────────────────────────────
        EnemyTemplate {
            id: "valley_wolf".to_string(),
            name: "Valley Wolf".to_string(),
            level: 1,
            base_hp: 28,
            stats: Stats::new(5, 2, 2, 4, 7, 1),
            armor: 0,
            weapon_damage: 6,
            damage_type: DamageType::Physical,
            resistances: ElementalResistances::none(),
            xp_reward: 22,
            gold_min: 0, gold_max: 1,
            loot_item_ids: vec!["wolf_pelt".to_string()],
            description: "A grey wolf that roams the Embervale valley floor.".to_string(),
            abilities: vec!["Bite".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "giant_bat".to_string(),
            name: "Giant Bat".to_string(),
            level: 2,
            base_hp: 22,
            stats: Stats::new(3, 2, 2, 3, 10, 1),
            armor: 0,
            weapon_damage: 5,
            damage_type: DamageType::Physical,
            resistances: ElementalResistances::none(),
            xp_reward: 20,
            gold_min: 0, gold_max: 0,
            loot_item_ids: vec![],
            description: "A large cave bat with leathery wings and a piercing shriek.".to_string(),
            abilities: vec!["Screech".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "cave_bear".to_string(),
            name: "Cave Bear".to_string(),
            level: 4,
            base_hp: 90,
            stats: Stats::new(13, 2, 3, 12, 5, 1),
            armor: 3,
            weapon_damage: 16,
            damage_type: DamageType::Physical,
            resistances: ElementalResistances::none(),
            xp_reward: 110,
            gold_min: 0, gold_max: 3,
            loot_item_ids: vec!["bear_claw".to_string()],
            description: "A massive brown bear that makes its den deep in the crystal cave.".to_string(),
            abilities: vec!["Maul".to_string(), "Roar".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "bandit".to_string(),
            name: "Valley Bandit".to_string(),
            level: 2,
            base_hp: 38,
            stats: Stats::new(6, 4, 3, 5, 6, 4),
            armor: 2,
            weapon_damage: 8,
            damage_type: DamageType::Physical,
            resistances: ElementalResistances::none(),
            xp_reward: 35,
            gold_min: 3, gold_max: 12,
            loot_item_ids: vec!["crude_knife".to_string(), "tattered_cloth".to_string()],
            description: "A desperate outlaw preying on travellers in the valley.".to_string(),
            abilities: vec!["Ambush".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "bandit_chief".to_string(),
            name: "Bandit Chief".to_string(),
            level: 4,
            base_hp: 72,
            stats: Stats::new(10, 6, 5, 8, 7, 6),
            armor: 5,
            weapon_damage: 13,
            damage_type: DamageType::Physical,
            resistances: ElementalResistances::none(),
            xp_reward: 120,
            gold_min: 15, gold_max: 35,
            loot_item_ids: vec!["iron_short_sword".to_string(), "bandit_cloak".to_string()],
            description: "The leader of the valley bandits — scarred, cruel, and well-armed.".to_string(),
            abilities: vec!["Rally".to_string(), "Power Strike".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "mountain_goat".to_string(),
            name: "Mountain Goat".to_string(),
            level: 1,
            base_hp: 20,
            stats: Stats::new(4, 1, 1, 4, 7, 1),
            armor: 0,
            weapon_damage: 4,
            damage_type: DamageType::Physical,
            resistances: ElementalResistances::none(),
            xp_reward: 10,
            gold_min: 0, gold_max: 0,
            loot_item_ids: vec!["meat".to_string()],
            description: "A sure-footed goat that has strayed from the mountain crags.".to_string(),
            abilities: vec![],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "stone_troll".to_string(),
            name: "Stone Troll".to_string(),
            level: 5,
            base_hp: 110,
            stats: Stats::new(16, 2, 2, 14, 4, 1),
            armor: 6,
            weapon_damage: 18,
            damage_type: DamageType::Physical,
            resistances: {
                let mut r = ElementalResistances::none();
                r.physical = 15;
                r.fire = -20;
                r
            },
            xp_reward: 180,
            gold_min: 0, gold_max: 8,
            loot_item_ids: vec!["iron_ingot".to_string(), "crystal_shard".to_string()],
            description: "A hulking troll of grey stone-like skin that guards the mountain passes.".to_string(),
            abilities: vec!["Boulder Smash".to_string(), "Regenerate".to_string()],
            on_death_status: None,
        },

        // ── Undead (Crypts & Tombs) ───────────────────────────────────────────
        EnemyTemplate {
            id: "skeleton_warrior".to_string(),
            name: "Skeleton Warrior".to_string(),
            level: 3,
            base_hp: 45,
            stats: Stats::new(7, 2, 1, 6, 5, 1),
            armor: 4,
            weapon_damage: 9,
            damage_type: DamageType::Physical,
            resistances: {
                let mut r = ElementalResistances::none();
                r.poison = 100;
                r.physical = 10;
                r.fire = -15;
                r
            },
            xp_reward: 50,
            gold_min: 0, gold_max: 5,
            loot_item_ids: vec!["bones".to_string(), "ancient_coin".to_string()],
            description: "An animated skeleton clad in corroded armour, obeying some \
                          long-dead will.".to_string(),
            abilities: vec!["Bone Rattle".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "skeleton_archer".to_string(),
            name: "Skeleton Archer".to_string(),
            level: 3,
            base_hp: 35,
            stats: Stats::new(4, 3, 2, 4, 8, 1),
            armor: 2,
            weapon_damage: 8,
            damage_type: DamageType::Physical,
            resistances: {
                let mut r = ElementalResistances::none();
                r.poison = 100;
                r.fire = -15;
                r
            },
            xp_reward: 45,
            gold_min: 0, gold_max: 3,
            loot_item_ids: vec!["bones".to_string(), "crude_bow".to_string()],
            description: "A skeleton that draws a crumbling bow with uncanny precision.".to_string(),
            abilities: vec!["Volley".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "crypt_ghoul".to_string(),
            name: "Crypt Ghoul".to_string(),
            level: 4,
            base_hp: 58,
            stats: Stats::new(9, 3, 2, 7, 8, 1),
            armor: 1,
            weapon_damage: 12,
            damage_type: DamageType::Poison,
            resistances: {
                let mut r = ElementalResistances::none();
                r.poison = 50;
                r.nature = 20;
                r
            },
            xp_reward: 80,
            gold_min: 0, gold_max: 4,
            loot_item_ids: vec!["tattered_cloth".to_string(), "ancient_coin".to_string()],
            description: "A hunched, ravening undead creature that feeds on the buried dead \
                          of the valley's ancient cemeteries.".to_string(),
            abilities: vec!["Paralyzing Bite".to_string(), "Frenzy".to_string()],
            on_death_status: Some(StatusEffect::Poison { damage_per_turn: 3, turns_remaining: 3 }),
        },
        EnemyTemplate {
            id: "wraith".to_string(),
            name: "Barrow Wraith".to_string(),
            level: 6,
            base_hp: 75,
            stats: Stats::new(6, 14, 12, 6, 9, 4),
            armor: 0,
            weapon_damage: 16,
            damage_type: DamageType::Psychic,
            resistances: {
                let mut r = ElementalResistances::none();
                r.poison = 100;
                r.physical = 50;
                r.fire = 25;
                r
            },
            xp_reward: 220,
            gold_min: 0, gold_max: 0,
            loot_item_ids: vec!["ghost_essence".to_string(), "ancient_coin".to_string()],
            description: "The tortured spirit of a slain warrior, bound to the barrow \
                          by ancient ritual. It drains life with a touch.".to_string(),
            abilities: vec!["Life Drain".to_string(), "Wail".to_string(), "Ethereal Form".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "tomb_guardian".to_string(),
            name: "Tomb Guardian".to_string(),
            level: 7,
            base_hp: 150,
            stats: Stats::new(14, 8, 6, 16, 6, 3),
            armor: 10,
            weapon_damage: 20,
            damage_type: DamageType::Physical,
            resistances: {
                let mut r = ElementalResistances::none();
                r.poison = 100;
                r.physical = 20;
                r.nature = 30;
                r
            },
            xp_reward: 450,
            gold_min: 20, gold_max: 50,
            loot_item_ids: vec!["ancient_coin".to_string(), "ghost_essence".to_string(), "iron_long_sword".to_string()],
            description: "A massive stone guardian animated by the Valley King's burial \
                          rites. Its eyes blaze with eldritch fire and its form is \
                          nearly impervious to mundane weapons.".to_string(),
            abilities: vec!["Stone Fist".to_string(), "Ancient Ward".to_string(), "Tremor Strike".to_string()],
            on_death_status: None,
        },

        // ── Cave & Dungeon Specialists ────────────────────────────────────────
        EnemyTemplate {
            id: "crystal_golem".to_string(),
            name: "Crystal Golem".to_string(),
            level: 5,
            base_hp: 100,
            stats: Stats::new(13, 5, 5, 14, 3, 1),
            armor: 8,
            weapon_damage: 15,
            damage_type: DamageType::Physical,
            resistances: {
                let mut r = ElementalResistances::none();
                r.physical = 25;
                r.nature = 15;
                r.poison = 100;
                r.fire = -15;
                r
            },
            xp_reward: 160,
            gold_min: 0, gold_max: 5,
            loot_item_ids: vec!["crystal_shard".to_string(), "crystalline_dust".to_string()],
            description: "A hulking construct of living crystal animated by the cave's \
                          ancient magical energies. Its faceted body refracts light \
                          into blinding beams.".to_string(),
            abilities: vec!["Crystal Shatter".to_string(), "Refraction Beam".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "cave_troll".to_string(),
            name: "Cave Troll".to_string(),
            level: 4,
            base_hp: 90,
            stats: Stats::new(13, 2, 2, 13, 3, 1),
            armor: 4,
            weapon_damage: 16,
            damage_type: DamageType::Physical,
            resistances: {
                let mut r = ElementalResistances::none();
                r.physical = 10;
                r.fire = -25;
                r
            },
            xp_reward: 130,
            gold_min: 0, gold_max: 5,
            loot_item_ids: vec!["iron_ingot".to_string(), "tattered_cloth".to_string()],
            description: "A cave-dwelling troll with moss-covered hide and massive fists. \
                          Slow but incredibly powerful, and capable of limited regeneration.".to_string(),
            abilities: vec!["Smash".to_string(), "Regenerate".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "barrow_knight".to_string(),
            name: "Barrow Knight".to_string(),
            level: 5,
            base_hp: 80,
            stats: Stats::new(11, 3, 3, 9, 7, 2),
            armor: 7,
            weapon_damage: 14,
            damage_type: DamageType::Physical,
            resistances: {
                let mut r = ElementalResistances::none();
                r.poison = 100;
                r.physical = 15;
                r.fire = -20;
                r
            },
            xp_reward: 140,
            gold_min: 0, gold_max: 8,
            loot_item_ids: vec!["ancient_coin".to_string(), "mouldering_chainmail".to_string(), "carved_bone".to_string()],
            description: "An ancient warrior bound forever to guard their lord's barrow. \
                          Clad in corroded armour and wielding a rusted but deadly blade, \
                          the barrow knight fights with centuries of martial memory.".to_string(),
            abilities: vec!["Shield Wall".to_string(), "Bone Cleave".to_string()],
            on_death_status: None,
        },
        EnemyTemplate {
            id: "mummified_guard".to_string(),
            name: "Mummified Guard".to_string(),
            level: 6,
            base_hp: 110,
            stats: Stats::new(12, 4, 4, 14, 4, 2),
            armor: 8,
            weapon_damage: 15,
            damage_type: DamageType::Physical,
            resistances: {
                let mut r = ElementalResistances::none();
                r.poison = 100;
                r.physical = 20;
                r.fire = -30;
                r.nature = 15;
                r
            },
            xp_reward: 200,
            gold_min: 0, gold_max: 10,
            loot_item_ids: vec!["ancient_coin".to_string(), "carved_bone".to_string(), "ghost_essence".to_string()],
            description: "A Valley King's guard, preserved for an age in aromatic resins. \
                          Wrapped in rotting linen strips and armed with an iron khopesh, \
                          it responds to intrusion with mechanical, relentless aggression.".to_string(),
            abilities: vec!["Ancient Grip".to_string(), "Cursed Blow".to_string()],
            on_death_status: Some(StatusEffect::Poison { damage_per_turn: 2, turns_remaining: 3 }),
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

        // Valley / cave / tomb materials and loot
        Item::new_consumable("bear_claw", "Bear Claw", ItemType::CraftingMaterial, 5),
        Item::new_consumable("crystal_shard", "Crystal Shard", ItemType::CraftingMaterial, 5),
        Item::new_consumable("bones", "Bones", ItemType::CraftingMaterial, 10),
        Item::new_consumable("ancient_coin", "Ancient Coin", ItemType::CraftingMaterial, 1),
        Item::new_consumable("ghost_essence", "Ghost Essence", ItemType::CraftingMaterial, 1),
        Item::new_consumable("tattered_cloth", "Tattered Cloth", ItemType::CraftingMaterial, 10),
        Item::new_armor("bandit_cloak", "Bandit's Cloak", EquipSlot::Cape, MaterialTier::Wood, ItemRarity::Common, 3),

        // New crafting materials
        Item::new_consumable("crystalline_dust", "Crystalline Dust", ItemType::CraftingMaterial, 10),
        Item::new_consumable("carved_bone", "Carved Bone", ItemType::CraftingMaterial, 10),
        Item::new_consumable("ancient_tome", "Ancient Tome", ItemType::CraftingMaterial, 1),
        Item::new_consumable("grave_dust", "Grave Dust", ItemType::CraftingMaterial, 10),

        // New armor from POIs
        Item::new_armor("mouldering_chainmail", "Mouldering Chainmail", EquipSlot::Torso, MaterialTier::Iron, ItemRarity::Common, 6),
        Item::new_armor("barrow_lord_helm", "Barrow Lord's Helm", EquipSlot::Helmet, MaterialTier::Iron, ItemRarity::Rare, 5),

        // New weapons from POIs
        Item::new_weapon("runic_short_sword", "Runic Short Sword", ItemType::ShortSword, MaterialTier::Iron, ItemRarity::Rare, 13),

        // Unique accessories — rings and amulets with stat bonuses
        Item {
            id: "crystal_ring".to_string(),
            name: "Crystal Ring".to_string(),
            item_type: ItemType::Ring,
            material: None,
            rarity: ItemRarity::Rare,
            weight: 0.1,
            value: 120,
            damage_base: 0,
            armor_base: 0,
            stat_requirements: Stats::zeroed(),
            stat_bonuses: Stats::new(0, 2, 1, 0, 0, 0),
            equip_slot: Some(EquipSlot::Ring1),
            stack_size: 1,
            quantity: 1,
            description: "A ring carved from a crystal shard, imbued with magical resonance. +2 INT, +1 WIS.".to_string(),
            effects: Vec::new(),
            is_two_handed: false,
        },
        {
            Item {
                id: "ancient_amulet".to_string(),
                name: "Ancient Amulet".to_string(),
                item_type: ItemType::Amulet,
                material: None,
                rarity: ItemRarity::Rare,
                weight: 0.2,
                value: 150,
                damage_base: 0,
                armor_base: 0,
                stat_requirements: Stats::zeroed(),
                stat_bonuses: Stats::new(1, 0, 0, 2, 0, 0),
                equip_slot: Some(EquipSlot::Amulet),
                stack_size: 1,
                quantity: 1,
                description: "A tarnished iron-age amulet of great antiquity. +1 STR, +2 CON.".to_string(),
                effects: Vec::new(),
                is_two_handed: false,
            }
        },
        Item {
            id: "tomb_seal_ring".to_string(),
            name: "Tomb Seal Ring".to_string(),
            item_type: ItemType::Ring,
            material: None,
            rarity: ItemRarity::Uncommon,
            weight: 0.1,
            value: 80,
            damage_base: 0,
            armor_base: 0,
            stat_requirements: Stats::zeroed(),
            stat_bonuses: Stats::new(0, 0, 2, 0, 0, 2),
            equip_slot: Some(EquipSlot::Ring1),
            stack_size: 1,
            quantity: 1,
            description: "The seal ring of a Valley King's official. +2 WIS, +2 CHA.".to_string(),
            effects: Vec::new(),
            is_two_handed: false,
        },
        Item {
            id: "valley_king_crown".to_string(),
            name: "Valley King's Crown".to_string(),
            item_type: ItemType::ArmorPiece(EquipSlot::Helmet),
            material: Some(MaterialTier::Iron),
            rarity: ItemRarity::Legendary,
            weight: 1.0,
            value: 500,
            damage_base: 0,
            armor_base: 8,
            stat_requirements: Stats::zeroed(),
            stat_bonuses: Stats::new(3, 0, 0, 3, 0, 3),
            equip_slot: Some(EquipSlot::Helmet),
            stack_size: 1,
            quantity: 1,
            description: "The legendary crown of Embervale's ancient Valley King. \
                          Its tarnished gold still carries immense authority. \
                          +3 STR, +3 CON, +3 CHA.".to_string(),
            effects: Vec::new(),
            is_two_handed: false,
        },
    ]
}

/// Find a catalog item by id.
pub fn find_item(id: &str) -> Option<Item> {
    all_items().into_iter().find(|i| i.id == id)
}

// ── Location Loot Tables ──────────────────────────────────────────────────────

/// A single entry in a loot table with weighted probability.
#[derive(Debug, Clone)]
pub struct LootEntry {
    pub item_id: String,
    /// Relative probability weight (higher = more likely).
    pub weight: u32,
    pub quantity_min: u32,
    pub quantity_max: u32,
}

/// A loot table associated with a searchable location.
#[derive(Debug, Clone)]
pub struct LootTable {
    pub id: String,
    /// Flavor text shown when the player searches this location.
    pub flavor_text: String,
    /// Items that are always present when the location is searched.
    pub guaranteed_items: Vec<(String, u32)>,
    /// Weighted pool of optional items.
    pub random_entries: Vec<LootEntry>,
    pub gold_min: u32,
    pub gold_max: u32,
    /// Number of times to draw from the random pool.
    pub roll_count: u32,
}

impl LootTable {
    /// Roll this loot table and return (gold, item_id/quantity pairs).
    pub fn roll(&self, rng: &mut impl Rng) -> (u32, Vec<(String, u32)>) {
        let gold = if self.gold_max > self.gold_min {
            rng.gen_range(self.gold_min..=self.gold_max)
        } else {
            self.gold_min
        };

        let mut items: Vec<(String, u32)> = self.guaranteed_items.clone();

        let total_weight: u32 = self.random_entries.iter().map(|e| e.weight).sum();
        if total_weight > 0 {
            for _ in 0..self.roll_count {
                let roll = rng.gen_range(0..total_weight);
                let mut cumulative = 0u32;
                for entry in &self.random_entries {
                    cumulative += entry.weight;
                    if roll < cumulative {
                        let qty = if entry.quantity_max > entry.quantity_min {
                            rng.gen_range(entry.quantity_min..=entry.quantity_max)
                        } else {
                            entry.quantity_min
                        };
                        if qty > 0 {
                            if let Some(existing) = items.iter_mut().find(|(id, _)| id == &entry.item_id) {
                                existing.1 += qty;
                            } else {
                                items.push((entry.item_id.clone(), qty));
                            }
                        }
                        break;
                    }
                }
            }
        }

        (gold, items)
    }
}

/// Find a loot table by location id.
pub fn find_loot_table(id: &str) -> Option<LootTable> {
    all_loot_tables().into_iter().find(|t| t.id == id)
}

/// All searchable location loot tables.
pub fn all_loot_tables() -> Vec<LootTable> {
    vec![
        // ── Wolf Den ─────────────────────────────────────────────────────────
        LootTable {
            id: "wolf_den_entrance".to_string(),
            flavor_text: "You search among the scattered bones and debris at the cave threshold. \
                          The remains of past victims yield a few useful items.".to_string(),
            guaranteed_items: vec![("bones".to_string(), 2)],
            random_entries: vec![
                LootEntry { item_id: "wolf_pelt".to_string(), weight: 60, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "meat".to_string(), weight: 40, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "sinew".to_string(), weight: 30, quantity_min: 1, quantity_max: 1 },
            ],
            gold_min: 0, gold_max: 3,
            roll_count: 1,
        },
        LootTable {
            id: "wolf_den_lair".to_string(),
            flavor_text: "You root through the alpha's trophies — gnawed gear and plunder \
                          from past victims. A hunter's pack lies intact beneath a shelf of rock.".to_string(),
            guaranteed_items: vec![
                ("dire_wolf_fang".to_string(), 1),
                ("wolf_pelt".to_string(), 2),
            ],
            random_entries: vec![
                LootEntry { item_id: "alpha_pelt".to_string(), weight: 70, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "health_potion".to_string(), weight: 50, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "leather".to_string(), weight: 40, quantity_min: 1, quantity_max: 3 },
                LootEntry { item_id: "crude_knife".to_string(), weight: 30, quantity_min: 1, quantity_max: 1 },
            ],
            gold_min: 5, gold_max: 15,
            roll_count: 2,
        },
        // ── Crystal Cave ─────────────────────────────────────────────────────
        LootTable {
            id: "crystal_cave_entrance".to_string(),
            flavor_text: "Crystal shards have fallen from the walls. You gather \
                          what you can without disturbing the delicate formations.".to_string(),
            guaranteed_items: vec![("crystal_shard".to_string(), 2)],
            random_entries: vec![
                LootEntry { item_id: "crystalline_dust".to_string(), weight: 60, quantity_min: 1, quantity_max: 3 },
                LootEntry { item_id: "crystal_shard".to_string(), weight: 40, quantity_min: 1, quantity_max: 2 },
            ],
            gold_min: 0, gold_max: 5,
            roll_count: 1,
        },
        LootTable {
            id: "crystal_cave_depths".to_string(),
            flavor_text: "The crystal formations here are magnificent. Embedded deep in the \
                          growth you find items drawn here over centuries by the crystal's power. \
                          A bear's hoard of curiosities spills from a rocky shelf.".to_string(),
            guaranteed_items: vec![
                ("bear_claw".to_string(), 1),
                ("crystal_shard".to_string(), 3),
                ("crystalline_dust".to_string(), 2),
            ],
            random_entries: vec![
                LootEntry { item_id: "crystal_ring".to_string(), weight: 40, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "health_potion".to_string(), weight: 60, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "iron_ingot".to_string(), weight: 50, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "crystal_shard".to_string(), weight: 80, quantity_min: 2, quantity_max: 4 },
            ],
            gold_min: 10, gold_max: 30,
            roll_count: 3,
        },
        // ── Shadow Gorge & Cave ───────────────────────────────────────────────
        LootTable {
            id: "shadow_gorge".to_string(),
            flavor_text: "You search the gorge floor among wet rocks and refuse. \
                          Someone camped here recently and left in a hurry.".to_string(),
            guaranteed_items: vec![],
            random_entries: vec![
                LootEntry { item_id: "meat".to_string(), weight: 40, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "herbs".to_string(), weight: 30, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "crude_knife".to_string(), weight: 20, quantity_min: 1, quantity_max: 1 },
            ],
            gold_min: 0, gold_max: 4,
            roll_count: 1,
        },
        LootTable {
            id: "shadow_cave_entrance".to_string(),
            flavor_text: "Goblin scratches cover every surface. Discarded weapons and refuse \
                          litter the ground — much of it stolen from travellers.".to_string(),
            guaranteed_items: vec![("tattered_cloth".to_string(), 1)],
            random_entries: vec![
                LootEntry { item_id: "crude_knife".to_string(), weight: 50, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "crude_bow".to_string(), weight: 30, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "sinew".to_string(), weight: 40, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "feather".to_string(), weight: 35, quantity_min: 2, quantity_max: 4 },
            ],
            gold_min: 2, gold_max: 8,
            roll_count: 2,
        },
        LootTable {
            id: "shadow_cave_depths".to_string(),
            flavor_text: "This is a goblin staging post. Stolen goods, crude weapons, \
                          and plundered supplies are piled high in makeshift chests. \
                          You pick through the haul carefully.".to_string(),
            guaranteed_items: vec![
                ("iron_ingot".to_string(), 2),
                ("crude_sword".to_string(), 1),
            ],
            random_entries: vec![
                LootEntry { item_id: "iron_short_sword".to_string(), weight: 30, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "health_potion".to_string(), weight: 50, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "leather".to_string(), weight: 40, quantity_min: 2, quantity_max: 4 },
                LootEntry { item_id: "tattered_cloth".to_string(), weight: 60, quantity_min: 1, quantity_max: 3 },
                LootEntry { item_id: "shaman_staff".to_string(), weight: 20, quantity_min: 1, quantity_max: 1 },
            ],
            gold_min: 15, gold_max: 40,
            roll_count: 3,
        },
        // ── Derelict Buildings ────────────────────────────────────────────────
        LootTable {
            id: "derelict_mill".to_string(),
            flavor_text: "You search the rotting interior. Old tools and long-forgotten \
                          stores are scattered about — some still salvageable.".to_string(),
            guaranteed_items: vec![("wood".to_string(), 2)],
            random_entries: vec![
                LootEntry { item_id: "iron_ingot".to_string(), weight: 30, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "wood_shaft".to_string(), weight: 50, quantity_min: 1, quantity_max: 3 },
                LootEntry { item_id: "crude_knife".to_string(), weight: 20, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "feather".to_string(), weight: 40, quantity_min: 2, quantity_max: 5 },
            ],
            gold_min: 0, gold_max: 6,
            roll_count: 2,
        },
        LootTable {
            id: "abandoned_farmstead".to_string(),
            flavor_text: "The bandit camp is littered with stolen supplies and discarded gear. \
                          You make the most of it — bandits always steal more than they can carry.".to_string(),
            guaranteed_items: vec![("tattered_cloth".to_string(), 1)],
            random_entries: vec![
                LootEntry { item_id: "iron_ingot".to_string(), weight: 40, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "health_potion".to_string(), weight: 30, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "crude_knife".to_string(), weight: 50, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "bandit_cloak".to_string(), weight: 20, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "leather".to_string(), weight: 40, quantity_min: 1, quantity_max: 2 },
            ],
            gold_min: 8, gold_max: 20,
            roll_count: 2,
        },
        LootTable {
            id: "valley_watchtower".to_string(),
            flavor_text: "You root through the tower's collapsed upper floor. \
                          Someone left supplies here not long ago — a guard cache, \
                          abandoned when the post was given up.".to_string(),
            guaranteed_items: vec![],
            random_entries: vec![
                LootEntry { item_id: "shortbow".to_string(), weight: 30, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "iron_ingot".to_string(), weight: 40, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "wood_shaft".to_string(), weight: 50, quantity_min: 1, quantity_max: 3 },
                LootEntry { item_id: "leather".to_string(), weight: 30, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "feather".to_string(), weight: 45, quantity_min: 3, quantity_max: 6 },
            ],
            gold_min: 3, gold_max: 12,
            roll_count: 2,
        },
        LootTable {
            id: "millford_ruins".to_string(),
            flavor_text: "You sift through the overgrown foundations. Old coins and personal \
                          trinkets remain from Millford's vanished inhabitants — undisturbed \
                          for two centuries.".to_string(),
            guaranteed_items: vec![("ancient_coin".to_string(), 2)],
            random_entries: vec![
                LootEntry { item_id: "herbs".to_string(), weight: 50, quantity_min: 1, quantity_max: 3 },
                LootEntry { item_id: "bones".to_string(), weight: 40, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "tattered_cloth".to_string(), weight: 30, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "ancient_coin".to_string(), weight: 40, quantity_min: 1, quantity_max: 3 },
            ],
            gold_min: 0, gold_max: 5,
            roll_count: 2,
        },
        // ── Crypts ────────────────────────────────────────────────────────────
        LootTable {
            id: "millford_crypt".to_string(),
            flavor_text: "You inspect the broken sarcophagi. The grave goods were plundered \
                          long ago, but fragments remain in the wall niches — offerings \
                          placed for the dead that survived the centuries.".to_string(),
            guaranteed_items: vec![
                ("bones".to_string(), 2),
                ("ancient_coin".to_string(), 1),
            ],
            random_entries: vec![
                LootEntry { item_id: "ancient_coin".to_string(), weight: 50, quantity_min: 1, quantity_max: 3 },
                LootEntry { item_id: "tattered_cloth".to_string(), weight: 40, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "carved_bone".to_string(), weight: 30, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "antidote".to_string(), weight: 20, quantity_min: 1, quantity_max: 1 },
            ],
            gold_min: 0, gold_max: 8,
            roll_count: 2,
        },
        LootTable {
            id: "millford_crypt_depths".to_string(),
            flavor_text: "The ossuary is a grim trove. Among the stacked bones you find \
                          offerings placed centuries ago — personal effects of Millford's \
                          dead that no one living knew to claim.".to_string(),
            guaranteed_items: vec![
                ("ancient_coin".to_string(), 3),
                ("ghost_essence".to_string(), 1),
            ],
            random_entries: vec![
                LootEntry { item_id: "mouldering_chainmail".to_string(), weight: 25, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "crude_sword".to_string(), weight: 20, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "carved_bone".to_string(), weight: 40, quantity_min: 1, quantity_max: 3 },
                LootEntry { item_id: "witch_talisman".to_string(), weight: 15, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "health_potion".to_string(), weight: 20, quantity_min: 1, quantity_max: 1 },
            ],
            gold_min: 5, gold_max: 20,
            roll_count: 3,
        },
        LootTable {
            id: "barrow_interior".to_string(),
            flavor_text: "You examine the grave-niche offerings of the ancient warriors. \
                          Some items have survived the ages in the sealed passages — \
                          corroded but still serviceable.".to_string(),
            guaranteed_items: vec![
                ("ancient_coin".to_string(), 2),
                ("carved_bone".to_string(), 1),
            ],
            random_entries: vec![
                LootEntry { item_id: "iron_spear".to_string(), weight: 25, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "mouldering_chainmail".to_string(), weight: 20, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "ancient_coin".to_string(), weight: 50, quantity_min: 2, quantity_max: 5 },
                LootEntry { item_id: "bones".to_string(), weight: 60, quantity_min: 1, quantity_max: 3 },
                LootEntry { item_id: "ghost_essence".to_string(), weight: 20, quantity_min: 1, quantity_max: 1 },
            ],
            gold_min: 5, gold_max: 20,
            roll_count: 2,
        },
        LootTable {
            id: "barrow_lord_chamber".to_string(),
            flavor_text: "You approach the chieftain's sarcophagus and claim the burial \
                          treasures of an iron-age king. The hoard is ancient and potent — \
                          weapons and armour wrought by smiths whose craft has been lost for \
                          two centuries.".to_string(),
            guaranteed_items: vec![
                ("ancient_coin".to_string(), 5),
                ("ghost_essence".to_string(), 1),
            ],
            random_entries: vec![
                LootEntry { item_id: "runic_short_sword".to_string(), weight: 50, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "ancient_amulet".to_string(), weight: 40, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "barrow_lord_helm".to_string(), weight: 35, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "health_potion".to_string(), weight: 30, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "carved_bone".to_string(), weight: 60, quantity_min: 2, quantity_max: 4 },
            ],
            gold_min: 20, gold_max: 50,
            roll_count: 3,
        },
        // ── Ironmere Keep ─────────────────────────────────────────────────────
        LootTable {
            id: "ironmere_courtyard".to_string(),
            flavor_text: "You search the goblin encampment. Stolen goods and crude supplies \
                          fill makeshift chests — plunder from raided caravans and farms.".to_string(),
            guaranteed_items: vec![
                ("iron_ingot".to_string(), 2),
                ("tattered_cloth".to_string(), 1),
            ],
            random_entries: vec![
                LootEntry { item_id: "crude_sword".to_string(), weight: 40, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "health_potion".to_string(), weight: 30, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "leather".to_string(), weight: 50, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "iron_ingot".to_string(), weight: 30, quantity_min: 1, quantity_max: 2 },
            ],
            gold_min: 10, gold_max: 25,
            roll_count: 2,
        },
        LootTable {
            id: "ironmere_tower".to_string(),
            flavor_text: "The plundered cache in the tower's ground floor holds considerable \
                          wealth — valley merchants' goods, tribute extorted from travellers, \
                          and the warlord's own hoard of spoils. A chest in the corner \
                          was clearly the warlord's personal treasury.".to_string(),
            guaranteed_items: vec![
                ("iron_ingot".to_string(), 3),
                ("health_potion".to_string(), 2),
                ("leather".to_string(), 3),
            ],
            random_entries: vec![
                LootEntry { item_id: "iron_short_sword".to_string(), weight: 30, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "iron_long_sword".to_string(), weight: 15, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "leather_helmet".to_string(), weight: 25, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "leather_torso".to_string(), weight: 20, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "shaman_staff".to_string(), weight: 20, quantity_min: 1, quantity_max: 1 },
            ],
            gold_min: 30, gold_max: 70,
            roll_count: 3,
        },
        // ── Valley King's Tomb ────────────────────────────────────────────────
        LootTable {
            id: "tomb_antechamber".to_string(),
            flavor_text: "You investigate the stone guardians' alcoves and the faded offering \
                          tables. The Valley King did not go unaccompanied into death — \
                          his honour guard took their worldly possessions with them.".to_string(),
            guaranteed_items: vec![
                ("ancient_coin".to_string(), 4),
                ("bones".to_string(), 2),
            ],
            random_entries: vec![
                LootEntry { item_id: "mouldering_chainmail".to_string(), weight: 30, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "ghost_essence".to_string(), weight: 25, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "carved_bone".to_string(), weight: 50, quantity_min: 1, quantity_max: 3 },
                LootEntry { item_id: "health_potion".to_string(), weight: 30, quantity_min: 1, quantity_max: 2 },
                LootEntry { item_id: "tomb_seal_ring".to_string(), weight: 20, quantity_min: 1, quantity_max: 1 },
            ],
            gold_min: 15, gold_max: 40,
            roll_count: 3,
        },
        LootTable {
            id: "tomb_sanctum".to_string(),
            flavor_text: "You stand before the Valley King's golden sarcophagus. \
                          The treasures of a lost kingdom surround you — tarnished gold, \
                          weapons of another age, and relics of immense power. \
                          This is the greatest hoard in all of Embervale.".to_string(),
            guaranteed_items: vec![
                ("ancient_coin".to_string(), 8),
                ("ghost_essence".to_string(), 2),
                ("ancient_tome".to_string(), 1),
            ],
            random_entries: vec![
                LootEntry { item_id: "valley_king_crown".to_string(), weight: 60, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "ancient_amulet".to_string(), weight: 50, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "tomb_seal_ring".to_string(), weight: 40, quantity_min: 1, quantity_max: 1 },
                LootEntry { item_id: "health_potion".to_string(), weight: 60, quantity_min: 2, quantity_max: 3 },
                LootEntry { item_id: "runic_short_sword".to_string(), weight: 30, quantity_min: 1, quantity_max: 1 },
            ],
            gold_min: 50, gold_max: 100,
            roll_count: 4,
        },
    ]
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
