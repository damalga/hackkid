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
    pub fn samuel(x: f64, y: f64) -> Self {
        Self {
            x, y,
            name: "Samuel".into(),
            greeting: "¡Por fin! Pensé que nunca despertarías.".into(),
            skin: (215, 175, 145),
            shirt: (70, 90, 130),
            pants: (50, 55, 65),
        }
    }

    pub fn selenia(x: f64, y: f64) -> Self {
        Self {
            x, y,
            name: "Selenia".into(),
            greeting: "¡Hey! ¿Tú también estás aquí?".into(),
            skin: (230, 190, 165),
            shirt: (140, 60, 90),
            pants: (60, 55, 70),
        }
    }
}
