use crate::game_state::GameState;
use crate::display;
use iron_age_combat::{Battle, BattleAction, BattleState};
use iron_age_data::{find_template, find_item, roll_loot};
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

                    // Check for random encounter (FF-style scaled probability)
                    let loc = state.world.current_location().unwrap();
                    let difficulty = loc.region_type.encounter_difficulty();
                    let is_safe = loc.is_safe;
                    let encounter_msgs = if difficulty > 0 {
                        let msgs = maybe_encounter(state, difficulty);
                        if !msgs.is_empty() {
                            state.danger_steps = 0;
                        } else {
                            state.danger_steps += 1;
                        }
                        msgs
                    } else {
                        // Safe area resets the danger counter
                        if is_safe {
                            state.danger_steps = 0;
                        }
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
                    if !npc.shop_item_ids.is_empty() {
                        out.push_str(&format!(
                            "\n  [Shop available — type 'shop {}' to browse wares]",
                            npc_id
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

        "shop" | "store" => {
            show_shop(state, arg)
        }

        "buy" | "purchase" => {
            if arg.is_empty() {
                return CommandResult::Message("Buy what? e.g. 'buy health_potion'.".to_string());
            }
            buy_item(state, arg)
        }

        "sell" => {
            if arg.is_empty() {
                return CommandResult::Message("Sell what? e.g. 'sell wolf_pelt'.".to_string());
            }
            sell_item(state, arg)
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

        "search" | "examine" | "loot" => {
            search_location(state)
        }

        "flee" => {
            "You back away cautiously. (Use 'flee' during a 'fight' encounter.)".to_string()
        }

        "craft" => {
            if arg == "list" || arg.is_empty() {
                let mut out = "── Known Recipes ──\n".to_string();
                if state.crafting.known_recipes.is_empty() {
                    out.push_str("  (none — use 'learn <recipe_id>' to learn recipes)\n");
                }
                for recipe in &state.crafting.known_recipes {
                    out.push_str(&format!(
                        "  [{}] {} — {}\n",
                        recipe.id, recipe.name, recipe.description
                    ));
                    let ings: Vec<String> = recipe.ingredients.iter()
                        .map(|i| format!("{} x{}", i.item_id, i.quantity))
                        .collect();
                    out.push_str(&format!("    Ingredients: {}\n", ings.join(", ")));
                    let station_str = recipe.station.name()
                        .map_or("None".to_string(), |s| s.to_string());
                    out.push_str(&format!(
                        "    Station: {} | Skill: {} (req. level {})\n",
                        station_str, recipe.profession.name(), recipe.required_skill_level
                    ));
                }
                out
            } else {
                // Look up the recipe in known recipes
                let recipe = state.crafting.known_recipes.iter().find(|r| r.id == arg).cloned();
                match recipe {
                    None => format!(
                        "Unknown recipe '{}'. Type 'craft list' to see your known recipes.",
                        arg
                    ),
                    Some(recipe) => {
                        // Check crafting station requirement
                        if let Some(required_station) = recipe.station.name() {
                            let current_station = state.world.current_location()
                                .and_then(|l| l.has_crafting_station.as_deref());
                            let has_station = current_station
                                .map_or(false, |s| s.eq_ignore_ascii_case(required_station));
                            if !has_station {
                                return CommandResult::Message(format!(
                                    "You need a {} to craft this. Look for one in the right location.",
                                    required_station
                                ));
                            }
                        }

                        // Use the character's actual skill level for this profession
                        let skill_name = recipe.profession.name();
                        let skill_level = state.player.character.get_craft_skill(skill_name);

                        let mut rng = rand::rngs::StdRng::seed_from_u64(
                            state.turn as u64 * 7 + 13
                        );
                        let int = state.player.character.stats.intelligence;
                        let wis = state.player.character.stats.wisdom;

                        match state.crafting.craft(
                            &recipe.id, &mut state.inventory,
                            skill_level, int, wis, &mut rng,
                        ) {
                            Ok((stub_item, quality)) => {
                                // Prefer the full catalog item; fall back to the stub
                                let mut item = find_item(&recipe.output_item_id)
                                    .unwrap_or(stub_item);
                                item.quantity = recipe.output_quantity;

                                // Apply quality bonus to damage/armor
                                let q_bonus = quality.stat_bonus();
                                item.damage_base = (item.damage_base + q_bonus).max(0);
                                item.armor_base = (item.armor_base + q_bonus).max(0);

                                let name = item.name.clone();
                                let q_label = format!("{:?}", quality);

                                // Gain crafting skill XP (25 base + 10 per required level)
                                let xp_gain = 25u64 + recipe.required_skill_level as u64 * 10;
                                let levelled_up = state.player.character
                                    .gain_craft_xp(skill_name, xp_gain);

                                match state.inventory.add_item(item) {
                                    Ok(_) => {
                                        let mut out =
                                            format!("Crafted {} ({}).", name, q_label);
                                        if levelled_up {
                                            let new_lvl = state.player.character
                                                .get_craft_skill(skill_name);
                                            out.push_str(&format!(
                                                "\n  📈 {} skill increased to level {}!",
                                                skill_name, new_lvl
                                            ));
                                        }
                                        out
                                    }
                                    Err(_) => "Inventory full — couldn't store the crafted item."
                                        .to_string(),
                                }
                            }
                            Err(e) => format!("{}", e),
                        }
                    }
                }
            }
        }

        "learn" => {
            if arg.is_empty() {
                "Learn which recipe? e.g. 'learn iron_long_sword'. \
                 Use 'craft list' to see already known recipes."
                    .to_string()
            } else {
                if state.crafting.known_recipes.iter().any(|r| r.id == arg) {
                    format!("You already know how to craft '{}'.", arg)
                } else if state.crafting.learn_recipe(arg) {
                    format!("You have learned the recipe for '{}'.", arg)
                } else {
                    format!(
                        "No recipe named '{}' exists. \
                         Check the recipe id carefully.",
                        arg
                    )
                }
            }
        }

        "save" => {
            save_game(state)
        }

        "load" => {
            match load_game(state) {
                Ok(msg) => msg,
                Err(e) => e,
            }
        }

        "map" => {
            display::world_map_display(&state.world)
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

    let found = find_in_inventory(state, item_name);

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
                ItemType::ClarityPotion => {
                    let restore = 20 + state.player.character.stats.intelligence * 2;
                    let c = &mut state.player.character;
                    let before = c.mana;
                    c.mana = (c.mana + restore).min(c.max_mana);
                    let actual = c.mana - before;
                    let _ = state.inventory.remove_item(&item.id, 1);
                    format!("You drink the clarity potion and restore {} mana.", actual)
                }
                ItemType::FortifyPotion => {
                    use iron_age_core::StatusEffect;
                    let c = &mut state.player.character;
                    c.status_effects.retain(|e| e.name() != "Regen");
                    c.status_effects.push(StatusEffect::Regen { heal_per_turn: 5, turns_remaining: 5 });
                    let _ = state.inventory.remove_item(&item.id, 1);
                    "You drink the fortify potion. You feel vigour coursing through you (Regen for 5 turns).".to_string()
                }
                _ => format!("You can't use {} that way.", item.name),
            }
        }
    }
}

fn equip_item(state: &mut GameState, item_name: &str) -> String {
    let item_id = item_name.to_lowercase().replace(' ', "_");
    let found = find_in_inventory(state, item_name);

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

pub fn try_load_game(state: &mut GameState) -> Result<String, String> {
    load_game(state)
}

fn load_game(state: &mut GameState) -> Result<String, String> {
    use std::fs;

    let path = "savegame.json";
    let json = fs::read_to_string(path)
        .map_err(|e| format!("No save file found ('{}'): {}", path, e))?;
    let save: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| format!("Save file corrupt: {}", e))?;

    // Restore player stats
    let p = &save["player"];
    let c = &mut state.player.character;
    c.name = p["name"].as_str().unwrap_or("Hero").to_string();
    c.level = p["level"].as_u64().unwrap_or(1) as u32;
    c.experience = p["experience"].as_u64().unwrap_or(0);
    c.hp = p["hp"].as_i64().unwrap_or(c.max_hp as i64) as i32;
    c.max_hp = p["max_hp"].as_i64().unwrap_or(c.max_hp as i64) as i32;
    c.stamina = p["stamina"].as_i64().unwrap_or(c.max_stamina as i64) as i32;
    c.max_stamina = p["max_stamina"].as_i64().unwrap_or(c.max_stamina as i64) as i32;
    c.mana = p["mana"].as_i64().unwrap_or(c.max_mana as i64) as i32;
    c.max_mana = p["max_mana"].as_i64().unwrap_or(c.max_mana as i64) as i32;
    c.stat_points = p["stat_points"].as_u64().unwrap_or(0) as u32;
    c.skill_points = p["skill_points"].as_u64().unwrap_or(0) as u32;
    if let Some(stats) = p["stats"].as_object() {
        c.stats.strength = stats.get("strength").and_then(|v| v.as_i64()).unwrap_or(5) as i32;
        c.stats.intelligence = stats.get("intelligence").and_then(|v| v.as_i64()).unwrap_or(5) as i32;
        c.stats.wisdom = stats.get("wisdom").and_then(|v| v.as_i64()).unwrap_or(5) as i32;
        c.stats.constitution = stats.get("constitution").and_then(|v| v.as_i64()).unwrap_or(5) as i32;
        c.stats.dexterity = stats.get("dexterity").and_then(|v| v.as_i64()).unwrap_or(5) as i32;
        c.stats.charisma = stats.get("charisma").and_then(|v| v.as_i64()).unwrap_or(5) as i32;
    }

    // Restore gold and turn
    state.gold = save["gold"].as_u64().unwrap_or(10) as u32;
    state.turn = save["turn"].as_u64().unwrap_or(0) as u32;

    // Restore current location
    if let Some(loc_id) = save["current_location"].as_str() {
        if state.world.locations.contains_key(loc_id) {
            state.world.player_location_id = loc_id.to_string();
            if let Some(loc) = state.world.locations.get_mut(loc_id) {
                loc.is_visited = true;
            }
        }
    }

    // Restore inventory
    state.inventory.items.clear();
    if let Some(items) = save["inventory"].as_array() {
        for entry in items {
            if let Some(id) = entry["id"].as_str() {
                let qty = entry["quantity"].as_u64().unwrap_or(1) as u32;
                state.give_item(id, qty);
            }
        }
    }

    // Restore equipped items
    state.equipment = iron_age_inventory::Equipment::default();
    let eq_keys: &[&str] = &[
        "main_hand", "off_hand", "helmet", "shoulders",
        "torso", "leggings", "cape", "amulet", "ring1", "ring2",
    ];
    if let Some(equipped) = save["equipped"].as_object() {
        for key in eq_keys {
            if let Some(item_id) = equipped.get(*key).and_then(|v| v.as_str()) {
                if let Some(item) = iron_age_data::find_item(item_id) {
                    let _ = state.equipment.equip(item);
                }
            }
        }
    }

    // Restore quest state
    // Start active quests
    if let Some(active) = save["active_quests"].as_array() {
        let completed: Vec<String> = save["completed_quests"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        for q_val in active {
            if let Some(qid) = q_val.as_str() {
                let _ = state.quest_log.start_quest(qid, &completed);
            }
        }
    }
    // Mark completed quests
    if let Some(completed) = save["completed_quests"].as_array() {
        for q_val in completed {
            if let Some(qid) = q_val.as_str() {
                let _ = state.quest_log.start_quest(qid, &[]);
                let _ = state.quest_log.try_complete_quest(qid);
            }
        }
    }

    Ok(format!("Game loaded from '{}'. Welcome back, {}!", path, state.player.character.name))
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

fn search_location(state: &mut GameState) -> String {
    let loc = state.world.current_location();
    let (loc_id, loot_table_id, is_looted) = match loc {
        None => return "You are nowhere.".to_string(),
        Some(l) => (l.id.clone(), l.loot_table_id.clone(), l.is_looted),
    };

    if is_looted {
        return "You have already searched this area thoroughly. Nothing remains.".to_string();
    }

    let Some(table_id) = loot_table_id else {
        return "You search the area carefully but find nothing of particular interest.".to_string();
    };

    let Some(table) = iron_age_data::find_loot_table(&table_id) else {
        return "You search the area but find nothing of interest.".to_string();
    };

    let mut rng = rand::rngs::StdRng::seed_from_u64(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(state.turn as u64, |d| d.subsec_nanos() as u64 + state.turn as u64)
    );

    let flavor = table.flavor_text.clone();
    let (gold, items) = table.roll(&mut rng);

    // Mark as looted
    if let Some(l) = state.world.locations.get_mut(&loc_id) {
        l.is_looted = true;
    }

    let mut out = format!("{}\n", flavor);

    let mut found_any = gold > 0;
    if gold > 0 {
        state.gold += gold;
        out.push_str(&format!("  Found: {} gold\n", gold));
    }

    for (item_id, qty) in &items {
        if state.give_item(item_id, *qty) {
            let name = iron_age_data::find_item(item_id)
                .map(|i| i.name)
                .unwrap_or_else(|| item_id.replace('_', " "));
            out.push_str(&format!("  Found: {} x{}\n", name, qty));
            found_any = true;
        }
    }

    if !found_any {
        out.push_str("  Nothing useful found.\n");
    }

    out
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

// ── Shop helpers ──────────────────────────────────────────────────────────────

/// Look up an item in the inventory by id or by normalised name.
fn find_in_inventory(state: &GameState, item_name: &str) -> Option<iron_age_inventory::Item> {
    let item_id = item_name.to_lowercase().replace(' ', "_");
    state.inventory.items.iter().find(|i| {
        i.id == item_id || i.name.to_lowercase().replace(' ', "_") == item_id
    }).cloned()
}

/// Find all NPC IDs that have a shop and are present at the current location.
fn shop_npcs_at_location(state: &GameState) -> Vec<String> {
    let npc_ids = state.world.current_location()
        .map(|l| l.npc_ids.clone())
        .unwrap_or_default();
    npc_ids.into_iter()
        .filter(|id| {
            state.npcs.get(id)
                .map_or(false, |n| !n.shop_item_ids.is_empty())
        })
        .collect()
}

/// `shop` / `shop <npc_id>` — display a merchant's wares.
fn show_shop(state: &GameState, npc_arg: &str) -> String {
    let npc_id = if npc_arg.is_empty() {
        // Auto-pick the first merchant at this location
        let merchants = shop_npcs_at_location(state);
        if merchants.is_empty() {
            return "There is no merchant here. \
                    Find a village or market.".to_string();
        }
        if merchants.len() > 1 {
            let names: Vec<String> = merchants.iter()
                .filter_map(|id| state.npcs.get(id).map(|n| format!("{} ({})", n.name, id)))
                .collect();
            return format!(
                "Multiple merchants here. Specify one: {}\nExample: 'shop {}'",
                names.join(", "), merchants[0]
            );
        }
        merchants[0].clone()
    } else {
        npc_arg.to_lowercase().replace(' ', "_")
    };

    let npc = match state.npcs.get(&npc_id) {
        None => return format!("There is no merchant called '{}'.", npc_arg),
        Some(n) => n,
    };

    if npc.shop_item_ids.is_empty() {
        return format!("{} doesn't sell anything.", npc.name);
    }

    let mut out = format!("── {}'s Wares ──\n", npc.name);
    out.push_str(&format!("  Your gold: {}\n", state.gold));
    out.push_str("  Item                        Buy    Sell\n");
    out.push_str("  ──────────────────────────────────────\n");
    for item_id in &npc.shop_item_ids {
        if let Some(item) = iron_age_data::find_item(item_id) {
            let buy_price = item.value.max(1);
            let sell_price = (item.value / 2).max(1);
            out.push_str(&format!(
                "  {:<28} {:>3}g   {:>3}g\n",
                item.name, buy_price, sell_price
            ));
        }
    }
    out.push_str("\n  Type 'buy <item_id>' to purchase, 'sell <item_id>' to sell.\n");
    out
}

/// `buy <item_id>` — purchase an item from a merchant at the current location.
fn buy_item(state: &mut GameState, item_name: &str) -> String {
    let item_id = item_name.to_lowercase().replace(' ', "_");

    // Find a merchant at the current location that stocks this item
    let merchant_id = {
        let merchants = shop_npcs_at_location(state);
        merchants.into_iter().find(|npc_id| {
            state.npcs.get(npc_id)
                .map_or(false, |n| n.shop_item_ids.iter().any(|id| id == &item_id))
        })
    };

    let merchant_id = match merchant_id {
        None => {
            return format!(
                "No merchant here sells '{}'. \
                 Type 'shop' to see available wares.",
                item_name
            );
        }
        Some(id) => id,
    };

    let item = match iron_age_data::find_item(&item_id) {
        None => return format!("Unknown item '{}'.", item_name),
        Some(i) => i,
    };

    let buy_price = item.value.max(1);
    if state.gold < buy_price {
        let merchant_name = state.npcs.get(&merchant_id)
            .map(|n| n.name.as_str())
            .unwrap_or("The merchant");
        return format!(
            "{} asks {} gold for {}, but you only have {} gold.",
            merchant_name, buy_price, item.name, state.gold
        );
    }

    let item_name_display = item.name.clone();
    match state.inventory.add_item(item) {
        Ok(_) => {
            state.gold -= buy_price;
            format!(
                "You buy {} for {} gold. Gold remaining: {}.",
                item_name_display, buy_price, state.gold
            )
        }
        Err(_) => "Your inventory is full.".to_string(),
    }
}

/// `sell <item_id>` — sell an item from inventory to a merchant at the current location.
fn sell_item(state: &mut GameState, item_name: &str) -> String {
    // There must be at least one merchant here
    let has_merchant = !shop_npcs_at_location(state).is_empty();
    if !has_merchant {
        return "There is no merchant here to sell to. \
                Find a village or market.".to_string();
    }

    let found = find_in_inventory(state, item_name);

    match found {
        None => format!("You don't have '{}'.", item_name),
        Some(item) => {
            let sell_price = (item.value / 2).max(1);
            let item_name_display = item.name.clone();
            let _ = state.inventory.remove_item(&item.id, 1);
            state.gold += sell_price;
            format!(
                "You sell {} for {} gold. Gold: {}.",
                item_name_display, sell_price, state.gold
            )
        }
    }
}

/// FF-style scaled random encounter.
///
/// Base encounter chance increases with each step taken in dangerous areas
/// since the last encounter (`state.danger_steps`). This mirrors the
/// "encounter accumulation" feel of Final Fantasy games — the longer you
/// roam without a fight, the more likely one becomes.
///
/// Probability formula:
///   chance = (difficulty * 0.06) + (danger_steps * 0.07)
///   capped at 0.95 (95%).
///
/// At valley difficulty (2), with danger_steps accumulated from prior steps:
///   step 1 → 12%   step 3 → 26%   step 5 → 40%   step 8 → 61%
///
/// Enemy selection prefers the current location's `enemy_spawn_ids`, falling
/// back to the global difficulty-filtered pool only when no spawns are defined.
fn maybe_encounter(state: &mut GameState, difficulty: u32) -> Vec<String> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(state.turn as u64, |d| d.subsec_nanos() as u64 + state.turn as u64)
    );
    let base = difficulty as f32 * 0.06;
    let step_bonus = state.danger_steps as f32 * 0.07;
    let encounter_chance = (base + step_bonus).min(0.95);

    let roll: f32 = rng.gen();
    if roll >= encounter_chance {
        return vec![];
    }

    // Prefer location-specific enemy spawns for thematic encounters
    let spawn_id: Option<String> = state.world.current_location()
        .filter(|l| !l.enemy_spawn_ids.is_empty())
        .map(|l| {
            let idx = rng.gen_index(l.enemy_spawn_ids.len());
            l.enemy_spawn_ids[idx].clone()
        });

    if let Some(id) = spawn_id {
        let msg = run_combat(state, &id, &mut rng);
        vec![msg]
    } else {
        // Fall back to the difficulty-filtered global pool
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
    }
}
