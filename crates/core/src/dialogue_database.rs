use crate::components::NPCMood;
use crate::dialogue::{
    DialogueChoice, DialogueNode, DialogueNodeID, DialogueTree, QuestType, TextEffect,
};
use std::collections::HashMap;

/// Central database for all dialogue trees in the game
/// Provides efficient access to dialogue content by NPC/quest ID
pub struct DialogueDatabase {
    trees: HashMap<String, DialogueTree>,
}

impl DialogueDatabase {
    /// Create a new dialogue database and load all dialogue trees
    pub fn new() -> Self {
        let mut database = Self {
            trees: HashMap::new(),
        };

        // Load all dialogue trees
        database.load_all_trees();
        database
    }

    /// Get a dialogue tree by ID
    pub fn get_tree(&self, tree_id: &str) -> Option<&DialogueTree> {
        self.trees.get(tree_id)
    }

    /// Get all available dialogue tree IDs
    pub fn get_tree_ids(&self) -> Vec<&String> {
        self.trees.keys().collect()
    }

    /// Load all dialogue trees into the database
    fn load_all_trees(&mut self) {
        // Tutorial quest NPCs
        self.trees.insert(
            "broken_android_tutorial".to_string(),
            Self::create_quest_tutorial_broken_android_tree(),
        );

        // Future dialogue trees will be added here:
        // self.trees.insert("merchant_basic".to_string(), Self::create_merchant_basic_tree());
        // self.trees.insert("guard_hostile".to_string(), Self::create_guard_hostile_tree());
        // self.trees.insert("scientist_friendly".to_string(), Self::create_scientist_friendly_tree());
    }

    /// Create dialogue tree for the broken SapienCorp android tutorial
    fn create_quest_tutorial_broken_android_tree() -> DialogueTree {
        let mut tree = DialogueTree::new();

        // Initial greeting
        tree.add_node(DialogueNode {
            id: DialogueNodeID::Start,
            speaker: "Broken Android".to_string(),
            text: "<1>Hello</1>... <2>...company android detected</2>. I am... <3>...SapienCorp maintenance unit</3>. My work task <4>is ... welding ... today</4>. How <2>can I assist you</2>... ... ... <5>today</5>? \n\n (You gaze at the rusty, flickering lump of metal on the floor. You wonder how it's still running. These construction and maintenance models were released only a few months ago. How is this one so messed up?)".to_string(),
            text_effects: vec![TextEffect::Static, TextEffect::Buzz, TextEffect::Crackle, TextEffect::PopHiss, TextEffect::Corrupt],
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "Are you alright?".to_string(),
                    leads_to: Some(DialogueNodeID::AreYouAlright),
                    requires_item: None,
                    npc_mood_change: None,
                    player_mood_change: Some(NPCMood::Neutral), // Player shows concern
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "What happened to you?".to_string(),
                    leads_to: Some(DialogueNodeID::WhatHappened),
                    requires_item: None,
                    npc_mood_change: None,
                    player_mood_change: Some(NPCMood::Weird), // Player is curious
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "Can I help?".to_string(),
                    leads_to: Some(DialogueNodeID::CanIHelp),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy), // Player feels helpful/compassionate
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "Where are we right now? Why is it so dark?".to_string(),
                    leads_to: Some(DialogueNodeID::WhereAreWe),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Neutral), // Player is confused/seeking info
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "Override Alpha-7: Emergency diagnostic mode. State model number and diagnostic info.".to_string(),
                    leads_to: Some(DialogueNodeID::OverrideModelNumber),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Weird),
                    player_mood_change: Some(NPCMood::Weird), // Player is being technical/commanding
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "[Give items] I have the diamond and scrap electronics you need.".to_string(),
                    leads_to: Some(DialogueNodeID::CompleteQuest),
                    requires_item: Some("Diamond".to_string()),
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: None,
                    requires_quest_active: Some(QuestType::RepairBrokenAndroid),
                    requires_quest_complete: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Response to "Are you alright?"
        tree.add_node(DialogueNode {
            id: DialogueNodeID::AreYouAlright,
            speaker: "Broken Android".to_string(),
            text: "<1>Systems... failing</1>. Memory core... <2>...damaged</2>. Cannot return to... <3>...base</3>.".to_string(),
            text_effects: vec![TextEffect::Static, TextEffect::Buzz, TextEffect::Crackle],
            mood: NPCMood::Sad,
            choices: vec![
                DialogueChoice {
                    text: "Maybe I can help repair you.".to_string(),
                    leads_to: Some(DialogueNodeID::CanIHelp),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "I'm sorry, I have to go.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Sad),
                    player_mood_change: Some(NPCMood::Sad),
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Response to "What happened to you?"
        tree.add_node(DialogueNode {
            id: DialogueNodeID::WhatHappened,
            speaker: "Broken Android".to_string(),
            text: "<1>Facility... explosion</1>. Lost contact with... <2>...SapienCorp</2>. Been here... days? Weeks? <3>*transmission degrades*</3>".to_string(),
            text_effects: vec![TextEffect::Buzz, TextEffect::Static, TextEffect::Crackle],
            mood: NPCMood::Sad,
            choices: vec![
                DialogueChoice {
                    text: "I'll help you get back online.".to_string(),
                    leads_to: Some(DialogueNodeID::CanIHelp),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Quest offer
        tree.add_node(DialogueNode {
            id: DialogueNodeID::CanIHelp,
            speaker: "Broken Android".to_string(),
            text: "<1>You would... help?</1> Need spare parts... <2>...to repair primary systems</2>. Basic components scattered around facility.".to_string(),
            text_effects: vec![TextEffect::Static, TextEffect::Buzz],
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "What do you need?".to_string(),
                    leads_to: Some(DialogueNodeID::WhatDoYouNeed),
                    requires_item: None,
                    npc_mood_change: None,
                    player_mood_change: None,
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Quest details
        tree.add_node(DialogueNode {
            id: DialogueNodeID::WhatDoYouNeed,
            speaker: "Broken Android".to_string(),
            text: "<1>Simple components</1>... <2>...a lab-grown diamond for my fusion reactor...and scrap electronics...</2>. Will... try to... <3>...stay online</3>.".to_string(),
            text_effects: vec![TextEffect::Crackle, TextEffect::Buzz, TextEffect::Static],
            mood: NPCMood::Neutral,
            choices: vec![
                DialogueChoice {
                    text: "I'll find a diamond and scrap electronics for you.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: Some(QuestType::RepairBrokenAndroid),
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "I can't do that right now.".to_string(),
                    leads_to: Some(DialogueNodeID::Start),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Sad),
                    player_mood_change: Some(NPCMood::Sad),
                    unlocks_quest:None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Response to "Where are we?"
        tree.add_node(DialogueNode {
            id: DialogueNodeID::WhereAreWe,
            speaker: "Broken Android".to_string(),
            text: "<1>Location... SapienCorp Factory</1>... <2>...welding section</2>. Dark because... <3>...power grid failure</3>... I think? <4>Sensors... malfunctioning</4>... cannot confirm... <5>...been here so long</5>... <6>999 days</6>? Or... was it... 9 days?".to_string(),
            text_effects: vec![TextEffect::Corrupt, TextEffect::Corrupt, TextEffect::Corrupt, TextEffect::Corrupt, TextEffect::Corrupt, TextEffect::Corrupt],
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "... [say nothing] (This poor thing has clearly got a few screws loose...)".to_string(),
                    leads_to: Some(DialogueNodeID::Start),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Response to override command
        tree.add_node(DialogueNode {
            id: DialogueNodeID::OverrideModelNumber,
            speaker: "Broken Android".to_string(),
            text: "<1>Override accepted</1>... <2>...Model SC-M4X7</2>... Maintenance unit... <3>...Primary systems: 15% operational</3>... Secondary systems: offline... <4>...Critical errors in navigation, visual processing, memory core</4>...".to_string(),
            text_effects: vec![TextEffect::Buzz, TextEffect::Static, TextEffect::Crackle, TextEffect::Buzz],
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "Understood. Can you be repaired?".to_string(),
                    leads_to: Some(DialogueNodeID::CanIHelp),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "Thank you for the diagnostic. Return to standard mode.".to_string(),
                    leads_to: Some(DialogueNodeID::Start),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Quest completion dialogue
        tree.add_node(DialogueNode {
            id: DialogueNodeID::CompleteQuest,
            speaker: "Broken Android".to_string(),
            text: "<1>*scanning items*</1>... Lab-grown diamond detected... Scrap electronics detected... <2>*systems activating*</2>... Thank you, user! <3>Initiating repair protocol</3>... Memory core stabilizing... <4>*mechanical whirring sounds*</4>... <5>Repair successful!</5> I am now operational. SapienCorp maintenance unit SC-M4X7 reporting for duty. How may I assist the facility today?".to_string(),
            text_effects: vec![TextEffect::Static, TextEffect::Buzz, TextEffect::Crackle, TextEffect::PopHiss, TextEffect::Fade],
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "You're welcome! Glad I could help.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "What will you do now?".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Once repaired, conversations open here instead of at Start
        tree.add_node(DialogueNode {
            id: DialogueNodeID::Repaired,
            speaker: "SC-M4X7".to_string(),
            text: "SapienCorp maintenance unit SC-M4X7, fully operational, thanks to you, user! Memory core stable, welding arm steady. What can I do for you?".to_string(),
            text_effects: vec![],
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "Do you know where I could find a new arm?".to_string(),
                    leads_to: Some(DialogueNodeID::NewArm),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "My arm works. What else did SapienCorp leave behind?".to_string(),
                    leads_to: Some(DialogueNodeID::BlackBoxOffer),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: Some(QuestType::ReplaceArm),
                },
                DialogueChoice {
                    text: "I brought the data shard from the datacenter.".to_string(),
                    leads_to: Some(DialogueNodeID::BlackBoxReveal),
                    requires_item: Some("Data Shard".to_string()),
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: None,
                    requires_quest_active: Some(QuestType::BlackBox),
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "Where is the sealed site under the river of fire?".to_string(),
                    leads_to: Some(DialogueNodeID::SealedSiteOffer),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: Some(QuestType::BlackBox),
                },
                DialogueChoice {
                    text: "I found Thale's journal.".to_string(),
                    leads_to: Some(DialogueNodeID::SealedSiteReveal),
                    requires_item: Some("Thale's Journal".to_string()),
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: None,
                    requires_quest_active: Some(QuestType::SealedSite),
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "Just checking on you. Goodbye!".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // The second quest, offered once the android is repaired
        tree.add_node(DialogueNode {
            id: DialogueNodeID::NewArm,
            speaker: "SC-M4X7".to_string(),
            text: "Your left arm... SapienCorp built that model at the assembly facility, about half a mile north of here. The line shut down mid-shift. There should still be a salvaged arm in a parts locker by the line. Watch out: the dogs got in after the staff left. Bring it back attached, user!".to_string(),
            text_effects: vec![],
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "I'll go and find it.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: Some(QuestType::ReplaceArm),
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "Thanks, I'll keep it in mind.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: None,
                    player_mood_change: None,
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // The third quest, offered once the arm is in
        tree.add_node(DialogueNode {
            id: DialogueNodeID::BlackBoxOffer,
            speaker: "SC-M4X7".to_string(),
            text: "SapienCorp backed everything up to its datacenter in the ruined city west of here. If anything remembers what happened to the reactor, it's in there. Bring something to burn, user, and watch for the security units.".to_string(),
            text_effects: vec![],
            mood: NPCMood::Neutral,
            choices: vec![
                DialogueChoice {
                    text: "I'll find the reactor's data.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: Some(QuestType::BlackBox),
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "Not yet.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: None,
                    player_mood_change: None,
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Handing in the shard: what the telemetry says
        tree.add_node(DialogueNode {
            id: DialogueNodeID::BlackBoxReveal,
            speaker: "SC-M4X7".to_string(),
            text: "<1>*reading shard*</1> The reactor was held past its limit for 41 days, on orders from Chief Executive Odessa Crane and the board, for a project run by Dr. Emeric Thale. His last entry: \"Hold output. The aperture is almost stable.\" Thale... a sealed site under the river of fire. Keep looking, user.".to_string(),
            text_effects: vec![TextEffect::Static],
            mood: NPCMood::Neutral,
            choices: vec![DialogueChoice {
                text: "I'll find out what Thale was building.".to_string(),
                leads_to: None,
                requires_item: None,
                npc_mood_change: Some(NPCMood::Neutral),
                player_mood_change: Some(NPCMood::Neutral),
                unlocks_quest: None,
                requires_quest_active: None,
                requires_quest_complete: None,
            }],
            auto_continue: false,
            shop_item: None,
        });

        // The fourth quest, offered once "Black Box" is done
        tree.add_node(DialogueNode {
            id: DialogueNodeID::SealedSiteOffer,
            speaker: "SC-M4X7".to_string(),
            text: "Thale's annex. A SapienCorp shaft goes down from a hut south-west of here, past the rivers of fire. The annex ran on its own reactor's leavings, user: if the door is dead, the leavings are still there.".to_string(),
            text_effects: vec![],
            mood: NPCMood::Neutral,
            choices: vec![
                DialogueChoice {
                    text: "I'll go down.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: Some(QuestType::SealedSite),
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
                DialogueChoice {
                    text: "Not yet.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: None,
                    player_mood_change: None,
                    unlocks_quest: None,
                    requires_quest_active: None,
                    requires_quest_complete: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Handing in the journal: the aperture
        tree.add_node(DialogueNode {
            id: DialogueNodeID::SealedSiteReveal,
            speaker: "SC-M4X7".to_string(),
            text: "<1>*reading journal*</1> The aperture: a gate to the far side, fed by the reactor. It opened once, four seconds. \"We heard the far side breathing. It turned toward us.\" Thale had the maintenance frames built to remember its shape. The frames were my model line, user. The last page names the foundry that cast the ring.".to_string(),
            text_effects: vec![TextEffect::Static],
            mood: NPCMood::Sad,
            choices: vec![DialogueChoice {
                text: "Then I'll find the foundry.".to_string(),
                leads_to: None,
                requires_item: None,
                npc_mood_change: Some(NPCMood::Neutral),
                player_mood_change: Some(NPCMood::Neutral),
                unlocks_quest: None,
                requires_quest_active: None,
                requires_quest_complete: None,
            }],
            auto_continue: false,
            shop_item: None,
        });

        tree
    }
}

impl Default for DialogueDatabase {
    fn default() -> Self {
        Self::new()
    }
}
