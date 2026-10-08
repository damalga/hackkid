use crate::game::items::{Item, ItemKind};
use crate::game::world::{MessageStyle, World};
use crate::objects::npc::Who;

use MessageStyle::{Neutral as N, Other as T, Protagonist as O};

const JULIAN_SCRIPT: [(MessageStyle, &str); 10] = [
    (O, "Olivia: 'Who... who are you? Where is everyone?'"),
    (T, "Julian: 'Jules. Julian. I woke up a week ago, in 106. Your IV line came loose in the surge. I've been checking on you every day.'"),
    (O, "Olivia: 'A week? What happened?'"),
    (T, "Julian: 'The Fanfare. One sound, everywhere at once, at 04:12. Everyone who was awake vanished mid-stride. Scrubs on the floor, monitors still beeping.'"),
    (O, "Olivia: 'And us?'"),
    (T, "Julian: 'Comas. Deep sedation. I study biology... studied. My guess is the tone resonated with waking brain activity, and ours was too quiet to catch.'"),
    (O, "Olivia: 'Is it just us?'"),
    (T, "Julian: 'There's Selenia. She woke up weeks before me and camps in the on-call room, down the west corridor. She knows this place better than anyone.'"),
    (T, "Julian: 'And your emergency terminal picks up a broadcast on 104.2 MHz. Something is still transmitting. Here, take the ward map.'"),
    (N, "Julian hands you a hospital map. It's on M, and in the HUD (Tab)."),
];

const SELENIA_SCRIPT: [(MessageStyle, &str); 8] = [
    (T, "Selenia: 'So the sleeper's finally up. I'm Selenia. Twenty-three days, if my marks on the locker are right.'"),
    (O, "Olivia: 'Twenty-three days? On your own?'"),
    (T, "Selenia: 'The first two weeks, yes. Then the boy woke up. Jules talks too much, but he's good company.'"),
    (O, "Olivia: 'How have you kept going?'"),
    (T, "Selenia: 'Fridges, vending machines, cupboards. Ration it. Drink from the taps while they still run. Wash. Sleep when you can.'"),
    (T, "Selenia: 'Surgery and the medication room are locked. The triage nurse kept a keycard in her desk drawer, off the lobby.'"),
    (T, "Selenia: 'And stay out of the car park at night. The lights are still on and nobody's there. That's worse, somehow.'"),
    (N, "Selenia presses a bottle of water into your hand. 'Don't argue. Drink it.'"),
];

impl World {
    pub(crate) fn talk_to(&mut self, idx: usize) {
        match self.npcs[idx].who {
            Who::Julian => self.julian_dialog(),
            Who::Selenia => self.selenia_dialog(),
        }
    }

    fn julian_dialog(&mut self) {
        if let Some(&(style, line)) = JULIAN_SCRIPT.get(self.julian_line) {
            self.speak(style, line);
            if self.julian_line == JULIAN_SCRIPT.len() - 1 {
                self.player.has_map = true;
            }
            self.julian_line += 1;
            return;
        }
        // after the first talk: whatever's useful now, then small talk in turn
        if !self.player.has_backpack && !self.equippables.is_empty() {
            return self.speak(T, "Julian: 'Your backpack's on the sofa in your room. I put it there so nobody... well. Habit.'");
        }
        if self.player.inventory.has(ItemKind::TriageKeycard) {
            return self.speak(T, "Julian: 'The triage keycard? That opens surgery, and the medication room. Bring back anything for headaches.'");
        }
        let idle = [
            "Julian: 'Selenia's in the on-call room, down the west corridor. She'll know where the food is.'",
            "Julian: 'The bathroom cabinets sometimes have painkillers. Most are empty. I've checked.'",
            "Julian: 'I keep counting the beeps on the monitors. Sorry. Habit.'",
            "Julian: 'Check the emergency terminal on 104.2 MHz. If it says anything new, tell me.'",
        ];
        let line = idle[self.julian_idle % idle.len()];
        self.julian_idle += 1;
        self.speak(T, line);
    }

    fn selenia_dialog(&mut self) {
        if let Some(&(style, line)) = SELENIA_SCRIPT.get(self.selenia_line) {
            self.speak(style, line);
            if self.selenia_line == SELENIA_SCRIPT.len() - 1 {
                self.give(Item::new(ItemKind::WaterBottle));
            }
            self.selenia_line += 1;
            return;
        }
        let days = 22 + self.day;
        let idle = [
            format!("Selenia: '{days} days. {} tomorrow. I keep counting so I don't stop.'", days + 1),
            "Selenia: 'If that terminal of yours ever says anything new, you tell me first.'".to_string(),
            "Selenia: 'Wash. Eat. Sleep. In that order, if you can manage it.'".to_string(),
            "Selenia: 'The chapel's quiet. I go there to think. Not to pray.'".to_string(),
            "Selenia: 'The kitchen fridges won't last forever. Neither will the taps.'".to_string(),
        ];
        let line = idle[self.selenia_idle % idle.len()].clone();
        self.selenia_idle += 1;
        self.speak(T, line);
    }
}
