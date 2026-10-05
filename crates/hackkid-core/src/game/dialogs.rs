use crate::game::world::{MessageStyle, World};

const JULIAN_SCRIPT: [(MessageStyle, &str); 9] = [
    (MessageStyle::Protagonist, "Olivia: 'Jules? What happened to everyone?'"),
    (MessageStyle::Other, "Julian: 'Your IV line came loose during the surge. The Fanfare hit without warning.'"),
    (MessageStyle::Protagonist, "Olivia: 'The Fanfare? What was that sound?'"),
    (MessageStyle::Other, "Julian: 'A global acoustic cataclysm. Every conscious human vanished instantly mid-stride, leaving only empty scrubs and beeping monitors behind.'"),
    (MessageStyle::Protagonist, "Olivia: 'And the staff? The doctors?'"),
    (MessageStyle::Other, "Julian: 'Gone. Only us ICU patients in deep comas remained. We survived because our nervous systems were locked.'"),
    (MessageStyle::Protagonist, "Olivia: 'How do we get out of here?'"),
    (MessageStyle::Other, "Julian: 'Radio towers are dead, but your emergency terminal can pick up a repeating broadcast on 104.2 MHz. Take this hospital map, and check the terminal.'"),
    (MessageStyle::Neutral, "Julian hands you a hospital map. It now shows in your HUD."),
];

impl World {
    pub(crate) fn talk_to(&mut self, idx: usize) {
        if self.npcs[idx].name.starts_with("Julian") {
            self.julian_dialog();
        } else {
            let msg = format!("{}: '{}'", self.npcs[idx].name, self.npcs[idx].greeting);
            self.speak(MessageStyle::Other, msg);
        }
    }

    fn julian_dialog(&mut self) {
        match JULIAN_SCRIPT.get(self.julian_line) {
            Some(&(style, line)) => {
                self.speak(style, line);
                if self.julian_line == JULIAN_SCRIPT.len() - 1 {
                    self.player.has_map = true;
                }
                self.julian_line += 1;
            }
            None => self.speak(MessageStyle::Other, "Julian: 'Check the emergency broadcast terminal on 104.2 MHz. It's in your backpack.'"),
        }
    }
}
