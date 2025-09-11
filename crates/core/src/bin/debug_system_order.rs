use lithicrivers_core::system_scheduler::SystemScheduler;
use tracing::info;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let scheduler = SystemScheduler::new();

    info!(target: "systems", "System Execution Order:");

    for (i, &system_id) in scheduler.get_execution_order().iter().enumerate() {
        info!(target: "systems", "{}. {:?}", i + 1, system_id);
    }
}
