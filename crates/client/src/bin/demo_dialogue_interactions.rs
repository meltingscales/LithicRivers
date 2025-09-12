use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::{io, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DialogueType {
    Linear,      // Simple linear conversation
    Branching,   // Player choices affect dialogue
    Shop,        // Trading interface with dialogue
    Quest,       // Quest giving with conditions
    Battle,      // Pre/post battle dialogue
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NPCMood {
    Friendly,
    Neutral,
    Hostile,
    Sad,
    Excited,
    Mysterious,
}

#[derive(Debug, Clone)]
struct DialogueChoice {
    text: String,
    leads_to: Option<usize>, // Index of next dialogue node, None = end conversation
    requires_item: Option<String>,
    mood_change: Option<NPCMood>,
    unlocks_quest: bool,
}

#[derive(Debug, Clone)]
struct DialogueNode {
    id: usize,
    speaker: String,
    text: String,
    mood: NPCMood,
    choices: Vec<DialogueChoice>,
    auto_continue: bool, // If true, automatically continues without player input
    shop_item: Option<String>, // If set, this node offers to sell/trade this item
}

#[derive(Debug, Clone)]
struct NPC {
    name: String,
    portrait: String, // ASCII art portrait
    dialogue_type: DialogueType,
    current_mood: NPCMood,
    initial_dialogue: usize, // Starting dialogue node ID
    met_before: bool,
    has_quest: bool,
    shop_inventory: Vec<String>,
}

#[derive(Debug, Clone)]
struct Player {
    name: String,
    inventory: Vec<String>,
    gold: u32,
    completed_quests: Vec<String>,
}

struct App {
    npcs: Vec<NPC>,
    dialogue_tree: Vec<DialogueNode>,
    player: Player,
    current_npc: Option<usize>,
    current_dialogue: Option<usize>,
    selected_choice: usize,
    message_log: Vec<String>,
    should_quit: bool,
    conversation_log: Vec<String>, // Track conversation history
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            npcs: Vec::new(),
            dialogue_tree: Vec::new(),
            player: Player {
                name: "Hero".to_string(),
                inventory: vec!["Rusty Sword".to_string(), "Health Potion".to_string()],
                gold: 50,
                completed_quests: Vec::new(),
            },
            current_npc: None,
            current_dialogue: None,
            selected_choice: 0,
            message_log: Vec::new(),
            should_quit: false,
            conversation_log: Vec::new(),
        };

        app.setup_npcs_and_dialogue();
        app
    }

    fn setup_npcs_and_dialogue(&mut self) {
        // Create NPCs with portraits inspired by classic RPGs
        self.npcs = vec![
            NPC {
                name: "Merchant Aldric".to_string(),
                portrait: "
    ╔═══════════╗
    ║  ◉     ◉  ║
    ║     ◡     ║
    ║   ┌───┐   ║
    ║   │ $ │   ║
    ║   └───┘   ║
    ╚═══════════╝
".to_string(),
                dialogue_type: DialogueType::Shop,
                current_mood: NPCMood::Friendly,
                initial_dialogue: 0,
                met_before: false,
                has_quest: false,
                shop_inventory: vec!["Iron Sword".to_string(), "Magic Scroll".to_string(), "Elixir".to_string()],
            },
            NPC {
                name: "Knight Captain Elena".to_string(),
                portrait: "
    ╔═══════════╗
    ║  ◉     ◉  ║
    ║     ─     ║
    ║   ┌───┐   ║
    ║   │ ⚔ │   ║
    ║   └───┘   ║
    ╚═══════════╝
".to_string(),
                dialogue_type: DialogueType::Quest,
                current_mood: NPCMood::Neutral,
                initial_dialogue: 10,
                met_before: false,
                has_quest: true,
                shop_inventory: Vec::new(),
            },
            NPC {
                name: "Mysterious Oracle".to_string(),
                portrait: "
    ╔═══════════╗
    ║  ◌     ◌  ║
    ║     ~     ║
    ║   ┌───┐   ║
    ║   │ 🔮 │   ║
    ║   └───┘   ║
    ╚═══════════╝
".to_string(),
                dialogue_type: DialogueType::Branching,
                current_mood: NPCMood::Mysterious,
                initial_dialogue: 20,
                met_before: false,
                has_quest: false,
                shop_inventory: Vec::new(),
            },
            NPC {
                name: "Innkeeper Marta".to_string(),
                portrait: "
    ╔═══════════╗
    ║  ♥     ♥  ║
    ║     ⌣     ║
    ║   ┌───┐   ║
    ║   │ 🍺 │   ║
    ║   └───┘   ║
    ╚═══════════╝
".to_string(),
                dialogue_type: DialogueType::Linear,
                current_mood: NPCMood::Friendly,
                initial_dialogue: 30,
                met_before: false,
                has_quest: false,
                shop_inventory: Vec::new(),
            },
            NPC {
                name: "Bandit Leader Raven".to_string(),
                portrait: "
    ╔═══════════╗
    ║  ▲     ▲  ║
    ║    \\▼/    ║
    ║   ┌───┐   ║
    ║   │ 💀 │   ║
    ║   └───┘   ║
    ╚═══════════╝
".to_string(),
                dialogue_type: DialogueType::Battle,
                current_mood: NPCMood::Hostile,
                initial_dialogue: 40,
                met_before: false,
                has_quest: false,
                shop_inventory: Vec::new(),
            },
        ];

        // Create comprehensive dialogue tree
        self.dialogue_tree = vec![
            // Merchant Aldric (0-9)
            DialogueNode {
                id: 0,
                speaker: "Merchant Aldric".to_string(),
                text: "Welcome, traveler! I have the finest wares in all the land. What catches your eye?".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "Show me your weapons".to_string(),
                        leads_to: Some(1),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "I need healing supplies".to_string(),
                        leads_to: Some(2),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Just browsing, thanks".to_string(),
                        leads_to: Some(3),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 1,
                speaker: "Merchant Aldric".to_string(),
                text: "Ah, a warrior! This Iron Sword is newly forged and sharp as winter wind. Only 30 gold!".to_string(),
                mood: NPCMood::Excited,
                choices: vec![
                    DialogueChoice {
                        text: "I'll take it!".to_string(),
                        leads_to: Some(4),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Too expensive for me".to_string(),
                        leads_to: Some(5),
                        requires_item: None,
                        mood_change: Some(NPCMood::Neutral),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: Some("Iron Sword".to_string()),
            },
            DialogueNode {
                id: 2,
                speaker: "Merchant Aldric".to_string(),
                text: "Smart thinking! An Elixir of Restoration - it'll bring you back from death's door. 25 gold.".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "Perfect, I need that".to_string(),
                        leads_to: Some(6),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "What else do you have?".to_string(),
                        leads_to: Some(0),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: Some("Elixir".to_string()),
            },
            DialogueNode {
                id: 3,
                speaker: "Merchant Aldric".to_string(),
                text: "Of course! Take your time. But don't wait too long - good items don't stay long!".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "Actually, let me see your wares".to_string(),
                        leads_to: Some(0),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Thanks, goodbye".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 4,
                speaker: "Merchant Aldric".to_string(),
                text: "Excellent choice! May it serve you well in battle. Come back anytime!".to_string(),
                mood: NPCMood::Excited,
                choices: vec![
                    DialogueChoice {
                        text: "Thank you!".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 5,
                speaker: "Merchant Aldric".to_string(),
                text: "I understand, times are tough. Perhaps when your purse is heavier, hmm?".to_string(),
                mood: NPCMood::Neutral,
                choices: vec![
                    DialogueChoice {
                        text: "I'll be back".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Show me something cheaper".to_string(),
                        leads_to: Some(2),
                        requires_item: None,
                        mood_change: Some(NPCMood::Friendly),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 6,
                speaker: "Merchant Aldric".to_string(),
                text: "Wise investment! This elixir has saved many heroes. Use it well!".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "Thank you!".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },

            // Knight Captain Elena (10-19)
            DialogueNode {
                id: 10,
                speaker: "Knight Captain Elena".to_string(),
                text: "Halt, citizen! I am Elena, Captain of the Royal Guard. Our town faces a terrible threat.".to_string(),
                mood: NPCMood::Neutral,
                choices: vec![
                    DialogueChoice {
                        text: "What kind of threat?".to_string(),
                        leads_to: Some(11),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "How can I help?".to_string(),
                        leads_to: Some(12),
                        requires_item: None,
                        mood_change: Some(NPCMood::Friendly),
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Not my problem".to_string(),
                        leads_to: Some(13),
                        requires_item: None,
                        mood_change: Some(NPCMood::Hostile),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 11,
                speaker: "Knight Captain Elena".to_string(),
                text: "Goblins! They've taken the old mine and are terrorizing travelers. We need someone brave enough to clear them out.".to_string(),
                mood: NPCMood::Neutral,
                choices: vec![
                    DialogueChoice {
                        text: "I'll do it!".to_string(),
                        leads_to: Some(14),
                        requires_item: None,
                        mood_change: Some(NPCMood::Friendly),
                        unlocks_quest: true,
                    },
                    DialogueChoice {
                        text: "Sounds dangerous...".to_string(),
                        leads_to: Some(15),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 12,
                speaker: "Knight Captain Elena".to_string(),
                text: "A noble heart! The realm needs more souls like yours. There are goblins in the old mine - clear them out and I'll reward you handsomely!".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "Consider it done!".to_string(),
                        leads_to: Some(14),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: true,
                    },
                    DialogueChoice {
                        text: "What's the reward?".to_string(),
                        leads_to: Some(16),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 13,
                speaker: "Knight Captain Elena".to_string(),
                text: "Hmph! Typical. When the goblins reach your doorstep, don't come crying to us!".to_string(),
                mood: NPCMood::Hostile,
                choices: vec![
                    DialogueChoice {
                        text: "Wait, maybe I can help...".to_string(),
                        leads_to: Some(11),
                        requires_item: None,
                        mood_change: Some(NPCMood::Neutral),
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Fine by me".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 14,
                speaker: "Knight Captain Elena".to_string(),
                text: "Excellent! The mine is north of town. Clear out all the goblins and return to me for your reward. May fortune favor you!".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "I'll return victorious!".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 15,
                speaker: "Knight Captain Elena".to_string(),
                text: "Yes, it is. But great rewards require great risks. I can offer you 100 gold and a blessed weapon if you succeed.".to_string(),
                mood: NPCMood::Neutral,
                choices: vec![
                    DialogueChoice {
                        text: "That changes things - I'm in!".to_string(),
                        leads_to: Some(14),
                        requires_item: None,
                        mood_change: Some(NPCMood::Friendly),
                        unlocks_quest: true,
                    },
                    DialogueChoice {
                        text: "Still too risky for me".to_string(),
                        leads_to: Some(17),
                        requires_item: None,
                        mood_change: Some(NPCMood::Sad),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 16,
                speaker: "Knight Captain Elena".to_string(),
                text: "100 gold pieces and a weapon blessed by our court mage. More than enough to set you up for greater adventures!".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "That's generous - I accept!".to_string(),
                        leads_to: Some(14),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: true,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 17,
                speaker: "Knight Captain Elena".to_string(),
                text: "I understand. Perhaps when you've gained more experience, you'll reconsider. The offer stands.".to_string(),
                mood: NPCMood::Sad,
                choices: vec![
                    DialogueChoice {
                        text: "Maybe another time".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },

            // Mysterious Oracle (20-29)
            DialogueNode {
                id: 20,
                speaker: "Mysterious Oracle".to_string(),
                text: "The threads of fate bring you to me... I see much in your future, young one. Do you wish to know what the shadows whisper?".to_string(),
                mood: NPCMood::Mysterious,
                choices: vec![
                    DialogueChoice {
                        text: "Tell me my future".to_string(),
                        leads_to: Some(21),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "What do you see in my past?".to_string(),
                        leads_to: Some(22),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "I don't believe in fortune telling".to_string(),
                        leads_to: Some(23),
                        requires_item: None,
                        mood_change: Some(NPCMood::Neutral),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 21,
                speaker: "Mysterious Oracle".to_string(),
                text: "I see... a great darkness approaching. You will face a choice between power and compassion. Choose wisely, for your decision will echo through ages.".to_string(),
                mood: NPCMood::Mysterious,
                choices: vec![
                    DialogueChoice {
                        text: "What kind of darkness?".to_string(),
                        leads_to: Some(24),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "How do I prepare?".to_string(),
                        leads_to: Some(25),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "That's rather vague...".to_string(),
                        leads_to: Some(26),
                        requires_item: None,
                        mood_change: Some(NPCMood::Neutral),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 22,
                speaker: "Mysterious Oracle".to_string(),
                text: "Your past... shrouded in mist. But I sense loss, and from that loss, strength was born. You carry the hopes of those who are gone.".to_string(),
                mood: NPCMood::Sad,
                choices: vec![
                    DialogueChoice {
                        text: "You're right... I lost everything".to_string(),
                        leads_to: Some(27),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "How could you know that?".to_string(),
                        leads_to: Some(28),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 23,
                speaker: "Mysterious Oracle".to_string(),
                text: "Skepticism is... wise. But the future has a way of proving itself, regardless of belief. You will remember these words.".to_string(),
                mood: NPCMood::Neutral,
                choices: vec![
                    DialogueChoice {
                        text: "We'll see about that".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Maybe you're right... tell me more".to_string(),
                        leads_to: Some(20),
                        requires_item: None,
                        mood_change: Some(NPCMood::Mysterious),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },

            // Innkeeper Marta (30-39) - Linear dialogue
            DialogueNode {
                id: 30,
                speaker: "Innkeeper Marta".to_string(),
                text: "Welcome to the Prancing Pony! You look weary, traveler. A hot meal and warm bed await you here!".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "That sounds perfect".to_string(),
                        leads_to: Some(31),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 31,
                speaker: "Innkeeper Marta".to_string(),
                text: "Wonderful! I'll have the cook prepare our special stew. It's made with herbs from my own garden - guaranteed to restore your strength!".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "How much for the meal and room?".to_string(),
                        leads_to: Some(32),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 32,
                speaker: "Innkeeper Marta".to_string(),
                text: "Just 5 gold for the meal and bed! And I'll throw in breakfast tomorrow morning - fresh baked bread and honey!".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "Perfect! Here's your gold".to_string(),
                        leads_to: Some(33),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Actually, I'll just rest outside".to_string(),
                        leads_to: Some(34),
                        requires_item: None,
                        mood_change: Some(NPCMood::Sad),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 33,
                speaker: "Innkeeper Marta".to_string(),
                text: "Bless you! Your room is upstairs, second door on the right. Sweet dreams, and may tomorrow bring you good fortune!".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "Thank you for your kindness".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 34,
                speaker: "Innkeeper Marta".to_string(),
                text: "Oh dear... if gold is tight, at least take this bread roll. No one should go hungry on my watch!".to_string(),
                mood: NPCMood::Sad,
                choices: vec![
                    DialogueChoice {
                        text: "You're too kind, thank you".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: Some(NPCMood::Friendly),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },

            // Bandit Leader Raven (40-49) - Battle dialogue
            DialogueNode {
                id: 40,
                speaker: "Bandit Leader Raven".to_string(),
                text: "Well, well... what have we here? Another fool wandering into MY territory! You picked the wrong road, stranger!".to_string(),
                mood: NPCMood::Hostile,
                choices: vec![
                    DialogueChoice {
                        text: "I'm not looking for trouble".to_string(),
                        leads_to: Some(41),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Bring it on, bandit scum!".to_string(),
                        leads_to: Some(42),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Maybe we can make a deal?".to_string(),
                        leads_to: Some(43),
                        requires_item: None,
                        mood_change: Some(NPCMood::Neutral),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 41,
                speaker: "Bandit Leader Raven".to_string(),
                text: "Ha! Too bad! Trouble found YOU! Drop your gold and weapons, and maybe I'll let you crawl away with your life!".to_string(),
                mood: NPCMood::Hostile,
                choices: vec![
                    DialogueChoice {
                        text: "Never! I'd rather die fighting!".to_string(),
                        leads_to: Some(44),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Fine, take what you want".to_string(),
                        leads_to: Some(45),
                        requires_item: None,
                        mood_change: Some(NPCMood::Neutral),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 42,
                speaker: "Bandit Leader Raven".to_string(),
                text: "Hah! I like your spirit! It'll make crushing it all the more satisfying! Come then, let's dance!".to_string(),
                mood: NPCMood::Excited,
                choices: vec![
                    DialogueChoice {
                        text: "[FIGHT!]".to_string(),
                        leads_to: Some(46),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 43,
                speaker: "Bandit Leader Raven".to_string(),
                text: "A deal? Interesting... I'm listening. But make it good, or my blade will do the talking!".to_string(),
                mood: NPCMood::Neutral,
                choices: vec![
                    DialogueChoice {
                        text: "I'll pay you 20 gold to let me pass".to_string(),
                        leads_to: Some(47),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "How about I help you rob someone richer?".to_string(),
                        leads_to: Some(48),
                        requires_item: None,
                        mood_change: Some(NPCMood::Friendly),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
        ];

        self.add_message("Welcome to the Dialogue Demo! Talk to NPCs with Enter, navigate with arrows/numbers.".to_string());
    }

    fn get_mood_color(&self, mood: NPCMood) -> Color {
        match mood {
            NPCMood::Friendly => Color::Green,
            NPCMood::Neutral => Color::White,
            NPCMood::Hostile => Color::Red,
            NPCMood::Sad => Color::Blue,
            NPCMood::Excited => Color::Yellow,
            NPCMood::Mysterious => Color::Magenta,
        }
    }

    fn get_mood_prefix(&self, mood: NPCMood) -> &'static str {
        match mood {
            NPCMood::Friendly => "😊 ",
            NPCMood::Neutral => "😐 ",
            NPCMood::Hostile => "😠 ",
            NPCMood::Sad => "😢 ",
            NPCMood::Excited => "😆 ",
            NPCMood::Mysterious => "🧙 ",
        }
    }

    fn get_mood_face(&self, mood: NPCMood) -> &'static str {
        match mood {
            NPCMood::Friendly => "◡",
            NPCMood::Neutral => "─",
            NPCMood::Hostile => "▼",
            NPCMood::Sad => "︶",
            NPCMood::Excited => "◠",
            NPCMood::Mysterious => "~",
        }
    }

    fn get_mood_eyes(&self, mood: NPCMood) -> &'static str {
        match mood {
            NPCMood::Friendly => "◉     ◉",
            NPCMood::Neutral => "○     ○",
            NPCMood::Hostile => "▲     ▲",
            NPCMood::Sad => "◌     ◌",
            NPCMood::Excited => "★     ★",
            NPCMood::Mysterious => "◇     ◇",
        }
    }

    fn get_portrait_with_mood(&self, npc: &NPC) -> String {
        let eyes = self.get_mood_eyes(npc.current_mood);
        let mouth = self.get_mood_face(npc.current_mood);
        let symbol = match npc.dialogue_type {
            DialogueType::Shop => "$",
            DialogueType::Quest => "⚔",
            DialogueType::Branching => "🔮",
            DialogueType::Linear => "🍺",
            DialogueType::Battle => "💀",
        };

        format!(
            "    ╔═══════════╗
    ║  {}  ║
    ║     {}     ║
    ║   ┌───┐   ║
    ║   │ {} │   ║
    ║   └───┘   ║
    ╚═══════════╝",
            eyes, mouth, symbol
        )
    }

    fn start_conversation(&mut self, npc_index: usize) {
        if npc_index >= self.npcs.len() {
            return;
        }

        self.current_npc = Some(npc_index);
        
        // Mark as met and get initial dialogue
        let (initial_dialogue, npc_name) = {
            let npc = &mut self.npcs[npc_index];
            if !npc.met_before {
                npc.met_before = true;
            }
            (npc.initial_dialogue, npc.name.clone())
        };

        self.current_dialogue = Some(initial_dialogue);
        self.selected_choice = 0;
        
        self.add_message(format!("Started conversation with {}", npc_name));
        self.conversation_log.clear();
    }

    fn select_choice(&mut self, choice_index: usize) {
        if let (Some(npc_idx), Some(dialogue_idx)) = (self.current_npc, self.current_dialogue) {
            // First, get all the data we need from the dialogue tree
            let (choice_data, shop_item) = if let Some(dialogue) = self.dialogue_tree.iter().find(|d| d.id == dialogue_idx) {
                if choice_index < dialogue.choices.len() {
                    let choice = &dialogue.choices[choice_index];
                    let choice_data = (
                        choice.text.clone(),
                        choice.mood_change,
                        choice.unlocks_quest,
                        choice.leads_to,
                    );
                    (Some(choice_data), dialogue.shop_item.clone())
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            if let Some((choice_text, mood_change, unlocks_quest, leads_to)) = choice_data {
                // Log the player's choice
                self.conversation_log.push(format!("You: {}", choice_text));
                
                // Handle mood changes
                if let Some(new_mood) = mood_change {
                    self.npcs[npc_idx].current_mood = new_mood;
                    self.add_message(format!("NPC mood changed to {:?}", new_mood));
                }

                // Handle quest unlocking
                if unlocks_quest {
                    self.add_message("New quest available!".to_string());
                }

                // Handle shop transactions
                if let Some(item) = shop_item {
                    // Simple shop logic
                    if choice_text.contains("take it") || choice_text.contains("buy") || choice_text.contains("I'll take") {
                        self.player.inventory.push(item.clone());
                        self.player.gold = self.player.gold.saturating_sub(25); // Simple price
                        self.add_message(format!("Purchased {} for 25 gold", item));
                    }
                }

                // Move to next dialogue or end conversation
                if let Some(next_dialogue) = leads_to {
                    self.current_dialogue = Some(next_dialogue);
                } else {
                    // End conversation
                    self.current_npc = None;
                    self.current_dialogue = None;
                    self.add_message("Conversation ended".to_string());
                }

                self.selected_choice = 0;
            }
        }
    }

    fn end_conversation(&mut self) {
        self.current_npc = None;
        self.current_dialogue = None;
        self.add_message("Conversation ended".to_string());
    }

    fn add_message(&mut self, msg: String) {
        self.message_log.push(msg);
        if self.message_log.len() > 10 {
            self.message_log.remove(0);
        }
    }

    fn get_current_dialogue(&self) -> Option<&DialogueNode> {
        if let Some(dialogue_idx) = self.current_dialogue {
            self.dialogue_tree.iter().find(|d| d.id == dialogue_idx)
        } else {
            None
        }
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            let num = c.to_digit(10).unwrap() as usize;
                            if num >= 1 && num <= 5 {
                                app.start_conversation(num - 1);
                            }
                        }
                        KeyCode::Up => {
                            if app.current_dialogue.is_some() {
                                if app.selected_choice > 0 {
                                    app.selected_choice -= 1;
                                }
                            }
                        }
                        KeyCode::Down => {
                            if let Some(dialogue) = app.get_current_dialogue() {
                                if app.selected_choice < dialogue.choices.len() - 1 {
                                    app.selected_choice += 1;
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if app.current_dialogue.is_some() {
                                app.select_choice(app.selected_choice);
                            }
                        }
                        KeyCode::Char('x') => {
                            if app.current_dialogue.is_some() {
                                app.end_conversation();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    if app.current_dialogue.is_some() {
        render_dialogue_screen(f, app, size);
    } else {
        render_npc_selection_screen(f, app, size);
    }
}

fn render_npc_selection_screen(f: &mut Frame, app: &App, area: Rect) {
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // NPCs
            Constraint::Length(8),  // Player info
            Constraint::Length(6),  // Message log
            Constraint::Length(3),  // Controls
        ])
        .split(area);

    // Title
    let title = Paragraph::new("🎭 Dialogue System Demo 🎭")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // NPCs grid
    let npc_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20); 5])
        .split(chunks[1]);

    for (i, npc) in app.npcs.iter().enumerate() {
        let mood_color = app.get_mood_color(npc.current_mood);
        let mood_prefix = app.get_mood_prefix(npc.current_mood);
        
        let npc_text = format!("{}\n{}\n\n{}{}",
            npc.portrait,
            npc.name,
            mood_prefix,
            format!("{:?}", npc.dialogue_type)
        );

        let mut block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", i + 1));
        
        if npc.met_before {
            block = block.border_style(Style::default().fg(Color::Green));
        }

        let npc_widget = Paragraph::new(npc_text)
            .block(block)
            .style(Style::default().fg(mood_color))
            .alignment(Alignment::Center);

        f.render_widget(npc_widget, npc_chunks[i]);
    }

    // Player info
    let player_info = format!(
        "Player: {} | Gold: {} | Items: {}",
        app.player.name,
        app.player.gold,
        app.player.inventory.len()
    );
    
    let inventory_text = app.player.inventory.join(", ");
    let full_player_text = format!("{}\nInventory: {}", player_info, inventory_text);

    let player_block = Block::default()
        .borders(Borders::ALL)
        .title(" Player Info ");
    
    let player_widget = Paragraph::new(full_player_text)
        .block(player_block)
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(player_widget, chunks[2]);

    // Message log
    let messages = app.message_log.join("\n");
    let log_block = Block::default()
        .borders(Borders::ALL)
        .title(" Messages ");
    
    let log_widget = Paragraph::new(messages)
        .block(log_block)
        .style(Style::default().fg(Color::White));

    f.render_widget(log_widget, chunks[3]);

    // Controls
    let controls = "1-5: Talk to NPC | q/Esc: Quit";
    let controls_widget = Paragraph::new(controls)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    f.render_widget(controls_widget, chunks[4]);
}

fn render_dialogue_screen(f: &mut Frame, app: &App, area: Rect) {
    if let (Some(npc_idx), Some(dialogue)) = (app.current_npc, app.get_current_dialogue()) {
        let npc = &app.npcs[npc_idx];
        
        // Summon Night style layout: portraits at top, message box at bottom
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // Portrait area
                Constraint::Min(1),     // Background/scene area
                Constraint::Length(12), // Message box area
                Constraint::Length(3),  // Controls
            ])
            .split(area);

        // Portrait area - side by side like Summon Night
        let portrait_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Player portrait
                Constraint::Percentage(40), // Empty space / scene info
                Constraint::Percentage(30), // NPC portrait
            ])
            .split(main_chunks[0]);

        // Player portrait (left side)
        let player_portrait = "
    ╔═══════════╗
    ║  ◉     ◉  ║
    ║     ◡     ║
    ║   ┌───┐   ║
    ║   │ 🗡️ │   ║
    ║   └───┘   ║
    ╚═══════════╝";

        let player_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", app.player.name))
            .border_style(Style::default().fg(Color::Cyan));
        
        let player_widget = Paragraph::new(player_portrait)
            .block(player_block)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);

        f.render_widget(player_widget, portrait_chunks[0]);

        // Scene info (center)
        let scene_info = format!("Location: Town Square\n\nGold: {}\nItems: {}", 
            app.player.gold, 
            app.player.inventory.len()
        );

        let scene_block = Block::default()
            .borders(Borders::ALL)
            .title(" Scene ")
            .border_style(Style::default().fg(Color::White));

        let scene_widget = Paragraph::new(scene_info)
            .block(scene_block)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);

        f.render_widget(scene_widget, portrait_chunks[1]);

        // NPC portrait (right side) - with dynamic mood
        let mood_color = app.get_mood_color(npc.current_mood);
        let npc_portrait = app.get_portrait_with_mood(npc);
        
        let npc_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", npc.name))
            .border_style(Style::default().fg(mood_color));
        
        let npc_widget = Paragraph::new(npc_portrait)
            .block(npc_block)
            .style(Style::default().fg(mood_color))
            .alignment(Alignment::Center);

        f.render_widget(npc_widget, portrait_chunks[2]);

        // Message box area (like Summon Night's dialogue box)
        let message_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Main dialogue text
                Constraint::Length(4),  // Choices/options
            ])
            .split(main_chunks[2]);

        // Main dialogue text with classic RPG styling
        let speaker_indicator = if dialogue.speaker == app.player.name {
            format!("🗡️ {}", dialogue.speaker)
        } else {
            format!("{} {}", app.get_mood_prefix(npc.current_mood), dialogue.speaker)
        };

        let dialogue_text = format!("{}\n\n\"{}\"", speaker_indicator, dialogue.text);
        
        let dialogue_block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .title(" Dialogue ")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Color::Yellow));
        
        let dialogue_widget = Paragraph::new(dialogue_text)
            .block(dialogue_block)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));

        f.render_widget(dialogue_widget, message_chunks[0]);

        // Choice selection area
        if !dialogue.choices.is_empty() {
            let choice_text = dialogue.choices
                .iter()
                .enumerate()
                .map(|(i, choice)| {
                    let prefix = if i == app.selected_choice { "→ " } else { "  " };
                    let style_marker = if i == app.selected_choice { "◆" } else { "◇" };
                    format!("{}{} {}", prefix, style_marker, choice.text)
                })
                .collect::<Vec<_>>()
                .join("\n");

            let choice_block = Block::default()
                .borders(Borders::ALL)
                .border_set(symbols::border::ROUNDED)
                .title(" Choose ")
                .border_style(Style::default().fg(Color::Green));

            let choice_widget = Paragraph::new(choice_text)
                .block(choice_block)
                .style(Style::default().fg(Color::White));

            f.render_widget(choice_widget, message_chunks[1]);
        }

        // Controls
        let controls = "↑↓: Select | Enter: Choose | x: End conversation | q: Quit";
        let controls_widget = Paragraph::new(controls)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        f.render_widget(controls_widget, main_chunks[3]);
    }
}