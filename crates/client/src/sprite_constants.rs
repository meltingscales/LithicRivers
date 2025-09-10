use ratatui::style::Color;

pub fn sprite_for_view_reticle() -> Vec<String> {
    let s = ["*", "**\n**", "***\n***\n***"];
    s.iter().map(|s| s.to_string()).collect()
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
        "  A  XXFXX  a ",
        " A   XFFFX   a",
        "     XXFXX    ",
        "     XXXXX    ",
        "     XX XX    ",
        "     L   l    ",
        "     L   l    ",
        "     L   l    ",
        "    L     l   ",
    ];
    art.iter().map(|s| s.to_string()).collect()
}
