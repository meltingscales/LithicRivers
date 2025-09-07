use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, symbols::border, widgets::*};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    io,
    time::{Duration, Instant},
    f32::consts::PI,
};

const TARGET_FPS: u64 = 60;
const NUM_CELLS: usize = 150;
const SPAWN_RADIUS: f32 = 30.0;
const ROOM_THRESHOLD: (u16, u16) = (7, 7); // min width, min height for rooms
const SEPARATION_FORCE: f32 = 0.5;
const CORRIDOR_WIDTH: u16 = 3;

#[derive(Debug, Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn distance(&self, other: &Vec2) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    fn normalize(&self) -> Self {
        let len = (self.x.powi(2) + self.y.powi(2)).sqrt();
        if len > 0.0 {
            Self { x: self.x / len, y: self.y / len }
        } else {
            Self { x: 0.0, y: 0.0 }
        }
    }
}

#[derive(Debug, Clone)]
struct Cell {
    position: Vec2,
    size: Vec2,
    velocity: Vec2,
    is_room: bool,
    id: usize,
}

impl Cell {
    fn new(x: f32, y: f32, width: f32, height: f32, id: usize) -> Self {
        Self {
            position: Vec2::new(x, y),
            size: Vec2::new(width, height),
            velocity: Vec2::new(0.0, 0.0),
            is_room: false,
            id,
        }
    }

    fn overlaps(&self, other: &Cell) -> bool {
        let left1 = self.position.x;
        let right1 = self.position.x + self.size.x;
        let top1 = self.position.y;
        let bottom1 = self.position.y + self.size.y;

        let left2 = other.position.x;
        let right2 = other.position.x + other.size.x;
        let top2 = other.position.y;
        let bottom2 = other.position.y + other.size.y;

        !(right1 <= left2 || right2 <= left1 || bottom1 <= top2 || bottom2 <= top1)
    }

    fn center(&self) -> Vec2 {
        Vec2::new(
            self.position.x + self.size.x / 2.0,
            self.position.y + self.size.y / 2.0,
        )
    }
}

#[derive(Debug, Clone)]
struct Edge {
    from: usize,
    to: usize,
    weight: f32,
}

struct App {
    cells: Vec<Cell>,
    rooms: Vec<usize>,
    edges: Vec<Edge>,
    corridors: HashSet<(i32, i32)>,
    rng: SplitMix64,
    step: usize,
    last_tick: Instant,
    should_quit: bool,
    auto_advance: bool,
    viewport: Rect,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            cells: Vec::new(),
            rooms: Vec::new(),
            edges: Vec::new(),
            corridors: HashSet::new(),
            rng: SplitMix64::new(42),
            step: 0,
            last_tick: Instant::now(),
            should_quit: false,
            auto_advance: false,
            viewport: Rect::default(),
        };
        app.generate_initial_cells();
        app
    }

    fn generate_initial_cells(&mut self) {
        self.cells.clear();
        for i in 0..NUM_CELLS {
            // Generate cells with normal distribution for size
            let width = self.normal_random(3.0, 15.0, 2.0, 12.0);
            let height = self.normal_random(3.0, 15.0, 2.0, 12.0);
            
            // Ensure reasonable aspect ratio
            let aspect_ratio = width / height;
            let (final_width, final_height) = if aspect_ratio > 2.0 {
                (width, width / 2.0)
            } else if aspect_ratio < 0.5 {
                (height / 2.0, height)
            } else {
                (width, height)
            };

            // Random position within spawn radius
            let angle = self.rng.next_f32() * 2.0 * PI;
            let radius = self.rng.next_f32() * SPAWN_RADIUS;
            let x = radius * angle.cos();
            let y = radius * angle.sin();

            self.cells.push(Cell::new(x, y, final_width, final_height, i));
        }
    }

    fn normal_random(&mut self, mean: f32, stddev: f32, min: f32, max: f32) -> f32 {
        // Box-Muller transform for normal distribution
        let u1 = self.rng.next_f32();
        let u2 = self.rng.next_f32();
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
        let value = mean + stddev * z0;
        value.clamp(min, max)
    }

    fn separate_cells(&mut self) {
        for _ in 0..200 { // Multiple iterations for better separation
            let mut forces = vec![Vec2::new(0.0, 0.0); self.cells.len()];
            let mut any_overlap = false;

            for i in 0..self.cells.len() {
                for j in i + 1..self.cells.len() {
                    if self.cells[i].overlaps(&self.cells[j]) {
                        any_overlap = true;
                        let center1 = self.cells[i].center();
                        let center2 = self.cells[j].center();
                        let diff = Vec2::new(center1.x - center2.x, center1.y - center2.y);
                        let distance = diff.x.abs() + diff.y.abs();
                        if distance > 0.0 {
                            let force = Vec2::new(diff.x / distance, diff.y / distance);
                            forces[i].x += force.x * SEPARATION_FORCE;
                            forces[i].y += force.y * SEPARATION_FORCE;
                            forces[j].x -= force.x * SEPARATION_FORCE;
                            forces[j].y -= force.y * SEPARATION_FORCE;
                        }
                    }
                }
            }

            for (i, force) in forces.iter().enumerate() {
                self.cells[i].velocity.x += force.x;
                self.cells[i].velocity.y += force.y;
                self.cells[i].velocity.x *= 0.8; // damping
                self.cells[i].velocity.y *= 0.8;
                self.cells[i].position.x += self.cells[i].velocity.x;
                self.cells[i].position.y += self.cells[i].velocity.y;
            }

            if !any_overlap {
                break;
            }
        }
    }

    fn identify_rooms(&mut self) {
        self.rooms.clear();
        for (i, cell) in self.cells.iter_mut().enumerate() {
            if cell.size.x as u16 >= ROOM_THRESHOLD.0 && cell.size.y as u16 >= ROOM_THRESHOLD.1 {
                cell.is_room = true;
                self.rooms.push(i);
            }
        }
    }

    fn generate_delaunay_triangulation(&mut self) {
        // Simplified triangulation - connect each room to its nearest neighbors
        self.edges.clear();
        if self.rooms.len() < 2 {
            return;
        }

        for &room_id in &self.rooms {
            let room_center = self.cells[room_id].center();
            let mut distances: Vec<(usize, f32)> = Vec::new();

            for &other_room_id in &self.rooms {
                if room_id != other_room_id {
                    let other_center = self.cells[other_room_id].center();
                    let distance = room_center.distance(&other_center);
                    distances.push((other_room_id, distance));
                }
            }

            distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            
            // Connect to 2-3 nearest neighbors
            for (other_id, distance) in distances.iter().take(3) {
                if !self.edges.iter().any(|e| 
                    (e.from == room_id && e.to == *other_id) || 
                    (e.from == *other_id && e.to == room_id)
                ) {
                    self.edges.push(Edge {
                        from: room_id,
                        to: *other_id,
                        weight: *distance,
                    });
                }
            }
        }
    }

    fn create_minimal_spanning_tree(&mut self) {
        if self.edges.is_empty() || self.rooms.is_empty() {
            return;
        }

        // Kruskal's algorithm
        self.edges.sort_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap());
        let mut parent: HashMap<usize, usize> = HashMap::new();
        let mut mst_edges = Vec::new();

        // Initialize union-find
        for &room_id in &self.rooms {
            parent.insert(room_id, room_id);
        }

        fn find(parent: &mut HashMap<usize, usize>, x: usize) -> usize {
            if parent[&x] != x {
                parent.insert(x, find(parent, parent[&x]));
            }
            parent[&x]
        }

        for edge in &self.edges {
            let root_from = find(&mut parent, edge.from);
            let root_to = find(&mut parent, edge.to);
            
            if root_from != root_to {
                mst_edges.push(edge.clone());
                parent.insert(root_from, root_to);
            }
        }

        // Add back 15% of remaining edges for loops
        let remaining_edges: Vec<_> = self.edges.iter()
            .filter(|e| !mst_edges.iter().any(|mst_e| 
                (mst_e.from == e.from && mst_e.to == e.to) ||
                (mst_e.from == e.to && mst_e.to == e.from)
            ))
            .collect();
        
        let num_to_add = (remaining_edges.len() as f32 * 0.15) as usize;
        for i in 0..num_to_add.min(remaining_edges.len()) {
            mst_edges.push(remaining_edges[i].clone());
        }

        self.edges = mst_edges;
    }

    fn generate_corridors(&mut self) {
        self.corridors.clear();
        
        for edge in &self.edges {
            let room1 = &self.cells[edge.from];
            let room2 = &self.cells[edge.to];
            
            let start = room1.center();
            let end = room2.center();

            // Create L-shaped corridor
            let mid_x = start.x;
            let mid_y = end.y;

            // Horizontal segment
            let y = mid_y as i32;
            let x_start = start.x.min(mid_x) as i32;
            let x_end = start.x.max(mid_x) as i32;
            for x in x_start..=x_end {
                for dy in -(CORRIDOR_WIDTH as i32 / 2)..=(CORRIDOR_WIDTH as i32 / 2) {
                    self.corridors.insert((x, y + dy));
                }
            }

            // Vertical segment
            let x = mid_x as i32;
            let y_start = start.y.min(end.y) as i32;
            let y_end = start.y.max(end.y) as i32;
            for y in y_start..=y_end {
                for dx in -(CORRIDOR_WIDTH as i32 / 2)..=(CORRIDOR_WIDTH as i32 / 2) {
                    self.corridors.insert((x + dx, y));
                }
            }
        }
    }

    fn next_step(&mut self) {
        match self.step {
            0 => {
                self.generate_initial_cells();
                self.step = 1;
            }
            1 => {
                self.separate_cells();
                self.step = 2;
            }
            2 => {
                self.identify_rooms();
                self.step = 3;
            }
            3 => {
                self.generate_delaunay_triangulation();
                self.step = 4;
            }
            4 => {
                self.create_minimal_spanning_tree();
                self.step = 5;
            }
            5 => {
                self.generate_corridors();
                self.step = 6;
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.step = 0;
        self.cells.clear();
        self.rooms.clear();
        self.edges.clear();
        self.corridors.clear();
        self.rng = SplitMix64::new(self.rng.next_u64()); // New seed
    }

    fn on_tick(&mut self) {
        if self.auto_advance {
            let frame_time = Duration::from_millis(1000);
            if self.last_tick.elapsed() >= frame_time && self.step < 6 {
                self.last_tick = Instant::now();
                self.next_step();
            }
        }
    }

    fn get_bounds(&self) -> (f32, f32, f32, f32) {
        if self.cells.is_empty() {
            return (-50.0, -50.0, 50.0, 50.0);
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for cell in &self.cells {
            min_x = min_x.min(cell.position.x);
            min_y = min_y.min(cell.position.y);
            max_x = max_x.max(cell.position.x + cell.size.x);
            max_y = max_y.max(cell.position.y + cell.size.y);
        }

        // Add padding
        let padding = 10.0;
        (min_x - padding, min_y - padding, max_x + padding, max_y + padding)
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(KeyEvent { code, kind, .. }) => {
                    if kind == KeyEventKind::Press {
                        match code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.should_quit = true;
                            }
                            KeyCode::Char(' ') | KeyCode::Enter => {
                                app.next_step();
                            }
                            KeyCode::Char('r') => {
                                app.reset();
                            }
                            KeyCode::Char('a') => {
                                app.auto_advance = !app.auto_advance;
                            }
                            _ => {}
                        }
                    }
                }
                Event::Resize(_, _) => {
                    app.viewport = terminal.size()?;
                }
                _ => {}
            }
        }

        app.on_tick();

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();
    
    // Title and instructions
    let title = format!(" Dungeon Generation - Step {} ", app.step);
    let instructions = if app.auto_advance {
        "Space/Enter: Next Step | R: Reset | A: Auto (ON) | Q: Quit"
    } else {
        "Space/Enter: Next Step | R: Reset | A: Auto (OFF) | Q: Quit"
    };

    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .title(title)
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::LightBlue));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(size);

    // Main content area
    let inner = main_block.inner(chunks[0]);
    render_dungeon(f, app, inner);
    f.render_widget(main_block, chunks[0]);

    // Instructions
    let instructions_block = Block::default()
        .borders(Borders::ALL)
        .title(" Controls ");
    let instructions_para = Paragraph::new(instructions)
        .block(instructions_block)
        .alignment(Alignment::Center);
    f.render_widget(instructions_para, chunks[1]);
}

fn render_dungeon(f: &mut Frame, app: &App, area: Rect) {
    if app.cells.is_empty() {
        let text = Paragraph::new("Press SPACE or ENTER to start generation")
            .alignment(Alignment::Center);
        f.render_widget(text, area);
        return;
    }

    let (min_x, min_y, max_x, max_y) = app.get_bounds();
    let world_width = max_x - min_x;
    let world_height = max_y - min_y;

    if world_width <= 0.0 || world_height <= 0.0 {
        return;
    }

    // Create a grid to render the dungeon
    let mut grid = vec![vec![' '; area.width as usize]; area.height as usize];

    // Scale factor to fit world in terminal
    let scale_x = (area.width as f32 - 2.0) / world_width;
    let scale_y = (area.height as f32 - 2.0) / world_height;
    let scale = scale_x.min(scale_y);

    // Draw corridors first
    if app.step >= 5 {
        for &(x, y) in &app.corridors {
            let screen_x = ((x as f32 - min_x) * scale) as usize;
            let screen_y = ((y as f32 - min_y) * scale) as usize;
            if screen_x < area.width as usize && screen_y < area.height as usize {
                grid[screen_y][screen_x] = '.';
            }
        }
    }

    // Draw cells
    for cell in &app.cells {
        let start_x = ((cell.position.x - min_x) * scale) as usize;
        let start_y = ((cell.position.y - min_y) * scale) as usize;
        let end_x = (((cell.position.x + cell.size.x) - min_x) * scale) as usize;
        let end_y = (((cell.position.y + cell.size.y) - min_y) * scale) as usize;

        let symbol = if cell.is_room && app.step >= 2 {
            '#'
        } else if app.step >= 1 {
            '░'
        } else {
            '▓'
        };

        for y in start_y..end_y.min(area.height as usize) {
            for x in start_x..end_x.min(area.width as usize) {
                if y < grid.len() && x < grid[y].len() {
                    grid[y][x] = symbol;
                }
            }
        }
    }

    // Draw connections
    if app.step >= 4 {
        for edge in &app.edges {
            let room1 = &app.cells[edge.from];
            let room2 = &app.cells[edge.to];
            let start = room1.center();
            let end = room2.center();

            let start_x = ((start.x - min_x) * scale) as usize;
            let start_y = ((start.y - min_y) * scale) as usize;
            let end_x = ((end.x - min_x) * scale) as usize;
            let end_y = ((end.y - min_y) * scale) as usize;

            // Simple line drawing
            if start_x < area.width as usize && start_y < area.height as usize &&
               end_x < area.width as usize && end_y < area.height as usize {
                if start_x < grid[0].len() && start_y < grid.len() {
                    grid[start_y][start_x] = '+';
                }
                if end_x < grid[0].len() && end_y < grid.len() {
                    grid[end_y][end_x] = '+';
                }
            }
        }
    }

    // Convert grid to string
    let mut content = String::new();
    for row in grid {
        content.push_str(&row.iter().collect::<String>());
        content.push('\n');
    }

    let paragraph = Paragraph::new(content);
    f.render_widget(paragraph, area);

    // Step description
    let step_desc = match app.step {
        0 => "Ready to generate dungeon",
        1 => "Generated cells with separation steering",
        2 => "Identified rooms (larger cells)",
        3 => "Created Delaunay triangulation",
        4 => "Built minimal spanning tree + loops",
        5 => "Generated L-shaped corridors",
        6 => "Dungeon complete!",
        _ => "Unknown step",
    };

    let desc_area = Rect {
        x: area.x + 1,
        y: area.y + area.height - 2,
        width: area.width - 2,
        height: 1,
    };
    let desc = Paragraph::new(step_desc)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(desc, desc_area);
}

// Simple PRNG implementation
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn next_f32(&mut self) -> f32 {
        let v = self.next_u64();
        (v as f32) / (u64::MAX as f32)
    }
}