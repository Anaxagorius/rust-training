use thiserror::Error;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Stats {
    pub strength: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub constitution: i32,
    pub dexterity: i32,
    pub charisma: i32,
}

impl Stats {
    pub fn new(str: i32, int: i32, wis: i32, con: i32, dex: i32, cha: i32) -> Self {
        Self { strength: str, intelligence: int, wisdom: wis, constitution: con, dexterity: dex, charisma: cha }
    }
    pub fn base_player() -> Self { Self::new(8, 5, 5, 7, 6, 6) }
    pub fn base_enemy() -> Self { Self::new(5, 3, 3, 5, 4, 2) }
    pub fn zeroed() -> Self { Self::new(0, 0, 0, 0, 0, 0) }
}

impl std::ops::Add for Stats {
    type Output = Stats;
    fn add(self, rhs: Stats) -> Stats {
        Stats {
            strength: self.strength + rhs.strength,
            intelligence: self.intelligence + rhs.intelligence,
            wisdom: self.wisdom + rhs.wisdom,
            constitution: self.constitution + rhs.constitution,
            dexterity: self.dexterity + rhs.dexterity,
            charisma: self.charisma + rhs.charisma,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StatusEffect {
    Bleed { damage_per_turn: i32, turns_remaining: u32 },
    Stagger { turns_remaining: u32 },
    Weaken { defense_reduction: i32, turns_remaining: u32 },
    Poison { damage_per_turn: i32, turns_remaining: u32 },
    Burn { damage_per_turn: i32, turns_remaining: u32 },
    Chill { speed_reduction: i32, turns_remaining: u32 },
    Fear { turns_remaining: u32 },
    Silence { turns_remaining: u32 },
    Entangle { turns_remaining: u32 },
    Slow { turns_remaining: u32 },
    Haste { turns_remaining: u32 },
    Shield { absorption: i32, turns_remaining: u32 },
    Regen { heal_per_turn: i32, turns_remaining: u32 },
    AttackUp { bonus: i32, turns_remaining: u32 },
    DefenseUp { bonus: i32, turns_remaining: u32 },
    Stunned { turns_remaining: u32 },
}

impl StatusEffect {
    pub fn tick(&mut self) -> bool {
        let tr = match self {
            StatusEffect::Bleed { turns_remaining, .. }
            | StatusEffect::Stagger { turns_remaining }
            | StatusEffect::Weaken { turns_remaining, .. }
            | StatusEffect::Poison { turns_remaining, .. }
            | StatusEffect::Burn { turns_remaining, .. }
            | StatusEffect::Chill { turns_remaining, .. }
            | StatusEffect::Fear { turns_remaining }
            | StatusEffect::Silence { turns_remaining }
            | StatusEffect::Entangle { turns_remaining }
            | StatusEffect::Slow { turns_remaining }
            | StatusEffect::Haste { turns_remaining }
            | StatusEffect::Shield { turns_remaining, .. }
            | StatusEffect::Regen { turns_remaining, .. }
            | StatusEffect::AttackUp { turns_remaining, .. }
            | StatusEffect::DefenseUp { turns_remaining, .. }
            | StatusEffect::Stunned { turns_remaining } => turns_remaining,
        };
        if *tr > 0 { *tr -= 1; }
        *tr > 0
    }

    pub fn name(&self) -> &str {
        match self {
            StatusEffect::Bleed { .. } => "Bleed",
            StatusEffect::Stagger { .. } => "Stagger",
            StatusEffect::Weaken { .. } => "Weaken",
            StatusEffect::Poison { .. } => "Poison",
            StatusEffect::Burn { .. } => "Burn",
            StatusEffect::Chill { .. } => "Chill",
            StatusEffect::Fear { .. } => "Fear",
            StatusEffect::Silence { .. } => "Silence",
            StatusEffect::Entangle { .. } => "Entangle",
            StatusEffect::Slow { .. } => "Slow",
            StatusEffect::Haste { .. } => "Haste",
            StatusEffect::Shield { .. } => "Shield",
            StatusEffect::Regen { .. } => "Regen",
            StatusEffect::AttackUp { .. } => "AttackUp",
            StatusEffect::DefenseUp { .. } => "DefenseUp",
            StatusEffect::Stunned { .. } => "Stunned",
        }
    }

    pub fn description(&self) -> String {
        match self {
            StatusEffect::Bleed { damage_per_turn, turns_remaining } =>
                format!("Bleeding: {} dmg/turn for {} turns", damage_per_turn, turns_remaining),
            StatusEffect::Poison { damage_per_turn, turns_remaining } =>
                format!("Poisoned: {} dmg/turn for {} turns", damage_per_turn, turns_remaining),
            StatusEffect::Burn { damage_per_turn, turns_remaining } =>
                format!("Burning: {} dmg/turn for {} turns", damage_per_turn, turns_remaining),
            StatusEffect::Regen { heal_per_turn, turns_remaining } =>
                format!("Regen: {} hp/turn for {} turns", heal_per_turn, turns_remaining),
            StatusEffect::Shield { absorption, turns_remaining } =>
                format!("Shield: {} absorption for {} turns", absorption, turns_remaining),
            StatusEffect::Weaken { defense_reduction, turns_remaining } =>
                format!("Weakened: -{} def for {} turns", defense_reduction, turns_remaining),
            StatusEffect::Chill { speed_reduction, turns_remaining } =>
                format!("Chilled: -{} spd for {} turns", speed_reduction, turns_remaining),
            StatusEffect::AttackUp { bonus, turns_remaining } =>
                format!("Attack Up +{} for {} turns", bonus, turns_remaining),
            StatusEffect::DefenseUp { bonus, turns_remaining } =>
                format!("Defense Up +{} for {} turns", bonus, turns_remaining),
            other => format!("{} active", other.name()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DamageType {
    Physical,
    Fire,
    Nature,
    Lightning,
    Frost,
    Psychic,
    Poison,
    Holy,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ElementalResistances {
    pub physical: i32,
    pub fire: i32,
    pub nature: i32,
    pub lightning: i32,
    pub frost: i32,
    pub psychic: i32,
    pub poison: i32,
    pub holy: i32,
}

impl ElementalResistances {
    pub fn none() -> Self {
        Self { physical: 0, fire: 0, nature: 0, lightning: 0, frost: 0, psychic: 0, poison: 0, holy: 0 }
    }

    pub fn get_resistance(&self, damage_type: &DamageType) -> i32 {
        match damage_type {
            DamageType::Physical => self.physical,
            DamageType::Fire => self.fire,
            DamageType::Nature => self.nature,
            DamageType::Lightning => self.lightning,
            DamageType::Frost => self.frost,
            DamageType::Psychic => self.psychic,
            DamageType::Poison => self.poison,
            DamageType::Holy => self.holy,
        }
    }

    pub fn apply_damage(&self, damage: i32, damage_type: &DamageType) -> i32 {
        let resistance = self.get_resistance(damage_type).clamp(-100, 100);
        let multiplier = (100 - resistance) as f32 / 100.0;
        ((damage as f32 * multiplier) as i32).max(0)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Row { Front, Back }

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Inventory full")]
    InventoryFull,
    #[error("Missing requirements: {0}")]
    MissingRequirements(String),
    #[error("Data error: {0}")]
    DataError(String),
}
