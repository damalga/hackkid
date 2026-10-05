use crate::game::player::Stats;

const LABEL_W: usize = 7;

pub fn stat_bar(label: &str, value: f64, width: usize) -> String {
    let filled = ((value / 100.0) * width as f64) as usize;
    let empty = width.saturating_sub(filled);
    let bar: String = format!("{}{}",
        "█".repeat(filled),
        "░".repeat(empty),
    );
    format!("{:<lw$} {} {:>3.0}", label, bar, value, lw = LABEL_W)
}

pub fn stat_bar_center(label: &str, value: f64, width: usize) -> String {
    let half = width / 2;
    let mut chars: Vec<char> = vec!['░'; width];
    let mag = ((value.abs() / 100.0) * half as f64).round() as usize;
    if value > 0.0 {
        for i in half..(half + mag).min(width) {
            chars[i] = '█';
        }
    } else if value < 0.0 {
        let start = half.saturating_sub(mag);
        for i in start..half {
            chars[i] = '█';
        }
    }
    if half < width { chars[half] = '│'; }
    let bar: String = chars.iter().collect();
    format!("{:<lw$} {} {:+4.0}", label, bar, value, lw = LABEL_W)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatDirection {
    HighGood,
    LowGood,
    Centered,
}

pub struct HudRow {
    pub line: String,
    pub value: f64,
    pub direction: StatDirection,
    pub is_stamina: bool,
}

pub struct HudGroup {
    pub title: &'static str,
    pub rows: Vec<HudRow>,
}

pub fn render_hud_groups(stats: &Stats, bar_w: usize) -> Vec<HudGroup> {
    vec![
        HudGroup {
            title: "Vital",
            rows: vec![
                HudRow { line: stat_bar("Body", stats.body, bar_w), value: stats.body, direction: StatDirection::HighGood, is_stamina: false },
                HudRow { line: stat_bar("Mind", stats.mind, bar_w), value: stats.mind, direction: StatDirection::HighGood, is_stamina: false },
                HudRow { line: stat_bar("Energy", stats.stamina, bar_w), value: stats.stamina, direction: StatDirection::HighGood, is_stamina: true },
                HudRow { line: stat_bar("Hygiene", stats.hygiene, bar_w), value: stats.hygiene, direction: StatDirection::HighGood, is_stamina: false },
            ],
        },
        HudGroup {
            title: "Needs",
            rows: vec![
                HudRow { line: stat_bar("Thirst", stats.thirst, bar_w), value: stats.thirst, direction: StatDirection::LowGood, is_stamina: false },
                HudRow { line: stat_bar("Hunger", stats.hunger, bar_w), value: stats.hunger, direction: StatDirection::LowGood, is_stamina: false },
                HudRow { line: stat_bar("Sleep", stats.sleep, bar_w), value: stats.sleep, direction: StatDirection::LowGood, is_stamina: false },
            ],
        },
        HudGroup {
            title: "Thermal",
            rows: vec![
                HudRow { line: stat_bar_center("Thermal", stats.thermal, bar_w), value: stats.thermal, direction: StatDirection::Centered, is_stamina: false },
            ],
        },
    ]
}

pub fn is_alarm(direction: StatDirection, value: f64) -> bool {
    match direction {
        StatDirection::HighGood => value <= 10.0,
        StatDirection::LowGood => value >= 90.0,
        StatDirection::Centered => value.abs() >= 90.0,
    }
}

pub fn render_top_status(day: u32, hour: f64, hidden: bool, weather: &str) -> String {
    let h = hour as u32;
    let m = ((hour - h as f64) * 60.0) as u32;
    let hidden_str = if hidden { "  [HIDDEN]" } else { "" };
    format!("FANFARE // EMERGENCY RECOVERY PROTOCOL — Day {}  {:02}:{:02}  {}{}", day, h, m, weather, hidden_str)
}
