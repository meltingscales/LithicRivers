//! Gamepad input (`gamepad-input` spec): a pure mapping from pad events to the key presses and
//! mouse-wheel steps the game already handles, shared by the game window and headless runs.
//!
//! The layout lives in the config's `gamepad` settings: each button or stick names a
//! keybind (`category.ACTION`, whose first key is sent) or a special action. Held stick and
//! D-pad directions repeat from the client clock, so headless runs stay deterministic.
// Mobile builds read no gamepads (the touch pad is used there)
#![cfg_attr(target_os = "android", allow(dead_code))]

use std::collections::BTreeSet;

use crossterm::event::KeyCode;

/// A gamepad button, by position (Xbox names: South is A, East B, West X, North Y).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PadButton {
    South,
    East,
    West,
    North,
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    Select,
    Start,
    LeftThumb,
    RightThumb,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

impl PadButton {
    /// Buttons with their config names, script names and labels.
    pub const ALL: [(PadButton, &'static str, &'static str, &'static str); 12] = [
        (PadButton::South, "SOUTH", "a", "A"),
        (PadButton::East, "EAST", "b", "B"),
        (PadButton::West, "WEST", "x", "X"),
        (PadButton::North, "NORTH", "y", "Y"),
        (PadButton::LeftBumper, "LEFT_BUMPER", "lb", "LB"),
        (PadButton::RightBumper, "RIGHT_BUMPER", "rb", "RB"),
        (PadButton::LeftTrigger, "LEFT_TRIGGER", "lt", "LT"),
        (PadButton::RightTrigger, "RIGHT_TRIGGER", "rt", "RT"),
        (PadButton::Select, "SELECT", "view", "View"),
        (PadButton::Start, "START", "menu", "Menu"),
        (PadButton::LeftThumb, "LEFT_THUMB", "l3", "L3"),
        (PadButton::RightThumb, "RIGHT_THUMB", "r3", "R3"),
    ];

    /// The button a script names (`a`, `rb`, `view`...); D-pad buttons are stick steps.
    pub fn from_script(name: &str) -> Option<PadButton> {
        Self::ALL
            .iter()
            .find(|(_, _, s, _)| s.eq_ignore_ascii_case(name))
            .map(|&(b, ..)| b)
    }

    pub fn script_name(self) -> &'static str {
        Self::ALL
            .iter()
            .find(|(b, ..)| *b == self)
            .map_or("dpad", |(_, _, s, _)| s)
    }

    pub fn label(self) -> &'static str {
        Self::ALL
            .iter()
            .find(|(b, ..)| *b == self)
            .map_or("D-pad", |(_, _, _, l)| l)
    }

    fn dpad_dir(self) -> Option<(i8, i8)> {
        match self {
            PadButton::DPadUp => Some((0, -1)),
            PadButton::DPadDown => Some((0, 1)),
            PadButton::DPadLeft => Some((-1, 0)),
            PadButton::DPadRight => Some((1, 0)),
            _ => None,
        }
    }
}

/// A stick, or the D-pad treated as one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stick {
    Left,
    Right,
    DPad,
}

impl Stick {
    pub const ALL: [(Stick, &'static str, &'static str); 3] = [
        (Stick::Left, "lstick", "LEFT_STICK"),
        (Stick::Right, "rstick", "RIGHT_STICK"),
        (Stick::DPad, "dpad", "DPAD"),
    ];

    pub fn from_script(name: &str) -> Option<Stick> {
        Self::ALL
            .iter()
            .find(|(_, s, _)| s.eq_ignore_ascii_case(name))
            .map(|&(stick, ..)| stick)
    }

    pub fn script_name(self) -> &'static str {
        Self::ALL
            .iter()
            .find(|(s, ..)| *s == self)
            .map_or("", |s| s.1)
    }
}

/// One of the eight directions, north up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir8 {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Dir8 {
    pub const ALL: [Dir8; 8] = [
        Dir8::N,
        Dir8::NE,
        Dir8::E,
        Dir8::SE,
        Dir8::S,
        Dir8::SW,
        Dir8::W,
        Dir8::NW,
    ];

    /// The keybind suffix: `NORTH`, `NORTHEAST`...
    pub fn suffix(self) -> &'static str {
        match self {
            Dir8::N => "NORTH",
            Dir8::NE => "NORTHEAST",
            Dir8::E => "EAST",
            Dir8::SE => "SOUTHEAST",
            Dir8::S => "SOUTH",
            Dir8::SW => "SOUTHWEST",
            Dir8::W => "WEST",
            Dir8::NW => "NORTHWEST",
        }
    }

    pub fn script_name(self) -> &'static str {
        match self {
            Dir8::N => "n",
            Dir8::NE => "ne",
            Dir8::E => "e",
            Dir8::SE => "se",
            Dir8::S => "s",
            Dir8::SW => "sw",
            Dir8::W => "w",
            Dir8::NW => "nw",
        }
    }

    pub fn from_script(name: &str) -> Option<Dir8> {
        Self::ALL
            .into_iter()
            .find(|d| d.script_name().eq_ignore_ascii_case(name))
    }

    /// From a step (`dx`, `dy`), y down; `None` for no step.
    pub fn from_step(dx: i8, dy: i8) -> Option<Dir8> {
        Some(match (dx.signum(), dy.signum()) {
            (0, -1) => Dir8::N,
            (1, -1) => Dir8::NE,
            (1, 0) => Dir8::E,
            (1, 1) => Dir8::SE,
            (0, 1) => Dir8::S,
            (-1, 1) => Dir8::SW,
            (-1, 0) => Dir8::W,
            (-1, -1) => Dir8::NW,
            _ => return None,
        })
    }
}

/// How far a stick must be pushed (0 to 1) before it counts.
pub const DEAD_ZONE: f32 = 0.5;
/// A held direction repeats after this long, then this often.
pub const REPEAT_DELAY_MS: u64 = 250;
pub const REPEAT_EVERY_MS: u64 = 120;

/// A stick position (x right, y down, each -1 to 1) as one of 8 directions, or `None`
/// inside the dead zone.
pub fn quantize(x: f32, y: f32) -> Option<Dir8> {
    if (x * x + y * y).sqrt() < DEAD_ZONE {
        return None;
    }
    // 0 is east, counting clockwise (y is down); each direction owns a 45-degree slice
    let angle = y.atan2(x).to_degrees();
    let slice = (((angle + 360.0 + 22.5) % 360.0) / 45.0) as usize % 8;
    Some(
        [
            Dir8::E,
            Dir8::SE,
            Dir8::S,
            Dir8::SW,
            Dir8::W,
            Dir8::NW,
            Dir8::N,
            Dir8::NE,
        ][slice],
    )
}

/// What a button or stick does, from the config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PadAction {
    /// Press the first key bound to `category.ACTION`
    Keybind(String, String),
    /// Interact on the World page, Enter elsewhere
    Confirm,
    /// One mouse wheel step
    Wheel { up: bool },
    /// The break or place key for a direction, by build mode
    BuildAim,
    /// The move key for a direction
    Movement,
    /// Go to a menu tab, by its config name (`MENU`, `INVENTORY`...)
    Tab(String),
}

impl PadAction {
    pub fn parse(value: &str) -> Option<PadAction> {
        Some(match value {
            "context.CONFIRM" => PadAction::Confirm,
            "wheel.UP" => PadAction::Wheel { up: true },
            "wheel.DOWN" => PadAction::Wheel { up: false },
            "build.AIM" => PadAction::BuildAim,
            "movement" => PadAction::Movement,
            other if other.starts_with("tab.") => PadAction::Tab(other[4..].to_string()),
            other => {
                let (category, action) = other.split_once('.')?;
                PadAction::Keybind(category.to_string(), action.to_string())
            }
        })
    }
}

impl PadAction {
    /// A short name for a wheel slot: "pick up", "torch", "rotate"...
    pub fn label(&self) -> String {
        let known = |c: &str, a: &str| -> Option<&'static str> {
            Some(match (c, a) {
                ("action", "PICKUP_ITEMS") => "pick up",
                ("action", "TOGGLE_TORCH") => "torch",
                ("action", "PASS_TIME") => "pass time",
                ("action", "LOOK_TOGGLE") => "look",
                ("action", "INTERACT") => "interact",
                ("action", "MINE") => "mine",
                ("ui", "TOGGLE_CONTEXT_HELP") => "help panel",
                ("build", "BLUEPRINT_MODE") => "blueprints",
                ("build", "BLUEPRINT_LIBRARY") => "bp library",
                ("build", "NEW_BLUEPRINT") => "new bp",
                ("build", "ROTATE_PLACEMENT") => "rotate",
                ("build", "TOGGLE_BREAK_PLACE_MODE") => "next mode",
                ("inventory", "USE_ITEM") => "use",
                ("inventory", "DROP_ITEM") => "drop",
                ("inventory", "TOGGLE_ITEM_AUTO_PICKUP_KEY") => "auto-pickup",
                _ => return None,
            })
        };
        match self {
            PadAction::Keybind(c, a) => known(c, a)
                .map(str::to_string)
                .unwrap_or_else(|| a.to_lowercase().replace('_', " ")),
            PadAction::Confirm => "confirm".to_string(),
            PadAction::Wheel { up: true } => "prev slot".to_string(),
            PadAction::Wheel { up: false } => "next slot".to_string(),
            PadAction::BuildAim => "aim".to_string(),
            PadAction::Movement => "move".to_string(),
            PadAction::Tab(name) => name.to_lowercase(),
        }
    }
}

/// Where a context wheel opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelKind {
    /// The World page in movement mode
    World,
    /// The World page in Break, Place or Rotate mode
    Build,
    /// The Inventory tab
    Inventory,
}

impl WheelKind {
    pub const ALL: [(WheelKind, &'static str); 3] = [
        (WheelKind::World, "WORLD"),
        (WheelKind::Build, "BUILD"),
        (WheelKind::Inventory, "INVENTORY"),
    ];

    pub fn title(self) -> &'static str {
        match self {
            WheelKind::World => "Actions",
            WheelKind::Build => "Building",
            WheelKind::Inventory => "Inventory",
        }
    }
}

/// How long LB must be held to open a wheel instead of acting.
pub const HOLD_MS: u64 = 250;

/// A wheel's eight slots, in `Dir8::ALL` order (N, NE, E, ... NW).
pub type WheelSlots = [Option<PadAction>; 8];

/// The layout: what each button and stick does, and the wheels.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Layout {
    pub buttons: Vec<(PadButton, PadAction)>,
    pub sticks: Vec<(Stick, PadAction)>,
    pub wheels: Vec<(WheelKind, WheelSlots)>,
}

impl Layout {
    /// Read the layout from the config's `gamepad` settings, warning on entries it doesn't
    /// understand.
    pub fn from_config(config: &lithicrivers_core::config::ConfigManager) -> Self {
        let Some(section) = config
            .data
            .settings
            .get("gamepad")
            .and_then(|v| v.as_object())
        else {
            return Self::default();
        };
        let mut layout = Self::default();
        for (name, value) in section {
            let action = value.as_str().and_then(PadAction::parse);
            let button = PadButton::ALL
                .iter()
                .find(|(_, c, ..)| c == name)
                .map(|&(b, ..)| b);
            let stick = Stick::ALL
                .iter()
                .find(|(_, _, c)| c == name)
                .map(|&(s, ..)| s);
            match (action, button, stick) {
                (Some(a), Some(b), _) => layout.buttons.push((b, a)),
                (Some(a), None, Some(s)) => layout.sticks.push((s, a)),
                _ => tracing::warn!(target: "game", "unknown gamepad entry {name}: {value}"),
            }
        }
        layout.buttons.sort_by_key(|(b, _)| *b);
        if let Some(wheels) = config
            .data
            .settings
            .get("gamepad_wheels")
            .and_then(|v| v.as_object())
        {
            for &(kind, name) in &WheelKind::ALL {
                let Some(list) = wheels.get(name).and_then(|v| v.as_array()) else {
                    continue;
                };
                let mut slots: WheelSlots = Default::default();
                for (slot, value) in slots.iter_mut().zip(list) {
                    let text = value.as_str().unwrap_or("");
                    if text.is_empty() {
                        continue;
                    }
                    *slot = PadAction::parse(text);
                    if slot.is_none() {
                        tracing::warn!(target: "game", "unknown gamepad wheel slot {name}: {text}");
                    }
                }
                layout.wheels.push((kind, slots));
            }
        }
        layout
    }

    /// The slots of a wheel, if the layout has one.
    pub fn wheel(&self, kind: WheelKind) -> Option<&WheelSlots> {
        self.wheels.iter().find(|(k, _)| *k == kind).map(|(_, s)| s)
    }

    fn button(&self, button: PadButton) -> Option<&PadAction> {
        self.buttons
            .iter()
            .find(|(b, _)| *b == button)
            .map(|(_, a)| a)
    }

    fn stick(&self, stick: Stick) -> Option<&PadAction> {
        self.sticks
            .iter()
            .find(|(s, _)| *s == stick)
            .map(|(_, a)| a)
    }

    /// The label of the control that sends `category.ACTION`, for hints: "A" for
    /// Interact (A confirms on the World page), "L-stick" for movement.
    pub fn label_for(&self, category: &str, action: &str) -> Option<&'static str> {
        if category == "movement"
            && action.starts_with("MOVE_")
            && self.stick(Stick::Left) == Some(&PadAction::Movement)
        {
            return Some("L-stick");
        }
        let aimed = ["BREAK_", "PLACE_"]
            .iter()
            .any(|p| action.starts_with(p) && !action.ends_with("CENTER"));
        if category == "build" && aimed && self.stick(Stick::Right) == Some(&PadAction::BuildAim) {
            return Some("R-stick");
        }
        let wanted = |a: &PadAction| match a {
            PadAction::Keybind(c, x) => c == category && x == action,
            PadAction::Confirm => {
                (category, action) == ("action", "INTERACT")
                    || (category, action) == ("ui", "MENU_ACTIVATE")
            }
            _ => false,
        };
        self.buttons
            .iter()
            .find(|(_, a)| wanted(a))
            .map(|(b, _)| b.label())
    }

    /// Every keybind the layout names, for checking it against the keybinds.
    #[cfg(test)]
    pub fn keybinds(&self) -> Vec<(String, String)> {
        self.buttons
            .iter()
            .map(|(_, a)| a)
            .chain(self.sticks.iter().map(|(_, a)| a))
            .chain(self.wheels.iter().flat_map(|(_, s)| s.iter().flatten()))
            .filter_map(|a| match a {
                PadAction::Keybind(c, x) => Some((c.clone(), x.clone())),
                _ => None,
            })
            .collect()
    }
}

/// What the app looks like to the mapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PadContext {
    /// On the World page with no window, menu, prompt or fight open
    pub world_free: bool,
    pub build_mode: crate::app_state::BuildMode,
    /// A text field takes keys
    pub typing: bool,
    /// The wheel holding LB opens here, if any
    pub wheel: Option<WheelKind>,
}

/// A pad event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadEvent {
    Button {
        button: PadButton,
        pressed: bool,
    },
    /// A stick's direction now (`None`: back in the dead zone)
    Stick {
        stick: Stick,
        dir: Option<Dir8>,
    },
}

/// What the game receives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadOutput {
    Key(KeyCode),
    Wheel {
        up: bool,
    },
    /// Open a menu tab, by config name (there is no single key for it)
    Tab(&'static str),
}

/// LB held: waiting to see if it is a tap, or a wheel is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Hold {
    since_ms: u64,
    kind: WheelKind,
    open: bool,
    highlight: Option<Dir8>,
}

/// The mapping, with what is held.
#[derive(Debug, Clone, Default)]
pub struct Mapper {
    pub layout: Layout,
    /// D-pad buttons held, which combine into diagonals
    dpad: BTreeSet<PadButton>,
    /// The held direction, which stick it is on, and when it next repeats (ms)
    held: Option<(Stick, Dir8, u64)>,
    /// LB held for a wheel
    hold: Option<Hold>,
}

/// The first key bound to `category.ACTION`.
pub type KeyLookup<'a> = &'a dyn Fn(&str, &str) -> Option<KeyCode>;

impl Mapper {
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
            ..Default::default()
        }
    }

    /// Handle one event at `now_ms`.
    pub fn handle(
        &mut self,
        event: PadEvent,
        ctx: PadContext,
        now_ms: u64,
        keys: KeyLookup,
    ) -> Vec<PadOutput> {
        // An open wheel takes directions and B; releasing LB picks
        if let Some(hold) = self.hold {
            if let Some(out) = self.wheel_event(hold, event, ctx, keys) {
                return out;
            }
        }
        if event
            == (PadEvent::Button {
                button: PadButton::LeftBumper,
                pressed: true,
            })
            && !ctx.typing
        {
            if let Some(kind) = ctx.wheel.filter(|&k| self.layout.wheel(k).is_some()) {
                self.hold = Some(Hold {
                    since_ms: now_ms,
                    kind,
                    open: false,
                    highlight: None,
                });
                return Vec::new();
            }
        }
        match event {
            PadEvent::Button { button, pressed } if button.dpad_dir().is_some() => {
                if pressed {
                    self.dpad.insert(button);
                } else {
                    self.dpad.remove(&button);
                }
                let (dx, dy) = self
                    .dpad
                    .iter()
                    .filter_map(|b| b.dpad_dir())
                    .fold((0, 0), |(x, y), (dx, dy)| (x + dx, y + dy));
                self.direction(Stick::DPad, Dir8::from_step(dx, dy), ctx, now_ms, keys)
            }
            PadEvent::Button {
                button,
                pressed: true,
            } => {
                // While typing, only B (leaving the field) gets through
                if ctx.typing && button != PadButton::East {
                    return Vec::new();
                }
                match self.layout.button(button).cloned() {
                    Some(action) => self.act(&action, None, ctx, keys),
                    None => Vec::new(),
                }
            }
            PadEvent::Button { pressed: false, .. } => Vec::new(),
            PadEvent::Stick { stick, dir } => self.direction(stick, dir, ctx, now_ms, keys),
        }
    }

    /// A stick now points `dir`: send it once if it changed, and hold it for repeats.
    fn direction(
        &mut self,
        stick: Stick,
        dir: Option<Dir8>,
        ctx: PadContext,
        now_ms: u64,
        keys: KeyLookup,
    ) -> Vec<PadOutput> {
        let held = self.held.filter(|(s, ..)| *s == stick).map(|(_, d, _)| d);
        if held == dir {
            return Vec::new();
        }
        match dir {
            None => {
                self.held = None;
                Vec::new()
            }
            Some(dir) => {
                self.held = Some((stick, dir, now_ms + REPEAT_DELAY_MS));
                self.stick_output(stick, dir, ctx, keys)
            }
        }
    }

    fn stick_output(
        &self,
        stick: Stick,
        dir: Dir8,
        ctx: PadContext,
        keys: KeyLookup,
    ) -> Vec<PadOutput> {
        if ctx.typing {
            return Vec::new();
        }
        match self.layout.stick(stick).cloned() {
            Some(action) => self.act(&action, Some(dir), ctx, keys),
            None => Vec::new(),
        }
    }

    /// An event while LB is held. `None` lets it through as usual (a held but not yet
    /// open wheel lets directions move).
    fn wheel_event(
        &mut self,
        hold: Hold,
        event: PadEvent,
        ctx: PadContext,
        keys: KeyLookup,
    ) -> Option<Vec<PadOutput>> {
        match event {
            PadEvent::Button {
                button: PadButton::LeftBumper,
                pressed: false,
            } => {
                self.hold = None;
                if !hold.open {
                    // A tap: LB's own action
                    let action = self.layout.button(PadButton::LeftBumper).cloned();
                    return Some(action.map_or_else(Vec::new, |a| self.act(&a, None, ctx, keys)));
                }
                let slot = hold.highlight.and_then(|d| {
                    let i = Dir8::ALL.iter().position(|&x| x == d)?;
                    self.layout.wheel(hold.kind)?[i].clone()
                });
                Some(slot.map_or_else(Vec::new, |a| self.act(&a, None, ctx, keys)))
            }
            _ if !hold.open => None,
            PadEvent::Button {
                button: PadButton::East,
                pressed: true,
            } => {
                self.hold = None;
                Some(Vec::new())
            }
            PadEvent::Stick { dir, .. } => {
                if let (Some(d), Some(h)) = (dir, self.hold.as_mut()) {
                    h.highlight = Some(d);
                }
                Some(Vec::new())
            }
            PadEvent::Button { button, pressed } if button.dpad_dir().is_some() => {
                if pressed {
                    self.dpad.insert(button);
                } else {
                    self.dpad.remove(&button);
                }
                let (dx, dy) = self
                    .dpad
                    .iter()
                    .filter_map(|b| b.dpad_dir())
                    .fold((0, 0), |(x, y), (dx, dy)| (x + dx, y + dy));
                if let (Some(d), Some(h)) = (Dir8::from_step(dx, dy), self.hold.as_mut()) {
                    h.highlight = Some(d);
                }
                Some(Vec::new())
            }
            // Other buttons wait until the wheel closes
            PadEvent::Button { .. } => Some(Vec::new()),
        }
    }

    /// The open wheel and its highlighted slot, for drawing.
    pub fn open_wheel(&self) -> Option<(WheelKind, Option<Dir8>)> {
        self.hold.filter(|h| h.open).map(|h| (h.kind, h.highlight))
    }

    /// Repeat the held direction while it is held, and open a wheel once LB has been held
    /// long enough: call once a frame.
    pub fn tick(&mut self, ctx: PadContext, now_ms: u64, keys: KeyLookup) -> Vec<PadOutput> {
        if let Some(h) = self.hold.as_mut() {
            if !h.open && now_ms >= h.since_ms + HOLD_MS {
                h.open = true;
                // The wheel takes the stick now; nothing keeps repeating under it
                self.held = None;
            }
            if h.open {
                return Vec::new();
            }
        }
        let mut out = Vec::new();
        while let Some((stick, dir, next)) = self.held {
            if next > now_ms {
                break;
            }
            self.held = Some((stick, dir, next + REPEAT_EVERY_MS));
            out.extend(self.stick_output(stick, dir, ctx, keys));
        }
        out
    }

    /// Release everything held (the pad was unplugged or turned off).
    pub fn release_all(&mut self) {
        self.dpad.clear();
        self.held = None;
        self.hold = None;
    }

    fn act(
        &self,
        action: &PadAction,
        dir: Option<Dir8>,
        ctx: PadContext,
        keys: KeyLookup,
    ) -> Vec<PadOutput> {
        use crate::app_state::BuildMode;
        let key = |c: &str, a: &str| keys(c, a).map(PadOutput::Key).into_iter().collect();
        match action {
            PadAction::Keybind(c, a) => key(c, a),
            PadAction::Confirm if ctx.world_free => key("action", "INTERACT"),
            PadAction::Confirm => vec![PadOutput::Key(KeyCode::Enter)],
            PadAction::Wheel { up } => vec![PadOutput::Wheel { up: *up }],
            PadAction::Movement => match dir {
                Some(d) => key("movement", &format!("MOVE_{}", d.suffix())),
                None => Vec::new(),
            },
            PadAction::BuildAim => {
                let prefix = match ctx.build_mode {
                    _ if !ctx.world_free => return Vec::new(),
                    BuildMode::Break => "BREAK",
                    BuildMode::Place | BuildMode::Rotate => "PLACE",
                    _ => return Vec::new(),
                };
                match dir {
                    Some(d) => key("build", &format!("{prefix}_{}", d.suffix())),
                    None => Vec::new(),
                }
            }
            PadAction::Tab(name) => match crate::MenuTab::from_config_name(name) {
                Some(tab) => vec![PadOutput::Tab(tab.config_name())],
                None => Vec::new(),
            },
        }
    }
}

impl crate::App {
    /// How hints name the control for `category.ACTION`: its gamepad button after the
    /// player last used a pad (for actions in the layout), else its key.
    pub(crate) fn key_label(&self, category: &str, action: &str) -> &'static str {
        if self.ui.last_input == InputDevice::Gamepad {
            if let Some(label) = self.ui.gamepad.layout.label_for(category, action) {
                return label;
            }
        }
        self.core
            .config_manager
            .get_printable_key_for_keybind(category, action)
    }

    /// Whether holding LB opens a wheel here, for the context help panel.
    pub(crate) fn pad_wheel_here(&self) -> bool {
        self.ui.last_input == InputDevice::Gamepad
            && context(self)
                .wheel
                .is_some_and(|k| self.ui.gamepad.layout.wheel(k).is_some())
    }
}

/// Which device the player used last, for button prompts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InputDevice {
    #[default]
    Keyboard,
    Gamepad,
}

/// The wheel holding LB opens now, if any.
fn wheel_here(app: &crate::App, in_game: bool) -> Option<WheelKind> {
    use crate::app_state::BuildMode;
    if !in_game || crate::app::input::mouse::modal_open(app) {
        return None;
    }
    match (app.ui.current_tab, app.panels.build.mode) {
        (crate::MenuTab::World, BuildMode::Movement) => Some(WheelKind::World),
        (crate::MenuTab::World, BuildMode::Break | BuildMode::Place | BuildMode::Rotate) => {
            Some(WheelKind::Build)
        }
        (crate::MenuTab::Inventory, _) => Some(WheelKind::Inventory),
        _ => None,
    }
}

/// What the app looks like to the mapper now.
pub fn context(app: &crate::App) -> PadContext {
    use crate::app::client_app::SplashState;
    let in_game = matches!(app.splash.state, SplashState::MainUI);
    PadContext {
        world_free: in_game
            && app.ui.current_tab == crate::MenuTab::World
            && !crate::app::input::mouse::modal_open(app),
        build_mode: app.panels.build.mode,
        typing: app.typing(),
        wheel: wheel_here(app, in_game),
    }
}

fn key_lookup(app: &crate::App) -> impl Fn(&str, &str) -> Option<KeyCode> + '_ {
    move |category: &str, action: &str| {
        app.ui
            .keybinds
            .get_map()
            .get(&format!("{category}:{action}"))
            .and_then(|codes| codes.first().copied())
    }
}

/// Send what the mapper produced to the game, as the keyboard and mouse would.
fn dispatch(app: &mut crate::App, outputs: Vec<PadOutput>) {
    for output in outputs {
        match output {
            PadOutput::Key(code) => {
                if let Err(e) = crate::handle_input(app, code) {
                    tracing::warn!(target: "game", "gamepad input error: {e}");
                }
            }
            PadOutput::Wheel { up } => {
                use crate::app::input::mouse::{MouseInput, MouseKind};
                // A cell in the game area, away from the menu bar
                crate::app::input::mouse::handle_mouse(
                    app,
                    MouseInput {
                        kind: if up {
                            MouseKind::ScrollUp
                        } else {
                            MouseKind::ScrollDown
                        },
                        col: 0,
                        row: 1,
                        shift: false,
                    },
                );
            }
            PadOutput::Tab(name) => {
                use crate::app::client_app::SplashState;
                let tab = crate::MenuTab::from_config_name(name)
                    .filter(|t| t.visible_with(app.ui.tutorial_visible));
                if let Some(tab) = tab.filter(|_| {
                    matches!(app.splash.state, SplashState::MainUI)
                        && !app.combat.is_active()
                        && !app.typing()
                }) {
                    app.ui.current_tab = tab;
                }
            }
        }
    }
}

/// One pad event from the player. Ignored while Gamepad is off in Options.
pub fn handle_event(app: &mut crate::App, event: PadEvent) {
    if !app.ui.window.gamepad {
        return;
    }
    app.ui.last_input = InputDevice::Gamepad;
    let ctx = context(app);
    let now = app.clock.animation_millis();
    let mut mapper = std::mem::take(&mut app.ui.gamepad);
    let outputs = mapper.handle(event, ctx, now, &key_lookup(app));
    app.ui.gamepad = mapper;
    dispatch(app, outputs);
}

/// Repeat held directions: once a frame.
pub fn tick(app: &mut crate::App) {
    if !app.ui.window.gamepad {
        app.ui.gamepad.release_all();
        return;
    }
    let ctx = context(app);
    let now = app.clock.animation_millis();
    let mut mapper = std::mem::take(&mut app.ui.gamepad);
    let outputs = mapper.tick(ctx, now, &key_lookup(app));
    app.ui.gamepad = mapper;
    dispatch(app, outputs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::{BuildMode, Keybinds};
    use lithicrivers_core::config::{ConfigManager, ConfigSourceKind};

    fn config() -> ConfigManager {
        ConfigManager {
            data: lithicrivers_core::default_config::default_config(),
            file_path: Default::default(),
            source: ConfigSourceKind::EmbeddedDefaults,
        }
    }

    struct Rig {
        mapper: Mapper,
        keys: Keybinds,
        ctx: PadContext,
        now: u64,
    }

    impl Rig {
        fn new() -> Self {
            let cfg = config();
            Self {
                mapper: Mapper::new(Layout::from_config(&cfg)),
                keys: Keybinds::from_config(&cfg),
                ctx: PadContext {
                    world_free: true,
                    build_mode: BuildMode::Movement,
                    typing: false,
                    wheel: Some(WheelKind::World),
                },
                now: 0,
            }
        }

        fn key(&self, category: &str, action: &str) -> PadOutput {
            PadOutput::Key(self.keys.get_map()[&format!("{category}:{action}")][0])
        }

        fn send(&mut self, event: PadEvent) -> Vec<PadOutput> {
            let keys = self.keys.clone();
            let lookup = move |c: &str, a: &str| {
                keys.get_map()
                    .get(&format!("{c}:{a}"))
                    .and_then(|k| k.first().copied())
            };
            self.mapper.handle(event, self.ctx, self.now, &lookup)
        }

        fn press(&mut self, button: PadButton) -> Vec<PadOutput> {
            let mut out = self.send(PadEvent::Button {
                button,
                pressed: true,
            });
            out.extend(self.send(PadEvent::Button {
                button,
                pressed: false,
            }));
            out
        }

        fn stick(&mut self, stick: Stick, dir: Option<Dir8>) -> Vec<PadOutput> {
            self.send(PadEvent::Stick { stick, dir })
        }

        /// Advance the clock to `ms` and run the frame's tick.
        fn at(&mut self, ms: u64) -> Vec<PadOutput> {
            self.now = ms;
            let keys = self.keys.clone();
            let lookup = move |c: &str, a: &str| {
                keys.get_map()
                    .get(&format!("{c}:{a}"))
                    .and_then(|k| k.first().copied())
            };
            self.mapper.tick(self.ctx, self.now, &lookup)
        }
    }

    #[test]
    fn the_shipped_layout_resolves_completely() {
        let cfg = config();
        let layout = Layout::from_config(&cfg);
        assert_eq!(layout.buttons.len(), PadButton::ALL.len(), "every button");
        assert_eq!(layout.sticks.len(), 3);
        let keys = Keybinds::from_config(&cfg);
        for (c, a) in layout.keybinds() {
            assert!(
                keys.get_map().contains_key(&format!("{c}:{a}")),
                "{c}.{a} is not a keybind"
            );
        }
        for (kind, name) in WheelKind::ALL {
            let slots = layout.wheel(kind).unwrap_or_else(|| panic!("{name} wheel"));
            assert!(slots.iter().flatten().count() >= 3, "{name}");
            for slot in slots.iter().flatten() {
                if let PadAction::Tab(tab) = slot {
                    assert!(crate::MenuTab::from_config_name(tab).is_some(), "{tab}");
                }
            }
        }
    }

    #[test]
    fn buttons_send_their_bound_keys() {
        let mut r = Rig::new();
        assert_eq!(r.press(PadButton::West), [r.key("action", "MINE")]);
        assert_eq!(r.press(PadButton::East), [PadOutput::Key(KeyCode::Esc)]);
        assert_eq!(r.press(PadButton::RightBumper), [r.key("ui", "MENU_NEXT")]);
        assert_eq!(
            r.press(PadButton::RightTrigger),
            [PadOutput::Wheel { up: false }]
        );
        assert_eq!(r.press(PadButton::Start), [PadOutput::Tab("MENU")]);
        assert_eq!(
            r.press(PadButton::LeftThumb),
            [r.key("action", "PASS_TIME")]
        );
        // A rebound key changes what the button sends
        let mut cfg = config();
        cfg.data.keybinds["action"]["MINE"] = serde_json::json!(["m"]);
        r.keys = Keybinds::from_config(&cfg);
        assert_eq!(
            r.press(PadButton::West),
            [PadOutput::Key(KeyCode::Char('m'))]
        );
    }

    #[test]
    fn a_confirms_by_context() {
        let mut r = Rig::new();
        assert_eq!(r.press(PadButton::South), [r.key("action", "INTERACT")]);
        r.ctx.world_free = false;
        r.ctx.wheel = None;
        assert_eq!(r.press(PadButton::South), [PadOutput::Key(KeyCode::Enter)]);
    }

    #[test]
    fn only_b_gets_through_while_typing() {
        let mut r = Rig::new();
        r.ctx.typing = true;
        assert!(r.press(PadButton::South).is_empty());
        assert!(r.stick(Stick::Left, Some(Dir8::N)).is_empty());
        assert_eq!(r.press(PadButton::East), [PadOutput::Key(KeyCode::Esc)]);
    }

    #[test]
    fn the_right_stick_aims_by_build_mode() {
        let mut r = Rig::new();
        assert!(
            r.stick(Stick::Right, Some(Dir8::S)).is_empty(),
            "movement mode"
        );
        r.stick(Stick::Right, None);
        r.ctx.build_mode = BuildMode::Place;
        assert_eq!(
            r.stick(Stick::Right, Some(Dir8::S)),
            [r.key("build", "PLACE_SOUTH")]
        );
        r.stick(Stick::Right, None);
        r.ctx.build_mode = BuildMode::Break;
        assert_eq!(
            r.stick(Stick::Right, Some(Dir8::NW)),
            [r.key("build", "BREAK_NORTHWEST")]
        );
    }

    #[test]
    fn sticks_quantize_to_eight_directions_past_the_dead_zone() {
        assert_eq!(quantize(0.3, 0.2), None, "dead zone");
        assert_eq!(quantize(1.0, 0.0), Some(Dir8::E));
        assert_eq!(quantize(0.0, -1.0), Some(Dir8::N));
        assert_eq!(quantize(0.7, 0.7), Some(Dir8::SE));
        assert_eq!(quantize(-0.7, -0.7), Some(Dir8::NW));
        assert_eq!(quantize(-1.0, 0.1), Some(Dir8::W));
        for dir in Dir8::ALL {
            assert_eq!(Dir8::from_script(dir.script_name()), Some(dir));
        }
    }

    #[test]
    fn the_dpad_combines_into_diagonals() {
        let mut r = Rig::new();
        let down = |b| PadEvent::Button {
            button: b,
            pressed: true,
        };
        assert_eq!(
            r.send(down(PadButton::DPadUp)),
            [r.key("movement", "MOVE_NORTH")]
        );
        assert_eq!(
            r.send(down(PadButton::DPadRight)),
            [r.key("movement", "MOVE_NORTHEAST")]
        );
    }

    #[test]
    fn held_directions_repeat_from_the_clock() {
        let mut r = Rig::new();
        let east = r.key("movement", "MOVE_EAST");
        assert_eq!(r.stick(Stick::Left, Some(Dir8::E)), [east]);
        assert!(r.at(REPEAT_DELAY_MS - 1).is_empty(), "not yet");
        assert_eq!(r.at(REPEAT_DELAY_MS), [east]);
        assert!(r.at(REPEAT_DELAY_MS + REPEAT_EVERY_MS - 1).is_empty());
        assert_eq!(
            r.at(REPEAT_DELAY_MS + 3 * REPEAT_EVERY_MS),
            [east, east, east]
        );
        assert!(
            r.stick(Stick::Left, Some(Dir8::E)).is_empty(),
            "same direction"
        );
        r.stick(Stick::Left, None);
        assert!(r.at(10_000).is_empty(), "released");
    }

    #[test]
    fn tapping_lb_changes_tabs_and_holding_opens_the_wheel() {
        let mut r = Rig::new();
        r.send(PadEvent::Button {
            button: PadButton::LeftBumper,
            pressed: true,
        });
        r.at(HOLD_MS - 50);
        assert_eq!(r.mapper.open_wheel(), None);
        let out = r.send(PadEvent::Button {
            button: PadButton::LeftBumper,
            pressed: false,
        });
        assert_eq!(out, [r.key("ui", "MENU_PREV")], "a tap");
        // Held: the wheel opens and directions highlight instead of moving
        r.send(PadEvent::Button {
            button: PadButton::LeftBumper,
            pressed: true,
        });
        r.at(r.now + HOLD_MS);
        assert_eq!(r.mapper.open_wheel(), Some((WheelKind::World, None)));
        assert!(r.stick(Stick::Right, Some(Dir8::E)).is_empty());
        assert!(
            r.stick(Stick::Left, Some(Dir8::N)).is_empty(),
            "no movement"
        );
        assert_eq!(
            r.mapper.open_wheel(),
            Some((WheelKind::World, Some(Dir8::N)))
        );
        assert!(r.press(PadButton::West).is_empty(), "other buttons wait");
        let out = r.send(PadEvent::Button {
            button: PadButton::LeftBumper,
            pressed: false,
        });
        assert_eq!(out, [r.key("action", "PICKUP_ITEMS")], "north slot");
        assert_eq!(r.mapper.open_wheel(), None);
    }

    #[test]
    fn b_cancels_a_wheel_and_releasing_on_nothing_does_nothing() {
        let mut r = Rig::new();
        let lb = |pressed| PadEvent::Button {
            button: PadButton::LeftBumper,
            pressed,
        };
        r.send(lb(true));
        r.at(HOLD_MS);
        r.stick(Stick::Right, Some(Dir8::S));
        assert!(r.press(PadButton::East).is_empty(), "B is swallowed");
        assert_eq!(r.mapper.open_wheel(), None);
        assert!(r.send(lb(false)).is_empty());
        r.send(lb(true));
        r.at(r.now + HOLD_MS);
        assert!(r.send(lb(false)).is_empty(), "nothing highlighted");
    }

    #[test]
    fn lb_acts_at_once_where_there_is_no_wheel() {
        let mut r = Rig::new();
        r.ctx.wheel = None;
        r.ctx.world_free = false;
        let out = r.send(PadEvent::Button {
            button: PadButton::LeftBumper,
            pressed: true,
        });
        assert_eq!(out, [r.key("ui", "MENU_PREV")]);
        assert_eq!(r.mapper.open_wheel(), None);
    }

    #[test]
    fn build_wheel_slots_send_their_actions() {
        let mut r = Rig::new();
        r.ctx.build_mode = BuildMode::Place;
        r.ctx.wheel = Some(WheelKind::Build);
        let lb = |pressed| PadEvent::Button {
            button: PadButton::LeftBumper,
            pressed,
        };
        r.send(lb(true));
        r.at(HOLD_MS);
        r.send(PadEvent::Button {
            button: PadButton::DPadUp,
            pressed: true,
        });
        assert_eq!(r.send(lb(false)), [r.key("build", "ROTATE_PLACEMENT")]);
        assert_eq!(
            PadAction::parse("inventory.TOGGLE_ITEM_AUTO_PICKUP_KEY")
                .unwrap()
                .label(),
            "auto-pickup"
        );
        assert_eq!(
            PadAction::parse("ui.SAVE_JSON").unwrap().label(),
            "save json"
        );
    }

    #[test]
    fn hints_name_buttons() {
        let layout = Layout::from_config(&config());
        assert_eq!(layout.label_for("action", "INTERACT"), Some("A"));
        assert_eq!(layout.label_for("ui", "CLOSE_HELP_MENU"), Some("B"));
        assert_eq!(layout.label_for("movement", "MOVE_EAST"), Some("L-stick"));
        assert_eq!(
            layout.label_for("inventory", "DROP_ITEM"),
            None,
            "no button"
        );
    }
}
