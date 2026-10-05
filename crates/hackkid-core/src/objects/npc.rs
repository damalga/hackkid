#[derive(Debug, Clone)]
pub struct Npc {
    pub x: f64,
    pub y: f64,
    pub name: String,
    pub greeting: String,
    pub skin: (u8, u8, u8),
    pub shirt: (u8, u8, u8),
    pub pants: (u8, u8, u8),
}

impl Npc {
    pub fn julian(x: f64, y: f64) -> Self {
        Self {
            x, y,
            name: "Julian (Jules)".into(),
            greeting: "Finally awake? Your IV line came loose during the surge.".into(),
            skin: (215, 175, 145),
            shirt: (70, 90, 130),
            pants: (50, 55, 65),
        }
    }
}
