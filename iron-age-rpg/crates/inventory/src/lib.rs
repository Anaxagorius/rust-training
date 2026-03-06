use iron_age_core::{Stats, GameError};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MaterialTier { Stone, Copper, Iron, Wood, Hardwood, Ironwood }

impl MaterialTier {
    pub fn damage_multiplier(&self) -> f32 {
        match self { Self::Stone => 0.7, Self::Copper => 0.85, Self::Iron => 1.0, Self::Wood => 0.6, Self::Hardwood => 0.8, Self::Ironwood => 1.1 }
    }
    pub fn weight_multiplier(&self) -> f32 {
        match self { Self::Stone => 1.3, Self::Copper => 0.9, Self::Iron => 1.0, Self::Wood => 0.5, Self::Hardwood => 0.6, Self::Ironwood => 0.7 }
    }
    pub fn value_multiplier(&self) -> f32 {
        match self { Self::Stone => 0.3, Self::Copper => 0.6, Self::Iron => 1.0, Self::Wood => 0.4, Self::Hardwood => 0.7, Self::Ironwood => 1.2 }
    }
    pub fn stat_requirement_bonus(&self) -> i32 {
        match self { Self::Stone => 0, Self::Copper => 1, Self::Iron => 2, Self::Wood => 0, Self::Hardwood => 1, Self::Ironwood => 2 }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ItemRarity { Common, Uncommon, Rare, Epic, Legendary }

impl ItemRarity {
    pub fn color_name(&self) -> &str {
        match self { Self::Common => "White", Self::Uncommon => "Green", Self::Rare => "Blue", Self::Epic => "Purple", Self::Legendary => "Gold" }
    }
    pub fn stat_bonus_multiplier(&self) -> f32 {
        match self { Self::Common => 1.0, Self::Uncommon => 1.2, Self::Rare => 1.5, Self::Epic => 2.0, Self::Legendary => 3.0 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EquipSlot { Ring1, Ring2, Amulet, MainHand, OffHand, Helmet, Shoulders, Torso, Leggings, Cape }

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ItemType {
    Club, PrimitiveSpear,
    ShortSword, LongSword, TwoHandedSword,
    HandAxe, BattleAxe, WarAxe,
    Mace, TwoHandedMace, Flail,
    Spear, IronSpear,
    Shortbow, Longbow, CompositeBow,
    HandCrossbow, TwoHandedCrossbow,
    Buckler, SmallShield, LargeShield, TowerShield,
    ArmorPiece(EquipSlot),
    HealthPotion, StaminaPotion, AntidotePotion, ClarityPotion, FortifyPotion,
    PitchBomb, ResinFlash, ClayFirePot,
    NightshadeOil, HemlockVial, BogFumeAmpule,
    CraftingMaterial, Ring, Amulet, CraftingTool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub item_type: ItemType,
    pub material: Option<MaterialTier>,
    pub rarity: ItemRarity,
    pub weight: f32,
    pub value: u32,
    pub damage_base: i32,
    pub armor_base: i32,
    pub stat_requirements: Stats,
    pub stat_bonuses: Stats,
    pub equip_slot: Option<EquipSlot>,
    pub stack_size: u32,
    pub quantity: u32,
    pub description: String,
    pub effects: Vec<String>,
    pub is_two_handed: bool,
}

impl Item {
    pub fn new_weapon(id: &str, name: &str, item_type: ItemType, material: MaterialTier, rarity: ItemRarity, base_damage: i32) -> Self {
        let two_h = matches!(item_type, ItemType::TwoHandedSword | ItemType::TwoHandedMace | ItemType::TwoHandedCrossbow | ItemType::Longbow | ItemType::CompositeBow);
        let weight = 2.0 * material.weight_multiplier();
        let value = (base_damage as f32 * 10.0 * material.value_multiplier() * rarity.stat_bonus_multiplier()) as u32;
        Item {
            id: id.to_string(), name: name.to_string(), item_type,
            material: Some(material), rarity, weight, value,
            damage_base: base_damage, armor_base: 0,
            stat_requirements: Stats::zeroed(), stat_bonuses: Stats::zeroed(),
            equip_slot: Some(EquipSlot::MainHand), stack_size: 1, quantity: 1,
            description: format!("A weapon: {}", name), effects: Vec::new(), is_two_handed: two_h,
        }
    }

    pub fn new_armor(id: &str, name: &str, slot: EquipSlot, material: MaterialTier, rarity: ItemRarity, base_armor: i32) -> Self {
        let weight = 1.5 * material.weight_multiplier();
        let value = (base_armor as f32 * 8.0 * material.value_multiplier() * rarity.stat_bonus_multiplier()) as u32;
        Item {
            id: id.to_string(), name: name.to_string(),
            item_type: ItemType::ArmorPiece(slot.clone()),
            material: Some(material), rarity, weight, value,
            damage_base: 0, armor_base: base_armor,
            stat_requirements: Stats::zeroed(), stat_bonuses: Stats::zeroed(),
            equip_slot: Some(slot), stack_size: 1, quantity: 1,
            description: format!("Armor: {}", name), effects: Vec::new(), is_two_handed: false,
        }
    }

    pub fn new_consumable(id: &str, name: &str, item_type: ItemType, stack_size: u32) -> Self {
        Item {
            id: id.to_string(), name: name.to_string(), item_type,
            material: None, rarity: ItemRarity::Common, weight: 0.2, value: 5,
            damage_base: 0, armor_base: 0,
            stat_requirements: Stats::zeroed(), stat_bonuses: Stats::zeroed(),
            equip_slot: None, stack_size, quantity: 1,
            description: format!("Consumable: {}", name), effects: Vec::new(), is_two_handed: false,
        }
    }

    pub fn effective_damage(&self) -> i32 {
        let mm = self.material.as_ref().map_or(1.0, |m| m.damage_multiplier());
        let rm = self.rarity.stat_bonus_multiplier();
        (self.damage_base as f32 * mm * rm) as i32
    }

    pub fn effective_armor(&self) -> i32 {
        let mm = self.material.as_ref().map_or(1.0, |m| m.damage_multiplier());
        let rm = self.rarity.stat_bonus_multiplier();
        (self.armor_base as f32 * mm * rm) as i32
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Equipment {
    pub ring1: Option<Item>, pub ring2: Option<Item>, pub amulet: Option<Item>,
    pub main_hand: Option<Item>, pub off_hand: Option<Item>,
    pub helmet: Option<Item>, pub shoulders: Option<Item>,
    pub torso: Option<Item>, pub leggings: Option<Item>, pub cape: Option<Item>,
}

impl Equipment {
    pub fn equip(&mut self, item: Item) -> Result<Option<Item>, GameError> {
        let slot = item.equip_slot.clone().ok_or_else(|| GameError::InvalidOperation("Item has no equip slot".to_string()))?;
        if item.is_two_handed { self.off_hand = None; }
        let old = match &slot {
            EquipSlot::Ring1 | EquipSlot::Ring2 => {
                if self.ring1.is_none() {
                    std::mem::replace(&mut self.ring1, Some(item))
                } else if self.ring2.is_none() {
                    std::mem::replace(&mut self.ring2, Some(item))
                } else {
                    return Err(GameError::InvalidOperation("Both ring slots occupied".to_string()));
                }
            }
            EquipSlot::Amulet => std::mem::replace(&mut self.amulet, Some(item)),
            EquipSlot::MainHand => std::mem::replace(&mut self.main_hand, Some(item)),
            EquipSlot::OffHand => {
                if self.is_two_handed_equipped() {
                    return Err(GameError::InvalidOperation("Cannot equip off-hand with two-handed weapon".to_string()));
                }
                std::mem::replace(&mut self.off_hand, Some(item))
            }
            EquipSlot::Helmet => std::mem::replace(&mut self.helmet, Some(item)),
            EquipSlot::Shoulders => std::mem::replace(&mut self.shoulders, Some(item)),
            EquipSlot::Torso => std::mem::replace(&mut self.torso, Some(item)),
            EquipSlot::Leggings => std::mem::replace(&mut self.leggings, Some(item)),
            EquipSlot::Cape => std::mem::replace(&mut self.cape, Some(item)),
        };
        Ok(old)
    }

    pub fn unequip(&mut self, slot: &EquipSlot) -> Option<Item> {
        match slot {
            EquipSlot::Ring1 => self.ring1.take(),
            EquipSlot::Ring2 => self.ring2.take(),
            EquipSlot::Amulet => self.amulet.take(),
            EquipSlot::MainHand => self.main_hand.take(),
            EquipSlot::OffHand => self.off_hand.take(),
            EquipSlot::Helmet => self.helmet.take(),
            EquipSlot::Shoulders => self.shoulders.take(),
            EquipSlot::Torso => self.torso.take(),
            EquipSlot::Leggings => self.leggings.take(),
            EquipSlot::Cape => self.cape.take(),
        }
    }

    pub fn get_slot(&self, slot: &EquipSlot) -> Option<&Item> {
        match slot {
            EquipSlot::Ring1 => self.ring1.as_ref(),
            EquipSlot::Ring2 => self.ring2.as_ref(),
            EquipSlot::Amulet => self.amulet.as_ref(),
            EquipSlot::MainHand => self.main_hand.as_ref(),
            EquipSlot::OffHand => self.off_hand.as_ref(),
            EquipSlot::Helmet => self.helmet.as_ref(),
            EquipSlot::Shoulders => self.shoulders.as_ref(),
            EquipSlot::Torso => self.torso.as_ref(),
            EquipSlot::Leggings => self.leggings.as_ref(),
            EquipSlot::Cape => self.cape.as_ref(),
        }
    }

    fn all_items(&self) -> Vec<&Item> {
        [&self.ring1, &self.ring2, &self.amulet, &self.main_hand, &self.off_hand,
         &self.helmet, &self.shoulders, &self.torso, &self.leggings, &self.cape]
            .iter().filter_map(|s| s.as_ref()).collect()
    }

    pub fn total_armor_mitigation(&self) -> i32 {
        [&self.helmet, &self.shoulders, &self.torso, &self.leggings, &self.cape, &self.off_hand]
            .iter().filter_map(|s| s.as_ref()).map(|i| i.effective_armor()).sum()
    }

    pub fn total_stat_bonuses(&self) -> Stats {
        self.all_items().iter().fold(Stats::zeroed(), |acc, item| acc + item.stat_bonuses.clone())
    }

    pub fn total_weight(&self) -> f32 {
        self.all_items().iter().map(|i| i.weight).sum()
    }

    pub fn is_two_handed_equipped(&self) -> bool {
        self.main_hand.as_ref().map_or(false, |i| i.is_two_handed)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Inventory {
    pub items: Vec<Item>,
    pub max_slots: usize,
    pub gold: u32,
}

impl Inventory {
    pub fn new(max_slots: usize) -> Self { Self { items: Vec::new(), max_slots, gold: 0 } }

    pub fn add_item(&mut self, mut item: Item) -> Result<(), GameError> {
        if item.stack_size > 1 {
            if let Some(existing) = self.items.iter_mut().find(|i| i.id == item.id && i.quantity < i.stack_size) {
                let space = existing.stack_size - existing.quantity;
                let to_add = item.quantity.min(space);
                existing.quantity += to_add;
                item.quantity -= to_add;
                if item.quantity == 0 { return Ok(()); }
            }
        }
        if self.items.len() >= self.max_slots { return Err(GameError::InventoryFull); }
        self.items.push(item);
        Ok(())
    }

    pub fn remove_item(&mut self, item_id: &str, quantity: u32) -> Result<Item, GameError> {
        let idx = self.items.iter().position(|i| i.id == item_id)
            .ok_or_else(|| GameError::NotFound(item_id.to_string()))?;
        if self.items[idx].quantity < quantity {
            return Err(GameError::InvalidOperation("Not enough quantity".to_string()));
        }
        self.items[idx].quantity -= quantity;
        let mut removed = self.items[idx].clone();
        removed.quantity = quantity;
        if self.items[idx].quantity == 0 { self.items.remove(idx); }
        Ok(removed)
    }

    pub fn find_item(&self, item_id: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.id == item_id)
    }

    pub fn find_item_mut(&mut self, item_id: &str) -> Option<&mut Item> {
        self.items.iter_mut().find(|i| i.id == item_id)
    }

    pub fn total_weight(&self) -> f32 {
        self.items.iter().map(|i| i.weight * i.quantity as f32).sum()
    }

    pub fn sort_by_type(&mut self) {
        self.items.sort_by(|a, b| format!("{:?}", a.item_type).cmp(&format!("{:?}", b.item_type)));
    }

    pub fn use_consumable(&mut self, item_id: &str) -> Result<Item, GameError> {
        let idx = self.items.iter().position(|i| i.id == item_id)
            .ok_or_else(|| GameError::NotFound(item_id.to_string()))?;
        let item = self.items[idx].clone();
        match &item.item_type {
            ItemType::HealthPotion | ItemType::StaminaPotion | ItemType::AntidotePotion
            | ItemType::ClarityPotion | ItemType::FortifyPotion | ItemType::PitchBomb
            | ItemType::ResinFlash | ItemType::ClayFirePot | ItemType::NightshadeOil
            | ItemType::HemlockVial | ItemType::BogFumeAmpule => {}
            _ => return Err(GameError::InvalidOperation("Not a consumable".to_string())),
        }
        self.remove_item(item_id, 1)?;
        Ok(item)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LootEntry {
    pub item_id: String,
    pub weight: u32,
    pub min_quantity: u32,
    pub max_quantity: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LootTable {
    pub entries: Vec<LootEntry>,
    pub guaranteed: Vec<String>,
    pub gold_min: u32,
    pub gold_max: u32,
}

impl LootTable {
    pub fn roll(&self, rng: &mut impl rand::Rng, num_rolls: u32) -> (Vec<(String, u32)>, u32) {
        use rand::Rng;
        let mut results = Vec::new();
        let gold_max = self.gold_max.max(self.gold_min);
        let gold = if gold_max > self.gold_min { rng.gen_range(self.gold_min..=gold_max) } else { self.gold_min };
        for item_id in &self.guaranteed { results.push((item_id.clone(), 1)); }
        let total_weight: u32 = self.entries.iter().map(|e| e.weight).sum();
        if total_weight > 0 {
            for _ in 0..num_rolls {
                let roll = rng.gen_range(0..total_weight);
                let mut cum = 0u32;
                for entry in &self.entries {
                    cum += entry.weight;
                    if roll < cum {
                        let max_q = entry.max_quantity.max(entry.min_quantity);
                        let qty = if max_q > entry.min_quantity { rng.gen_range(entry.min_quantity..=max_q) } else { entry.min_quantity };
                        results.push((entry.item_id.clone(), qty));
                        break;
                    }
                }
            }
        }
        (results, gold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ring(id: &str) -> Item {
        Item {
            id: id.to_string(), name: id.to_string(), item_type: ItemType::Ring,
            material: None, rarity: ItemRarity::Common, weight: 0.1, value: 10,
            damage_base: 0, armor_base: 0,
            stat_requirements: Stats::zeroed(), stat_bonuses: Stats::zeroed(),
            equip_slot: Some(EquipSlot::Ring1), stack_size: 1, quantity: 1,
            description: "A ring".to_string(), effects: Vec::new(), is_two_handed: false,
        }
    }

    #[test]
    fn test_equip_two_rings_max() {
        let mut eq = Equipment::default();
        eq.equip(make_ring("r1")).unwrap();
        eq.equip(make_ring("r2")).unwrap();
        assert!(eq.equip(make_ring("r3")).is_err());
    }

    #[test]
    fn test_two_handed_blocks_offhand() {
        let mut eq = Equipment::default();
        let mut tw = Item::new_weapon("tw", "Two-Hander", ItemType::TwoHandedSword, MaterialTier::Iron, ItemRarity::Common, 15);
        tw.is_two_handed = true;
        eq.equip(tw).unwrap();
        let shield = Item::new_armor("sh", "Shield", EquipSlot::OffHand, MaterialTier::Iron, ItemRarity::Common, 5);
        assert!(eq.equip(shield).is_err());
    }

    #[test]
    fn test_loot_table_roll() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let table = LootTable {
            entries: vec![LootEntry { item_id: "sword".to_string(), weight: 100, min_quantity: 1, max_quantity: 1 }],
            guaranteed: vec!["herb".to_string()],
            gold_min: 10, gold_max: 20,
        };
        let (items, gold) = table.roll(&mut rng, 1);
        assert!(gold >= 10 && gold <= 20);
        assert!(items.iter().any(|(id, _)| id == "herb"));
    }

    #[test]
    fn test_inventory_full_returns_error() {
        let mut inv = Inventory::new(2);
        inv.add_item(Item::new_consumable("h1", "Pot1", ItemType::HealthPotion, 1)).unwrap();
        inv.add_item(Item::new_consumable("h2", "Pot2", ItemType::HealthPotion, 1)).unwrap();
        assert!(inv.add_item(Item::new_consumable("h3", "Pot3", ItemType::HealthPotion, 1)).is_err());
    }

    #[test]
    fn test_material_tiers_affect_damage() {
        let iron = Item::new_weapon("i", "Iron", ItemType::ShortSword, MaterialTier::Iron, ItemRarity::Common, 10);
        let stone = Item::new_weapon("s", "Stone", ItemType::ShortSword, MaterialTier::Stone, ItemRarity::Common, 10);
        assert!(iron.effective_damage() > stone.effective_damage());
    }
}
