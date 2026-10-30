// On Windows the game opens its own window, so it needs no console window
#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    lithicrivers_client::run_desktop()
}
