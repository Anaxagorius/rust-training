use iron_age_core::{DamageType, ElementalResistances, StatusEffect};
use iron_age_character::Character;
use rand::Rng;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AbilityCategory { Physical, Magic, Psionic }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ability {
    pub name: String,
    pub category: AbilityCategory,
    pub description: String,
    pub stamina_cost: i32,
    pub mana_cost: i32,
    pub cooldown: u32,
    pub current_cooldown: u32,
    pub damage_type: DamageType,
}

pub fn all_abilities() -> Vec<Ability> {
    vec![
        // Physical (10)
        Ability { name: "True Aim".into(), category: AbilityCategory::Physical,
            description: "Next attack guaranteed hit, +crit chance.".into(),
            stamina_cost: 15, mana_cost: 0, cooldown: 2, current_cooldown: 0, damage_type: DamageType::Physical },
        Ability { name: "Braced Strike".into(), category: AbilityCategory::Physical,
            description: "Heavy hit that ignores 30% armor.".into(),
            stamina_cost: 20, mana_cost: 0, cooldown: 3, current_cooldown: 0, damage_type: DamageType::Physical },
        Ability { name: "Feint".into(), category: AbilityCategory::Physical,
            description: "Lowers enemy guard, boosts next ally hit.".into(),
            stamina_cost: 12, mana_cost: 0, cooldown: 2, current_cooldown: 0, damage_type: DamageType::Physical },
        Ability { name: "Shield Bash".into(), category: AbilityCategory::Physical,
            description: "Damage + stun chance.".into(),
            stamina_cost: 18, mana_cost: 0, cooldown: 3, current_cooldown: 0, damage_type: DamageType::Physical },
        Ability { name: "Riposte Stance".into(), category: AbilityCategory::Physical,
            description: "Counterattack on being hit.".into(),
            stamina_cost: 10, mana_cost: 0, cooldown: 4, current_cooldown: 0, damage_type: DamageType::Physical },
        Ability { name: "War Cry".into(), category: AbilityCategory::Physical,
            description: "Party ATK up, enemies' morale down.".into(),
            stamina_cost: 15, mana_cost: 0, cooldown: 4, current_cooldown: 0, damage_type: DamageType::Physical },
        Ability { name: "Second Wind".into(), category: AbilityCategory::Physical,
            description: "Restore HP/stamina based on missing HP.".into(),
            stamina_cost: 5, mana_cost: 0, cooldown: 5, current_cooldown: 0, damage_type: DamageType::Physical },
        Ability { name: "Hamstring".into(), category: AbilityCategory::Physical,
            description: "Damage + slow.".into(),
            stamina_cost: 14, mana_cost: 0, cooldown: 2, current_cooldown: 0, damage_type: DamageType::Physical },
        Ability { name: "Perfect Form".into(), category: AbilityCategory::Physical,
            description: "Short buff to accuracy/evasion.".into(),
            stamina_cost: 8, mana_cost: 0, cooldown: 3, current_cooldown: 0, damage_type: DamageType::Physical },
        Ability { name: "Last Stand".into(), category: AbilityCategory::Physical,
            description: "Survive fatal blow at 1 HP once per battle.".into(),
            stamina_cost: 0, mana_cost: 0, cooldown: 999, current_cooldown: 0, damage_type: DamageType::Physical },
        // Magic (10)
        Ability { name: "Ember Bind".into(), category: AbilityCategory::Magic,
            description: "Fire DoT + root.".into(),
            stamina_cost: 0, mana_cost: 18, cooldown: 2, current_cooldown: 0, damage_type: DamageType::Fire },
        Ability { name: "Bog Shroud".into(), category: AbilityCategory::Magic,
            description: "Nature mist; enemy accuracy down, stealth up.".into(),
            stamina_cost: 0, mana_cost: 15, cooldown: 3, current_cooldown: 0, damage_type: DamageType::Nature },
        Ability { name: "Stone Ward".into(), category: AbilityCategory::Magic,
            description: "Physical shield absorbs damage.".into(),
            stamina_cost: 0, mana_cost: 20, cooldown: 3, current_cooldown: 0, damage_type: DamageType::Physical },
        Ability { name: "Oak's Grasp".into(), category: AbilityCategory::Magic,
            description: "Nature damage + entangle.".into(),
            stamina_cost: 0, mana_cost: 22, cooldown: 2, current_cooldown: 0, damage_type: DamageType::Nature },
        Ability { name: "River's Grace".into(), category: AbilityCategory::Magic,
            description: "Heal-over-time + cleanse minor debuffs.".into(),
            stamina_cost: 0, mana_cost: 25, cooldown: 3, current_cooldown: 0, damage_type: DamageType::Holy },
        Ability { name: "Thunder Wick".into(), category: AbilityCategory::Magic,
            description: "Chain lightning; risks friendly fire at high power.".into(),
            stamina_cost: 0, mana_cost: 30, cooldown: 3, current_cooldown: 0, damage_type: DamageType::Lightning },
        Ability { name: "Frost Veil".into(), category: AbilityCategory::Magic,
            description: "Damage + slow + resist Fire.".into(),
            stamina_cost: 0, mana_cost: 20, cooldown: 2, current_cooldown: 0, damage_type: DamageType::Frost },
        Ability { name: "Wight's Lantern".into(), category: AbilityCategory::Magic,
            description: "Reveal hidden/ethereal enemies.".into(),
            stamina_cost: 0, mana_cost: 10, cooldown: 5, current_cooldown: 0, damage_type: DamageType::Holy },
        Ability { name: "Runic Surge".into(), category: AbilityCategory::Magic,
            description: "Convert WIS into spell power for 3 turns.".into(),
            stamina_cost: 0, mana_cost: 15, cooldown: 4, current_cooldown: 0, damage_type: DamageType::Nature },
        Ability { name: "Ashen Gale".into(), category: AbilityCategory::Magic,
            description: "AOE Fire/Nature hybrid; synergy with resin/explosives.".into(),
            stamina_cost: 0, mana_cost: 35, cooldown: 4, current_cooldown: 0, damage_type: DamageType::Fire },
        // Psionic (10)
        Ability { name: "Mind Lance".into(), category: AbilityCategory::Psionic,
            description: "Pure psychic damage; bypasses armor.".into(),
            stamina_cost: 0, mana_cost: 20, cooldown: 1, current_cooldown: 0, damage_type: DamageType::Psychic },
        Ability { name: "Aegis Thought".into(), category: AbilityCategory::Psionic,
            description: "Mental barrier; resist fear/charm.".into(),
            stamina_cost: 0, mana_cost: 15, cooldown: 3, current_cooldown: 0, damage_type: DamageType::Psychic },
        Ability { name: "Stillness".into(), category: AbilityCategory::Psionic,
            description: "Silence enemy caster.".into(),
            stamina_cost: 0, mana_cost: 18, cooldown: 3, current_cooldown: 0, damage_type: DamageType::Psychic },
        Ability { name: "Echo Step".into(), category: AbilityCategory::Psionic,
            description: "Act earlier next round; initiative boost.".into(),
            stamina_cost: 0, mana_cost: 12, cooldown: 3, current_cooldown: 0, damage_type: DamageType::Psychic },
        Ability { name: "Premonition".into(), category: AbilityCategory::Psionic,
            description: "See next enemy action; +dodge.".into(),
            stamina_cost: 0, mana_cost: 14, cooldown: 2, current_cooldown: 0, damage_type: DamageType::Psychic },
        Ability { name: "Sunder Will".into(), category: AbilityCategory::Psionic,
            description: "Reduce enemy damage for 2 turns.".into(),
            stamina_cost: 0, mana_cost: 16, cooldown: 3, current_cooldown: 0, damage_type: DamageType::Psychic },
        Ability { name: "Shared Pain".into(), category: AbilityCategory::Psionic,
            description: "Link two enemies; split incoming damage.".into(),
            stamina_cost: 0, mana_cost: 22, cooldown: 4, current_cooldown: 0, damage_type: DamageType::Psychic },
        Ability { name: "Calm the Flock".into(), category: AbilityCategory::Psionic,
            description: "Lower encounter chance/escape success up.".into(),
            stamina_cost: 0, mana_cost: 10, cooldown: 10, current_cooldown: 0, damage_type: DamageType::Psychic },
        Ability { name: "Mnemonic Spark".into(), category: AbilityCategory::Psionic,
            description: "Instantly learn one recipe if materials present.".into(),
            stamina_cost: 0, mana_cost: 30, cooldown: 20, current_cooldown: 0, damage_type: DamageType::Psychic },
        Ability { name: "Soul Beacon".into(), category: AbilityCategory::Psionic,
            description: "Revive downed ally with penalty.".into(),
            stamina_cost: 0, mana_cost: 40, cooldown: 10, current_cooldown: 0, damage_type: DamageType::Holy },
    ]
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BattleAction {
    Attack,
    Guard,
    UseAbility(String),
    UseItem(String),
    Flee,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CombatResult {
    pub damage_dealt: i32,
    pub damage_type: DamageType,
    pub is_critical: bool,
    pub is_hit: bool,
    pub effects_applied: Vec<String>,
    pub message: String,
}

pub struct CombatFormulas;

impl CombatFormulas {
    pub fn calculate_damage(
        weapon_base: i32,
        attacker_stat: i32,
        weapon_quality: i32,
        armor_mitigation: i32,
        damage_type: &DamageType,
        resistances: &ElementalResistances,
    ) -> i32 {
        let quality_bonus = (weapon_quality as f32 / 100.0 * weapon_base as f32) as i32;
        let raw = weapon_base + attacker_stat / 3 + quality_bonus;
        let after_armor = (raw - armor_mitigation).max(1);
        resistances.apply_damage(after_armor, damage_type)
    }

    pub fn calculate_hit_chance(attacker_dex: i32, hit_bonus: i32, target_evasion: i32) -> f32 {
        let raw = 0.7 + (attacker_dex + hit_bonus - target_evasion) as f32 * 0.02;
        raw.clamp(0.05, 0.99)
    }

    pub fn calculate_crit_chance(base: f32, gear_bonus: f32, ability_bonus: f32) -> f32 {
        (base + gear_bonus + ability_bonus).clamp(0.0, 0.75)
    }

    pub fn calculate_magic_damage(base: i32, intelligence: i32, level: u32) -> i32 {
        let int_scale = 1.0 + intelligence as f32 * 0.05;
        let level_scale = 1.0 + level as f32 * 0.01;
        (base as f32 * int_scale * level_scale) as i32
    }

    pub fn calculate_magic_cost(base_cost: i32, wisdom: i32) -> i32 {
        let reduction = (wisdom as f32 * 0.02).min(0.4);
        ((base_cost as f32) * (1.0 - reduction)) as i32
    }

    pub fn calculate_psionic_damage(base: i32, wisdom: i32, level: u32) -> i32 {
        let wis_scale = 1.0 + wisdom as f32 * 0.06;
        let level_scale = 1.0 + level as f32 * 0.01;
        (base as f32 * wis_scale * level_scale) as i32
    }

    pub fn calculate_psionic_hit_chance(base_chance: f32, intelligence: i32) -> f32 {
        (base_chance + intelligence as f32 * 0.01).clamp(0.1, 0.99)
    }

    pub fn calculate_initiative(dex: i32, random_tiebreak: f32) -> f32 {
        dex as f32 + random_tiebreak
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Combatant {
    pub character: Character,
    pub is_player: bool,
    pub weapon_base_damage: i32,
    pub weapon_quality: i32,
    pub armor_mitigation: i32,
    pub evasion: i32,
    pub initiative: f32,
    pub is_guarding: bool,
    pub last_stand_used: bool,
}

impl Combatant {
    pub fn new(character: Character, is_player: bool) -> Self {
        let weapon_base_damage = 5 + character.stats.strength / 2;
        let armor_mitigation = character.stats.constitution / 3;
        let evasion = character.stats.dexterity / 2;
        Self {
            character, is_player, weapon_base_damage, weapon_quality: 50,
            armor_mitigation, evasion, initiative: 0.0,
            is_guarding: false, last_stand_used: false,
        }
    }

    pub fn calculate_initiative(&mut self, rng: &mut impl Rng) {
        let tiebreak: f32 = rng.gen();
        self.initiative = CombatFormulas::calculate_initiative(self.character.stats.dexterity, tiebreak);
    }

    pub fn take_damage_in_combat(&mut self, mut damage: i32) -> i32 {
        if self.is_guarding { damage = (damage as f32 * 0.5) as i32; }
        // Check Last Stand
        if !self.last_stand_used && damage >= self.character.hp && self.character.hp > 1 {
            self.last_stand_used = true;
            self.character.hp = 1;
            return damage;
        }
        self.character.take_damage(damage)
    }

    pub fn take_turn(
        &mut self,
        action: BattleAction,
        targets: &mut Vec<Combatant>,
        rng: &mut impl Rng,
    ) -> Vec<CombatResult> {
        self.is_guarding = false;
        match action {
            BattleAction::Attack => {
                let hit_chance = CombatFormulas::calculate_hit_chance(
                    self.character.stats.dexterity, 0,
                    targets.first().map_or(0, |t| t.evasion),
                );
                let hit: f32 = rng.gen();
                if hit > hit_chance {
                    return vec![CombatResult {
                        damage_dealt: 0, damage_type: DamageType::Physical,
                        is_critical: false, is_hit: false,
                        effects_applied: Vec::new(),
                        message: format!("{} missed!", self.character.name),
                    }];
                }
                let crit_roll: f32 = rng.gen();
                let crit_chance = CombatFormulas::calculate_crit_chance(0.05, 0.0, 0.0);
                let is_crit = crit_roll < crit_chance;
                let mut results = Vec::new();
                if let Some(target) = targets.first_mut() {
                    let res = &ElementalResistances::none();
                    let mut dmg = CombatFormulas::calculate_damage(
                        self.weapon_base_damage, self.character.stats.strength,
                        self.weapon_quality, target.armor_mitigation,
                        &DamageType::Physical, res,
                    );
                    if is_crit { dmg = (dmg as f32 * 1.75) as i32; }
                    let actual = target.take_damage_in_combat(dmg);
                    results.push(CombatResult {
                        damage_dealt: actual, damage_type: DamageType::Physical,
                        is_critical: is_crit, is_hit: true,
                        effects_applied: Vec::new(),
                        message: format!("{} attacks {} for {} damage{}",
                            self.character.name, target.character.name, actual,
                            if is_crit { " (CRITICAL!)" } else { "" }),
                    });
                }
                results
            }
            BattleAction::Guard => {
                self.is_guarding = true;
                vec![CombatResult {
                    damage_dealt: 0, damage_type: DamageType::Physical,
                    is_critical: false, is_hit: true,
                    effects_applied: vec!["Guarding".to_string()],
                    message: format!("{} takes a defensive stance.", self.character.name),
                }]
            }
            BattleAction::Flee => {
                vec![CombatResult {
                    damage_dealt: 0, damage_type: DamageType::Physical,
                    is_critical: false, is_hit: true,
                    effects_applied: vec!["Flee".to_string()],
                    message: format!("{} attempts to flee!", self.character.name),
                }]
            }
            BattleAction::UseAbility(name) => {
                vec![CombatResult {
                    damage_dealt: 0, damage_type: DamageType::Physical,
                    is_critical: false, is_hit: true,
                    effects_applied: vec![name.clone()],
                    message: format!("{} uses {}!", self.character.name, name),
                }]
            }
            BattleAction::UseItem(name) => {
                vec![CombatResult {
                    damage_dealt: 0, damage_type: DamageType::Physical,
                    is_critical: false, is_hit: true,
                    effects_applied: vec![name.clone()],
                    message: format!("{} uses {}!", self.character.name, name),
                }]
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BattleState {
    InProgress,
    PlayerVictory { xp_gained: u64, loot: Vec<String> },
    PlayerDefeat,
    Fled,
}

pub struct Battle {
    pub combatants: Vec<Combatant>,
    pub turn_order: Vec<usize>,
    pub current_turn_index: usize,
    pub round: u32,
    pub state: BattleState,
    pub log: Vec<String>,
}

impl Battle {
    pub fn new(mut player_party: Vec<Combatant>, mut enemy_party: Vec<Combatant>, rng: &mut impl Rng) -> Self {
        for c in player_party.iter_mut().chain(enemy_party.iter_mut()) {
            c.calculate_initiative(rng);
        }
        let mut combatants = player_party;
        combatants.append(&mut enemy_party);
        let mut battle = Self {
            combatants, turn_order: Vec::new(),
            current_turn_index: 0, round: 1,
            state: BattleState::InProgress, log: Vec::new(),
        };
        battle.calculate_turn_order();
        battle
    }

    pub fn calculate_turn_order(&mut self) {
        let mut order: Vec<usize> = (0..self.combatants.len()).collect();
        order.sort_by(|&a, &b| {
            self.combatants[b].initiative
                .partial_cmp(&self.combatants[a].initiative)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.turn_order = order;
    }

    pub fn current_combatant_index(&self) -> usize {
        self.turn_order[self.current_turn_index % self.turn_order.len()]
    }

    pub fn process_turn(&mut self, action: BattleAction, rng: &mut impl Rng) -> &BattleState {
        // Check flee
        if matches!(action, BattleAction::Flee) {
            let roll: f32 = rng.gen();
            if roll < 0.6 {
                self.state = BattleState::Fled;
                self.log.push("Escaped!".to_string());
                return &self.state;
            } else {
                self.log.push("Failed to flee!".to_string());
            }
        } else {
            let actor_idx = self.current_combatant_index();
            // Find first living enemy target
            let target_idx = if self.combatants[actor_idx].is_player {
                self.combatants.iter().position(|c| !c.is_player && c.character.is_alive())
            } else {
                self.combatants.iter().position(|c| c.is_player && c.character.is_alive())
            };

            if let Some(t_idx) = target_idx {
                // Split to avoid borrow conflict
                let (actor_part, target_part) = if actor_idx < t_idx {
                    let (left, right) = self.combatants.split_at_mut(t_idx);
                    (&mut left[actor_idx], &mut right[0])
                } else {
                    let (left, right) = self.combatants.split_at_mut(actor_idx);
                    (&mut right[0], &mut left[t_idx])
                };
                let mut targets = vec![target_part.clone()];
                let results = actor_part.take_turn(action, &mut targets, rng);
                // Apply results back
                if let Some(updated_target) = targets.into_iter().next() {
                    self.combatants[t_idx] = updated_target;
                }
                for r in &results {
                    self.log.push(r.message.clone());
                }
            }
        }

        self.current_turn_index += 1;
        // End of round
        if self.current_turn_index >= self.turn_order.len() {
            self.current_turn_index = 0;
            self.round += 1;
            self.tick_end_of_round();
        }

        self.check_battle_end();
        &self.state
    }

    pub fn tick_end_of_round(&mut self) {
        for c in &mut self.combatants {
            c.character.tick_status_effects();
        }
    }

    pub fn check_battle_end(&mut self) -> bool {
        if matches!(self.state, BattleState::Fled) { return true; }
        let all_enemies_dead = self.combatants.iter().filter(|c| !c.is_player).all(|c| !c.character.is_alive());
        let all_players_dead = self.combatants.iter().filter(|c| c.is_player).all(|c| !c.character.is_alive());
        if all_enemies_dead {
            let xp = self.calculate_xp_reward();
            let loot = self.calculate_loot();
            self.state = BattleState::PlayerVictory { xp_gained: xp, loot };
            return true;
        }
        if all_players_dead {
            self.state = BattleState::PlayerDefeat;
            return true;
        }
        false
    }

    fn calculate_xp_reward(&self) -> u64 {
        self.combatants.iter().filter(|c| !c.is_player)
            .map(|c| 20u64 * c.character.level as u64).sum()
    }

    fn calculate_loot(&self) -> Vec<String> {
        vec!["gold_coin".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn make_combatant(name: &str, dex: i32, is_player: bool) -> Combatant {
        let mut c = Character::new(name.to_string());
        c.stats.dexterity = dex;
        Combatant::new(c, is_player)
    }

    #[test]
    fn test_all_thirty_abilities_loaded() {
        let abilities = all_abilities();
        assert_eq!(abilities.len(), 30);
        let physical: Vec<_> = abilities.iter().filter(|a| a.category == AbilityCategory::Physical).collect();
        let magic: Vec<_> = abilities.iter().filter(|a| a.category == AbilityCategory::Magic).collect();
        let psionic: Vec<_> = abilities.iter().filter(|a| a.category == AbilityCategory::Psionic).collect();
        assert_eq!(physical.len(), 10);
        assert_eq!(magic.len(), 10);
        assert_eq!(psionic.len(), 10);
    }

    #[test]
    fn test_damage_formula() {
        let res = ElementalResistances::none();
        let dmg = CombatFormulas::calculate_damage(10, 8, 50, 2, &DamageType::Physical, &res);
        assert!(dmg > 0);
    }

    #[test]
    fn test_turn_order_by_dexterity() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let p = make_combatant("Player", 20, true);
        let e = make_combatant("Enemy", 4, false);
        let mut battle = Battle::new(vec![p], vec![e], &mut rng);
        // Player (index 0) should have higher initiative with DEX 20 vs DEX 4
        let first_idx = battle.turn_order[0];
        assert!(battle.combatants[first_idx].is_player);
    }

    #[test]
    fn test_hit_chance_formula() {
        let high_dex = CombatFormulas::calculate_hit_chance(20, 0, 5);
        let low_dex = CombatFormulas::calculate_hit_chance(4, 0, 5);
        assert!(high_dex > low_dex);
    }

    #[test]
    fn test_magic_damage_scales_with_int() {
        let low_int = CombatFormulas::calculate_magic_damage(20, 5, 10);
        let high_int = CombatFormulas::calculate_magic_damage(20, 20, 10);
        assert!(high_int > low_int);
    }

    #[test]
    fn test_crit_multiplier() {
        let res = ElementalResistances::none();
        let normal = CombatFormulas::calculate_damage(10, 0, 0, 0, &DamageType::Physical, &res);
        let crit = (normal as f32 * 1.75) as i32;
        assert!(crit > normal);
    }

    #[test]
    fn test_battle_creates_and_runs() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut player = make_combatant("Player", 10, true);
        player.character.hp = 200;
        player.character.max_hp = 200;
        let mut enemy = make_combatant("Wolf", 5, false);
        enemy.character.hp = 30;
        enemy.character.max_hp = 30;
        let mut battle = Battle::new(vec![player], vec![enemy], &mut rng);
        // Run a few turns
        for _ in 0..20 {
            if !matches!(battle.state, BattleState::InProgress) { break; }
            battle.process_turn(BattleAction::Attack, &mut rng);
        }
        // Should have ended
        assert!(!matches!(battle.state, BattleState::InProgress));
    }
}
