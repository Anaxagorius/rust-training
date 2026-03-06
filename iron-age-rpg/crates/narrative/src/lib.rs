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
        .with_quest("drive_back_goblins"),
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
        .with_quest("gather_iron"),
    );

    npcs.register(
        Npc::new(
            "merchant_serah", "Merchant Serah", NpcRole::Merchant,
            "Fresh supplies, fair prices! What can I get for you?",
        )
        .with_shop_item("health_potion")
        .with_shop_item("antidote")
        .with_shop_item("iron_ingot")
        .with_shop_item("leather"),
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
        )),
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
        )),
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
}
