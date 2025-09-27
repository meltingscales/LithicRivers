use crate::app_state::{ConversationState, NPCMood};
use crate::sprite_loader::SpriteLoader;
use hecs::{Entity, World};
use lithicrivers_core::components::{NPCMood as CoreNPCMood, SpriteRef};
use lithicrivers_core::dialogue::DialogueTree;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

/// UI presentation layer for dialogue system - handles only formatting and display
pub struct DialoguePresenter;

impl DialoguePresenter {
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
            let dialogue_lines =
                Self::format_dialogue_lines(dialogue_tree, conversation, selected_choice);
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

            // Add speaker line with normal white color
            let speaker_text = format!("{}{}: \"{}\"", mood_prefix, node.speaker, node.text);
            lines.push(Line::from(Span::styled(
                speaker_text,
                Style::default().fg(Color::White),
            )));

            // Add empty line for spacing
            lines.push(Line::from(""));

            // Add choice lines with different colors
            for (i, choice) in node.choices.iter().enumerate() {
                let prefix = if i == selected_choice { ">" } else { " " };
                let choice_text = format!("{} {}. {}", prefix, i + 1, choice.text);

                let choice_color = if i == selected_choice {
                    Color::Yellow // Highlighted choice
                } else {
                    Color::Cyan // Available choices
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
