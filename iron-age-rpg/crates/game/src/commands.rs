use crate::game_state::GameState;
use crate::display;
use iron_age_combat::{Battle, BattleAction, BattleState};
use iron_age_data::find_template;
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
