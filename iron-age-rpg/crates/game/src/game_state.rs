use iron_age_character::Character;
use iron_age_combat::Combatant;
use iron_age_inventory::{Equipment, Inventory};
use iron_age_crafting::CraftingSystem;
use iron_age_world::{WorldMap, build_starting_world};
use iron_age_narrative::{NpcRegistry, QuestLog, build_narrative};
use iron_age_data::{starter_items, find_item};

pub struct GameState {
    pub player: Combatant,
    pub equipment: Equipment,
    pub inventory: Inventory,
    pub crafting: CraftingSystem,
    pub world: WorldMap,
    pub npcs: NpcRegistry,
    pub quest_log: QuestLog,
    pub gold: u32,
    pub turn: u32,
    #[allow(dead_code)]
    pub resting: bool,
    /// Steps taken in dangerous areas since the last random encounter.
    /// This drives the FF-style scaling encounter probability.
    pub danger_steps: u32,
}

impl GameState {
    pub fn new_game() -> Self {
        let mut character = Character::new("Hero".to_string());
        // Give the player a slightly better starting kit
        character.stats.strength += 2;
        character.stats.dexterity += 1;
        character.max_hp = Character::max_hp_base(character.stats.constitution);
        character.hp = character.max_hp;

        let player = Combatant::new(character, true);
        let mut inventory = Inventory::new(30);

        for item in starter_items() {
            let _ = inventory.add_item(item);
        }

        let crafting = CraftingSystem::with_starter_recipes();
        let world = build_starting_world();
        let (npcs, quest_log) = build_narrative();

        Self {
            player,
            equipment: Equipment::default(),
            inventory,
            crafting,
            world,
            npcs,
            quest_log,
            gold: 10,
            turn: 0,
            resting: false,
            danger_steps: 0,
        }
    }

    #[allow(dead_code)]
    pub fn player_name(&self) -> &str {
        &self.player.character.name
    }

    pub fn active_quest_ids(&self) -> Vec<String> {
        self.quest_log.active_quests().iter().map(|q| q.id.clone()).collect()
    }

    /// Try to give the player an item from the data catalog by id.
    pub fn give_item(&mut self, item_id: &str, qty: u32) -> bool {
        if let Some(mut item) = find_item(item_id) {
            item.quantity = qty;
            self.inventory.add_item(item).is_ok()
        } else {
            false
        }
    }

    /// Record a kill for quest tracking. Returns any quest update messages.
    pub fn on_enemy_killed(&mut self, enemy_id: &str) -> Vec<String> {
        self.quest_log.on_kill(enemy_id)
    }

    /// Try to unlock the exit in the current direction if the player has the required key.
    pub fn try_unlock_with_inventory(&mut self, direction: &str) -> bool {
        let key_id = {
            let loc = match self.world.current_location() {
                Some(l) => l,
                None => return false,
            };
            let exit = match loc.exit_for_direction(direction) {
                Some(e) => e,
                None => return false,
            };
            match &exit.requires_key {
                Some(k) => k.clone(),
                None => return false,
            }
        };
        if self.inventory.find_item(&key_id).is_some() {
            self.world.unlock_exit(direction);
            true
        } else {
            false
        }
    }

    /// Rest at a safe location (inn or safe zone), restoring hp/stamina/mana.
    pub fn rest(&mut self) -> String {
        let is_safe = self.world.current_location().map_or(false, |l| l.is_safe);
        if is_safe {
            let c = &mut self.player.character;
            c.hp = c.max_hp;
            c.stamina = c.max_stamina;
            c.mana = c.max_mana;
            c.status_effects.clear();
            self.danger_steps = 0;
            format!(
                "You rest and recover fully. HP: {}/{}, Stamina: {}/{}, Mana: {}/{}",
                c.hp, c.max_hp, c.stamina, c.max_stamina, c.mana, c.max_mana
            )
        } else {
            "You cannot rest here — it is not safe.".to_string()
        }
    }
}
