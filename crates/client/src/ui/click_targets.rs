//! Click targets: every widget that draws clickable rows records each visible row's screen
//! area while drawing, and mouse clicks are resolved only from these records, so what you
//! click is always what's on screen. Rebuilt every frame.

use crate::MenuTab;
use ratatui::layout::Rect;
use std::cell::RefCell;

/// What a recorded area is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickTarget {
    Tab(MenuTab),
    /// A window's whole area: clicks inside it never reach what is under it
    ModalArea,
    TitleChoice(usize),
    WorldRow(usize),
    WorldAction(usize),
    DeleteChoice(usize),
    CreateField(usize),
    OptionsRow(usize),
    InteractionChoice(usize),
    DialogueChoice(usize),
    FilterRow(usize),
    BlockPickerRow(usize),
    CorpsePickerRow(usize),
    CombatMove(usize),
    Suggestion(usize),
    /// A row of a two-panel item window: the container's panel or Your Items
    TransferRow {
        container: bool,
        row: usize,
    },
    FurnaceSlot(usize),
    FurnacePlayerRow(usize),
    RecipeRow(usize),
    DictionaryRow(usize),
    BlueprintRow(usize),
    StairsChoice(usize),
    DeathChoice(usize),
}

/// The inside of a bordered box.
pub fn inset(rect: Rect) -> Rect {
    Rect::new(
        rect.x + 1,
        rect.y + 1,
        rect.width.saturating_sub(2),
        rect.height.saturating_sub(2),
    )
}

/// The result of resolving a click against the recorded targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolved {
    Target(ClickTarget),
    /// Outside the topmost open window: close it
    Outside,
    /// No target and no window: the world (or nothing) handles it
    Nothing,
}

#[derive(Debug, Default)]
pub struct ClickTargets(RefCell<Vec<(Rect, ClickTarget)>>);

impl ClickTargets {
    pub fn clear(&self) {
        self.0.borrow_mut().clear();
    }

    pub fn push(&self, rect: Rect, target: ClickTarget) {
        self.0.borrow_mut().push((rect, target));
    }

    /// Record `count` one-line rows starting at the top of `area` (clipped to it); row `i`
    /// on screen is item `first + i`.
    pub fn rows(
        &self,
        area: Rect,
        first: usize,
        count: usize,
        target: impl Fn(usize) -> ClickTarget,
    ) {
        for i in 0..count.min(area.height as usize) {
            let rect = Rect::new(area.x, area.y + i as u16, area.width, 1);
            self.push(rect, target(first + i));
        }
    }

    /// The topmost target (the last one drawn) under the cell.
    pub fn hit(&self, col: u16, row: u16) -> Option<(Rect, ClickTarget)> {
        self.hit_index(col, row).map(|i| self.0.borrow()[i])
    }

    fn hit_index(&self, col: u16, row: u16) -> Option<usize> {
        self.0.borrow().iter().rposition(|(r, _)| {
            (r.x..r.x + r.width).contains(&col) && (r.y..r.y + r.height).contains(&row)
        })
    }

    /// What a click at the cell means: a target of the topmost window (or of the screen,
    /// when no window is open), or `Outside` an open window.
    pub fn resolve(&self, col: u16, row: u16) -> Resolved {
        let targets = self.0.borrow();
        let window = targets
            .iter()
            .rposition(|(_, t)| *t == ClickTarget::ModalArea);
        drop(targets);
        match (self.hit_index(col, row), window) {
            (Some(hit), Some(w)) if hit < w => Resolved::Outside,
            (None, Some(_)) => Resolved::Outside,
            (Some(hit), _) => Resolved::Target(self.0.borrow()[hit].1),
            (None, None) => Resolved::Nothing,
        }
    }
}
