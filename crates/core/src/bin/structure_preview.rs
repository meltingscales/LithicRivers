//! `just util-structure-preview <name>`: print a structure layer by layer, with its
//! legend and anything the lint finds.

fn main() {
    let Some(name) = std::env::args().nth(1) else {
        eprintln!("usage: structure_preview <name>, e.g. small_temple");
        std::process::exit(2);
    };
    match lithicrivers_core::structure_lint::preview(&name) {
        Ok(text) => print!("{text}"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
