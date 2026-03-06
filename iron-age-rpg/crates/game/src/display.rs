use iron_age_world::Location;
use iron_age_character::Character;
use iron_age_combat::Combatant;
use iron_age_narrative::Quest;
use iron_age_inventory::{Equipment, EquipSlot, Inventory};

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
    use iron_age_character::Skill;
    let crafting_skills: Vec<(&str, &Skill)> = vec![
        ("Weaponsmithing", &Skill::Weaponsmithing),
        ("Armorsmithing",  &Skill::Armorsmithing),
        ("Alchemy",        &Skill::Alchemy),
        ("Cooking",        &Skill::Cooking),
        ("Mining",         &Skill::Mining),
        ("Gathering",      &Skill::Gathering),
        ("BoyerFletcher",  &Skill::BoyerFletcher),
    ];
    let craft_str: String = crafting_skills.iter()
        .filter_map(|(label, skill)| {
            c.skills.get(skill).filter(|sl| sl.level > 0 || sl.experience > 0)
                .map(|sl| format!("{}: {}", label, sl.level))
        })
        .collect::<Vec<_>>()
        .join(", ");
    let craft_display = if craft_str.is_empty() {
        "None".to_string()
    } else {
        craft_str
    };

    format!(
        "── {} (Level {}) ──\n\
         HP: {}/{} | Stamina: {}/{} | Mana: {}/{}\n\
         Gold: {} | XP to next: {}\n\
         STR:{} INT:{} WIS:{} CON:{} DEX:{} CHA:{}\n\
         Stat points: {} | Skill points: {}\n\
         Crafting skills: {}\n\
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
        craft_display,
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
            let equip_tag = if item.equip_slot.is_some() { " [equippable]" } else { "" };
            out.push_str(&format!("  {} x{}{} — {}\n",
                item.name, item.quantity, equip_tag, item.description));
        }
    }
    out
}

pub fn equipment_display(eq: &Equipment) -> String {
    let slots: &[(&str, &EquipSlot)] = &[
        ("Main Hand  ", &EquipSlot::MainHand),
        ("Off Hand   ", &EquipSlot::OffHand),
        ("Helmet     ", &EquipSlot::Helmet),
        ("Shoulders  ", &EquipSlot::Shoulders),
        ("Torso      ", &EquipSlot::Torso),
        ("Leggings   ", &EquipSlot::Leggings),
        ("Cape       ", &EquipSlot::Cape),
        ("Amulet     ", &EquipSlot::Amulet),
        ("Ring 1     ", &EquipSlot::Ring1),
        ("Ring 2     ", &EquipSlot::Ring2),
    ];
    let mut out = "── Equipped Gear ──\n".to_string();
    for (label, slot) in slots {
        let item_str = eq.get_slot(slot)
            .map(|i| format!("{} — {}", i.name, i.description))
            .unwrap_or_else(|| "(empty)".to_string());
        out.push_str(&format!("  {}: {}\n", label, item_str));
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
  search / examine / loot  — Search the current location for treasure and items
  talk <npc>               — Talk to an NPC
  attack                   — Attack a hostile creature
  flee                     — Attempt to flee from combat
  stats                    — Show your character sheet
  inventory / inv          — Show your inventory
  equipment / gear / eq    — Show your equipped gear
  equip <item>             — Equip an item from your inventory
  unequip <slot>           — Unequip an item (mainhand, offhand, helmet, etc.)
  alloc <stat> [n]         — Spend stat points (str/int/wis/con/dex/cha)
  quests / q               — Show your quest log
  rest                     — Rest at a safe location
  use <item>               — Use a consumable item
  shop [npc_id]            — Browse a merchant's wares
  buy <item_id>            — Buy an item from a merchant
  sell <item_id>           — Sell an item to a merchant
  craft list               — List known crafting recipes (with station & skill info)
  craft <recipe_id>        — Craft an item (must be at the required station)
  learn <recipe_id>        — Learn a new crafting recipe
  accept <quest_id>        — Accept a quest from an NPC
  complete <quest_id>      — Turn in a completed quest
  save                     — Save your progress to savegame.json
  help                     — Show this help
  quit                     — Quit the game
"
}
