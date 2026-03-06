use iron_age_world::Location;
use iron_age_character::Character;
use iron_age_combat::Combatant;
use iron_age_narrative::Quest;
use iron_age_inventory::Inventory;

pub fn title_screen() -> String {
    "
╔══════════════════════════════════════════════════════╗
║           I R O N   A G E   R . P . G .             ║
║          A Rust Text Adventure                       ║
╚══════════════════════════════════════════════════════╝
Type 'help' for a list of commands.
".trim_start().to_string()
}

pub fn intro_text() -> String {
    "
You are a wandering sellsword who has arrived at the village of Thornvale.
Word has reached you of goblin raids from the north — and where there is danger,
there is coin to be made. Speak to Elder Aldric in the village square to begin.
".trim_start().to_string()
}

pub fn location_display(loc: &Location) -> String {
    let mut out = format!(
        "── {} ──\n{}\n",
        loc.name, loc.description
    );

    if !loc.exits.is_empty() {
        out.push_str("\nExits: ");
        let exits: Vec<String> = loc.exits.iter().map(|e| {
            if e.is_locked {
                format!("{} [locked]", e.direction)
            } else {
                e.direction.clone()
            }
        }).collect();
        out.push_str(&exits.join(", "));
        out.push('\n');
    }

    if !loc.npc_ids.is_empty() {
        out.push_str(&format!("People here: {}\n", loc.npc_ids.join(", ")));
    }

    if !loc.enemy_spawn_ids.is_empty() && !loc.is_safe {
        out.push_str("⚠ This area may contain hostile creatures.\n");
    }

    if let Some(station) = &loc.has_crafting_station {
        out.push_str(&format!("🔨 Crafting station: {}\n", station));
    }

    out
}

pub fn character_sheet(c: &Character, gold: u32) -> String {
    format!(
        "── {} (Level {}) ──\n\
         HP: {}/{} | Stamina: {}/{} | Mana: {}/{}\n\
         Gold: {} | XP to next: {}\n\
         STR:{} INT:{} WIS:{} CON:{} DEX:{} CHA:{}\n\
         Stat points: {} | Skill points: {}\n\
         Perks: {}",
        c.name, c.level,
        c.hp, c.max_hp,
        c.stamina, c.max_stamina,
        c.mana, c.max_mana,
        gold,
        Character::xp_for_level(c.level + 1).saturating_sub(c.experience),
        c.stats.strength, c.stats.intelligence, c.stats.wisdom,
        c.stats.constitution, c.stats.dexterity, c.stats.charisma,
        c.stat_points, c.skill_points,
        if c.perks.is_empty() {
            "None".to_string()
        } else {
            c.perks.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>().join(", ")
        }
    )
}

pub fn inventory_display(inv: &Inventory, gold: u32) -> String {
    let mut out = format!("── Inventory ({}/{}) | Gold: {} ──\n",
        inv.items.len(), inv.max_slots, gold);
    if inv.items.is_empty() {
        out.push_str("  (empty)\n");
    } else {
        for item in &inv.items {
            out.push_str(&format!("  {} x{} — {}\n",
                item.name, item.quantity, item.description));
        }
    }
    out
}

pub fn quest_log_display(quests: &[&Quest]) -> String {
    if quests.is_empty() {
        return "No active quests.".to_string();
    }
    let mut out = "── Active Quests ──\n".to_string();
    for q in quests {
        out.push_str(&format!("\n[{}] {}\n{}\n", q.id, q.name, q.description));
        for obj in &q.objectives {
            let check = if obj.is_complete() { "✓" } else { "○" };
            out.push_str(&format!("  {} {}\n", check, obj.kind.progress_text()));
        }
        out.push_str(&format!(
            "  Reward: {} XP | {} gold\n",
            q.reward.experience, q.reward.gold
        ));
    }
    out
}

#[allow(dead_code)]
pub fn combat_display(combatants: &[Combatant]) -> String {
    let mut out = "── Combat ──\n".to_string();
    for c in combatants {
        let side = if c.is_player { "PLAYER" } else { "ENEMY " };
        out.push_str(&format!(
            "  [{}] {} — HP: {}/{}\n",
            side, c.character.name, c.character.hp, c.character.max_hp
        ));
    }
    out
}

pub fn help_text() -> &'static str {
    "
── Commands ──
  look / l                 — Describe your current location
  go <direction>           — Move in a direction (north, south, east, west)
  talk <npc>               — Talk to an NPC
  attack                   — Attack a hostile creature
  flee                     — Attempt to flee from combat
  stats                    — Show your character sheet
  inventory / inv          — Show your inventory
  quests / q               — Show your quest log
  rest                     — Rest at a safe location
  use <item>               — Use a consumable item
  craft list               — List known crafting recipes
  craft <recipe_id>        — Craft an item
  accept <quest_id>        — Accept a quest from an NPC
  complete <quest_id>      — Turn in a completed quest
  help                     — Show this help
  quit                     — Quit the game
"
}
