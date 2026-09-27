use crate::game::world::World;

impl World {
    pub fn talk_to_nearby_npc(&mut self) -> bool {
        if let Some(idx) = self.nearby_npc_idx() {
            let name = self.npcs[idx].name.clone();
            if name == "Samuel" {
                self.samuel_dialog();
            } else if name == "Selenia" {
                self.selenia_dialog();
            } else {
                let msg = format!("{}: '{}'", name, self.npcs[idx].greeting);
                self.set_message(msg, 20);
            }
            self.tick();
            return true;
        }
        false
    }

    pub(crate) fn samuel_dialog(&mut self) {
        let base: Vec<String> = vec![
            "Tú: 'Hola, ¿quién eres?'".into(),
            "Samuel: 'Me llamo Samuel. Desperté hace un par de días. Parece que todo el mundo se ha esfumado.'".into(),
            "Tú: '... no entiendo nada.'".into(),
            "Samuel: 'Desperté hace un mes y en todo este tiempo solo he visto a una persona más, se llama Selenia, está fuera del hospital...'".into(),
            "Tú: 'Espera... ¿y dónde está el baño?'".into(),
            "Samuel: 'Toma un mapa del hospital, igual te ayuda a entender el espacio en el que estás, algo es algo :)'".into(),
            "Has recibido el mapa del hospital. Pulsa Q para verlo.".into(),
        ];
        let post_selenia: Vec<String> = vec![
            "Selenia: 'Samuel, este es quien te dije. Estaba dormido cuando desperté.'".into(),
            "Samuel: 'Vaya, así que también aparecimos aquí sin explicación.'".into(),
            "Selenia: 'Estuve una semana viviendo de las máquinas, gripando, sin ver a nadie.'".into(),
            "Samuel: 'Cuando llegué yo tú ya no estabas. Pensé que soñaba.'".into(),
            "Tú: '¿Alguna idea de qué ha pasado?'".into(),
            "Samuel: 'Ninguna. Solo sabemos que la ciudad se llama Copenlada.'".into(),
            "Selenia: 'Y que ya no queda casi nadie. Toca averiguar qué pasó.'".into(),
        ];

        if !self.selenia_met {
            if self.samuel_line < base.len() {
                let line = base[self.samuel_line].clone();
                self.set_message(line, 40);
                if self.samuel_line == base.len() - 1 {
                    self.player.has_map = true;
                }
                self.samuel_line += 1;
            } else {
                self.set_message("Samuel: '...'".into(), 10);
            }
        } else {
            if self.samuel_line < base.len() { self.samuel_line = base.len(); }
            let idx = self.samuel_line - base.len();
            if idx < post_selenia.len() {
                let line = post_selenia[idx].clone();
                self.set_message(line, 40);
                self.samuel_line = base.len() + idx + 1;
            } else {
                self.set_message("Samuel: '...'".into(), 10);
            }
        }
    }

    pub(crate) fn selenia_dialog(&mut self) {
        let script: Vec<String> = vec![
            "Tú: '¡Hey! ¿Tú también estás aquí?'".into(),
            "Selenia: 'Menos mal. Me llamo Selenia. Cuando desperté estabais tú y Samuel dormidos, no había nadie más.'".into(),
            "Tú: '¿Nadie? ¿En serio?'".into(),
            "Selenia: 'En serio. Estuve una semana sin ver a nadie. Gripando, viviendo de las máquinas expendedoras.'".into(),
            "Tú: '¿Y qué es esto?'".into(),
            "Selenia: 'Ni idea. Voy a entrar a buscar a Samuel, tengo cosas que contarle.'".into(),
            "Selenia: 'Toma, un mapa de la ciudad. Se llama Copenlada.'".into(),
            "Has recibido el mapa de Copenlada.".into(),
        ];
        if self.selenia_line < script.len() {
            let line = script[self.selenia_line].clone();
            self.set_message(line, 40);
            if self.selenia_line == script.len() - 1 {
                self.player.has_city_map = true;
                self.selenia_met = true;
                if let Some(sel_idx) = self.npcs.iter().position(|n| n.name == "Selenia") {
                    self.npcs[sel_idx].x = 18.5;
                    self.npcs[sel_idx].y = 17.5;
                }
            }
            self.selenia_line += 1;
        } else {
            self.set_message("Selenia: '...'".into(), 10);
        }
    }
}
