use std::collections::HashMap;
use iron_age_core::{Stats, StatusEffect, Row, GameError};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Skill {
    Perception, Diplomacy, SurvivalUrban, SurvivalNature, FaithRituals,
    Stealth, Acrobatics, Tracking, Investigation, Tinkering,
    Mining, Gathering, Cooking, Weaponsmithing, Armorsmithing, Alchemy, BoyerFletcher,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillLevel {
    pub level: u32,
    pub experience: u64,
}

impl SkillLevel {
    pub fn new() -> Self { Self { level: 0, experience: 0 } }

    pub fn add_experience(&mut self, xp: u64) -> bool {
        self.experience += xp;
        let needed = self.xp_needed_for_next();
        if self.experience >= needed && self.level < 100 {
            self.experience -= needed;
            self.level += 1;
            true
        } else {
            false
        }
    }

    pub fn xp_needed_for_next(&self) -> u64 { 100 + self.level as u64 * 50 }
}

impl Default for SkillLevel {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Perk {
    Resilient, Sharpened, Mastermind, Transcendent,
    IronWill, NaturalHealer, BattleHardened, SwiftFoot,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Character {
    pub name: String,
    pub level: u32,
    pub experience: u64,
    pub stats: Stats,
    pub hp: i32,
    pub max_hp: i32,
    pub stamina: i32,
    pub max_stamina: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub skills: HashMap<Skill, SkillLevel>,
    pub perks: Vec<Perk>,
    pub stat_points: u32,
    pub skill_points: u32,
    pub row: Row,
    pub status_effects: Vec<StatusEffect>,
}

impl Character {
    pub fn new(name: String) -> Self {
        let stats = Stats::base_player();
        let max_hp = Self::max_hp_base(stats.constitution);
        let max_stamina = Self::max_stamina_base(stats.constitution, stats.strength);
        let max_mana = Self::max_mana_base(stats.intelligence, stats.wisdom);
        Self {
            name, level: 1, experience: 0, stats,
            hp: max_hp, max_hp,
            stamina: max_stamina, max_stamina,
            mana: max_mana, max_mana,
            skills: HashMap::new(), perks: Vec::new(),
            stat_points: 0, skill_points: 0,
            row: Row::Front, status_effects: Vec::new(),
        }
    }

    pub fn is_alive(&self) -> bool { self.hp > 0 }

    pub fn take_damage(&mut self, amount: i32) -> i32 {
        let actual = amount.max(0);
        self.hp = (self.hp - actual).max(0);
        actual
    }

    pub fn heal(&mut self, amount: i32) -> i32 {
        let before = self.hp;
        self.hp = (self.hp + amount).min(self.max_hp);
        self.hp - before
    }

    pub fn add_experience(&mut self, xp: u64) -> Vec<u32> {
        let xp = if self.perks.contains(&Perk::Transcendent) {
            (xp as f64 * 1.15) as u64
        } else { xp };
        self.experience += xp;
        let mut levels = Vec::new();
        loop {
            let needed = Self::xp_for_level(self.level + 1);
            if self.experience >= needed {
                self.experience -= needed;
                self.level_up();
                levels.push(self.level);
            } else { break; }
        }
        levels
    }

    pub fn level_up(&mut self) {
        self.level += 1;
        self.stat_points += Self::stat_points_per_level(self.level);
        self.skill_points += Self::skill_points_per_level(self.level);
        self.max_hp = Self::max_hp_base(self.stats.constitution);
        self.max_stamina = Self::max_stamina_base(self.stats.constitution, self.stats.strength);
        self.max_mana = Self::max_mana_base(self.stats.intelligence, self.stats.wisdom);
        self.hp = self.max_hp;
        self.stamina = self.max_stamina;
        self.mana = self.max_mana;
        self.check_perk_unlock();
    }

    pub fn stat_points_per_level(level: u32) -> u32 {
        if level <= 50 { return 3; }
        let extra = level - 50;
        let divisor = 2u32.pow(extra / 10);
        let pts = 3 / divisor.max(1);
        if pts == 0 { if extra % 2 == 0 { 1 } else { 0 } } else { pts }
    }

    pub fn skill_points_per_level(level: u32) -> u32 {
        if level <= 50 { 2 } else { 1 }
    }

    pub fn xp_for_level(level: u32) -> u64 {
        if level <= 1 { return 0; }
        if level <= 50 {
            (100.0 * 1.2_f64.powi(level as i32 - 1)) as u64
        } else {
            let base = (100.0 * 1.2_f64.powi(49)) as f64;
            (base * 1.5_f64.powi((level - 50) as i32)) as u64
        }
    }

    pub fn allocate_stat(&mut self, stat: &str, points: u32) -> Result<(), GameError> {
        if self.stat_points < points {
            return Err(GameError::InvalidOperation("Not enough stat points".to_string()));
        }
        match stat {
            "STR" | "strength" => self.stats.strength += points as i32,
            "INT" | "intelligence" => self.stats.intelligence += points as i32,
            "WIS" | "wisdom" => self.stats.wisdom += points as i32,
            "CON" | "constitution" => self.stats.constitution += points as i32,
            "DEX" | "dexterity" => self.stats.dexterity += points as i32,
            "CHA" | "charisma" => self.stats.charisma += points as i32,
            _ => return Err(GameError::NotFound(format!("Stat '{}' not found", stat))),
        }
        self.stat_points -= points;
        self.max_hp = Self::max_hp_base(self.stats.constitution);
        self.max_stamina = Self::max_stamina_base(self.stats.constitution, self.stats.strength);
        self.max_mana = Self::max_mana_base(self.stats.intelligence, self.stats.wisdom);
        Ok(())
    }

    pub fn add_status_effect(&mut self, effect: StatusEffect) {
        self.status_effects.push(effect);
    }

    pub fn tick_status_effects(&mut self) {
        let effects = self.status_effects.clone();
        for effect in &effects {
            match effect {
                StatusEffect::Bleed { damage_per_turn, .. }
                | StatusEffect::Poison { damage_per_turn, .. }
                | StatusEffect::Burn { damage_per_turn, .. } => {
                    self.hp = (self.hp - damage_per_turn).max(0);
                }
                StatusEffect::Regen { heal_per_turn, .. } => {
                    self.hp = (self.hp + heal_per_turn).min(self.max_hp);
                }
                _ => {}
            }
        }
        self.status_effects.retain_mut(|e| e.tick());
    }

    pub fn has_status(&self, effect_name: &str) -> bool {
        self.status_effects.iter().any(|e| e.name() == effect_name)
    }

    pub fn check_perk_unlock(&mut self) {
        if self.level >= 10 && !self.perks.contains(&Perk::Resilient) {
            self.perks.push(Perk::Resilient);
            self.max_hp = (self.max_hp as f32 * 1.10) as i32;
            self.hp = self.hp.min(self.max_hp);
        }
        if self.level >= 25 && !self.perks.contains(&Perk::Sharpened) {
            self.perks.push(Perk::Sharpened);
        }
        if self.level >= 50 && !self.perks.contains(&Perk::Mastermind) {
            self.perks.push(Perk::Mastermind);
            self.stats.strength += 1; self.stats.intelligence += 1;
            self.stats.wisdom += 1; self.stats.constitution += 1;
            self.stats.dexterity += 1; self.stats.charisma += 1;
        }
        if self.level >= 75 && !self.perks.contains(&Perk::Transcendent) {
            self.perks.push(Perk::Transcendent);
        }
    }

    pub fn max_hp_base(con: i32) -> i32 { 50 + con * 10 }
    pub fn max_stamina_base(con: i32, str: i32) -> i32 { 30 + (con + str) * 3 }
    pub fn max_mana_base(int: i32, wis: i32) -> i32 { 20 + (int + wis) * 5 }

    /// Return the current level for a named crafting skill (0 if untrained).
    pub fn get_craft_skill(&self, skill_name: &str) -> u32 {
        let skill = Self::crafting_skill_from_name(skill_name);
        skill.and_then(|s| self.skills.get(&s)).map_or(0, |sl| sl.level)
    }

    /// Add XP to a named crafting skill. Returns `true` if the skill levelled up.
    pub fn gain_craft_xp(&mut self, skill_name: &str, xp: u64) -> bool {
        if let Some(skill) = Self::crafting_skill_from_name(skill_name) {
            self.skills.entry(skill).or_insert_with(SkillLevel::new).add_experience(xp)
        } else {
            false
        }
    }

    fn crafting_skill_from_name(name: &str) -> Option<Skill> {
        match name {
            "Weaponsmithing" => Some(Skill::Weaponsmithing),
            "Armorsmithing"  => Some(Skill::Armorsmithing),
            "Alchemy"        => Some(Skill::Alchemy),
            "Cooking"        => Some(Skill::Cooking),
            "Mining"         => Some(Skill::Mining),
            "Gathering"      => Some(Skill::Gathering),
            "BoyerFletcher"  => Some(Skill::BoyerFletcher),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_curve_increases() {
        for i in 1..10u32 {
            assert!(Character::xp_for_level(i + 1) > Character::xp_for_level(i));
        }
    }

    #[test]
    fn test_diminishing_returns_after_50() {
        assert!(Character::stat_points_per_level(51) <= Character::stat_points_per_level(1));
    }

    #[test]
    fn test_level_up_grants_stat_points() {
        let mut c = Character::new("Test".to_string());
        c.add_experience(500);
        assert!(c.stat_points > 0 || c.level > 1);
    }

    #[test]
    fn test_status_effects_tick_and_expire() {
        let mut c = Character::new("Test".to_string());
        c.add_status_effect(StatusEffect::Bleed { damage_per_turn: 1, turns_remaining: 2 });
        assert!(c.has_status("Bleed"));
        c.tick_status_effects();
        assert!(c.has_status("Bleed"));
        c.tick_status_effects();
        assert!(!c.has_status("Bleed"));
    }

    #[test]
    fn test_perk_unlock_at_level_10() {
        let mut c = Character::new("Test".to_string());
        for _ in 0..9 { c.level_up(); }
        assert!(c.perks.contains(&Perk::Resilient));
    }

    #[test]
    fn test_character_cannot_go_below_zero_hp() {
        let mut c = Character::new("Test".to_string());
        c.take_damage(99999);
        assert_eq!(c.hp, 0);
    }

    #[test]
    fn test_xp_for_level_grows_monotonically() {
        for i in 1..100u32 {
            assert!(Character::xp_for_level(i + 1) > Character::xp_for_level(i),
                "level {} -> {} failed", i, i+1);
        }
    }

    #[test]
    fn test_get_craft_skill_returns_zero_for_untrained() {
        let c = Character::new("Test".to_string());
        assert_eq!(c.get_craft_skill("Weaponsmithing"), 0);
        assert_eq!(c.get_craft_skill("Alchemy"), 0);
        assert_eq!(c.get_craft_skill("UnknownSkill"), 0);
    }

    #[test]
    fn test_gain_craft_xp_levels_up_skill() {
        let mut c = Character::new("Test".to_string());
        assert_eq!(c.get_craft_skill("Alchemy"), 0);
        // First level requires 100 XP
        let levelled = c.gain_craft_xp("Alchemy", 100);
        assert!(levelled, "Should have levelled up after 100 XP");
        assert_eq!(c.get_craft_skill("Alchemy"), 1);
    }

    #[test]
    fn test_gain_craft_xp_unknown_skill_returns_false() {
        let mut c = Character::new("Test".to_string());
        assert!(!c.gain_craft_xp("Teleportation", 9999));
    }
}
