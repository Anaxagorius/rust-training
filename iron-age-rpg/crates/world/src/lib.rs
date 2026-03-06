use std::collections::HashMap;
use iron_age_core::GameError;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RegionType {
    Village,
    Forest,
    Dungeon,
    Plains,
    Swamp,
    Mountains,
    Coast,
    Cave,
    Ruins,
    Road,
}

impl RegionType {
    pub fn description(&self) -> &str {
        match self {
            Self::Village => "A settlement with traders and townsfolk.",
            Self::Forest => "Dense woodland filled with wildlife and hidden paths.",
            Self::Dungeon => "A dark underground complex teeming with danger.",
            Self::Plains => "Open grasslands, easy to traverse but exposed.",
            Self::Swamp => "Boggy marshland concealing poison and ancient secrets.",
            Self::Mountains => "Rugged high terrain, home to hardy creatures.",
            Self::Coast => "Rocky shores swept by sea winds.",
            Self::Cave => "A natural cavern in the earth.",
            Self::Ruins => "Crumbling remnants of a bygone civilization.",
            Self::Road => "A worn trade road connecting settlements.",
        }
    }

    pub fn encounter_difficulty(&self) -> u32 {
        match self {
            Self::Village | Self::Road => 0,
            Self::Plains | Self::Coast => 1,
            Self::Forest | Self::Swamp => 2,
            Self::Mountains | Self::Cave | Self::Ruins => 3,
            Self::Dungeon => 4,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Exit {
    pub direction: String,
    pub destination_id: String,
    pub description: String,
    pub requires_key: Option<String>,
    pub is_locked: bool,
}

impl Exit {
    pub fn new(direction: &str, destination_id: &str, description: &str) -> Self {
        Self {
            direction: direction.to_string(),
            destination_id: destination_id.to_string(),
            description: description.to_string(),
            requires_key: None,
            is_locked: false,
        }
    }

    pub fn locked(mut self, key_id: &str) -> Self {
        self.requires_key = Some(key_id.to_string());
        self.is_locked = true;
        self
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub description: String,
    pub region_type: RegionType,
    pub exits: Vec<Exit>,
    pub npc_ids: Vec<String>,
    pub enemy_spawn_ids: Vec<String>,
    pub loot_table_id: Option<String>,
    pub is_safe: bool,
    pub is_visited: bool,
    pub has_crafting_station: Option<String>,
}

impl Location {
    pub fn new(id: &str, name: &str, description: &str, region_type: RegionType) -> Self {
        let is_safe = matches!(region_type, RegionType::Village | RegionType::Road);
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            region_type,
            exits: Vec::new(),
            npc_ids: Vec::new(),
            enemy_spawn_ids: Vec::new(),
            loot_table_id: None,
            is_safe,
            is_visited: false,
            has_crafting_station: None,
        }
    }

    pub fn with_exit(mut self, exit: Exit) -> Self {
        self.exits.push(exit);
        self
    }

    pub fn with_npc(mut self, npc_id: &str) -> Self {
        self.npc_ids.push(npc_id.to_string());
        self
    }

    pub fn with_enemy_spawn(mut self, enemy_id: &str) -> Self {
        self.enemy_spawn_ids.push(enemy_id.to_string());
        self
    }

    pub fn with_crafting_station(mut self, station: &str) -> Self {
        self.has_crafting_station = Some(station.to_string());
        self
    }

    pub fn exit_for_direction(&self, direction: &str) -> Option<&Exit> {
        self.exits.iter().find(|e| e.direction.eq_ignore_ascii_case(direction))
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WorldMap {
    pub locations: HashMap<String, Location>,
    pub player_location_id: String,
}

impl WorldMap {
    pub fn new(starting_location_id: &str) -> Self {
        Self {
            locations: HashMap::new(),
            player_location_id: starting_location_id.to_string(),
        }
    }

    pub fn add_location(&mut self, location: Location) {
        self.locations.insert(location.id.clone(), location);
    }

    pub fn current_location(&self) -> Option<&Location> {
        self.locations.get(&self.player_location_id)
    }

    pub fn current_location_mut(&mut self) -> Option<&mut Location> {
        self.locations.get_mut(&self.player_location_id)
    }

    pub fn travel(&mut self, direction: &str) -> Result<&Location, GameError> {
        let destination_id = {
            let current = self.current_location()
                .ok_or_else(|| GameError::NotFound("Current location".to_string()))?;
            let exit = current.exit_for_direction(direction)
                .ok_or_else(|| GameError::InvalidOperation(format!("No exit to the '{}'", direction)))?;
            if exit.is_locked {
                return Err(GameError::InvalidOperation(
                    format!("The way {} is locked. You need a key.", direction)
                ));
            }
            exit.destination_id.clone()
        };
        self.player_location_id = destination_id.clone();
        if let Some(loc) = self.locations.get_mut(&destination_id) {
            loc.is_visited = true;
        }
        self.locations.get(&self.player_location_id)
            .ok_or_else(|| GameError::NotFound(destination_id))
    }

    pub fn unlock_exit(&mut self, direction: &str) -> bool {
        if let Some(loc) = self.current_location_mut() {
            if let Some(exit) = loc.exits.iter_mut().find(|e| e.direction.eq_ignore_ascii_case(direction)) {
                exit.is_locked = false;
                return true;
            }
        }
        false
    }

    pub fn visited_count(&self) -> usize {
        self.locations.values().filter(|l| l.is_visited).count()
    }
}

/// Build the default starting world map.
pub fn build_starting_world() -> WorldMap {
    let mut map = WorldMap::new("thornvale_square");

    // --- Thornvale Village ---
    map.add_location(
        Location::new(
            "thornvale_square",
            "Thornvale Village Square",
            "The central square of Thornvale, a small iron-age village. \
             Mud-brick houses ring a well. A crier hawks news of goblin raids \
             to the north. Smoke rises from the smithy to the east.",
            RegionType::Village,
        )
        .with_exit(Exit::new("north", "thornvale_gate", "The north gate leads to the King's Road."))
        .with_exit(Exit::new("east", "thornvale_smithy", "The clang of hammer on anvil echoes from the smithy."))
        .with_exit(Exit::new("south", "thornvale_market", "Stalls and hawkers crowd the market district."))
        .with_exit(Exit::new("west", "thornvale_inn", "A warm glow and smell of stew comes from the inn."))
        .with_npc("elder_aldric")
        .with_npc("town_crier"),
    );

    map.add_location(
        Location::new(
            "thornvale_smithy",
            "Thornvale Smithy",
            "A hot, smoky forge-house. Iron tools and weapons hang on the walls. \
             The blacksmith Grund wipes his brow and nods at your entrance.",
            RegionType::Village,
        )
        .with_exit(Exit::new("west", "thornvale_square", "Back to the village square."))
        .with_npc("blacksmith_grund")
        .with_crafting_station("Forge"),
    );

    map.add_location(
        Location::new(
            "thornvale_market",
            "Thornvale Market",
            "Rows of stalls sell grain, cloth, and oddities. A travelling merchant \
             eyes you with interest. Supplies can be purchased here.",
            RegionType::Village,
        )
        .with_exit(Exit::new("north", "thornvale_square", "Back to the village square."))
        .with_npc("merchant_serah"),
    );

    map.add_location(
        Location::new(
            "thornvale_inn",
            "The Rusted Helm Inn",
            "Low beams, a roaring fire, and the smell of ale. A bard strums \
             in the corner. This is a safe place to rest and recover.",
            RegionType::Village,
        )
        .with_exit(Exit::new("east", "thornvale_square", "Back to the village square."))
        .with_npc("innkeeper_marta"),
    );

    // --- King's Road ---
    map.add_location(
        Location::new(
            "thornvale_gate",
            "Thornvale North Gate",
            "The village gate opens onto the King's Road. Cart-ruts in the mud \
             lead north toward the Ashwood Forest. A guard watches the road.",
            RegionType::Road,
        )
        .with_exit(Exit::new("south", "thornvale_square", "Back into Thornvale village."))
        .with_exit(Exit::new("north", "kings_road_south", "The King's Road stretches north."))
        .with_npc("guard_torven"),
    );

    map.add_location(
        Location::new(
            "kings_road_south",
            "King's Road (South)",
            "A well-worn road flanked by low hedges. The outline of Ashwood Forest \
             is visible to the north-east. The land is open and quiet — for now.",
            RegionType::Road,
        )
        .with_exit(Exit::new("south", "thornvale_gate", "Thornvale's gate is to the south."))
        .with_exit(Exit::new("north", "kings_road_fork", "The road continues north to a fork."))
        .with_exit(Exit::new("east", "ashwood_edge", "A narrow trail breaks into the forest edge."))
        .with_enemy_spawn("goblin_scout"),
    );

    map.add_location(
        Location::new(
            "kings_road_fork",
            "King's Road Fork",
            "The road splits here. A weathered signpost points north to \
             'Ironmere Keep' and east deeper into Ashwood. Crows circle overhead.",
            RegionType::Road,
        )
        .with_exit(Exit::new("south", "kings_road_south", "The road south toward Thornvale."))
        .with_exit(Exit::new("north", "ironmere_approach", "North toward the ruined keep."))
        .with_exit(Exit::new("east", "ashwood_clearing", "Into the heart of Ashwood Forest."))
        .with_enemy_spawn("wolf")
        .with_enemy_spawn("goblin_scout"),
    );

    // --- Ashwood Forest ---
    map.add_location(
        Location::new(
            "ashwood_edge",
            "Ashwood Forest Edge",
            "Ancient ash trees crowd close, their roots buckling the ground. \
             Light filters through the canopy in dusty shafts. Something watches \
             from deeper in the forest.",
            RegionType::Forest,
        )
        .with_exit(Exit::new("west", "kings_road_south", "Back to the King's Road."))
        .with_exit(Exit::new("north", "ashwood_clearing", "Deeper into the forest."))
        .with_enemy_spawn("wolf")
        .with_enemy_spawn("forest_spider"),
    );

    map.add_location(
        Location::new(
            "ashwood_clearing",
            "Ashwood Forest Clearing",
            "A mossy clearing where old standing stones lean at odd angles. \
             Strange carvings mark the stones — iron-age glyphs no one alive can \
             fully read. A path continues north-east toward the Bog.",
            RegionType::Forest,
        )
        .with_exit(Exit::new("south", "ashwood_edge", "Back south toward the road."))
        .with_exit(Exit::new("west", "kings_road_fork", "West to the road fork."))
        .with_exit(Exit::new("north", "ashwood_depths", "Deeper into the dark forest."))
        .with_exit(Exit::new("east", "bog_trail", "A muddy trail leads to the bog."))
        .with_enemy_spawn("wolf")
        .with_enemy_spawn("goblin_warrior"),
    );

    map.add_location(
        Location::new(
            "ashwood_depths",
            "Ashwood Forest Depths",
            "The canopy closes overhead and the air grows cold. You can hear \
             howling in the distance. The forest is at its most dangerous here. \
             A cave mouth gapes in the hillside to the north.",
            RegionType::Forest,
        )
        .with_exit(Exit::new("south", "ashwood_clearing", "Back south to the clearing."))
        .with_exit(Exit::new("north", "wolf_den_entrance", "The wolf den cave entrance."))
        .with_enemy_spawn("wolf")
        .with_enemy_spawn("dire_wolf")
        .with_enemy_spawn("forest_spider"),
    );

    // --- Wolf Den ---
    map.add_location(
        Location::new(
            "wolf_den_entrance",
            "Wolf Den Entrance",
            "The cave reeks of animal musk. Bones litter the threshold. \
             Low growls echo from within.",
            RegionType::Cave,
        )
        .with_exit(Exit::new("south", "ashwood_depths", "Back into the forest."))
        .with_exit(Exit::new("north", "wolf_den_lair", "Into the den proper."))
        .with_enemy_spawn("dire_wolf"),
    );

    map.add_location(
        Location::new(
            "wolf_den_lair",
            "Wolf Den Lair",
            "The main chamber of the wolf den. A massive dire wolf, scarred \
             from countless battles, guards its pups. Trophies of past \
             adventurers litter the cave floor.",
            RegionType::Cave,
        )
        .with_exit(Exit::new("south", "wolf_den_entrance", "Back to the den entrance."))
        .with_enemy_spawn("dire_wolf_alpha"),
    );

    // --- Bog ---
    map.add_location(
        Location::new(
            "bog_trail",
            "Bog Trail",
            "A precarious path through sucking marsh. Twisted willows claw \
             the sky and strange lights drift over the water at night. \
             The smell of rot is heavy.",
            RegionType::Swamp,
        )
        .with_exit(Exit::new("west", "ashwood_clearing", "Back toward the forest clearing."))
        .with_exit(Exit::new("east", "bog_heart", "Deeper into the bog."))
        .with_enemy_spawn("bog_crawler")
        .with_enemy_spawn("swamp_witch"),
    );

    map.add_location(
        Location::new(
            "bog_heart",
            "Heart of the Bog",
            "The bog opens into a wide, still lake. A crumbling stone circle \
             rises from the black water. Ancient offerings hang from the willows. \
             This place hums with forgotten power.",
            RegionType::Swamp,
        )
        .with_exit(Exit::new("west", "bog_trail", "Back along the bog trail."))
        .with_enemy_spawn("bog_crawler")
        .with_enemy_spawn("swamp_witch"),
    );

    // --- Ironmere Keep ---
    map.add_location(
        Location::new(
            "ironmere_approach",
            "Ironmere Keep Approach",
            "A stone road, broken and overgrown, leads to the ruins of \
             Ironmere Keep. The walls are crumbled but the gatehouse still \
             stands. Goblin banners hang from the battlements — the raiders \
             have made it their base.",
            RegionType::Ruins,
        )
        .with_exit(Exit::new("south", "kings_road_fork", "Back south to the fork."))
        .with_exit(Exit::new("north", "ironmere_courtyard", "Through the ruined gatehouse."))
        .with_enemy_spawn("goblin_warrior")
        .with_enemy_spawn("goblin_archer"),
    );

    map.add_location(
        Location::new(
            "ironmere_courtyard",
            "Ironmere Keep Courtyard",
            "The courtyard is littered with goblin camp-fires and makeshift \
             shelters. A ramshackle forge has been set up in one corner. \
             The keep tower looms to the north, its door banded with iron.",
            RegionType::Ruins,
        )
        .with_exit(Exit::new("south", "ironmere_approach", "Back to the approach road."))
        .with_exit(Exit::new("north", "ironmere_tower", "To the iron-banded tower door.").locked("iron_key"))
        .with_enemy_spawn("goblin_warrior")
        .with_enemy_spawn("goblin_shaman"),
    );

    map.add_location(
        Location::new(
            "ironmere_tower",
            "Ironmere Keep Tower",
            "The interior of the ancient tower. Spiralling stairs lead up to \
             the warlord's throne room. A cache of plundered goods fills the \
             ground floor. This is the goblin warlord's lair.",
            RegionType::Dungeon,
        )
        .with_exit(Exit::new("south", "ironmere_courtyard", "Back to the courtyard."))
        .with_enemy_spawn("goblin_warlord"),
    );

    // Mark the starting location as visited
    if let Some(start) = map.locations.get_mut("thornvale_square") {
        start.is_visited = true;
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_starting_world_has_locations() {
        let world = build_starting_world();
        assert!(world.locations.len() >= 10);
        assert!(world.current_location().is_some());
    }

    #[test]
    fn test_travel_valid_direction() {
        let mut world = build_starting_world();
        let result = world.travel("east");
        assert!(result.is_ok());
        assert_eq!(world.player_location_id, "thornvale_smithy");
    }

    #[test]
    fn test_travel_invalid_direction() {
        let mut world = build_starting_world();
        let result = world.travel("up");
        assert!(result.is_err());
    }

    #[test]
    fn test_locked_exit_requires_unlock() {
        let mut world = build_starting_world();
        world.player_location_id = "ironmere_courtyard".to_string();
        let result = world.travel("north");
        assert!(result.is_err());
        world.unlock_exit("north");
        let result = world.travel("north");
        assert!(result.is_ok());
    }

    #[test]
    fn test_visited_count_increments() {
        let mut world = build_starting_world();
        let initial = world.visited_count();
        world.travel("east").unwrap();
        assert_eq!(world.visited_count(), initial + 1);
    }

    #[test]
    fn test_safe_locations_are_villages() {
        let world = build_starting_world();
        let smithy = world.locations.get("thornvale_smithy").unwrap();
        assert!(smithy.is_safe);
        let forest = world.locations.get("ashwood_edge").unwrap();
        assert!(!forest.is_safe);
    }

    #[test]
    fn test_region_encounter_difficulty() {
        assert_eq!(RegionType::Village.encounter_difficulty(), 0);
        assert_eq!(RegionType::Dungeon.encounter_difficulty(), 4);
        assert!(RegionType::Forest.encounter_difficulty() > RegionType::Plains.encounter_difficulty());
    }
}
