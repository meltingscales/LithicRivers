const CORRUPTION_RATE: f32 = 0.0;
const CORRUPTION_SEED: u64 = 42;

pub fn get_boot_message() -> String {
    let lines = read_boot_lines();

    // Apply a single corruption pass to each line with a fixed seed
    let corrupted_lines: Vec<String> = lines
        .iter()
        .map(|line| {
            // Use a fixed seed for consistent corruption
            let corrupted = corrupt_text(line, CORRUPTION_RATE, CORRUPTION_SEED);
            // Ensure line length is reasonable
            if corrupted.len() > 120 {
                corrupted[..120].to_string()
            } else {
                corrupted
            }
        })
        .collect();

    // Join all lines with newlines
    corrupted_lines.join("\n")
}

fn read_boot_lines() -> Vec<String> {
    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "assets/config/"]
    struct Asset;

    Asset::get("boot_message.dat")
        .and_then(|f| std::str::from_utf8(&f.data).ok().map(|s| s.to_string()))
        .unwrap_or_else(|| panic!("boot_message.dat not found"))
        .lines()
        .map(|s| s.to_string())
        .collect()
}

fn corrupt_text(line: &str, rate: f32, seed: u64) -> String {
    if line.is_empty() {
        return String::new();
    }

    let mut rng = SplitMix64::new(seed);
    let mut result = String::with_capacity(line.len());

    for c in line.chars() {
        if c.is_whitespace() {
            result.push(c);
            continue;
        }

        if rng.next_f32() < rate {
            // Corrupt this character
            let new_char = (b'!' + (rng.next_u32() % 94) as u8) as char;
            result.push(new_char);
        } else {
            result.push(c);
        }
    }

    result
}

// Minimal deterministic RNG (SplitMix64)
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

    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }
}
