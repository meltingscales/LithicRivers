use ratatui::style::Color;

pub fn sprite_for_view_reticle() -> String {
    "*".to_string()
}

pub fn sprite_for_view_reticle_color() -> Color {
    Color::Magenta
}

pub fn body_art() -> Vec<String> {
    let art = [
        "      HHH     ",
        "     HHHHH    ",
        "      HHH     ",
        "   A  XXX  a  ",
        "  A  XXXXX  a ",
        " A   XXXXX   a",
        "     XXXXX    ",
        "     XXXXX    ",
        "     XX XX    ",
        "     L   l    ",
        "     L   l    ",
        "     L   l    ",
        "    L     l   ",
    ];
    art.iter().map(|s| s.to_string()).collect()
}
