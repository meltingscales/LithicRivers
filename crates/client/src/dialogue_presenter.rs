use crate::app_state::{ConversationState, NPCMood};
use crate::sprite_loader::SpriteLoader;
use hecs::{Entity, World};
use lithicrivers_core::components::{NPCMood as CoreNPCMood, SpriteRef};
use lithicrivers_core::dialogue::{DialogueTree, TextEffect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::time::{SystemTime, UNIX_EPOCH};

/// UI presentation layer for dialogue system - handles only formatting and display
pub struct DialoguePresenter;

impl DialoguePresenter {
    /// Render text with TextEffect visual styling and animations
    fn render_text_with_effects(text: &str, effects: &[TextEffect]) -> Vec<Span<'static>> {
        if effects.is_empty() {
            return vec![Span::styled(
                text.to_string(),
                Style::default().fg(Color::White),
            )];
        }

        // Get current time for animations
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Find all effect segments first (without modifying the original text)
        let mut effect_segments = Vec::new();

        for (effect_idx, effect) in effects.iter().enumerate() {
            let open_tag = format!("<{}>", effect_idx + 1);
            let close_tag = format!("</{}>", effect_idx + 1);

            let mut search_pos = 0;
            while let Some(start) = text[search_pos..].find(&open_tag) {
                let actual_start = search_pos + start;
                if let Some(end_pos) = text[actual_start..].find(&close_tag) {
                    let actual_end = actual_start + end_pos;
                    let content_start = actual_start + open_tag.len();
                    let content = text[content_start..actual_end].to_string();

                    effect_segments.push((
                        actual_start,
                        actual_end + close_tag.len(),
                        content,
                        effect.clone(),
                    ));
                    search_pos = actual_end + close_tag.len();
                } else {
                    break;
                }
            }
        }

        // Sort segments by position
        effect_segments.sort_by_key(|&(start, _, _, _)| start);

        // Build spans
        let mut result = Vec::new();
        let mut current_pos = 0;

        for (start, end, content, effect) in effect_segments {
            // Add text before this effect
            if current_pos < start {
                let before_text = text[current_pos..start].to_string();
                if !before_text.is_empty() {
                    result.push(Span::styled(before_text, Style::default().fg(Color::White)));
                }
            }

            // Add effect text with animation and corruption
            if !content.is_empty() {
                let corrupted_text = Self::apply_text_corruption(&content, &effect, now);
                let animated_style = Self::get_animated_text_effect_style(&effect, now);
                result.push(Span::styled(corrupted_text, animated_style));
            }

            current_pos = end;
        }

        // Add remaining text
        if current_pos < text.len() {
            let remaining_text = text[current_pos..].to_string();
            if !remaining_text.is_empty() {
                result.push(Span::styled(
                    remaining_text,
                    Style::default().fg(Color::White),
                ));
            }
        }

        if result.is_empty() {
            vec![Span::styled(
                text.to_string(),
                Style::default().fg(Color::White),
            )]
        } else {
            result
        }
    }

    /// Get animated visual style for a specific TextEffect
    fn get_animated_text_effect_style(effect: &TextEffect, time_ms: u64) -> Style {
        match effect {
            TextEffect::Static => {
                // Flicker between light blue and blue
                let cycle = (time_ms / 200) % 2;
                if cycle == 0 {
                    Style::default().fg(Color::LightBlue)
                } else {
                    Style::default().fg(Color::Blue)
                }
            }
            TextEffect::Glitch => {
                // Rapid color cycling
                let colors = [Color::Magenta, Color::Red, Color::Cyan, Color::Yellow];
                let index = (time_ms / 100) % 4;
                Style::default().fg(colors[index as usize])
            }
            TextEffect::Corrupt => {
                // Pulsing red intensity
                let cycle = (time_ms / 300) % 3;
                match cycle {
                    0 => Style::default().fg(Color::Red),
                    1 => Style::default().fg(Color::LightRed),
                    _ => Style::default().fg(Color::DarkGray),
                }
            }
            TextEffect::Fade => {
                // Fade in and out
                let cycle = (time_ms / 500) % 2;
                if cycle == 0 {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Gray)
                }
            }
            TextEffect::Buzz => {
                // Fast yellow flicker
                let cycle = (time_ms / 150) % 2;
                if cycle == 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::LightYellow)
                }
            }
            TextEffect::Crackle => {
                // Cyan with occasional white flashes
                let cycle = (time_ms / 250) % 5;
                if cycle == 4 {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Cyan)
                }
            }
            TextEffect::PopHiss => {
                // Quick red flashes
                let cycle = (time_ms / 100) % 4;
                match cycle {
                    0 => Style::default().fg(Color::LightRed),
                    1 => Style::default().fg(Color::Red),
                    2 => Style::default().fg(Color::LightRed),
                    _ => Style::default().fg(Color::White),
                }
            }
        }
    }

    /// Apply text corruption effects for Glitch and Corrupt
    fn apply_text_corruption(text: &str, effect: &TextEffect, time_ms: u64) -> String {
        match effect {
            TextEffect::Glitch => {
                // Minecraft-style mystical glyphs
                let glitch_chars = [
                    'ᚴ', 'ᛖ', 'ᚱ', 'ᛈ', 'ᚲ', 'ᛚ', 'ᚢ', 'ᛏ', 'ᛒ', 'ᚦ', 'ᚠ', 'ᚨ', 'ᚱ', 'ᚲ', 'ᚷ', 'ᚹ',
                    'ᚺ', 'ᚾ', 'ᛁ', 'ᛃ', 'ᛇ', 'ᛈ', 'ᛉ', 'ᛊ', 'ᛏ', 'ᛒ', 'ᛖ', 'ᛗ', 'ᛚ', 'ᛜ', 'ᛟ', 'ᛞ',
                ];
                let mut result = String::new();
                let corruption_rate = 0.4; // 40% of characters get corrupted

                for (i, ch) in text.char_indices() {
                    // Use position and time for deterministic randomness
                    let seed = (time_ms / 200) + (i as u64) * 17;
                    let random_val = (seed * 1103515245 + 12345) % 100;

                    if ch.is_whitespace() || ch.is_ascii_punctuation() {
                        result.push(ch); // Keep spaces and punctuation
                    } else if (random_val as f64 / 100.0) < corruption_rate {
                        let glyph_index = (seed * 7) % (glitch_chars.len() as u64);
                        result.push(glitch_chars[glyph_index as usize]);
                    } else {
                        result.push(ch);
                    }
                }
                result
            }
            TextEffect::Corrupt => {
                // Random static/noise characters
                let static_chars = [
                    '█', '▓', '▒', '░', '▄', '▀', '■', '□', '▪', '▫', '●', '○', '◆', '◇', '★', '☆',
                    '#', '@', '%', '&', '*', '+', '=', '~', '^',
                ];
                let noise_chars = ['a', 'e', 'i', 'o', 'u', 'x', 'z', '0', '1', '2', '7', '9'];
                let mut result = String::new();
                let corruption_rate = 0.2; // what % of characters get corrupted

                for (i, ch) in text.char_indices() {
                    let seed = (time_ms / 1000) + (i as u64) * 23;
                    let random_val = (seed * 1103515245 + 12345) % 100;

                    if ch.is_whitespace() || ch.is_ascii_punctuation() {
                        result.push(ch); // Keep spaces and punctuation
                    } else if (random_val as f64 / 100.0) < corruption_rate {
                        // Mix of static blocks and noise characters
                        let use_static = (seed * 3) % 2 == 0;
                        if use_static {
                            let static_index = (seed * 11) % (static_chars.len() as u64);
                            result.push(static_chars[static_index as usize]);
                        } else {
                            let noise_index = (seed * 13) % (noise_chars.len() as u64);
                            result.push(noise_chars[noise_index as usize]);
                        }
                    } else {
                        result.push(ch);
                    }
                }
                result
            }
            _ => text.to_string(), // No corruption for other effects
        }
    }

    /// Convert dialogue mood to core NPC mood for sprite loader
    fn dialogue_mood_to_core_mood(mood: NPCMood) -> CoreNPCMood {
        match mood {
            NPCMood::Friendly => CoreNPCMood::Happy,
            NPCMood::Neutral => CoreNPCMood::Neutral,
            NPCMood::Hostile => CoreNPCMood::Weird, // Hostile maps to "weird" expression
            NPCMood::Sad => CoreNPCMood::Sad,
            NPCMood::Excited => CoreNPCMood::Happy, // Excited is also happy
            NPCMood::Mysterious => CoreNPCMood::Weird, // Mysterious maps to "weird" expression
        }
    }

    /// Get character portrait for NPC in current mood
    pub fn get_character_portrait(
        sprite_loader: &mut SpriteLoader,
        world: &World,
        npc_entity: Entity,
        mood: NPCMood,
    ) -> Option<String> {
        // Get the SpriteRef component directly from the entity instead of manual mapping
        if let Ok(sprite_ref) = world.get::<&SpriteRef>(npc_entity) {
            let core_mood = Self::dialogue_mood_to_core_mood(mood);
            sprite_loader.get_mood_portrait(&sprite_ref, core_mood)
        } else {
            None
        }
    }

    /// Format dialogue with Summon Night-style layout including both player and NPC portraits
    pub fn format_dialogue_with_portraits(
        sprite_loader: &mut SpriteLoader,
        world: &World,
        npc_entity: Entity,
        dialogue_tree: &DialogueTree,
        conversation: &ConversationState,
        selected_choice: usize,
        resources: &lithicrivers_core::resources::Resources,
    ) -> (Vec<Line<'static>>, Option<String>, Option<String>) {
        if let Some(node) = conversation
            .current_node_id
            .as_ref()
            .and_then(|id| dialogue_tree.get_node(id))
        {
            // Convert core NPCMood to client NPCMood for display
            let display_mood = match node.mood {
                CoreNPCMood::Happy => NPCMood::Friendly,
                CoreNPCMood::Sad => NPCMood::Sad,
                CoreNPCMood::Neutral => NPCMood::Neutral,
                CoreNPCMood::Weird => NPCMood::Mysterious,
            };
            let npc_portrait =
                Self::get_character_portrait(sprite_loader, world, npc_entity, display_mood);
            let player_portrait = Self::get_player_portrait(sprite_loader, conversation);
            let dialogue_lines = Self::format_dialogue_lines(
                dialogue_tree,
                conversation,
                selected_choice,
                resources,
                world,
            );
            (dialogue_lines, npc_portrait, player_portrait)
        } else {
            (
                vec![Line::from(Span::styled(
                    "No dialogue available",
                    Style::default().fg(Color::Red),
                ))],
                None,
                None,
            )
        }
    }

    /// Get player portrait based on current dialogue context (player mood/response)
    pub fn get_player_portrait(
        sprite_loader: &mut SpriteLoader,
        conversation: &ConversationState,
    ) -> Option<String> {
        let sprite_ref = SpriteRef {
            category: "entities".to_string(),
            name: "player".to_string(),
        };

        // Use the player's current mood from conversation state
        let core_mood = Self::dialogue_mood_to_core_mood(conversation.player_mood);
        sprite_loader.get_mood_portrait(&sprite_ref, core_mood)
    }

    /// Format dialogue with styled lines for colored choices
    pub fn format_dialogue_lines(
        dialogue_tree: &DialogueTree,
        conversation: &ConversationState,
        selected_choice: usize,
        resources: &lithicrivers_core::resources::Resources,
        world: &World,
    ) -> Vec<Line<'static>> {
        if let Some(node) = conversation
            .current_node_id
            .as_ref()
            .and_then(|id| dialogue_tree.get_node(id))
        {
            let mut lines = Vec::new();

            // Convert core NPCMood to client NPCMood for display
            let display_mood = match node.mood {
                CoreNPCMood::Happy => NPCMood::Friendly,
                CoreNPCMood::Sad => NPCMood::Sad,
                CoreNPCMood::Neutral => NPCMood::Neutral,
                CoreNPCMood::Weird => NPCMood::Mysterious,
            };
            let mood_prefix = Self::get_mood_prefix(display_mood);

            // Add speaker line with text effects
            let speaker_prefix = format!("{}{}: \"", mood_prefix, node.speaker);
            let quote_suffix = "\"";

            let mut speaker_spans = vec![Span::styled(
                speaker_prefix,
                Style::default().fg(Color::White),
            )];
            speaker_spans.extend(Self::render_text_with_effects(
                &node.text,
                &node.text_effects,
            ));
            speaker_spans.push(Span::styled(
                quote_suffix,
                Style::default().fg(Color::White),
            ));

            lines.push(Line::from(speaker_spans));

            // Add empty line for spacing
            lines.push(Line::from(""));

            // Get player entity for item checks
            let player_entity = world
                .query::<&lithicrivers_core::components::Player>()
                .iter()
                .next()
                .map(|(entity, _)| entity);

            // Filter choices based on quest and item requirements
            let filtered_choices: Vec<(usize, &lithicrivers_core::dialogue::DialogueChoice)> = node
                .choices
                .iter()
                .enumerate()
                .filter(|(_, choice)| {
                    // Check quest requirements
                    if let Some(required_quest) = &choice.requires_quest_active {
                        if !resources.is_quest_active(*required_quest) {
                            return false;
                        }
                    }
                    if let Some(required_quest) = &choice.requires_quest_complete {
                        if !resources.is_quest_completed(*required_quest) {
                            return false;
                        }
                    }

                    // Check item requirements
                    if let Some(required_item) = &choice.requires_item {
                        if let Some(player_entity) = player_entity {
                            // Special case for quest completion choices - check if all quest objectives are met
                            if choice.leads_to == Some(lithicrivers_core::dialogue::DialogueNodeID::CompleteQuest) {
                                // For quest completion, check if the active quest has all items needed
                                if let Some(required_quest) = &choice.requires_quest_active {
                                    let quest_items_ready = if let Ok(inventory) = world.get::<&lithicrivers_core::components::Inventory>(player_entity) {
                                        // Check if player has all items needed for this quest
                                        match required_quest {
                                            lithicrivers_core::dialogue::QuestType::RepairBrokenAndroid => {
                                                let has_diamond = inventory.slots.iter().any(|stack| {
                                                    lithicrivers_core::components::itemkind_name(stack.kind) == "Diamond" && stack.qty > 0
                                                });
                                                let has_electronics = inventory.slots.iter().any(|stack| {
                                                    lithicrivers_core::components::itemkind_name(stack.kind) == "Scrap Electronics" && stack.qty > 0
                                                });
                                                has_diamond && has_electronics
                                            }
                                        }
                                    } else {
                                        false
                                    };
                                    if !quest_items_ready {
                                        return false;
                                    }
                                }
                            } else {
                                // Standard single item requirement check
                                let has_item = if let Ok(inventory) = world.get::<&lithicrivers_core::components::Inventory>(player_entity) {
                                    inventory.slots.iter().any(|stack| {
                                        lithicrivers_core::components::itemkind_name(stack.kind) == required_item
                                            && stack.qty > 0
                                    })
                                } else {
                                    false
                                };
                                if !has_item {
                                    return false;
                                }
                            }
                        } else {
                            // No player entity found, can't check items
                            return false;
                        }
                    }

                    true
                })
                .collect();

            // Add choice lines with different colors
            for (display_index, (_original_index, choice)) in filtered_choices.iter().enumerate() {
                let prefix = if display_index == selected_choice {
                    ">"
                } else {
                    " "
                };

                // Add quest marker for choices that unlock quests
                let quest_marker = if choice.unlocks_quest.is_some() {
                    " [ACCEPT QUEST]"
                } else {
                    ""
                };
                let choice_text = format!(
                    "{} {}. {} {}",
                    prefix,
                    display_index + 1,
                    quest_marker,
                    choice.text
                );

                let choice_color =
                    if choice.unlocks_quest.is_some() && display_index == selected_choice {
                        Color::LightYellow // Highlighted quest choice
                    } else if choice.unlocks_quest.is_some() {
                        Color::Green // Available quest choice
                    } else if display_index == selected_choice {
                        Color::Yellow // Highlighted regular choice
                    } else {
                        Color::Cyan // Available regular choices
                    };

                lines.push(Line::from(Span::styled(
                    choice_text,
                    Style::default().fg(choice_color),
                )));
            }

            lines
        } else {
            vec![Line::from(Span::styled(
                "No dialogue available",
                Style::default().fg(Color::Red),
            ))]
        }
    }

    /// Get mood prefix for text display
    pub fn get_mood_prefix(mood: NPCMood) -> &'static str {
        match mood {
            NPCMood::Friendly => "> ",
            NPCMood::Neutral => "- ",
            NPCMood::Hostile => "! ",
            NPCMood::Sad => "~ ",
            NPCMood::Excited => "* ",
            NPCMood::Mysterious => "? ",
        }
    }
}
