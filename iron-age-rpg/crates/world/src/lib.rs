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
    /// Wide valley floor between towering mountain ranges.
    Valley,
    /// Damp underground burial chambers filled with restless dead.
    Crypt,
    /// Ancient sealed tombs, guarded by powerful undead.
    Tomb,
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
            Self::Valley => "An open valley floor between towering mountain ranges.",
            Self::Crypt => "A damp underground burial chamber, home to restless dead.",
            Self::Tomb => "An ancient sealed tomb guarded by powerful undead.",
        }
    }

    pub fn encounter_difficulty(&self) -> u32 {
        match self {
            Self::Village | Self::Road => 0,
            Self::Plains | Self::Coast => 1,
            Self::Valley | Self::Forest | Self::Swamp => 2,
            Self::Mountains | Self::Cave | Self::Ruins | Self::Crypt => 3,
            Self::Dungeon | Self::Tomb => 4,
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
            "The central square of Thornvale, a small iron-age village nestled \
             in the heart of Embervale — a wide valley cradled between two ancient \
             mountain ranges. Mud-brick houses ring a well. A crier hawks news of \
             goblin raids to the north. Smoke rises from the smithy to the east. \
             Through the north gate, the valley stretches wide and untamed.",
            RegionType::Village,
        )
        .with_exit(Exit::new("north", "thornvale_gate", "The north gate leads to the King's Road and the open valley."))
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
            "thornvale_inn",
            "The Rusted Helm Inn",
            "Low beams, a roaring fire, and the smell of ale. A bard strums \
             in the corner. This is a safe place to rest and recover.",
            RegionType::Village,
        )
        .with_exit(Exit::new("east", "thornvale_square", "Back to the village square."))
        .with_npc("innkeeper_marta")
        .with_crafting_station("Campfire"),
    );

    map.add_location(
        Location::new(
            "thornvale_market",
            "Thornvale Market",
            "Rows of stalls sell grain, cloth, and oddities. A travelling merchant \
             eyes you with interest. Supplies can be purchased here. A rough trail \
             leads south into the open valley meadows.",
            RegionType::Village,
        )
        .with_exit(Exit::new("north", "thornvale_square", "Back to the village square."))
        .with_exit(Exit::new("south", "valley_south_meadow", "A dirt trail leads south into the valley meadows."))
        .with_npc("merchant_serah")
        .with_crafting_station("TanningRack"),
    );

    // --- King's Road ---
    map.add_location(
        Location::new(
            "thornvale_gate",
            "Thornvale North Gate",
            "The village gate opens onto the King's Road and the wide sweep of \
             Embervale. Rugged mountain ridges loom to the north and south, \
             framing the valley between them. A guard watches the road. Trails \
             branch northwest into older, quieter parts of the valley.",
            RegionType::Road,
        )
        .with_exit(Exit::new("south", "thornvale_square", "Back into Thornvale village."))
        .with_exit(Exit::new("north", "kings_road_south", "The King's Road stretches north through the valley."))
        .with_exit(Exit::new("east", "valley_northeast", "A rough path leads northeast across the valley floor."))
        .with_exit(Exit::new("west", "valley_northwest", "A faint trail winds northwest into quieter parts of the valley."))
        .with_npc("guard_torven"),
    );

    map.add_location(
        Location::new(
            "kings_road_south",
            "King's Road (South)",
            "A well-worn road flanked by low hedges winding through Embervale. \
             The outline of Ashwood Forest is visible to the north-east. The land \
             is open — exposed to whatever haunts the valley at night. The valley \
             floor opens westward into rolling grassland.",
            RegionType::Road,
        )
        .with_exit(Exit::new("south", "thornvale_gate", "Thornvale's gate is to the south."))
        .with_exit(Exit::new("north", "kings_road_fork", "The road continues north to a fork."))
        .with_exit(Exit::new("east", "ashwood_edge", "A narrow trail breaks into the forest edge."))
        .with_exit(Exit::new("west", "valley_west", "The valley floor opens west into open grassland."))
        .with_enemy_spawn("goblin_scout")
        .with_enemy_spawn("valley_wolf"),
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
        .with_enemy_spawn("goblin_warrior")
        .with_crafting_station("FletchingBench"),
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
        .with_enemy_spawn("swamp_witch")
        .with_crafting_station("AlchemyStone"),
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

    // =========================================================
    // --- Embervale — The Valley World ---
    // The player's starting region: a wide valley between two
    // ancient mountain ranges. Embervale is expansive and filled
    // with adventure: caves, derelict buildings, ruined settlements,
    // crypts, and ancient tombs.
    // =========================================================

    // --- Valley Floor (North) ---
    map.add_location(
        Location::new(
            "valley_northwest",
            "Embervale — Northwest Reach",
            "The northwest corner of the valley is quiet and unsettling. \
             Knee-high grass sways in the wind and an old goat track winds \
             between tumbledown stone walls — the remnants of long-abandoned \
             field boundaries. The ruined mill looms to the west. The valley \
             opens north toward the mountain foothills.",
            RegionType::Valley,
        )
        .with_exit(Exit::new("east", "thornvale_gate", "East back to Thornvale's north gate."))
        .with_exit(Exit::new("north", "valley_north", "North into the higher valley floor."))
        .with_exit(Exit::new("west", "derelict_mill", "West toward the old ruined mill."))
        .with_exit(Exit::new("south", "valley_west", "South toward the western meadows."))
        .with_enemy_spawn("bandit")
        .with_enemy_spawn("valley_wolf"),
    );

    map.add_location(
        Location::new(
            "valley_northeast",
            "Embervale — Northeast Reach",
            "The northeast sweep of the valley is rougher terrain — the grass \
             gives way to scattered boulders and patches of scrub. A dark ravine \
             cuts south toward the shadow gorge. Ahead, the eastern crags begin \
             to rise. The old watchtower is visible on a rise to the east.",
            RegionType::Valley,
        )
        .with_exit(Exit::new("west", "thornvale_gate", "West back to Thornvale's north gate."))
        .with_exit(Exit::new("north", "valley_north", "North toward the upper valley."))
        .with_exit(Exit::new("south", "shadow_gorge", "South into the dark shadow gorge."))
        .with_exit(Exit::new("east", "valley_watchtower", "East toward the ruined watchtower on the rise."))
        .with_enemy_spawn("bandit")
        .with_enemy_spawn("goblin_scout"),
    );

    map.add_location(
        Location::new(
            "valley_north",
            "Embervale — Northern Valley",
            "The northern reach of the valley, where the mountains close in on \
             both sides. The air is colder here and the grass grows pale. Ridges \
             of bare stone mark the start of the mountain foothills. A narrow cleft \
             in the western ridge leads to a cave entrance. To the east, a burial \
             mound rises from the earth — an ancient barrow, silent and foreboding. \
             North lies the high mountain pass.",
            RegionType::Valley,
        )
        .with_exit(Exit::new("south", "valley_northwest", "South back toward the northwest valley floor."))
        .with_exit(Exit::new("southeast", "valley_northeast", "Southeast back toward the northeast valley."))
        .with_exit(Exit::new("north", "north_mountain_pass", "North into the forbidding mountain pass."))
        .with_exit(Exit::new("west", "crystal_cave_entrance", "West into a cleft in the ridge — a cave entrance glitters."))
        .with_exit(Exit::new("east", "ancient_barrow_mound", "East toward the brooding burial mound."))
        .with_enemy_spawn("stone_troll")
        .with_enemy_spawn("goblin_scout")
        .with_enemy_spawn("valley_wolf"),
    );

    // --- Valley Floor (West & South) ---
    map.add_location(
        Location::new(
            "valley_west",
            "Embervale — Western Meadows",
            "Rolling meadowland stretches westward from the King's Road. \
             Wildflowers nod in the breeze and the distant sound of running water \
             can be heard. The western mountain range rises to the south, \
             its crags dusted with cloud. An old farmstead crumbles on a hillside \
             to the southwest.",
            RegionType::Valley,
        )
        .with_exit(Exit::new("east", "kings_road_south", "East back to the King's Road."))
        .with_exit(Exit::new("north", "valley_northwest", "North toward the northwest valley."))
        .with_exit(Exit::new("south", "valley_south_meadow", "South into the southern meadows."))
        .with_exit(Exit::new("southwest", "abandoned_farmstead", "Southwest toward the crumbling farmstead."))
        .with_exit(Exit::new("west", "west_mountain_crags", "West into the rocky western mountain crags."))
        .with_enemy_spawn("valley_wolf")
        .with_enemy_spawn("giant_bat"),
    );

    map.add_location(
        Location::new(
            "valley_south_meadow",
            "Embervale — South Meadows",
            "The southern meadows of Embervale are wide and deceptively peaceful. \
             Long grass conceals old foundations — signs of a settlement that once \
             stood here. The southern mountain range is a dark wall at the horizon. \
             A collapsed hamlet is visible to the southeast. Thornvale's market \
             gate is a short walk north.",
            RegionType::Valley,
        )
        .with_exit(Exit::new("north", "thornvale_market", "North back toward Thornvale market."))
        .with_exit(Exit::new("west", "valley_west", "West into the western meadows."))
        .with_exit(Exit::new("east", "valley_east", "East across the valley floor."))
        .with_exit(Exit::new("south", "south_mountain_foothills", "South toward the southern mountain foothills."))
        .with_exit(Exit::new("southeast", "millford_ruins", "Southeast toward the ruins of a collapsed hamlet."))
        .with_enemy_spawn("bandit")
        .with_enemy_spawn("valley_wolf"),
    );

    map.add_location(
        Location::new(
            "valley_east",
            "Embervale — Eastern Flats",
            "The eastern side of the valley is flatter and windier. Scattered \
             rocks protrude from the turf. A gorge entrance yawns to the northeast, \
             carved deep into the earth. The King's Road and the forest of Ashwood \
             are just to the north.",
            RegionType::Valley,
        )
        .with_exit(Exit::new("west", "valley_south_meadow", "West back toward the south meadows."))
        .with_exit(Exit::new("north", "kings_road_south", "North toward the King's Road."))
        .with_exit(Exit::new("northeast", "shadow_gorge", "Northeast into the shadow gorge ravine."))
        .with_exit(Exit::new("south", "valley_tomb_approach", "South toward a sealed stone gateway."))
        .with_enemy_spawn("goblin_scout")
        .with_enemy_spawn("valley_wolf"),
    );

    // --- Mountains ---
    map.add_location(
        Location::new(
            "north_mountain_pass",
            "North Mountain Pass",
            "A treacherous high-altitude pass through the northern range. \
             Biting wind howls between ice-streaked cliffs. Only the hardiest — \
             or most desperate — venture here. A stone troll hunches beneath \
             an overhang ahead, guarding the narrow path. Far below, Embervale \
             stretches south.",
            RegionType::Mountains,
        )
        .with_exit(Exit::new("south", "valley_north", "South back down into the valley."))
        .with_enemy_spawn("stone_troll")
        .with_enemy_spawn("mountain_goat"),
    );

    map.add_location(
        Location::new(
            "west_mountain_crags",
            "Western Mountain Crags",
            "Sheer crags of dark granite rise to either side. Loose scree \
             shifts underfoot with every step. The wind carries a high-pitched \
             shriek — bats roost in the cliff faces. The valley lies to the east.",
            RegionType::Mountains,
        )
        .with_exit(Exit::new("east", "valley_west", "East back down to the valley floor."))
        .with_enemy_spawn("giant_bat")
        .with_enemy_spawn("stone_troll"),
    );

    map.add_location(
        Location::new(
            "south_mountain_foothills",
            "Southern Mountain Foothills",
            "The foothills of the southern range are layered with gorse and \
             loose stone. Goat paths wind between boulders. A sealed stone \
             door is set into the hillside — an old tomb, long forgotten by \
             the valley's living. The valley meadows are north.",
            RegionType::Mountains,
        )
        .with_exit(Exit::new("north", "valley_south_meadow", "North back into the valley meadows."))
        .with_exit(Exit::new("east", "valley_tomb_approach", "East toward the sealed tomb gateway."))
        .with_enemy_spawn("mountain_goat")
        .with_enemy_spawn("stone_troll"),
    );

    // --- Caves ---
    map.add_location(
        Location::new(
            "crystal_cave_entrance",
            "Crystal Cave Entrance",
            "A narrow cleft in the western mountain ridge opens into a \
             surprisingly large cave. The walls catch the fading daylight and \
             throw it back as prismatic sparks — embedded crystals line every \
             surface. It is beautiful and deadly in equal measure. Strange \
             scratching sounds come from deeper within.",
            RegionType::Cave,
        )
        .with_exit(Exit::new("east", "valley_north", "East back out to the valley."))
        .with_exit(Exit::new("north", "crystal_cave_depths", "Deeper into the sparkling crystal cave."))
        .with_enemy_spawn("giant_bat")
        .with_enemy_spawn("cave_bear"),
    );

    map.add_location(
        Location::new(
            "crystal_cave_depths",
            "Crystal Cave — Inner Chamber",
            "The deeper chamber pulses with dim blue-green light from fist-sized \
             crystals. The floor is slick with mineral-rich water. A massive cave \
             bear claims this chamber as its den, bones of past meals heaped in \
             one corner. Crystal shards glitter on the ground — treasure for those \
             bold enough to survive.",
            RegionType::Cave,
        )
        .with_exit(Exit::new("south", "crystal_cave_entrance", "South back to the cave entrance."))
        .with_enemy_spawn("cave_bear")
        .with_enemy_spawn("giant_bat"),
    );

    map.add_location(
        Location::new(
            "shadow_gorge",
            "Shadow Gorge",
            "A deep ravine slashed into the valley floor. The walls are so high \
             that sunlight never reaches the bottom — the air is cold and perpetually \
             dim. The sound of dripping water mingles with distant inhuman growls. \
             A cave mouth gapes at the northern end of the gorge, reeking of musk \
             and old blood.",
            RegionType::Cave,
        )
        .with_exit(Exit::new("north", "valley_northeast", "North back up to the valley floor."))
        .with_exit(Exit::new("south", "valley_east", "South toward the eastern valley flats."))
        .with_exit(Exit::new("down", "shadow_cave_entrance", "Into the cave mouth at the gorge floor."))
        .with_enemy_spawn("goblin_scout")
        .with_enemy_spawn("giant_bat")
        .with_enemy_spawn("valley_wolf"),
    );

    map.add_location(
        Location::new(
            "shadow_cave_entrance",
            "Shadow Cave — Entrance",
            "The cave is pitch-black save for a faint phosphorescent glow from \
             some of the fungal growths on the walls. The stench is overwhelming — \
             equal parts rot, animal musk, and something older and fouler. Crude \
             goblin scratches mar the stone walls. Further in, the cave widens \
             into a larger chamber.",
            RegionType::Cave,
        )
        .with_exit(Exit::new("up", "shadow_gorge", "Back up out of the cave to the gorge."))
        .with_exit(Exit::new("north", "shadow_cave_depths", "Deeper into the shadow cave."))
        .with_enemy_spawn("goblin_scout")
        .with_enemy_spawn("goblin_warrior")
        .with_enemy_spawn("giant_bat"),
    );

    map.add_location(
        Location::new(
            "shadow_cave_depths",
            "Shadow Cave — Depths",
            "The main chamber of the shadow cave is a sprawling, low-ceilinged \
             space riddled with side tunnels. This is a goblin staging post — \
             fire pits, crude weapons, and stolen goods fill the space. A goblin \
             shaman has claimed the back alcove as a ritual space, the walls \
             daubed with dark symbols. Loot is piled high.",
            RegionType::Cave,
        )
        .with_exit(Exit::new("south", "shadow_cave_entrance", "South back toward the cave entrance."))
        .with_enemy_spawn("goblin_warrior")
        .with_enemy_spawn("goblin_archer")
        .with_enemy_spawn("goblin_shaman"),
    );

    // --- Derelict Buildings & Settlements ---
    map.add_location(
        Location::new(
            "derelict_mill",
            "Derelict Mill",
            "An old water-mill, long since stopped. The great wheel hangs \
             cracked and still over a dry millrace. Inside, the floor has \
             rotted through in places and the roof beams groan ominously. \
             Someone — or something — has made a nest in the upper floor. \
             Old grinding stones and rusted implements litter the space.",
            RegionType::Ruins,
        )
        .with_exit(Exit::new("east", "valley_northwest", "East back out to the valley."))
        .with_exit(Exit::new("south", "abandoned_farmstead", "South along a crumbling field wall to the farmstead."))
        .with_enemy_spawn("bandit")
        .with_enemy_spawn("giant_bat"),
    );

    map.add_location(
        Location::new(
            "abandoned_farmstead",
            "Abandoned Farmstead",
            "A cluster of collapsed stone buildings that once formed a working \
             farm. The barn still stands, barely — its timbers black with age. \
             Rusted tools hang from hooks and a broken cart sits in the yard. \
             Bandits have camped here recently; the ashes of a fire are still \
             warm. A path leads west toward the mountain crags.",
            RegionType::Ruins,
        )
        .with_exit(Exit::new("north", "derelict_mill", "North toward the old mill."))
        .with_exit(Exit::new("east", "valley_west", "East back toward the valley meadows."))
        .with_exit(Exit::new("west", "west_mountain_crags", "West into the mountain crags."))
        .with_enemy_spawn("bandit")
        .with_enemy_spawn("bandit_chief"),
    );

    map.add_location(
        Location::new(
            "valley_watchtower",
            "Ruined Watchtower",
            "A squat stone watchtower — old even by valley standards — squats \
             on a low rise. Its top floor has collapsed inward but the ground \
             level still stands. Arrow slits look out over the northeastern \
             valley. Graffiti and old camp gear suggest it has been used as \
             shelter by travellers and bandits alike. From the rise, you can \
             see much of the valley spread below.",
            RegionType::Ruins,
        )
        .with_exit(Exit::new("west", "valley_northeast", "West back into the valley."))
        .with_enemy_spawn("bandit")
        .with_enemy_spawn("goblin_scout"),
    );

    map.add_location(
        Location::new(
            "millford_ruins",
            "Ruins of Millford",
            "The collapsed remains of Millford — a hamlet that predated Thornvale \
             by two centuries. Only foundation stones and the shells of a few \
             buildings remain. The cobbled central square is cracked and overgrown \
             with weeds. Local legend says Millford was abandoned after a plague \
             that turned its dead into something else. The entrance to an old crypt \
             lies beneath the ruined chapel at the hamlet's edge.",
            RegionType::Ruins,
        )
        .with_exit(Exit::new("northwest", "valley_south_meadow", "Northwest back toward the valley meadows."))
        .with_exit(Exit::new("down", "millford_crypt", "Down into the crypt beneath the ruined chapel."))
        .with_enemy_spawn("skeleton_warrior")
        .with_enemy_spawn("crypt_ghoul"),
    );

    // --- Crypts ---
    map.add_location(
        Location::new(
            "millford_crypt",
            "Millford Crypt",
            "A low, arched stone crypt beneath the ruins of Millford's chapel. \
             Tallow candles have burned to stubs on iron holders — someone was \
             here recently. Rows of stone sarcophagi line the walls, several \
             broken open from the inside. The smell of grave-dirt and decay is \
             thick. Bones scrape against stone in the darkness ahead.",
            RegionType::Crypt,
        )
        .with_exit(Exit::new("up", "millford_ruins", "Up and out of the crypt, back to the ruins."))
        .with_exit(Exit::new("north", "millford_crypt_depths", "Deeper into the crypt's lower passages."))
        .with_enemy_spawn("skeleton_warrior")
        .with_enemy_spawn("skeleton_archer")
        .with_enemy_spawn("crypt_ghoul"),
    );

    map.add_location(
        Location::new(
            "millford_crypt_depths",
            "Millford Crypt — Lower Passages",
            "The lower passages of the Millford crypt are older and cruder — \
             hand-carved into the bedrock long before the chapel was built. \
             Crude iron sconces hold guttering torches (recently lit — by whom?). \
             Niches in the walls hold mummified remains wrapped in rotting cloth. \
             At the far end, a larger chamber serves as an ossuary, floor to \
             ceiling with stacked bones. Something enormous moves among them.",
            RegionType::Crypt,
        )
        .with_exit(Exit::new("south", "millford_crypt", "South back to the upper crypt."))
        .with_enemy_spawn("crypt_ghoul")
        .with_enemy_spawn("skeleton_warrior")
        .with_enemy_spawn("wraith"),
    );

    // --- Ancient Barrow ---
    map.add_location(
        Location::new(
            "ancient_barrow_mound",
            "Ancient Barrow Mound",
            "A large earthen mound rises from the valley floor, roughly circular \
             and clearly artificial. Tall standing stones ring it at intervals, \
             carved with spiral designs that match nothing in living memory. A \
             low stone lintel marks the entrance to the barrow passage — a darkness \
             that smells of old stone and something older still. The mound has \
             clearly been disturbed recently: fresh tracks lead in and do not \
             come back out.",
            RegionType::Valley,
        )
        .with_exit(Exit::new("west", "valley_north", "West back to the northern valley."))
        .with_exit(Exit::new("in", "barrow_interior", "Into the barrow passage entrance."))
        .with_enemy_spawn("skeleton_warrior")
        .with_enemy_spawn("crypt_ghoul"),
    );

    map.add_location(
        Location::new(
            "barrow_interior",
            "Barrow Interior",
            "The barrow passage is tighter than it looked from outside — you \
             have to duck. Dressed stone walls glisten with moisture. Side niches \
             hold the grave goods of ancient warriors: corroded swords, shattered \
             pottery, crumbling bones. The passage opens into a small antechamber \
             where a stone altar stands. Deeper still, a sealed iron door \
             separates the antechamber from the burial lord's chamber.",
            RegionType::Crypt,
        )
        .with_exit(Exit::new("out", "ancient_barrow_mound", "Back out through the barrow passage."))
        .with_exit(Exit::new("north", "barrow_lord_chamber", "Through the iron door into the burial lord's chamber."))
        .with_enemy_spawn("skeleton_warrior")
        .with_enemy_spawn("skeleton_archer")
        .with_enemy_spawn("wraith"),
    );

    map.add_location(
        Location::new(
            "barrow_lord_chamber",
            "Barrow Lord's Chamber",
            "The main burial chamber of the barrow. A massive sarcophagus of \
             black granite dominates the centre, its lid carved in the likeness \
             of an iron-age chieftain in full war gear. Treasures of another age \
             surround it — tarnished gold, iron arms and armour, offerings long \
             since desiccated. The chieftain does not rest easily: a wraith of \
             terrible power rises from the sarcophagus, ancient hatred in its \
             hollow eyes.",
            RegionType::Tomb,
        )
        .with_exit(Exit::new("south", "barrow_interior", "South back to the barrow antechamber."))
        .with_enemy_spawn("wraith")
        .with_enemy_spawn("skeleton_warrior"),
    );

    // --- Valley King's Tomb ---
    map.add_location(
        Location::new(
            "valley_tomb_approach",
            "Valley King's Tomb — Approach",
            "A wide stone gateway, sealed with two massive stone slabs, rises \
             from the valley floor. Carved figures of warriors flank the door, \
             their stone eyes watching all who approach. The stonework is far \
             older and more sophisticated than anything Thornvale's people know \
             how to create. Moss fills the carvings. A cold draught emanates \
             from a crack between the slabs.",
            RegionType::Valley,
        )
        .with_exit(Exit::new("north", "valley_east", "North back to the eastern valley flats."))
        .with_exit(Exit::new("west", "south_mountain_foothills", "West toward the southern foothills."))
        .with_exit(Exit::new("in", "tomb_antechamber", "Through the cracked stone slabs into the tomb."))
        .with_enemy_spawn("skeleton_warrior")
        .with_enemy_spawn("crypt_ghoul"),
    );

    map.add_location(
        Location::new(
            "tomb_antechamber",
            "Valley King's Tomb — Antechamber",
            "The antechamber is vast — far larger than the entrance suggested. \
             Pillars of carved stone support a vaulted ceiling lost in shadow. \
             Faded frescoes depict a valley kingdom of great power: armies, \
             harvest, kingship — all ground to dust. The air is utterly still \
             and tainted with the smell of preserved death. Stone guardians \
             flank the inner doorway, and they are not merely decorative.",
            RegionType::Tomb,
        )
        .with_exit(Exit::new("out", "valley_tomb_approach", "Out through the stone slabs to the approach."))
        .with_exit(Exit::new("north", "tomb_sanctum", "Into the tomb's inner sanctum."))
        .with_enemy_spawn("skeleton_warrior")
        .with_enemy_spawn("skeleton_archer")
        .with_enemy_spawn("tomb_guardian"),
    );

    map.add_location(
        Location::new(
            "tomb_sanctum",
            "Valley King's Tomb — Sanctum",
            "The sanctum is the heart of the tomb — a circular chamber where the \
             Valley King himself was laid to rest an age ago. His golden sarcophagus \
             stands atop a raised dais, surrounded by the mummified remains of his \
             honour guard, still upright in their decayed armour. Something ancient \
             and terrible has claimed this place as its own: a tomb guardian of \
             immense power stands sentinel, its stone flesh cracked but unyielding, \
             its eyes burning with eldritch light.",
            RegionType::Tomb,
        )
        .with_exit(Exit::new("south", "tomb_antechamber", "South back to the antechamber."))
        .with_enemy_spawn("tomb_guardian")
        .with_enemy_spawn("wraith"),
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

    #[test]
    fn test_valley_biome_properties() {
        assert_eq!(RegionType::Valley.encounter_difficulty(), 2);
        assert_eq!(RegionType::Crypt.encounter_difficulty(), 3);
        assert_eq!(RegionType::Tomb.encounter_difficulty(), 4);
    }

    #[test]
    fn test_valley_world_is_large() {
        let world = build_starting_world();
        // The valley world should have many locations
        assert!(world.locations.len() >= 30, "Expected at least 30 locations, got {}", world.locations.len());
    }

    #[test]
    fn test_valley_locations_exist() {
        let world = build_starting_world();
        let valley_locations = [
            "valley_north", "valley_northwest", "valley_northeast",
            "valley_west", "valley_east", "valley_south_meadow",
        ];
        for loc_id in &valley_locations {
            assert!(world.locations.contains_key(*loc_id), "Missing valley location: {}", loc_id);
        }
    }

    #[test]
    fn test_caves_exist() {
        let world = build_starting_world();
        let cave_ids = ["crystal_cave_entrance", "crystal_cave_depths", "shadow_cave_entrance", "shadow_cave_depths", "shadow_gorge"];
        for id in &cave_ids {
            assert!(world.locations.contains_key(*id), "Missing cave: {}", id);
        }
    }

    #[test]
    fn test_crypts_and_tombs_exist() {
        let world = build_starting_world();
        let crypt_ids = ["millford_crypt", "millford_crypt_depths", "barrow_interior", "barrow_lord_chamber", "tomb_antechamber", "tomb_sanctum"];
        for id in &crypt_ids {
            assert!(world.locations.contains_key(*id), "Missing crypt/tomb: {}", id);
        }
    }

    #[test]
    fn test_derelict_buildings_exist() {
        let world = build_starting_world();
        let derelict_ids = ["derelict_mill", "abandoned_farmstead", "valley_watchtower", "millford_ruins"];
        for id in &derelict_ids {
            assert!(world.locations.contains_key(*id), "Missing derelict building: {}", id);
        }
    }

    #[test]
    fn test_valley_locations_are_dangerous() {
        let world = build_starting_world();
        let valley = world.locations.get("valley_north").unwrap();
        assert!(!valley.is_safe);
        assert!(valley.region_type.encounter_difficulty() > 0);
    }

    #[test]
    fn test_mountain_regions_exist() {
        let world = build_starting_world();
        assert!(world.locations.contains_key("north_mountain_pass"));
        assert!(world.locations.contains_key("west_mountain_crags"));
        let pass = world.locations.get("north_mountain_pass").unwrap();
        assert_eq!(pass.region_type, RegionType::Mountains);
    }

    #[test]
    fn test_thornvale_connects_to_valley() {
        let world = build_starting_world();
        let gate = world.locations.get("thornvale_gate").unwrap();
        let has_valley_exit = gate.exits.iter()
            .any(|e| e.destination_id == "valley_northwest" || e.destination_id == "valley_northeast");
        assert!(has_valley_exit, "thornvale_gate should connect to the valley");
    }
}
