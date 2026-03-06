use crate::game_state::GameState;
use crate::display;
use iron_age_combat::{Battle, BattleAction, BattleState};
use iron_age_data::{find_template, roll_loot};
use rand::{Rng, SeedableRng};

pub enum CommandResult {
    Message(String),
    Quit,
}

pub fn handle_command(input: &str, state: &mut GameState) -> CommandResult {
    let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
    let cmd = parts[0].to_lowercase();
    let arg = parts.get(1).copied().unwrap_or("").trim();

    let msg = match cmd.as_str() {
        "help" | "?" => display::help_text().to_string(),

        "look" | "l" => {
            match state.world.current_location() {
                Some(loc) => display::location_display(loc),
                None => "You are nowhere.".to_string(),
            }
        }

        "go" | "move" | "travel" => {
            if arg.is_empty() {
                return CommandResult::Message("Go where? Try 'go north'.".to_string());
            }
            // Try unlocking with inventory first
            state.try_unlock_with_inventory(arg);

            match state.world.travel(arg) {
                Ok(loc) => {
                    let loc_id = loc.id.clone();
                    let loc_display = display::location_display(loc);

                    // Quest: location tracking
                    let quest_msgs = state.quest_log.on_reach_location(&loc_id);

                    // Check for random encounter
                    let loc = state.world.current_location().unwrap();
                    let difficulty = loc.region_type.encounter_difficulty();
                    let encounter_msgs = if difficulty > 0 {
                        maybe_encounter(state, difficulty)
                    } else {
                        Vec::new()
                    };

                    let mut out = loc_display;
                    for m in &quest_msgs { out.push_str(&format!("\n{}", m)); }
                    for m in &encounter_msgs { out.push_str(&format!("\n{}", m)); }
                    out
                }
                Err(e) => format!("{}", e),
            }
        }

        "talk" => {
            if arg.is_empty() {
                return CommandResult::Message("Talk to whom? Try 'talk elder_aldric'.".to_string());
            }
            let npc_id = arg.to_lowercase().replace(' ', "_");
            match state.npcs.get(&npc_id) {
                None => format!("There is no one called '{}' here.", arg),
                Some(npc) => {
                    let active_ids = state.active_quest_ids();
                    let lines = npc.available_lines(&active_ids);
                    let mut out = format!("{}: \"{}\"", npc.name, npc.greeting);
                    for line in lines {
                        out.push_str(&format!("\n  \"{}\"", line.text));
                    }
                    if !npc.quest_ids.is_empty() {
                        out.push_str(&format!(
                            "\n  [Available quests: {}]",
                            npc.quest_ids.join(", ")
                        ));
                    }
                    // Quest tracking
                    state.quest_log.on_talk(&npc_id);
                    out
                }
            }
        }

        "accept" => {
            if arg.is_empty() {
                return CommandResult::Message("Accept which quest? e.g. 'accept drive_back_goblins'.".to_string());
            }
            let completed = state.quest_log.completed_quest_ids();
            match state.quest_log.start_quest(arg, &completed) {
                Ok(_) => format!("Quest accepted: '{}'.", arg),
                Err(e) => format!("{}", e),
            }
        }

        "complete" => {
            if arg.is_empty() {
                return CommandResult::Message("Complete which quest?".to_string());
            }
            match state.quest_log.try_complete_quest(arg) {
                Ok(reward) => {
                    let mut out = format!(
                        "Quest '{}' completed!\nReward: {} XP, {} gold",
                        arg, reward.experience, reward.gold
                    );
                    state.gold += reward.gold;
                    let levels = state.player.character.add_experience(reward.experience);
                    for lvl in &levels {
                        out.push_str(&format!("\n🎉 Level up! You are now level {}.", lvl));
                    }
                    for item_id in &reward.item_ids {
                        if state.give_item(item_id, 1) {
                            out.push_str(&format!("\nReceived: {}", item_id));
                        }
                    }
                    out
                }
                Err(e) => format!("{}", e),
            }
        }

        "quests" | "q" | "journal" => {
            let active = state.quest_log.active_quests();
            display::quest_log_display(&active)
        }

        "stats" | "status" | "char" => {
            display::character_sheet(&state.player.character, state.gold)
        }

        "inventory" | "inv" | "i" => {
            display::inventory_display(&state.inventory, state.gold)
        }

        "rest" | "sleep" => state.rest(),

        "use" => {
            if arg.is_empty() {
                return CommandResult::Message("Use what? e.g. 'use health_potion'.".to_string());
            }
            use_item(state, arg)
        }

        "equip" => {
            if arg.is_empty() {
                return CommandResult::Message("Equip what? e.g. 'equip iron_sword'.".to_string());
            }
            equip_item(state, arg)
        }

        "unequip" => {
            if arg.is_empty() {
                return CommandResult::Message("Unequip which slot? e.g. 'unequip mainhand'.".to_string());
            }
            unequip_slot(state, arg)
        }

        "equipment" | "gear" | "eq" => {
            display::equipment_display(&state.equipment)
        }

        "alloc" | "allocate" => {
            alloc_stat(state, arg)
        }

        "attack" | "fight" => {
            let loc = state.world.current_location().unwrap();
            if loc.is_safe {
                "There is nothing to fight here.".to_string()
            } else if loc.enemy_spawn_ids.is_empty() {
                "There are no enemies here right now.".to_string()
            } else {
                let mut rng = rand::rngs::StdRng::seed_from_u64(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(42, |d| d.subsec_nanos() as u64)
                );
                let spawn_id = loc.enemy_spawn_ids[rng.gen_index(loc.enemy_spawn_ids.len())].clone();
                run_combat(state, &spawn_id, &mut rng)
            }
        }

        "flee" => {
            "You back away cautiously. (Use 'flee' during a 'fight' encounter.)".to_string()
        }

        "craft" => {
            if arg == "list" || arg.is_empty() {
                let mut out = "── Known Recipes ──\n".to_string();
                for recipe in &state.crafting.known_recipes {
                    out.push_str(&format!(
                        "  [{}] {} — {}\n",
                        recipe.id, recipe.name, recipe.description
                    ));
                    let ings: Vec<String> = recipe.ingredients.iter()
                        .map(|i| format!("{} x{}", i.item_id, i.quantity))
                        .collect();
                    out.push_str(&format!("    Ingredients: {}\n", ings.join(", ")));
                }
                out
            } else {
                let mut rng = rand::rngs::StdRng::seed_from_u64(state.turn as u64 * 7 + 13);
                let int = state.player.character.stats.intelligence;
                let wis = state.player.character.stats.wisdom;
                match state.crafting.craft(arg, &mut state.inventory, 0, int, wis, &mut rng) {
                    Ok((item, quality)) => {
                        let name = item.name.clone();
                        let q_label = format!("{:?}", quality);
                        let added = state.inventory.add_item(item);
                        match added {
                            Ok(_) => format!("Crafted {} ({}).", name, q_label),
                            Err(_) => "Inventory full — couldn't store the crafted item.".to_string(),
                        }
                    }
                    Err(e) => format!("{}", e),
                }
            }
        }

        "save" => {
            save_game(state)
        }

        "load" => {
            return CommandResult::Message(
                "To load a save, restart the game and type 'load' at the main prompt, or run 'cargo run --bin iron-age-rpg -- --load'.".to_string()
            );
        }

        "quit" | "exit" => return CommandResult::Quit,

        _ => format!("Unknown command: '{}'. Type 'help' for a list of commands.", cmd),
    };

    state.turn += 1;
    CommandResult::Message(msg)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn use_item(state: &mut GameState, item_name: &str) -> String {
    use iron_age_inventory::ItemType;

    let item_id = item_name.to_lowercase().replace(' ', "_");
    // Check inventory
    let found = state.inventory.items.iter().find(|i| {
        i.id == item_id || i.name.to_lowercase().replace(' ', "_") == item_id
    }).cloned();

    match found {
        None => format!("You don't have '{}'.", item_name),
        Some(item) => {
            match item.item_type {
                ItemType::HealthPotion => {
                    let heal = 30 + state.player.character.stats.constitution * 2;
                    let actual = state.player.character.heal(heal);
                    let _ = state.inventory.remove_item(&item.id, 1);
                    format!("You drink the health potion and recover {} HP.", actual)
                }
                ItemType::StaminaPotion => {
                    let restore = 25;
                    let c = &mut state.player.character;
                    let before = c.stamina;
                    c.stamina = (c.stamina + restore).min(c.max_stamina);
                    let actual = c.stamina - before;
                    let _ = state.inventory.remove_item(&item.id, 1);
                    format!("You drink the stamina potion and recover {} stamina.", actual)
                }
                ItemType::AntidotePotion => {
                    state.player.character.status_effects.retain(|e| e.name() != "Poison");
                    let _ = state.inventory.remove_item(&item.id, 1);
                    "The antidote clears your system.".to_string()
                }
                _ => format!("You can't use {} that way.", item.name),
            }
        }
    }
}

fn equip_item(state: &mut GameState, item_name: &str) -> String {
    let item_id = item_name.to_lowercase().replace(' ', "_");
    // Find matching item in inventory
    let found = state.inventory.items.iter().find(|i| {
        i.id == item_id || i.name.to_lowercase().replace(' ', "_") == item_id
    }).cloned();

    match found {
        None => format!("You don't have '{}'.", item_name),
        Some(item) => {
            if item.equip_slot.is_none() {
                return format!("{} cannot be equipped.", item.name);
            }
            let item_name_display = item.name.clone();
            let _ = state.inventory.remove_item(&item.id, 1);
            match state.equipment.equip(item) {
                Ok(Some(old_item)) => {
                    // Return old item to inventory
                    let old_name = old_item.name.clone();
                    let _ = state.inventory.add_item(old_item);
                    format!("Equipped {}. {} returned to inventory.", item_name_display, old_name)
                }
                Ok(None) => format!("Equipped {}.", item_name_display),
                Err(e) => {
                    // Re-add to inventory since equip failed
                    if let Some(item) = iron_age_data::find_item(&item_id) {
                        let _ = state.inventory.add_item(item);
                    }
                    format!("Cannot equip: {}", e)
                }
            }
        }
    }
}

fn unequip_slot(state: &mut GameState, slot_name: &str) -> String {
    use iron_age_inventory::EquipSlot;
    let slot = match slot_name.to_lowercase().as_str() {
        "mainhand" | "main" | "main_hand" | "weapon" => EquipSlot::MainHand,
        "offhand" | "off" | "off_hand" | "shield" => EquipSlot::OffHand,
        "helmet" | "head" | "helm" => EquipSlot::Helmet,
        "shoulders" | "shoulder" | "pauldrons" => EquipSlot::Shoulders,
        "torso" | "chest" | "body" => EquipSlot::Torso,
        "leggings" | "legs" | "pants" => EquipSlot::Leggings,
        "cape" | "cloak" | "back" => EquipSlot::Cape,
        "amulet" | "neck" | "necklace" => EquipSlot::Amulet,
        "ring1" | "ring" => EquipSlot::Ring1,
        "ring2" => EquipSlot::Ring2,
        _ => return format!("Unknown slot '{}'. Try: mainhand, offhand, helmet, shoulders, torso, leggings, cape, amulet, ring1, ring2.", slot_name),
    };
    match state.equipment.unequip(&slot) {
        Some(item) => {
            let name = item.name.clone();
            let _ = state.inventory.add_item(item);
            format!("Unequipped {} → inventory.", name)
        }
        None => format!("Nothing equipped in that slot."),
    }
}

fn alloc_stat(state: &mut GameState, args: &str) -> String {
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.is_empty() {
        let pts = state.player.character.stat_points;
        return if pts == 0 {
            "You have no stat points to spend.".to_string()
        } else {
            format!(
                "You have {} stat point(s). Use 'alloc <stat> [amount]' — stats: str, int, wis, con, dex, cha.",
                pts
            )
        };
    }
    let stat = parts[0].to_lowercase();
    let amount: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    match state.player.character.allocate_stat(&stat, amount) {
        Ok(()) => {
            let c = &state.player.character;
            format!(
                "Allocated {} point(s) to {}. Remaining points: {}.",
                amount, stat, c.stat_points
            )
        }
        Err(e) => format!("{}", e),
    }
}

fn save_game(state: &GameState) -> String {
    use std::fs;
    use iron_age_inventory::EquipSlot;

    // Build a simple save structure as JSON
    let save = serde_json::json!({
        "version": 1,
        "player": {
            "name": state.player.character.name,
            "level": state.player.character.level,
            "experience": state.player.character.experience,
            "hp": state.player.character.hp,
            "max_hp": state.player.character.max_hp,
            "stamina": state.player.character.stamina,
            "max_stamina": state.player.character.max_stamina,
            "mana": state.player.character.mana,
            "max_mana": state.player.character.max_mana,
            "stat_points": state.player.character.stat_points,
            "skill_points": state.player.character.skill_points,
            "stats": {
                "strength": state.player.character.stats.strength,
                "intelligence": state.player.character.stats.intelligence,
                "wisdom": state.player.character.stats.wisdom,
                "constitution": state.player.character.stats.constitution,
                "dexterity": state.player.character.stats.dexterity,
                "charisma": state.player.character.stats.charisma,
            }
        },
        "gold": state.gold,
        "turn": state.turn,
        "current_location": state.world.current_location().map(|l| &l.id),
        "inventory": state.inventory.items.iter().map(|i| serde_json::json!({
            "id": i.id, "quantity": i.quantity
        })).collect::<Vec<_>>(),
        "equipped": {
            "main_hand": state.equipment.get_slot(&EquipSlot::MainHand).map(|i| &i.id),
            "off_hand": state.equipment.get_slot(&EquipSlot::OffHand).map(|i| &i.id),
            "helmet": state.equipment.get_slot(&EquipSlot::Helmet).map(|i| &i.id),
            "shoulders": state.equipment.get_slot(&EquipSlot::Shoulders).map(|i| &i.id),
            "torso": state.equipment.get_slot(&EquipSlot::Torso).map(|i| &i.id),
            "leggings": state.equipment.get_slot(&EquipSlot::Leggings).map(|i| &i.id),
            "cape": state.equipment.get_slot(&EquipSlot::Cape).map(|i| &i.id),
            "amulet": state.equipment.get_slot(&EquipSlot::Amulet).map(|i| &i.id),
            "ring1": state.equipment.get_slot(&EquipSlot::Ring1).map(|i| &i.id),
            "ring2": state.equipment.get_slot(&EquipSlot::Ring2).map(|i| &i.id),
        },
        "active_quests": state.quest_log.active_quests().iter().map(|q| &q.id).collect::<Vec<_>>(),
        "completed_quests": state.quest_log.completed_quest_ids(),
    });

    let path = "savegame.json";
    let json = match serde_json::to_string_pretty(&save) {
        Ok(j) => j,
        Err(e) => return format!("Failed to serialise save data: {}", e),
    };
    match fs::write(path, json) {
        Ok(()) => format!("Game saved to '{}'.", path),
        Err(e) => format!("Failed to save: {}", e),
    }
}

fn run_combat_with(state: &mut GameState, enemy: iron_age_combat::Combatant, enemy_template_id: &str, rng: &mut impl rand::Rng) -> String {
    let enemy_name = enemy.character.name.clone();
    let enemy_template_id = enemy_template_id.to_string();
    let mut out = format!("A {} appears!\n", enemy_name);

    let player_snapshot = state.player.clone();
    let mut battle = Battle::new(vec![player_snapshot], vec![enemy], rng);

    let mut turn = 0;
    loop {
        if !matches!(battle.state, BattleState::InProgress) { break; }
        battle.process_turn(BattleAction::Attack, rng);
        for msg in battle.log.iter().skip(turn) {
            out.push_str(&format!("  {}\n", msg));
        }
        turn = battle.log.len();
        if turn > 50 { break; }
    }

    if let Some(pc) = battle.combatants.iter().find(|c| c.is_player) {
        state.player.character.hp = pc.character.hp;
        state.player.character.status_effects = pc.character.status_effects.clone();
    }

    match &battle.state {
        BattleState::PlayerVictory { xp_gained, .. } => {
            let xp = *xp_gained;
            out.push_str(&format!("\n⚔ Victory! {} defeated.\n", enemy_name));
            out.push_str(&format!("  +{} XP\n", xp));
            let levels = state.player.character.add_experience(xp);
            for lvl in &levels {
                out.push_str(&format!("  🎉 Level up! You are now level {}.\n", lvl));
                if state.player.character.stat_points > 0 {
                    out.push_str(&format!(
                        "  You have {} stat point(s) to spend. Use 'alloc <stat>' (str/int/wis/con/dex/cha).\n",
                        state.player.character.stat_points
                    ));
                }
            }
            // Roll loot from the enemy template
            let (gold_drop, item_drops) = roll_loot(&enemy_template_id, rng);
            if gold_drop > 0 {
                state.gold += gold_drop;
                out.push_str(&format!("  +{} gold\n", gold_drop));
            }
            for item_id in &item_drops {
                if state.give_item(item_id, 1) {
                    out.push_str(&format!("  Found: {}\n", item_id.replace('_', " ")));
                }
            }
            let quest_msgs = state.on_enemy_killed(&enemy_template_id);
            for m in quest_msgs {
                out.push_str(&format!("  {}\n", m));
            }
        }
        BattleState::PlayerDefeat => {
            out.push_str("\n💀 You have been defeated.\n");
            out.push_str("  (Rest at a safe location to continue.)\n");
            state.player.character.hp = 1;
        }
        BattleState::Fled => {
            out.push_str("\nYou managed to escape!\n");
        }
        BattleState::InProgress => {
            out.push_str("\nThe battle was inconclusive.\n");
        }
    }
    out
}

fn run_combat(state: &mut GameState, enemy_id: &str, rng: &mut impl rand::Rng) -> String {
    match find_template(enemy_id) {
        Some(t) => {
            let enemy = t.spawn();
            run_combat_with(state, enemy, enemy_id, rng)
        }
        None => format!("No enemy found with id '{}'.", enemy_id),
    }
}

trait GenIndex {
    fn gen_index(&mut self, len: usize) -> usize;
}

impl<R: rand::Rng> GenIndex for R {
    fn gen_index(&mut self, len: usize) -> usize {
        if len == 0 { return 0; }
        self.gen_range(0..len)
    }
}

fn maybe_encounter(state: &mut GameState, difficulty: u32) -> Vec<String> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(state.turn as u64, |d| d.subsec_nanos() as u64 + state.turn as u64)
    );
    // 40% chance of encounter when entering a dangerous area
    let roll: f32 = rng.gen();
    if roll < 0.40 {
        // Pick a template from the valid pool
        use iron_age_data::all_enemy_templates;
        let pool: Vec<_> = all_enemy_templates()
            .into_iter()
            .filter(|t| (t.level.saturating_sub(1) / 2) <= difficulty.saturating_sub(1))
            .collect();
        if pool.is_empty() { return vec![]; }
        let idx = rng.gen_index(pool.len());
        let template = &pool[idx];
        let enemy = template.spawn();
        let template_id = template.id.clone();
        let msg = run_combat_with(state, enemy, &template_id, &mut rng);
        vec![msg]
    } else {
        vec![]
    }
}
