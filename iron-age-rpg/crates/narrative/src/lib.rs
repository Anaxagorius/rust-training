use std::collections::HashMap;
use iron_age_core::GameError;

// ── Quest system ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum QuestStatus {
    NotStarted,
    Active,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ObjectiveKind {
    KillEnemy { enemy_id: String, required: u32, current: u32 },
    CollectItem { item_id: String, required: u32, current: u32 },
    ReachLocation { location_id: String, reached: bool },
    TalkToNpc { npc_id: String, talked: bool },
    SurviveRounds { rounds: u32, survived: u32 },
}

impl ObjectiveKind {
    pub fn is_complete(&self) -> bool {
        match self {
            Self::KillEnemy { required, current, .. } => current >= required,
            Self::CollectItem { required, current, .. } => current >= required,
            Self::ReachLocation { reached, .. } => *reached,
            Self::TalkToNpc { talked, .. } => *talked,
            Self::SurviveRounds { rounds, survived } => survived >= rounds,
        }
    }

    pub fn progress_text(&self) -> String {
        match self {
            Self::KillEnemy { enemy_id, required, current } =>
                format!("Kill {} ({}/{})", enemy_id, current, required),
            Self::CollectItem { item_id, required, current } =>
                format!("Collect {} ({}/{})", item_id, current, required),
            Self::ReachLocation { location_id, reached } =>
                format!("Reach {} ({})", location_id, if *reached { "done" } else { "pending" }),
            Self::TalkToNpc { npc_id, talked } =>
                format!("Talk to {} ({})", npc_id, if *talked { "done" } else { "pending" }),
            Self::SurviveRounds { rounds, survived } =>
                format!("Survive {} rounds ({}/{})", rounds, survived, rounds),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuestObjective {
    pub description: String,
    pub kind: ObjectiveKind,
}

impl QuestObjective {
    pub fn is_complete(&self) -> bool {
        self.kind.is_complete()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuestReward {
    pub experience: u64,
    pub gold: u32,
    pub item_ids: Vec<String>,
}

impl QuestReward {
    pub fn new(experience: u64, gold: u32) -> Self {
        Self { experience, gold, item_ids: Vec::new() }
    }

    pub fn with_item(mut self, item_id: &str) -> Self {
        self.item_ids.push(item_id.to_string());
        self
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Quest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub giver_npc_id: String,
    pub objectives: Vec<QuestObjective>,
    pub reward: QuestReward,
    pub status: QuestStatus,
    pub prerequisite_quest_ids: Vec<String>,
}

impl Quest {
    pub fn new(
        id: &str, name: &str, description: &str, giver_npc_id: &str,
        objectives: Vec<QuestObjective>, reward: QuestReward,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            giver_npc_id: giver_npc_id.to_string(),
            objectives,
            reward,
            status: QuestStatus::NotStarted,
            prerequisite_quest_ids: Vec::new(),
        }
    }

    pub fn with_prerequisite(mut self, quest_id: &str) -> Self {
        self.prerequisite_quest_ids.push(quest_id.to_string());
        self
    }

    pub fn all_objectives_complete(&self) -> bool {
        self.objectives.iter().all(|o| o.is_complete())
    }

    pub fn is_active(&self) -> bool {
        self.status == QuestStatus::Active
    }

    pub fn start(&mut self) -> Result<(), GameError> {
        if self.status != QuestStatus::NotStarted {
            return Err(GameError::InvalidOperation(
                format!("Quest '{}' is already {:?}", self.name, self.status)
            ));
        }
        self.status = QuestStatus::Active;
        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), GameError> {
        if !self.all_objectives_complete() {
            return Err(GameError::InvalidOperation(
                "Not all objectives are complete".to_string()
            ));
        }
        self.status = QuestStatus::Completed;
        Ok(())
    }

    /// Record a kill event and return true if any objective was advanced.
    pub fn on_kill(&mut self, enemy_id: &str) -> bool {
        let mut advanced = false;
        for obj in &mut self.objectives {
            if let ObjectiveKind::KillEnemy { enemy_id: eid, required, current } = &mut obj.kind {
                if eid == enemy_id && current < required {
                    *current += 1;
                    advanced = true;
                }
            }
        }
        advanced
    }

    /// Record an item collection and return true if any objective was advanced.
    pub fn on_collect(&mut self, item_id: &str, quantity: u32) -> bool {
        let mut advanced = false;
        for obj in &mut self.objectives {
            if let ObjectiveKind::CollectItem { item_id: iid, required, current } = &mut obj.kind {
                if iid == item_id {
                    *current = (*current + quantity).min(*required);
                    advanced = true;
                }
            }
        }
        advanced
    }

    /// Record reaching a location and return true if any objective was advanced.
    pub fn on_reach_location(&mut self, location_id: &str) -> bool {
        let mut advanced = false;
        for obj in &mut self.objectives {
            if let ObjectiveKind::ReachLocation { location_id: lid, reached } = &mut obj.kind {
                if lid == location_id && !*reached {
                    *reached = true;
                    advanced = true;
                }
            }
        }
        advanced
    }

    /// Record talking to an NPC and return true if any objective was advanced.
    pub fn on_talk(&mut self, npc_id: &str) -> bool {
        let mut advanced = false;
        for obj in &mut self.objectives {
            if let ObjectiveKind::TalkToNpc { npc_id: nid, talked } = &mut obj.kind {
                if nid == npc_id && !*talked {
                    *talked = true;
                    advanced = true;
                }
            }
        }
        advanced
    }
}

// ── NPC & Dialogue ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum NpcRole {
    QuestGiver,
    Merchant,
    Blacksmith,
    Innkeeper,
    Guard,
    Civilian,
    Companion,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DialogueLine {
    pub speaker_id: String,
    pub text: String,
    /// If `Some`, this line is only shown when the named quest is in the given status.
    pub quest_condition: Option<(String, QuestStatus)>,
}

impl DialogueLine {
    pub fn new(speaker_id: &str, text: &str) -> Self {
        Self {
            speaker_id: speaker_id.to_string(),
            text: text.to_string(),
            quest_condition: None,
        }
    }

    pub fn when_quest(mut self, quest_id: &str, status: QuestStatus) -> Self {
        self.quest_condition = Some((quest_id.to_string(), status));
        self
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Npc {
    pub id: String,
    pub name: String,
    pub role: NpcRole,
    pub greeting: String,
    pub dialogue_lines: Vec<DialogueLine>,
    pub quest_ids: Vec<String>,
    pub shop_item_ids: Vec<String>,
}

impl Npc {
    pub fn new(id: &str, name: &str, role: NpcRole, greeting: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            role,
            greeting: greeting.to_string(),
            dialogue_lines: Vec::new(),
            quest_ids: Vec::new(),
            shop_item_ids: Vec::new(),
        }
    }

    pub fn with_dialogue(mut self, line: DialogueLine) -> Self {
        self.dialogue_lines.push(line);
        self
    }

    pub fn with_quest(mut self, quest_id: &str) -> Self {
        self.quest_ids.push(quest_id.to_string());
        self
    }

    pub fn with_shop_item(mut self, item_id: &str) -> Self {
        self.shop_item_ids.push(item_id.to_string());
        self
    }

    /// Return lines valid given current active quest ids.
    pub fn available_lines<'a>(&'a self, active_quests: &[String]) -> Vec<&'a DialogueLine> {
        self.dialogue_lines.iter().filter(|line| {
            match &line.quest_condition {
                None => true,
                Some((qid, status)) => {
                    let quest_active = active_quests.contains(qid);
                    match status {
                        QuestStatus::Active => quest_active,
                        QuestStatus::NotStarted => !quest_active,
                        _ => false,
                    }
                }
            }
        }).collect()
    }
}

// ── Quest Log ─────────────────────────────────────────────────────────────────

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct QuestLog {
    pub quests: HashMap<String, Quest>,
}

impl QuestLog {
    pub fn new() -> Self {
        Self { quests: HashMap::new() }
    }

    pub fn register(&mut self, quest: Quest) {
        self.quests.insert(quest.id.clone(), quest);
    }

    pub fn start_quest(&mut self, quest_id: &str, completed_quests: &[String]) -> Result<(), GameError> {
        let quest = self.quests.get(quest_id)
            .ok_or_else(|| GameError::NotFound(quest_id.to_string()))?;
        for prereq in &quest.prerequisite_quest_ids {
            if !completed_quests.contains(prereq) {
                return Err(GameError::MissingRequirements(
                    format!("Must complete '{}' first", prereq)
                ));
            }
        }
        self.quests.get_mut(quest_id).unwrap().start()
    }

    pub fn active_quests(&self) -> Vec<&Quest> {
        self.quests.values().filter(|q| q.status == QuestStatus::Active).collect()
    }

    pub fn completed_quests(&self) -> Vec<&Quest> {
        self.quests.values().filter(|q| q.status == QuestStatus::Completed).collect()
    }

    pub fn completed_quest_ids(&self) -> Vec<String> {
        self.completed_quests().iter().map(|q| q.id.clone()).collect()
    }

    pub fn on_kill(&mut self, enemy_id: &str) -> Vec<String> {
        let mut messages = Vec::new();
        for quest in self.quests.values_mut().filter(|q| q.is_active()) {
            if quest.on_kill(enemy_id) {
                messages.push(format!("[Quest: {}] Updated.", quest.name));
                if quest.all_objectives_complete() {
                    messages.push(format!("[Quest: {}] All objectives complete! Return to {}.",
                        quest.name, quest.giver_npc_id));
                }
            }
        }
        messages
    }

    pub fn on_collect(&mut self, item_id: &str, quantity: u32) -> Vec<String> {
        let mut messages = Vec::new();
        for quest in self.quests.values_mut().filter(|q| q.is_active()) {
            if quest.on_collect(item_id, quantity) {
                messages.push(format!("[Quest: {}] Updated.", quest.name));
            }
        }
        messages
    }

    pub fn on_reach_location(&mut self, location_id: &str) -> Vec<String> {
        let mut messages = Vec::new();
        for quest in self.quests.values_mut().filter(|q| q.is_active()) {
            if quest.on_reach_location(location_id) {
                messages.push(format!("[Quest: {}] Updated.", quest.name));
            }
        }
        messages
    }

    pub fn on_talk(&mut self, npc_id: &str) -> Vec<String> {
        let mut messages = Vec::new();
        for quest in self.quests.values_mut().filter(|q| q.is_active()) {
            if quest.on_talk(npc_id) {
                messages.push(format!("[Quest: {}] Updated.", quest.name));
            }
        }
        messages
    }

    pub fn try_complete_quest(&mut self, quest_id: &str) -> Result<QuestReward, GameError> {
        let quest = self.quests.get_mut(quest_id)
            .ok_or_else(|| GameError::NotFound(quest_id.to_string()))?;
        quest.complete()?;
        Ok(quest.reward.clone())
    }
}

// ── NPC Registry ──────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct NpcRegistry {
    pub npcs: HashMap<String, Npc>,
}

impl NpcRegistry {
    pub fn new() -> Self { Self { npcs: HashMap::new() } }

    pub fn register(&mut self, npc: Npc) {
        self.npcs.insert(npc.id.clone(), npc);
    }

    pub fn get(&self, id: &str) -> Option<&Npc> {
        self.npcs.get(id)
    }
}

/// Build the default NPC registry and quest log for the starting world.
pub fn build_narrative() -> (NpcRegistry, QuestLog) {
    let mut npcs = NpcRegistry::new();
    let mut quest_log = QuestLog::new();

    // ── NPCs ──────────────────────────────────────────────────────────────────

    npcs.register(
        Npc::new(
            "elder_aldric", "Elder Aldric", NpcRole::QuestGiver,
            "Ah, a capable-looking sort. We are in desperate need of your skills, traveller.",
        )
        .with_dialogue(DialogueLine::new(
            "elder_aldric",
            "The goblins from Ironmere Keep have been raiding our farms. \
             We need someone to drive them back. Will you help us?"
        ).when_quest("drive_back_goblins", QuestStatus::NotStarted))
        .with_dialogue(DialogueLine::new(
            "elder_aldric",
            "Thank the gods you are out there fighting. Keep at it — drive \
             them from the courtyard and find the iron key to reach their warlord."
        ).when_quest("drive_back_goblins", QuestStatus::Active))
        .with_quest("drive_back_goblins")
        .with_quest("clear_wolf_den")
        .with_quest("valley_explorer")
        .with_quest("bandit_camp_raid")
        .with_quest("bog_witch_queen")
        .with_quest("ironmere_free_captives")
        .with_quest("ironmere_dungeon_clear")
        .with_quest("barrow_high_knight")
        .with_quest("tomb_skeleton_hunt")
        .with_quest("tomb_mummy_hunt"),
    );

    npcs.register(
        Npc::new(
            "town_crier", "Town Crier", NpcRole::Civilian,
            "Hear ye! Goblin raiders spotted on the King's Road!",
        )
        .with_dialogue(DialogueLine::new(
            "town_crier",
            "Goblins have taken Ironmere Keep! Elder Aldric seeks a champion!"
        )),
    );

    npcs.register(
        Npc::new(
            "blacksmith_grund", "Grund the Blacksmith", NpcRole::Blacksmith,
            "Need a weapon or some armour? You've come to the right place.",
        )
        .with_dialogue(DialogueLine::new(
            "blacksmith_grund",
            "I can forge you a sword or some armour if you bring the materials. \
             Iron ingots and leather are what you'll need."
        ))
        .with_dialogue(DialogueLine::new(
            "blacksmith_grund",
            "The forge is yours to use. Just don't melt anything expensive."
        ))
        .with_shop_item("iron_ingot")
        .with_shop_item("leather")
        .with_shop_item("wood_shaft")
        .with_quest("gather_iron")
        .with_quest("ashwood_lumber_run"),
    );

    npcs.register(
        Npc::new(
            "merchant_serah", "Merchant Serah", NpcRole::Merchant,
            "Fresh supplies, fair prices! What can I get for you?",
        )
        .with_shop_item("health_potion")
        .with_shop_item("stamina_potion")
        .with_shop_item("antidote")
        .with_shop_item("clarity_potion")
        .with_shop_item("iron_ingot")
        .with_shop_item("leather")
        .with_shop_item("herbs")
        .with_shop_item("clean_water")
        .with_quest("thornvale_market_supply")
        .with_quest("ironmere_warlord_hoard"),
    );

    npcs.register(
        Npc::new(
            "innkeeper_marta", "Marta the Innkeeper", NpcRole::Innkeeper,
            "Welcome to the Rusted Helm! Rest your boots — a bed and a meal \
             will set you right.",
        )
        .with_dialogue(DialogueLine::new(
            "innkeeper_marta",
            "The inn is safe. Sleep here to fully restore your health and stamina."
        ))
        .with_quest("thornvale_herbalist")
        .with_quest("ashwood_spider_hunt"),
    );

    npcs.register(
        Npc::new(
            "guard_torven", "Guard Torven", NpcRole::Guard,
            "Halt. State your business... actually, go on through. We need \
             all the help we can get right now.",
        )
        .with_dialogue(DialogueLine::new(
            "guard_torven",
            "Watch yourself on the King's Road. There are wolf packs and \
             goblin scouts between here and the forest."
        ))
        .with_quest("ashwood_forest_patrol")
        .with_quest("valley_explorer")
        .with_quest("bandit_camp_raid")
        .with_quest("valley_watchtower_survey")
        .with_quest("crystal_cave_bat_nest")
        .with_quest("ironmere_archer_hunt"),
    );

    // NPCs outside Thornvale

    npcs.register(
        Npc::new(
            "hermit_bogdan", "Hermit Bogdan", NpcRole::Civilian,
            "Not many venture into the Bog-heart willingly. You've got courage, \
             or you're a fool. Perhaps both.",
        )
        .with_dialogue(DialogueLine::new(
            "hermit_bogdan",
            "The Crystal Cave to the north holds crystal shards of great power. \
             Crystalline dust can be brewed into potions at my alchemy stone — feel \
             free to use it. In return, I ask only for herbs when you find them."
        ))
        .with_dialogue(DialogueLine::new(
            "hermit_bogdan",
            "The shadow gorge to the north-east is stalked by cave trolls. \
             Iron weapons work best; they hate bright torchlight too.",
        ).when_quest("shadow_cave_delve", QuestStatus::Active))
        .with_shop_item("antidote")
        .with_shop_item("herbs")
        .with_shop_item("bog_moss")
        .with_shop_item("nightshade_leaf")
        .with_quest("shadow_cave_delve")
        .with_quest("bog_moss_harvest")
        .with_quest("bog_pest_control")
        .with_quest("bog_witch_warning")
        .with_quest("shadow_hidden_chamber")
        .with_quest("crystal_dust_harvest"),
    );

    npcs.register(
        Npc::new(
            "ranger_vex", "Ranger Vex", NpcRole::Guard,
            "I patrol these mountain passes alone. It's dangerous work, but \
             someone has to keep the road clear.",
        )
        .with_dialogue(DialogueLine::new(
            "ranger_vex",
            "The Crystal Cave glitters with promise but those golems don't \
             take kindly to trespassers. If you're brave enough to clear them out, \
             I'll make it worth your while."
        ))
        .with_dialogue(DialogueLine::new(
            "ranger_vex",
            "Good progress in there. The golems are weakest against blunt weapons — \
             their crystal shells shatter rather than flex.",
        ).when_quest("crystal_cave_clear", QuestStatus::Active))
        .with_quest("crystal_cave_clear")
        .with_quest("ashwood_ancient_grove_discovery")
        .with_quest("ashwood_treant_lord")
        .with_quest("crystal_cave_bear_trophy")
        .with_quest("crystal_cave_depths_expedition")
        .with_quest("crystal_elemental_boss"),
    );

    npcs.register(
        Npc::new(
            "scholar_lyria", "Scholar Lyria", NpcRole::QuestGiver,
            "Careful with those old stones! I've been cataloguing these ruins \
             for months. You wouldn't believe what lies beneath.",
        )
        .with_dialogue(DialogueLine::new(
            "scholar_lyria",
            "The Ancient Barrow to the north-east predates Thornvale by a thousand \
             years. Barrow knights still guard it — but the burial lord's chamber \
             holds artefacts of immeasurable historical value. Could you retrieve \
             the Barrow Lord's Helm for me? It would complete my research."
        ))
        .with_dialogue(DialogueLine::new(
            "scholar_lyria",
            "The Valley King's Tomb to the south-east is even older than the barrow. \
             Legends say the Valley King's Crown was buried with him. Whoever retrieves \
             it would be hailed as a true champion of Embervale.",
        ).when_quest("barrow_research", QuestStatus::Active))
        .with_dialogue(DialogueLine::new(
            "scholar_lyria",
            "You've found the Barrow Lord's Helm! Extraordinary. Now I wonder — \
             dare you venture into the Valley King's Tomb? The crown must be there.",
        ).when_quest("valley_king_tomb", QuestStatus::NotStarted))
        .with_quest("barrow_research")
        .with_quest("valley_king_tomb")
        .with_quest("ashwood_ancient_grove_discovery")
        .with_quest("barrow_wraith_hunt")
        .with_quest("barrow_coin_collection")
        .with_quest("barrow_high_knight")
        .with_quest("tomb_inscription_research")
        .with_quest("tomb_spectral_cleansing")
        .with_quest("temple_serpent_purge")
        .with_quest("temple_guardian_rites")
        .with_quest("idol_construct_smash")
        .with_quest("temple_relic_recovery")
        .with_quest("temple_coin_recovery"),
    );

    // ── New Zone NPCs ─────────────────────────────────────────────────────────

    npcs.register(
        Npc::new(
            "fisherman_aldric", "Fisherman Aldric", NpcRole::QuestGiver,
            "Ay, the sea's not been right lately. Wraiths in the mist, serpents in the shallows. \
             You look like someone who can handle themselves.",
        )
        .with_dialogue(DialogueLine::new(
            "fisherman_aldric",
            "Those coastal raiders have blockaded our fishing grounds. They must be driven off — \
             follow the shore and find their camp in the old sea-fort."
        ))
        .with_dialogue(DialogueLine::new(
            "fisherman_aldric",
            "The tide wraiths... they appear when the old temple stirs. Something down there \
             is waking up. The sunken temple beneath the cliff has been sealed for good reason.",
        ).when_quest("tide_wraith_hunt", QuestStatus::Active))
        .with_shop_item("sea_salt")
        .with_shop_item("driftwood")
        .with_shop_item("antidote")
        .with_quest("coastal_raiders_clear")
        .with_quest("tide_wraith_hunt")
        .with_quest("sea_serpent_hunt")
        .with_quest("sunken_temple_delve")
        .with_quest("saltstone_crab_hunt"),
    );

    npcs.register(
        Npc::new(
            "mining_overseer", "Overseer Maren", NpcRole::QuestGiver,
            "The quarry was ours before those brigands took it. We need someone \
             brave enough to take it back.",
        )
        .with_dialogue(DialogueLine::new(
            "mining_overseer",
            "Clear the quarry bandits from the entrance first, then deal with whatever \
             is deeper down. Iron is desperately needed in Thornvale."
        ))
        .with_dialogue(DialogueLine::new(
            "mining_overseer",
            "Mine crawlers are the worst — they burrow through the tunnels and collapse \
             the shoring timbers. Kill as many as you can.",
        ).when_quest("quarry_mine_crawlers", QuestStatus::Active))
        .with_shop_item("iron_ore")
        .with_shop_item("iron_ingot")
        .with_shop_item("black_iron_ingot")
        .with_quest("quarry_bandit_rout")
        .with_quest("quarry_mine_crawlers")
        .with_quest("black_iron_harvest")
        .with_quest("iron_golem_destroy")
        .with_quest("quarry_ore_delivery"),
    );

    npcs.register(
        Npc::new(
            "frost_sage", "Frost Sage Erindel", NpcRole::QuestGiver,
            "Few make it to the summit. Fewer still survive what dwells here. \
             You must be exceptional — or foolhardy.",
        )
        .with_dialogue(DialogueLine::new(
            "frost_sage",
            "The glacial wraith has haunted this peak for a century, \
             feeding on the life-force of climbers. If you can banish it, \
             the summit will be safe again."
        ))
        .with_dialogue(DialogueLine::new(
            "frost_sage",
            "Ice trolls have multiplied this season. Drive them from the approach \
             so that the passage to the summit is safe.",
        ).when_quest("ice_troll_cull", QuestStatus::Active))
        .with_shop_item("frost_crystal")
        .with_shop_item("glacier_shard")
        .with_shop_item("clarity_potion")
        .with_quest("glacial_wraith_banish")
        .with_quest("ice_troll_cull")
        .with_quest("frost_wolf_pelts")
        .with_quest("frostpeak_summit_reach")
        .with_quest("frozen_eagle_hunt"),
    );

    npcs.register(
        Npc::new(
            "merchant_aldis", "Merchant Aldis", NpcRole::Merchant,
            "Welcome to Merchant's Crossing! Best prices on the river, I guarantee it.",
        )
        .with_dialogue(DialogueLine::new(
            "merchant_aldis",
            "We see all sorts passing through. Travellers, traders, adventurers. \
             You want supplies? We have them. Want to make some coin? Try the dice table \
             — Brom runs an honest game. Mostly."
        ))
        .with_shop_item("health_potion")
        .with_shop_item("stamina_potion")
        .with_shop_item("antidote")
        .with_shop_item("iron_ingot")
        .with_shop_item("leather")
        .with_shop_item("herbs")
        .with_shop_item("clean_water")
        .with_shop_item("driftwood")
        .with_shop_item("sea_salt"),
    );

    npcs.register(
        Npc::new(
            "dice_master_brom", "Dice Master Brom", NpcRole::Civilian,
            "Want to play a hand of dice? Put up some gold and we'll see \
             who Lady Luck favours today!",
        )
        .with_dialogue(DialogueLine::new(
            "dice_master_brom",
            "The rules are simple: we each roll two dice and add them up. \
             Highest total wins the pot. Type 'dice <bet>' to play."
        ))
        .with_quest("dice_champion")
        .with_quest("reach_merchants_crossing")
        .with_quest("merchant_goods_delivery"),
    );

    npcs.register(
        Npc::new(
            "barge_master_finn", "Barge Master Finn", NpcRole::QuestGiver,
            "Those river serpents are bad for business. I've lost two good deckhands \
             this season already.",
        )
        .with_dialogue(DialogueLine::new(
            "barge_master_finn",
            "Kill enough river serpents and harpies around the docks and I'll \
             make it worth your while. Salvage trade needs those waters clear."
        ))
        .with_shop_item("driftwood")
        .with_shop_item("sea_salt")
        .with_shop_item("serpent_scale")
        .with_shop_item("harpy_feather")
        .with_quest("river_dock_clear")
        .with_quest("harpy_feather_collect"),
    );

    // ── Quests ────────────────────────────────────────────────────────────────

    // Quest 1: Drive back the goblins (main quest)
    quest_log.register(Quest::new(
        "drive_back_goblins",
        "Drive Back the Raiders",
        "The village elder Aldric has tasked you with dealing with the goblin \
         raiders who have taken Ironmere Keep. Clear the courtyard of goblins, \
         find the iron key dropped by their shaman, and defeat the warlord \
         in the tower.",
        "elder_aldric",
        vec![
            QuestObjective {
                description: "Defeat goblin warriors in the courtyard (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "goblin_warrior".to_string(), required: 3, current: 0,
                },
            },
            QuestObjective {
                description: "Defeat the goblin shaman".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "goblin_shaman".to_string(), required: 1, current: 0,
                },
            },
            QuestObjective {
                description: "Collect the iron key from the shaman".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "iron_key".to_string(), required: 1, current: 0,
                },
            },
            QuestObjective {
                description: "Reach the keep tower".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "ironmere_tower".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Defeat the goblin warlord".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "goblin_warlord".to_string(), required: 1, current: 0,
                },
            },
        ],
        QuestReward::new(500, 50).with_item("iron_long_sword"),
    ));

    // Quest 2: Gather iron (crafting side quest)
    quest_log.register(Quest::new(
        "gather_iron",
        "Iron for Grund",
        "Blacksmith Grund needs iron ingots to restock his forge. \
         Search the ruins or fell enemies who carry ore.",
        "blacksmith_grund",
        vec![
            QuestObjective {
                description: "Collect iron ingots (5)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "iron_ingot".to_string(), required: 5, current: 0,
                },
            },
        ],
        QuestReward::new(150, 20).with_item("leather_torso"),
    ));

    // Quest 3: Clear the wolf den
    quest_log.register(Quest::new(
        "clear_wolf_den",
        "The Wolf Menace",
        "Wolves from the Ashwood Forest have been attacking livestock. \
         Track them to their den and deal with the alpha.",
        "elder_aldric",
        vec![
            QuestObjective {
                description: "Kill wolves in Ashwood Forest (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "wolf".to_string(), required: 3, current: 0,
                },
            },
            QuestObjective {
                description: "Reach the wolf den".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "wolf_den_lair".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Defeat the dire wolf alpha".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "dire_wolf_alpha".to_string(), required: 1, current: 0,
                },
            },
        ],
        QuestReward::new(300, 30).with_item("wolf_pelt_armor"),
    ));

    // Quest 4: Crystal Cave — clear the golems
    quest_log.register(Quest::new(
        "crystal_cave_clear",
        "Shards of Light",
        "Ranger Vex has asked you to clear the crystal golems from the Crystal \
         Cave so that prospectors can safely harvest the crystal shards within.",
        "ranger_vex",
        vec![
            QuestObjective {
                description: "Reach the Crystal Cave entrance".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "crystal_cave_entrance".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Defeat crystal golems (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "crystal_golem".to_string(), required: 3, current: 0,
                },
            },
            QuestObjective {
                description: "Reach the Crystal Cave depths".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "crystal_cave_depths".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(350, 40).with_item("crystal_ring"),
    ));

    // Quest 5: Shadow Cave — cave trolls
    quest_log.register(Quest::new(
        "shadow_cave_delve",
        "Darkness Below",
        "Hermit Bogdan has warned you about cave trolls in the Shadow Gorge. \
         Venture into the shadow cave and eliminate the troll threat.",
        "hermit_bogdan",
        vec![
            QuestObjective {
                description: "Enter the shadow gorge".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "shadow_gorge".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Defeat cave trolls (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "cave_troll".to_string(), required: 3, current: 0,
                },
            },
            QuestObjective {
                description: "Reach the shadow cave depths".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "shadow_cave_depths".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(300, 35).with_item("ancient_amulet"),
    ));

    // Quest 6: Barrow — scholar's research
    quest_log.register(Quest::new(
        "barrow_research",
        "Secrets of the Barrow",
        "Scholar Lyria needs someone to retrieve the Barrow Lord's Helm from \
         the ancient barrow to the north-east. The interior is guarded by \
         barrow knights and worse. Retrieve the helm and return it to Lyria \
         in Millford Ruins.",
        "scholar_lyria",
        vec![
            QuestObjective {
                description: "Reach the barrow interior".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "barrow_interior".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Defeat barrow knights (2)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "barrow_knight".to_string(), required: 2, current: 0,
                },
            },
            QuestObjective {
                description: "Collect the Barrow Lord's Helm".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "barrow_lord_helm".to_string(), required: 1, current: 0,
                },
            },
            QuestObjective {
                description: "Return to Scholar Lyria".to_string(),
                kind: ObjectiveKind::TalkToNpc {
                    npc_id: "scholar_lyria".to_string(), talked: false,
                },
            },
        ],
        QuestReward::new(450, 60).with_item("runic_short_sword"),
    ));

    // Quest 7: Valley King's Tomb — the final challenge
    quest_log.register(
        Quest::new(
            "valley_king_tomb",
            "The Valley King's Legacy",
            "Scholar Lyria believes the legendary Valley King's Crown lies within \
             the Valley King's Tomb to the south-east. Navigate the antechamber \
             and sanctum, defeat the tomb guardian, and claim the crown.",
            "scholar_lyria",
            vec![
                QuestObjective {
                    description: "Reach the tomb antechamber".to_string(),
                    kind: ObjectiveKind::ReachLocation {
                        location_id: "tomb_antechamber".to_string(), reached: false,
                    },
                },
                QuestObjective {
                    description: "Defeat mummified guards (2)".to_string(),
                    kind: ObjectiveKind::KillEnemy {
                        enemy_id: "mummified_guard".to_string(), required: 2, current: 0,
                    },
                },
                QuestObjective {
                    description: "Reach the tomb sanctum".to_string(),
                    kind: ObjectiveKind::ReachLocation {
                        location_id: "tomb_sanctum".to_string(), reached: false,
                    },
                },
                QuestObjective {
                    description: "Defeat the tomb guardian".to_string(),
                    kind: ObjectiveKind::KillEnemy {
                        enemy_id: "tomb_guardian".to_string(), required: 1, current: 0,
                    },
                },
                QuestObjective {
                    description: "Collect the Valley King's Crown".to_string(),
                    kind: ObjectiveKind::CollectItem {
                        item_id: "valley_king_crown".to_string(), required: 1, current: 0,
                    },
                },
            ],
            QuestReward::new(1000, 150).with_item("ancient_amulet"),
        )
        .with_prerequisite("barrow_research"),
    );

    // ── Zone 1: Ember Coast Quests (4) ───────────────────────────────────────

    // Quest: Clear coastal raiders
    quest_log.register(Quest::new(
        "coastal_raiders_clear",
        "Drive Off the Raiders",
        "Fisherman Aldric has asked you to clear the coastal raiders from the Ember \
         Coast shore and ruins. They have blockaded the fishing grounds and must be \
         driven back into the sea.",
        "fisherman_aldric",
        vec![
            QuestObjective {
                description: "Reach the Ember Coast shore".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "ember_coast_shore".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Defeat coastal raiders (4)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "coastal_raider".to_string(), required: 4, current: 0,
                },
            },
            QuestObjective {
                description: "Reach the sea-fort ruins".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "ember_coast_ruins".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(320, 40).with_item("iron_short_sword"),
    ));

    // Quest: Hunt tide wraiths
    quest_log.register(Quest::new(
        "tide_wraith_hunt",
        "Banish the Tide Wraiths",
        "Fisherman Aldric warns that tide wraiths have appeared in the sea mist, \
         luring sailors to their doom. Venture to the Ember Coast and drive them off.",
        "fisherman_aldric",
        vec![
            QuestObjective {
                description: "Defeat tide wraiths (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "tide_wraith".to_string(), required: 3, current: 0,
                },
            },
        ],
        QuestReward::new(280, 35).with_item("antidote"),
    ));

    // Quest: Sea serpent hunt
    quest_log.register(Quest::new(
        "sea_serpent_hunt",
        "Scales of the Deep",
        "The sea serpents of the Ember Coast are attacking fishing boats. \
         Hunt them and collect their scales — a prize of considerable value.",
        "fisherman_aldric",
        vec![
            QuestObjective {
                description: "Defeat sea serpents (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "sea_serpent".to_string(), required: 3, current: 0,
                },
            },
            QuestObjective {
                description: "Collect serpent scales (3)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "serpent_scale".to_string(), required: 3, current: 0,
                },
            },
        ],
        QuestReward::new(350, 45).with_item("serpent_scale_armor"),
    ));

    // Quest: Sunken temple delve
    quest_log.register(Quest::new(
        "sunken_temple_delve",
        "Depths of the Sunken Temple",
        "The Sunken Temple beneath the Ember Coast cliff is stirring. \
         Venture inside, defeat the cursed acolytes and guardians, and \
         claim the Temple Seal Amulet from the sanctum altar.",
        "fisherman_aldric",
        vec![
            QuestObjective {
                description: "Reach the Sunken Temple entrance".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "sunken_temple_entrance".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Defeat cursed acolytes (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "cursed_acolyte".to_string(), required: 3, current: 0,
                },
            },
            QuestObjective {
                description: "Reach the Sunken Temple sanctum".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "sunken_temple_sanctum".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Defeat the Bone Colossus".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "bone_colossus".to_string(), required: 1, current: 0,
                },
            },
            QuestObjective {
                description: "Collect the Temple Seal Amulet".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "temple_seal_amulet".to_string(), required: 1, current: 0,
                },
            },
        ],
        QuestReward::new(900, 120).with_item("black_iron_sword"),
    ));

    // ── Zone 2: Iron Quarry Quests (4) ────────────────────────────────────────

    // Quest: Quarry bandit rout
    quest_log.register(Quest::new(
        "quarry_bandit_rout",
        "Reclaim the Quarry",
        "Overseer Maren needs the Iron Quarry cleared of brigands so that \
         legitimate mining can resume. Drive off the quarry bandits.",
        "mining_overseer",
        vec![
            QuestObjective {
                description: "Reach the Iron Quarry entrance".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "iron_quarry_entrance".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Defeat quarry bandits (4)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "quarry_bandit".to_string(), required: 4, current: 0,
                },
            },
        ],
        QuestReward::new(280, 30).with_item("iron_ingot"),
    ));

    // Quest: Mine crawler extermination
    quest_log.register(Quest::new(
        "quarry_mine_crawlers",
        "Infestation",
        "Mine crawlers have infested the lower quarry tunnels, collapsing shoring \
         timbers and making the depths impassable. Overseer Maren asks you to \
         exterminate them.",
        "mining_overseer",
        vec![
            QuestObjective {
                description: "Defeat mine crawlers (5)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "mine_crawler".to_string(), required: 5, current: 0,
                },
            },
            QuestObjective {
                description: "Reach the quarry depths".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "iron_quarry_depths".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(300, 35).with_item("iron_chainmail"),
    ));

    // Quest: Collect black iron
    quest_log.register(Quest::new(
        "black_iron_harvest",
        "The Black Seam",
        "Overseer Maren has heard rumours of a rich black iron seam deep in the quarry. \
         Descend to the depths and collect black iron ingots from the deepest tunnels.",
        "mining_overseer",
        vec![
            QuestObjective {
                description: "Collect black iron ingots (3)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "black_iron_ingot".to_string(), required: 3, current: 0,
                },
            },
        ],
        QuestReward::new(250, 40).with_item("iron_round_shield"),
    ));

    // Quest: Defeat the iron golem
    quest_log.register(Quest::new(
        "iron_golem_destroy",
        "The Iron Sentinel",
        "An iron golem in the quarry depths has gone rogue, attacking both the \
         bandits and the mine crawlers indiscriminately. Overseer Maren asks \
         you to destroy it before it collapses the entire quarry.",
        "mining_overseer",
        vec![
            QuestObjective {
                description: "Defeat the iron golem".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "iron_golem".to_string(), required: 1, current: 0,
                },
            },
            QuestObjective {
                description: "Reach the quarry depths".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "iron_quarry_depths".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(400, 55).with_item("black_iron_sword"),
    ));

    // ── Zone 3: Sunken Temple Quests (4) ─────────────────────────────────────
    // (Sunken temple quests already handled in Zone 1 coastal quests above,
    // so here we add temple-specific quests from the scholar)

    // Quest: Temple serpent purge
    quest_log.register(Quest::new(
        "temple_serpent_purge",
        "Serpents of the Deep",
        "The sunken temple is overrun with temple serpents. Scholar Lyria believes \
         they guard something important in the inner sanctum. Clear them out.",
        "scholar_lyria",
        vec![
            QuestObjective {
                description: "Defeat temple serpents (4)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "temple_serpent".to_string(), required: 4, current: 0,
                },
            },
        ],
        QuestReward::new(260, 30).with_item("antidote"),
    ));

    // Quest: Guardian spirits
    quest_log.register(Quest::new(
        "temple_guardian_rites",
        "Ward of the Ancients",
        "Scholar Lyria has identified temple guardian spirits bound to the \
         sunken temple by ancient rites. Defeat them to weaken the temple's \
         supernatural defences.",
        "scholar_lyria",
        vec![
            QuestObjective {
                description: "Defeat temple guardian spirits (2)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "temple_guardian_spirit".to_string(), required: 2, current: 0,
                },
            },
            QuestObjective {
                description: "Reach the temple sanctum".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "sunken_temple_sanctum".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(380, 50).with_item("temple_seal_amulet"),
    ));

    // Quest: Idol construct smash
    quest_log.register(Quest::new(
        "idol_construct_smash",
        "Shatter the Idols",
        "Idol constructs animated by the temple's residual power block access \
         to the inner sanctum. Destroy them to open the way.",
        "scholar_lyria",
        vec![
            QuestObjective {
                description: "Defeat idol constructs (2)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "idol_construct".to_string(), required: 2, current: 0,
                },
            },
        ],
        QuestReward::new(320, 40).with_item("ancient_amulet"),
    ));

    // Quest: Collect temple relics
    quest_log.register(Quest::new(
        "temple_relic_recovery",
        "Relics of the Deep Faith",
        "Scholar Lyria wants to catalogue the temple relics for academic study. \
         Collect relics from the sunken temple and return them to her.",
        "scholar_lyria",
        vec![
            QuestObjective {
                description: "Collect temple relics (4)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "temple_relic".to_string(), required: 4, current: 0,
                },
            },
            QuestObjective {
                description: "Return to Scholar Lyria".to_string(),
                kind: ObjectiveKind::TalkToNpc {
                    npc_id: "scholar_lyria".to_string(), talked: false,
                },
            },
        ],
        QuestReward::new(280, 35).with_item("clarity_potion"),
    ));

    // ── Zone 4: Frostpeak Quests (4) ─────────────────────────────────────────

    // Quest: Banish the glacial wraith
    quest_log.register(Quest::new(
        "glacial_wraith_banish",
        "The Eternal Cold",
        "Frost Sage Erindel has asked you to banish the glacial wraith that \
         haunts the Frostpeak summit. It has fed on climbers for a century \
         and must be put to rest.",
        "frost_sage",
        vec![
            QuestObjective {
                description: "Reach the Frostpeak summit".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "frostpeak_summit".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Defeat the glacial wraith".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "glacial_wraith".to_string(), required: 1, current: 0,
                },
            },
        ],
        QuestReward::new(420, 55).with_item("frostpeak_amulet"),
    ));

    // Quest: Ice troll cull
    quest_log.register(Quest::new(
        "ice_troll_cull",
        "Thinning the Pack",
        "Ice trolls have multiplied on the Frostpeak approach, making the path \
         to the summit impassable. Frost Sage Erindel asks you to cull their numbers.",
        "frost_sage",
        vec![
            QuestObjective {
                description: "Defeat ice trolls (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "ice_troll".to_string(), required: 3, current: 0,
                },
            },
        ],
        QuestReward::new(360, 45).with_item("frost_helm"),
    ));

    // Quest: Frost wolf pelts
    quest_log.register(Quest::new(
        "frost_wolf_pelts",
        "Winter's Fur",
        "Frost Sage Erindel needs frost wolf pelts to keep warm through the mountain \
         winter. Hunt the frost wolves on the approach.",
        "frost_sage",
        vec![
            QuestObjective {
                description: "Kill frost wolves (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "frost_wolf".to_string(), required: 3, current: 0,
                },
            },
            QuestObjective {
                description: "Collect wolf pelts (3)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "wolf_pelt".to_string(), required: 3, current: 0,
                },
            },
        ],
        QuestReward::new(240, 30).with_item("stamina_potion"),
    ));

    // Quest: Frostpeak summit reach
    quest_log.register(Quest::new(
        "frostpeak_summit_reach",
        "Peak of the World",
        "Frost Sage Erindel challenges you to reach the summit of Frostpeak and \
         return alive — a test of true mettle that few have passed.",
        "frost_sage",
        vec![
            QuestObjective {
                description: "Reach the Frostpeak approach".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "frostpeak_approach".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Reach the Frostpeak summit".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "frostpeak_summit".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(200, 20).with_item("glacier_shard"),
    ));

    // ── Zone 5: Merchant's Crossing Quests (4) ────────────────────────────────

    // Quest: Dice champion
    quest_log.register(Quest::new(
        "dice_champion",
        "Lucky Bones",
        "Dice Master Brom has challenged you to prove yourself at the dice table. \
         Win three games of dice in the populated areas of the valley.",
        "dice_master_brom",
        vec![
            QuestObjective {
                description: "Win dice games (3)".to_string(),
                kind: ObjectiveKind::SurviveRounds { rounds: 3, survived: 0 },
            },
        ],
        QuestReward::new(100, 25),
    ));

    // Quest: River dock clear
    quest_log.register(Quest::new(
        "river_dock_clear",
        "Clear the Docks",
        "Barge Master Finn needs the river docks cleared of river serpents and \
         harpies so that trading barges can operate safely again.",
        "barge_master_finn",
        vec![
            QuestObjective {
                description: "Defeat river serpents (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "river_serpent".to_string(), required: 3, current: 0,
                },
            },
            QuestObjective {
                description: "Defeat harpies (2)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "harpy".to_string(), required: 2, current: 0,
                },
            },
        ],
        QuestReward::new(220, 30).with_item("iron_short_sword"),
    ));

    // Quest: Collect harpy feathers
    quest_log.register(Quest::new(
        "harpy_feather_collect",
        "Fine Fletching",
        "Barge Master Finn trades in rare goods — harpy feathers fetch a good price \
         from fletchers. Collect them from the dock harpies.",
        "barge_master_finn",
        vec![
            QuestObjective {
                description: "Collect harpy feathers (5)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "harpy_feather".to_string(), required: 5, current: 0,
                },
            },
        ],
        QuestReward::new(150, 20).with_item("shortbow"),
    ));

    // Quest: Reach Merchant's Crossing
    quest_log.register(Quest::new(
        "reach_merchants_crossing",
        "Follow the River",
        "Merchant Aldis at Merchant's Crossing has goods to sell, but the road \
         there is long and not entirely safe. Simply reach the crossing to \
         unlock his wares.",
        "merchant_aldis",
        vec![
            QuestObjective {
                description: "Reach Merchant's Crossing".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "merchants_crossing".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(80, 10),
    ));

    // ── Thornvale Village Quests (total: 5+) ─────────────────────────────────

    // Quest: Marta's Remedy
    quest_log.register(Quest::new(
        "thornvale_herbalist",
        "Marta's Remedy",
        "Innkeeper Marta needs fresh herbs to brew medicine for sick villagers. \
         Venture into the valley or forest and gather what you can.",
        "innkeeper_marta",
        vec![
            QuestObjective {
                description: "Collect herbs (5)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "herbs".to_string(), required: 5, current: 0,
                },
            },
        ],
        QuestReward::new(120, 15).with_item("health_potion"),
    ));

    // Quest: Market Day
    quest_log.register(Quest::new(
        "thornvale_market_supply",
        "Market Day",
        "Merchant Serah needs to restock her market stall before the week's end. \
         Bring her leather for armour work and iron ingots for trade.",
        "merchant_serah",
        vec![
            QuestObjective {
                description: "Collect leather (4)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "leather".to_string(), required: 4, current: 0,
                },
            },
            QuestObjective {
                description: "Collect iron ingots (2)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "iron_ingot".to_string(), required: 2, current: 0,
                },
            },
        ],
        QuestReward::new(150, 25),
    ));

    // ── Ashwood Forest Quests (total: 6+) ────────────────────────────────────

    // Quest: Wood for the Forge
    quest_log.register(Quest::new(
        "ashwood_lumber_run",
        "Wood for the Forge",
        "Blacksmith Grund needs hardwood fuel for his forge. A good supply \
         of wood can be found in the Ashwood Forest — if you can survive long \
         enough to collect it.",
        "blacksmith_grund",
        vec![
            QuestObjective {
                description: "Reach the Ashwood Forest edge".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "ashwood_edge".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Collect wood (4)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "wood".to_string(), required: 4, current: 0,
                },
            },
        ],
        QuestReward::new(100, 15).with_item("wood_shaft"),
    ));

    // Quest: Spider Problem
    quest_log.register(Quest::new(
        "ashwood_spider_hunt",
        "Spider Problem",
        "Innkeeper Marta says forest spiders have been creeping closer to Thornvale, \
         frightening travellers on the roads. She wants them dealt with.",
        "innkeeper_marta",
        vec![
            QuestObjective {
                description: "Kill forest spiders (5)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "forest_spider".to_string(), required: 5, current: 0,
                },
            },
        ],
        QuestReward::new(200, 20).with_item("antidote"),
    ));

    // Quest: Forest Watch
    quest_log.register(Quest::new(
        "ashwood_forest_patrol",
        "Forest Watch",
        "Guard Torven has been asked to patrol the Ashwood Forest roads but \
         cannot leave his post at the gate. Patrol the forest on his behalf — \
         clear out the wolves and push through to the depths.",
        "guard_torven",
        vec![
            QuestObjective {
                description: "Kill wolves (5)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "wolf".to_string(), required: 5, current: 0,
                },
            },
            QuestObjective {
                description: "Reach the Ashwood depths".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "ashwood_depths".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(250, 25).with_item("shortbow"),
    ));

    // Quest: The Ancient Grove
    quest_log.register(Quest::new(
        "ashwood_ancient_grove_discovery",
        "The Ancient Grove",
        "Scholar Lyria has read references to a sacred grove deep in the Ashwood — \
         a place of ancient druidic power that was corrupted long ago. She asks you \
         to find it and document what you find.",
        "scholar_lyria",
        vec![
            QuestObjective {
                description: "Reach the Ashwood Ancient Grove".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "ashwood_ancient_grove".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(300, 30),
    ));

    // Quest: The Treant Lord — Ashwood Boss
    quest_log.register(Quest::new(
        "ashwood_treant_lord",
        "Lord of the Blighted Grove",
        "Ranger Vex has determined that the source of the forest's corruption is the \
         Treant Lord — an ancient, twisted treant that commands the Ashwood Ancient \
         Grove. Destroy it and collect its bark as proof of the deed.",
        "ranger_vex",
        vec![
            QuestObjective {
                description: "Reach the Ashwood Ancient Grove".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "ashwood_ancient_grove".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Defeat the Treant Lord".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "treant_lord".to_string(), required: 1, current: 0,
                },
            },
            QuestObjective {
                description: "Collect Treant Bark (3)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "treant_bark".to_string(), required: 3, current: 0,
                },
            },
        ],
        QuestReward::new(800, 80).with_item("ancient_treant_staff"),
    ));

    // ── Bog & Shadow Cave Quests (total: 6+) ─────────────────────────────────

    // Quest: Bogdan's Brew
    quest_log.register(Quest::new(
        "bog_moss_harvest",
        "Bogdan's Brew",
        "Hermit Bogdan needs bog moss to brew his restorative tonics. \
         The moss grows in abundance through the bog — if you survive long \
         enough to collect it.",
        "hermit_bogdan",
        vec![
            QuestObjective {
                description: "Collect bog moss (5)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "bog_moss".to_string(), required: 5, current: 0,
                },
            },
        ],
        QuestReward::new(150, 15).with_item("antidote"),
    ));

    // Quest: Bog Pests
    quest_log.register(Quest::new(
        "bog_pest_control",
        "Bog Pests",
        "Hermit Bogdan complains that bog crawlers have been multiplying out of \
         control this season, destroying his carefully tended herb patches. \
         Cull their numbers.",
        "hermit_bogdan",
        vec![
            QuestObjective {
                description: "Kill bog crawlers (5)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "bog_crawler".to_string(), required: 5, current: 0,
                },
            },
        ],
        QuestReward::new(200, 25),
    ));

    // Quest: Swamp Witch Coven
    quest_log.register(Quest::new(
        "bog_witch_warning",
        "The Witch Coven",
        "Hermit Bogdan warns that the swamp witches have been growing bolder — \
         their rituals are disturbing the bog's balance and poisoning the water. \
         Drive off their coven.",
        "hermit_bogdan",
        vec![
            QuestObjective {
                description: "Kill swamp witches (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "swamp_witch".to_string(), required: 3, current: 0,
                },
            },
        ],
        QuestReward::new(300, 35).with_item("clarity_potion"),
    ));

    // Quest: The Queen of the Bog — Bog Boss
    quest_log.register(Quest::new(
        "bog_witch_queen",
        "The Queen of the Bog",
        "Elder Aldric has heard that the swamp witches are led by an ancient queen \
         of terrible power, dwelling in a hut at the deepest part of the bog. \
         She must be dealt with before her influence reaches Thornvale.",
        "elder_aldric",
        vec![
            QuestObjective {
                description: "Reach the Bog Witch's Hut".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "bog_witchhut".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Defeat the Swamp Witch Queen".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "swamp_witch_queen".to_string(), required: 1, current: 0,
                },
            },
        ],
        QuestReward::new(700, 70).with_item("bog_queen_amulet"),
    ));

    // Quest: Secrets Below
    quest_log.register(Quest::new(
        "shadow_hidden_chamber",
        "Secrets Below",
        "Hermit Bogdan suspects the goblins in the shadow cave are hiding something \
         of value in a secret chamber. Explore the depths and find the hidden room.",
        "hermit_bogdan",
        vec![
            QuestObjective {
                description: "Reach the shadow cave hidden chamber".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "shadow_cave_hidden_chamber".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(350, 40),
    ));

    // ── Ironmere Keep Quests (total: 5+) ─────────────────────────────────────

    // Quest: Silence the Archers
    quest_log.register(Quest::new(
        "ironmere_archer_hunt",
        "Silence the Archers",
        "Guard Torven reports that goblin archers on the Ironmere approach are \
         picking off valley travellers from the ruins. Clear them out before \
         the road becomes completely impassable.",
        "guard_torven",
        vec![
            QuestObjective {
                description: "Kill goblin archers (4)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "goblin_archer".to_string(), required: 4, current: 0,
                },
            },
        ],
        QuestReward::new(300, 30).with_item("shortbow"),
    ));

    // Quest: Free the Captives
    quest_log.register(Quest::new(
        "ironmere_free_captives",
        "Free the Captives",
        "Elder Aldric has word that the goblins have taken prisoners — valley folk \
         dragged from their farms and locked in the keep's dungeon. Find the dungeon \
         and see what can be done.",
        "elder_aldric",
        vec![
            QuestObjective {
                description: "Reach the Ironmere dungeon".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "ironmere_dungeon".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(400, 50),
    ));

    // Quest: The Iron Dungeon
    quest_log.register(Quest::new(
        "ironmere_dungeon_clear",
        "The Iron Dungeon",
        "The Ironmere dungeon harbours not just living goblins but the shades of \
         those who perished there. Elder Aldric wants both threats eliminated \
         so that the dungeon can be reclaimed.",
        "elder_aldric",
        vec![
            QuestObjective {
                description: "Defeat the Ironmere Jailer".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "ironmere_jailer".to_string(), required: 1, current: 0,
                },
            },
            QuestObjective {
                description: "Defeat dungeon shades (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "dungeon_shade".to_string(), required: 3, current: 0,
                },
            },
        ],
        QuestReward::new(500, 55).with_item("iron_chainmail"),
    ));

    // Quest: The Warlord's Hoard
    quest_log.register(Quest::new(
        "ironmere_warlord_hoard",
        "The Warlord's Hoard",
        "Merchant Serah has heard that the goblin warlord hoards a significant supply \
         of iron ingots seized from valley merchants. Retrieve them from the keep tower.",
        "merchant_serah",
        vec![
            QuestObjective {
                description: "Reach the keep tower".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "ironmere_tower".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Collect iron ingots (3)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "iron_ingot".to_string(), required: 3, current: 0,
                },
            },
        ],
        QuestReward::new(350, 45),
    ));

    // ── Crystal Cave Quests (total: 6+) ──────────────────────────────────────

    // Quest: Bat Problem
    quest_log.register(Quest::new(
        "crystal_cave_bat_nest",
        "Bat Problem",
        "Guard Torven reports that giant bats from the Crystal Cave have been \
         swooping down on travellers near the northern valley. Their numbers need \
         to be reduced.",
        "guard_torven",
        vec![
            QuestObjective {
                description: "Kill giant bats (5)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "giant_bat".to_string(), required: 5, current: 0,
                },
            },
        ],
        QuestReward::new(180, 20),
    ));

    // Quest: The Cave Bear
    quest_log.register(Quest::new(
        "crystal_cave_bear_trophy",
        "The Cave Bear",
        "Ranger Vex wants proof that the cave bear menace in the Crystal Cave has \
         been dealt with. Defeat two cave bears and collect their claws as trophies.",
        "ranger_vex",
        vec![
            QuestObjective {
                description: "Kill cave bears (2)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "cave_bear".to_string(), required: 2, current: 0,
                },
            },
            QuestObjective {
                description: "Collect bear claws (2)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "bear_claw".to_string(), required: 2, current: 0,
                },
            },
        ],
        QuestReward::new(250, 30),
    ));

    // Quest: Crystalline Ingredients
    quest_log.register(Quest::new(
        "crystal_dust_harvest",
        "Crystalline Ingredients",
        "Hermit Bogdan needs crystalline dust for his alchemical experiments. \
         It can only be gathered from the Crystal Cave's inner formations.",
        "hermit_bogdan",
        vec![
            QuestObjective {
                description: "Collect crystalline dust (5)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "crystalline_dust".to_string(), required: 5, current: 0,
                },
            },
        ],
        QuestReward::new(200, 25).with_item("clarity_potion"),
    ));

    // Quest: The Crystal Depths
    quest_log.register(Quest::new(
        "crystal_cave_depths_expedition",
        "The Crystal Depths",
        "Ranger Vex has heard rumours of a hidden seam deep in the Crystal Cave \
         containing void crystals — crystals of exceptional power unlike any seen \
         before. Reach the hidden seam and bring back a sample.",
        "ranger_vex",
        vec![
            QuestObjective {
                description: "Reach the Crystal Cave hidden seam".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "crystal_cave_hidden_seam".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Collect a void crystal".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "void_crystal".to_string(), required: 1, current: 0,
                },
            },
        ],
        QuestReward::new(400, 45).with_item("crystal_ring"),
    ));

    // Quest: The Crystal Elemental — Crystal Cave Boss
    quest_log.register(Quest::new(
        "crystal_elemental_boss",
        "The Crystal Elemental",
        "Ranger Vex warns that a crystal elemental of immense power guards the \
         deepest seam of the Crystal Cave. It must be defeated before the seam \
         can be safely mined.",
        "ranger_vex",
        vec![
            QuestObjective {
                description: "Defeat the Crystal Elemental".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "crystal_elemental".to_string(), required: 1, current: 0,
                },
            },
        ],
        QuestReward::new(800, 90).with_item("void_crystal_ring"),
    ));

    // ── Valley Floor Quests (total: 3+) ──────────────────────────────────────

    // Quest: Know the Land
    quest_log.register(Quest::new(
        "valley_explorer",
        "Know the Land",
        "Guard Torven asks that you scout the northern and eastern reaches of the \
         valley, noting any goblin activity or bandit movements.",
        "guard_torven",
        vec![
            QuestObjective {
                description: "Reach the northern valley".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "valley_north".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Reach the eastern valley flats".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "valley_east".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(200, 25),
    ));

    // Quest: Bandit Camp
    quest_log.register(Quest::new(
        "bandit_camp_raid",
        "Clean Out the Camp",
        "Guard Torven has located a bandit camp at the abandoned farmstead. \
         Drive out the bandits and push their chief back.",
        "guard_torven",
        vec![
            QuestObjective {
                description: "Kill valley bandits (5)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "bandit".to_string(), required: 5, current: 0,
                },
            },
            QuestObjective {
                description: "Reach the abandoned farmstead".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "abandoned_farmstead".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(300, 30).with_item("bandit_cloak"),
    ));

    // Quest: Old Watchtower
    quest_log.register(Quest::new(
        "valley_watchtower_survey",
        "Old Watchtower",
        "Guard Torven recalls that an old watchtower on the valley's northeast rise \
         once gave fine views of goblin approaches. Survey it and report back.",
        "guard_torven",
        vec![
            QuestObjective {
                description: "Reach the ruined watchtower".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "valley_watchtower".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(150, 15).with_item("shortbow"),
    ));

    // ── Barrow Quests (additional, total: 5+) ────────────────────────────────

    // Quest: Barrow Wraiths
    quest_log.register(Quest::new(
        "barrow_wraith_hunt",
        "Barrow Wraiths",
        "Scholar Lyria is studying the barrow's supernatural manifestations. \
         She needs you to drive off the barrow wraiths to allow her research \
         team to safely excavate.",
        "scholar_lyria",
        vec![
            QuestObjective {
                description: "Defeat barrow wraiths (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "wraith".to_string(), required: 3, current: 0,
                },
            },
        ],
        QuestReward::new(400, 45).with_item("ghost_essence"),
    ));

    // Quest: Ancient Currency
    quest_log.register(Quest::new(
        "barrow_coin_collection",
        "Ancient Currency",
        "Scholar Lyria wants to study the ancient coinage found in the barrow \
         and surrounding ruins. Collect enough ancient coins for her research.",
        "scholar_lyria",
        vec![
            QuestObjective {
                description: "Collect ancient coins (8)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "ancient_coin".to_string(), required: 8, current: 0,
                },
            },
        ],
        QuestReward::new(200, 30),
    ));

    // Quest: Barrow High Knight — Barrow Boss
    quest_log.register(Quest::new(
        "barrow_high_knight",
        "The High Knight's Rest",
        "Elder Aldric has been told that a particularly powerful barrow knight — \
         a chieftain's champion — haunts the burial lord's chamber. This knight's \
         restless spirit cannot be allowed to grow stronger.",
        "elder_aldric",
        vec![
            QuestObjective {
                description: "Reach the barrow lord's chamber".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "barrow_lord_chamber".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Defeat barrow knights (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "barrow_knight".to_string(), required: 3, current: 0,
                },
            },
            QuestObjective {
                description: "Defeat the barrow wraith".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "wraith".to_string(), required: 1, current: 0,
                },
            },
        ],
        QuestReward::new(600, 65).with_item("barrow_lord_helm"),
    ));

    // ── Valley King's Tomb Quests (additional, total: 5+) ────────────────────

    // Quest: Restless Dead
    quest_log.register(Quest::new(
        "tomb_skeleton_hunt",
        "Restless Dead",
        "Elder Aldric worries that the animated skeletons spilling out of the \
         Valley King's Tomb will threaten the southern valley. Drive them back.",
        "elder_aldric",
        vec![
            QuestObjective {
                description: "Kill skeleton warriors (5)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "skeleton_warrior".to_string(), required: 5, current: 0,
                },
            },
        ],
        QuestReward::new(300, 30).with_item("iron_short_sword"),
    ));

    // Quest: The Mummified Legion
    quest_log.register(Quest::new(
        "tomb_mummy_hunt",
        "The Mummified Legion",
        "The Valley King's tomb is guarded by an army of mummified guards preserved \
         for an age. Elder Aldric asks you to thin their numbers before sending a \
         proper expedition.",
        "elder_aldric",
        vec![
            QuestObjective {
                description: "Defeat mummified guards (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "mummified_guard".to_string(), required: 3, current: 0,
                },
            },
        ],
        QuestReward::new(450, 50).with_item("ancient_coin"),
    ));

    // Quest: Valley King's Records
    quest_log.register(Quest::new(
        "tomb_inscription_research",
        "Valley King's Records",
        "Scholar Lyria needs to personally document the inscriptions in both \
         the tomb antechamber and sanctum. She needs someone to clear the way.",
        "scholar_lyria",
        vec![
            QuestObjective {
                description: "Reach the tomb antechamber".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "tomb_antechamber".to_string(), reached: false,
                },
            },
            QuestObjective {
                description: "Reach the tomb sanctum".to_string(),
                kind: ObjectiveKind::ReachLocation {
                    location_id: "tomb_sanctum".to_string(), reached: false,
                },
            },
        ],
        QuestReward::new(400, 40).with_item("ancient_tome"),
    ));

    // Quest: Cleanse the Sanctum
    quest_log.register(Quest::new(
        "tomb_spectral_cleansing",
        "Cleanse the Sanctum",
        "Scholar Lyria believes the wraiths in the Valley King's Tomb are anchored \
         to the sanctum by the burial rites. Defeating them may allow the tomb to \
         be safely studied.",
        "scholar_lyria",
        vec![
            QuestObjective {
                description: "Defeat wraiths in the tomb (3)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "wraith".to_string(), required: 3, current: 0,
                },
            },
        ],
        QuestReward::new(500, 55).with_item("clarity_potion"),
    ));

    // ── Additional Zone Quests (bringing each zone to 5+) ────────────────────

    // Ember Coast: Saltstone Crab Hunt (→ total 5)
    quest_log.register(Quest::new(
        "saltstone_crab_hunt",
        "Crab Season",
        "Fisherman Aldric says saltstone crabs have been multiplying on the shore, \
         making it impossible to set traps. Cull their numbers.",
        "fisherman_aldric",
        vec![
            QuestObjective {
                description: "Kill saltstone crabs (5)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "saltstone_crab".to_string(), required: 5, current: 0,
                },
            },
        ],
        QuestReward::new(200, 20).with_item("sea_salt"),
    ));

    // Iron Quarry: Ore Delivery (→ total 5)
    quest_log.register(Quest::new(
        "quarry_ore_delivery",
        "Iron for Thornvale",
        "Overseer Maren needs ore delivered to Thornvale's smith. Gather iron ore \
         from the quarry entrance and help keep the valley supplied.",
        "mining_overseer",
        vec![
            QuestObjective {
                description: "Collect iron ore (5)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "iron_ore".to_string(), required: 5, current: 0,
                },
            },
        ],
        QuestReward::new(180, 25),
    ));

    // Sunken Temple: Temple Offerings (→ total 5)
    quest_log.register(Quest::new(
        "temple_coin_recovery",
        "Temple Offerings",
        "Scholar Lyria wants to catalogue the ancient coin offerings left in the \
         Sunken Temple by its last worshippers. Collect what you find there.",
        "scholar_lyria",
        vec![
            QuestObjective {
                description: "Collect ancient coins (6) in the temple".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "ancient_coin".to_string(), required: 6, current: 0,
                },
            },
        ],
        QuestReward::new(250, 35),
    ));

    // Frostpeak: Snow Eagle Menace (→ total 5)
    quest_log.register(Quest::new(
        "frozen_eagle_hunt",
        "Snow Eagle Menace",
        "Frost Sage Erindel asks you to cull the snow eagles on the Frostpeak \
         approach — their swooping attacks have been injuring climbers.",
        "frost_sage",
        vec![
            QuestObjective {
                description: "Kill snow eagles (5)".to_string(),
                kind: ObjectiveKind::KillEnemy {
                    enemy_id: "snow_eagle".to_string(), required: 5, current: 0,
                },
            },
        ],
        QuestReward::new(180, 20).with_item("harpy_feather"),
    ));

    // Merchant's Crossing: Trade Supplies (→ total 5)
    quest_log.register(Quest::new(
        "merchant_goods_delivery",
        "Trade Supplies",
        "Merchant Aldis at the Crossing needs a shipment of driftwood and sea salt \
         for the river traders. Gather these from the coastal areas.",
        "merchant_aldis",
        vec![
            QuestObjective {
                description: "Collect driftwood (3)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "driftwood".to_string(), required: 3, current: 0,
                },
            },
            QuestObjective {
                description: "Collect sea salt (3)".to_string(),
                kind: ObjectiveKind::CollectItem {
                    item_id: "sea_salt".to_string(), required: 3, current: 0,
                },
            },
        ],
        QuestReward::new(150, 20).with_item("health_potion"),
    ));

    (npcs, quest_log)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_objectives_track_kills() {
        let (_, mut log) = build_narrative();
        log.start_quest("drive_back_goblins", &[]).unwrap();

        let messages = log.on_kill("goblin_warrior");
        assert!(!messages.is_empty());

        let q = log.quests.get("drive_back_goblins").unwrap();
        if let ObjectiveKind::KillEnemy { current, .. } = &q.objectives[0].kind {
            assert_eq!(*current, 1);
        }
    }

    #[test]
    fn test_quest_complete_requires_objectives() {
        let (_, mut log) = build_narrative();
        log.start_quest("gather_iron", &[]).unwrap();
        assert!(log.try_complete_quest("gather_iron").is_err());
    }

    #[test]
    fn test_quest_can_be_completed() {
        let (_, mut log) = build_narrative();
        log.start_quest("gather_iron", &[]).unwrap();
        for _ in 0..5 {
            log.on_collect("iron_ingot", 1);
        }
        let reward = log.try_complete_quest("gather_iron");
        assert!(reward.is_ok());
        assert_eq!(reward.unwrap().gold, 20);
    }

    #[test]
    fn test_npc_dialogue_filters_by_quest() {
        let (npcs, mut log) = build_narrative();
        let npc = npcs.get("elder_aldric").unwrap();
        let active: Vec<String> = vec![];
        let lines_before = npc.available_lines(&active);
        log.start_quest("drive_back_goblins", &[]).unwrap();
        let active_ids: Vec<String> = vec!["drive_back_goblins".to_string()];
        let lines_after = npc.available_lines(&active_ids);
        // The text shown changes based on quest state
        assert_ne!(lines_before[0].text, lines_after[0].text);
    }

    #[test]
    fn test_location_objective_triggers() {
        let (_, mut log) = build_narrative();
        log.start_quest("clear_wolf_den", &[]).unwrap();
        let msgs = log.on_reach_location("wolf_den_lair");
        assert!(!msgs.is_empty());
    }

    #[test]
    fn test_prerequisite_quest_blocks_start() {
        let (_, mut log) = build_narrative();
        let dependent = Quest::new(
            "test_dep", "Dependent", "desc", "npc",
            vec![], QuestReward::new(0, 0),
        ).with_prerequisite("drive_back_goblins");
        log.register(dependent);
        let result = log.start_quest("test_dep", &[]);
        assert!(result.is_err());
        let result2 = log.start_quest("test_dep", &["drive_back_goblins".to_string()]);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_merchant_serah_has_shop_items() {
        let (npcs, _) = build_narrative();
        let serah = npcs.get("merchant_serah").unwrap();
        assert!(!serah.shop_item_ids.is_empty(), "Merchant Serah should have shop items");
        assert!(serah.shop_item_ids.contains(&"health_potion".to_string()));
        assert!(serah.shop_item_ids.contains(&"antidote".to_string()));
    }

    #[test]
    fn test_blacksmith_grund_has_shop_items() {
        let (npcs, _) = build_narrative();
        let grund = npcs.get("blacksmith_grund").unwrap();
        assert!(!grund.shop_item_ids.is_empty(), "Blacksmith Grund should stock materials");
        assert!(grund.shop_item_ids.contains(&"iron_ingot".to_string()));
        assert!(grund.shop_item_ids.contains(&"leather".to_string()));
    }

    #[test]
    fn test_new_npcs_registered() {
        let (npcs, _) = build_narrative();
        assert!(npcs.get("hermit_bogdan").is_some(), "Hermit Bogdan should be registered");
        assert!(npcs.get("ranger_vex").is_some(), "Ranger Vex should be registered");
        assert!(npcs.get("scholar_lyria").is_some(), "Scholar Lyria should be registered");
    }

    #[test]
    fn test_hermit_bogdan_has_shop_and_quest() {
        let (npcs, _) = build_narrative();
        let bogdan = npcs.get("hermit_bogdan").unwrap();
        assert!(!bogdan.shop_item_ids.is_empty());
        assert!(bogdan.quest_ids.contains(&"shadow_cave_delve".to_string()));
    }

    #[test]
    fn test_new_quests_registered() {
        let (_, log) = build_narrative();
        for qid in &["crystal_cave_clear", "shadow_cave_delve", "barrow_research", "valley_king_tomb"] {
            assert!(log.quests.contains_key(*qid), "Quest '{}' should be registered", qid);
        }
    }

    #[test]
    fn test_new_instance_quests_registered() {
        let (_, log) = build_narrative();
        let new_quests = [
            "ashwood_treant_lord", "bog_witch_queen", "ironmere_dungeon_clear",
            "crystal_elemental_boss", "shadow_hidden_chamber",
            "thornvale_herbalist", "thornvale_market_supply",
            "ashwood_lumber_run", "ashwood_spider_hunt", "ashwood_forest_patrol",
            "ashwood_ancient_grove_discovery",
            "bog_moss_harvest", "bog_pest_control", "bog_witch_warning",
            "ironmere_archer_hunt", "ironmere_free_captives", "ironmere_warlord_hoard",
            "crystal_cave_bat_nest", "crystal_cave_bear_trophy", "crystal_dust_harvest",
            "crystal_cave_depths_expedition",
            "valley_explorer", "bandit_camp_raid", "valley_watchtower_survey",
            "barrow_wraith_hunt", "barrow_coin_collection", "barrow_high_knight",
            "tomb_skeleton_hunt", "tomb_mummy_hunt", "tomb_inscription_research",
            "tomb_spectral_cleansing",
            "saltstone_crab_hunt", "quarry_ore_delivery", "temple_coin_recovery",
            "frozen_eagle_hunt", "merchant_goods_delivery",
        ];
        for qid in &new_quests {
            assert!(log.quests.contains_key(*qid), "Quest '{}' should be registered", qid);
        }
    }

    #[test]
    fn test_boss_quests_target_boss_enemies() {
        let (_, log) = build_narrative();
        let treant = log.quests.get("ashwood_treant_lord").unwrap();
        let has_treant_lord = treant.objectives.iter().any(|o|
            matches!(&o.kind, ObjectiveKind::KillEnemy { enemy_id, .. } if enemy_id == "treant_lord")
        );
        assert!(has_treant_lord, "ashwood_treant_lord quest should target treant_lord");

        let bog = log.quests.get("bog_witch_queen").unwrap();
        let has_queen = bog.objectives.iter().any(|o|
            matches!(&o.kind, ObjectiveKind::KillEnemy { enemy_id, .. } if enemy_id == "swamp_witch_queen")
        );
        assert!(has_queen, "bog_witch_queen quest should target swamp_witch_queen");

        let crystal = log.quests.get("crystal_elemental_boss").unwrap();
        let has_elemental = crystal.objectives.iter().any(|o|
            matches!(&o.kind, ObjectiveKind::KillEnemy { enemy_id, .. } if enemy_id == "crystal_elemental")
        );
        assert!(has_elemental, "crystal_elemental_boss quest should target crystal_elemental");
    }

    #[test]
    fn test_valley_king_tomb_requires_barrow_research() {
        let (_, mut log) = build_narrative();
        // Should fail without prerequisite
        let result = log.start_quest("valley_king_tomb", &[]);
        assert!(result.is_err());
        // Should succeed when prerequisite is met
        let result2 = log.start_quest("valley_king_tomb", &["barrow_research".to_string()]);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_crystal_cave_quest_tracks_kills_and_location() {
        let (_, mut log) = build_narrative();
        log.start_quest("crystal_cave_clear", &[]).unwrap();
        // Location objective
        let msgs = log.on_reach_location("crystal_cave_entrance");
        assert!(!msgs.is_empty());
        // Kill objective
        for _ in 0..3 {
            log.on_kill("crystal_golem");
        }
        let q = log.quests.get("crystal_cave_clear").unwrap();
        let kill_obj = q.objectives.iter().find(|o| matches!(&o.kind, ObjectiveKind::KillEnemy { enemy_id, .. } if enemy_id == "crystal_golem")).unwrap();
        if let ObjectiveKind::KillEnemy { current, .. } = &kill_obj.kind {
            assert_eq!(*current, 3);
        }
    }
}
