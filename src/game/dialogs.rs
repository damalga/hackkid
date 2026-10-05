use crate::game::world::World;

impl World {
    pub fn talk_to_nearby_npc(&mut self) -> bool {
        if let Some(idx) = self.nearby_npc_idx() {
            let name = self.npcs[idx].name.clone();
            if name.starts_with("Julian") {
                self.julian_dialog();
            } else {
                let msg = format!("{}: '{}'", name, self.npcs[idx].greeting);
                self.set_message(msg, 20);
            }
            self.tick();
            return true;
        }
        false
    }

    pub(crate) fn julian_dialog(&mut self) {
        let script: Vec<String> = vec![
            "Olivia: 'Jules? What happened to everyone?'".into(),
            "Julian: 'Your IV line came loose during the surge. The Fanfare hit without warning.'".into(),
            "Olivia: 'The Fanfare? What was that sound?'".into(),
            "Julian: 'A global acoustic cataclysm. Every conscious human vanished instantly mid-stride—left behind only empty scrubs and beeping monitors.'".into(),
            "Olivia: 'And the staff? The doctors?'".into(),
            "Julian: 'Gone. Only us ICU patients in deep comas remained unaroused. We survived because our nervous systems were locked.'".into(),
            "Olivia: 'How do we get out of here?'".into(),
            "Julian: 'Radio towers are dead, but the emergency terminal in Ward 104 is intercepting a repeating broadcast on 104.2 MHz. Take this hospital map. Check the terminal.'".into(),
            "You received the hospital map. Press Q to view it.".into(),
        ];

        if self.samuel_line < script.len() {
            let line = script[self.samuel_line].clone();
            self.set_message(line, 40);
            if self.samuel_line == script.len() - 1 {
                self.player.has_map = true;
            }
            self.samuel_line += 1;
        } else {
            self.set_message("Julian (Jules): 'Check the emergency broadcast terminal on 104.2 MHz.'".into(), 10);
        }
    }
}
