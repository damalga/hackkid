//! The hospital's building network, as the NET screen of the emergency terminal sees it:
//! a handful of machines still running after The Fanfare. Finding them, reading what the
//! staff left on them and releasing the electronic locks is a puzzle layer over the
//! exploring: the passwords are written down somewhere in the building, and where they
//! aren't there is the override, a timing test. The card-backed locks ask for a relay
//! code, a memory test. Everything here is invented for the game, hosts, services and
//! commands alike, and new puzzles plug in as further [`Minigame`] variants, hosts or
//! commands.
//!
//! Progress that matters (which hosts have been found and got into, which locks have
//! been released) lives in [`Network`] and the doors and is saved; the shell's text is
//! not.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::game::items::ItemKind;
use crate::game::loot::Rng;
use crate::game::world::World;
use crate::objects::Lock;

/// The hospital's subnet; a host's address is this and its last octet.
pub const SUBNET: &str = "10.7.0";
const SCROLLBACK: usize = 300;
const HISTORY: usize = 40;
const INPUT_MAX: usize = 72;
/// What takes time takes this long (real milliseconds).
const SCAN_MS: u64 = 3_600;
const PORTS_MS: u64 = 1_400;
const CONNECT_MS: u64 = 800;
/// Three misses on the override and the host refuses connections for this long.
pub const LOCKOUT_MS: u64 = 30_000;
const OVERRIDE_MISSES: u8 = 3;
/// How many wrong relay codes before the panel times out.
const RELAY_MISSES: u8 = 2;

pub const GATEWAY: usize = 0;
pub const INTRANET: usize = 1;
pub const RECEPTION: usize = 2;
pub const AHMED: usize = 3;
pub const DOORCTL: usize = 4;

/// A machine on the network.
#[derive(Debug, Clone)]
pub struct Host {
    pub ip: u8,
    pub name: &'static str,
    pub role: &'static str,
    /// Open ports and what answers on them.
    pub ports: &'static [(u16, &'static str)],
    /// The account that gets you in, if the host takes logins at all.
    pub account: Option<Account>,
    /// What's on it once you're in: file name and contents.
    pub files: Vec<(&'static str, String)>,
    /// The override, if its login can be timed past: how many hits it takes and how wide
    /// the window is (0..1 of the sweep). `None`: the password or nothing.
    pub bypass: Option<(u8, f64)>,
    pub found: bool,
    /// Logged into at least once: the session reopens without the password.
    pub owned: bool,
    pub lockout_until_ms: u64,
}

#[derive(Debug, Clone)]
pub struct Account {
    pub user: &'static str,
    pub password: String,
}

impl Host {
    pub fn addr(&self) -> String {
        format!("{SUBNET}.{}", self.ip)
    }
}

/// Everything on the network, with how far the player has got into it.
#[derive(Debug, Clone)]
pub struct Network {
    pub hosts: Vec<Host>,
}

/// The part of [`Network`] that goes in a save.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetSave {
    pub found: Vec<bool>,
    pub owned: Vec<bool>,
}

const WORDS: [&str; 16] = [
    "Marigold", "Harbour", "Lantern", "Saffron", "Meridian", "Cobalt", "Juniper", "Velvet", "Orchid", "Pelican", "Sparrow",
    "Granite", "Willow", "Ember", "Compass", "Tulip",
];

fn password(rng: &mut Rng) -> String {
    let word = WORDS[rng.range(0, WORDS.len() as u32 - 1) as usize];
    format!("{word}-{}", rng.range(10, 99))
}

impl Network {
    /// The hospital's machines, with this game's passwords.
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed ^ 0x6E65_7477_6F72_6B00);
        let pw_reception = password(&mut rng);
        let pw_ahmed = password(&mut rng);
        let pw_doors = password(&mut rng);
        let host = |ip, name, role, ports, account: Option<(&'static str, &str)>, bypass| Host {
            ip,
            name,
            role,
            ports,
            account: account.map(|(user, pw)| Account { user, password: pw.to_string() }),
            files: Vec::new(),
            bypass,
            found: false,
            owned: false,
            lockout_until_ms: 0,
        };
        let mut hosts = vec![
            host(1, "hospital-edge", "Edge router", &[(53, "dns"), (443, "https: management, IT VLAN only")], None, None),
            host(10, "intranet", "Staff intranet", &[(80, "http: notices, directory"), (389, "ldap")], None, None),
            host(21, "ws-reception", "Reception / pharmacy workstation", &[(22, "ssh"), (445, "smb: shared files")], Some(("lmoreno", &pw_reception)), Some((3, 0.11))),
            host(31, "ws-ahmed", "Dr. Ahmed's workstation (ICU)", &[(22, "ssh"), (445, "smb: shared files")], Some(("rahmed", &pw_ahmed)), Some((3, 0.09))),
            host(40, "doorctl", "Facilities controller: electronic locks", &[(22, "ssh"), (4840, "relay panel")], Some(("svc-doors", &pw_doors)), Some((4, 0.08))),
        ];
        hosts[RECEPTION].files = vec![
            (
                "handover.txt",
                "PHARMACY HANDOVER, April 13. Stock moved to the dispensary behind the shop. The new door has no card reader: it is released from the facilities controller (doorctl, 10.7.0.40), lock DISP-1. Facilities set us up with the svc-doors account; its password is in helpdesk ticket 4411. - L. Moreno".into(),
            ),
            (
                "ticket_4411.txt",
                format!("HELPDESK TICKET #4411 (closed). Request: facilities controller access for pharmacy. Resolution: svc-doors password reset to {pw_doors}. Please change it at first login. Status: password not changed (reminder sent 3x)."),
            ),
            (
                "visitors_apr14.txt",
                "VISITOR LOG, April 14. 03:50 M. Okafor, to ICU 104 (daughter). 04:05 Delivery, pharmacy. 04:12 ".into(),
            ),
        ];
        hosts[AHMED].files = vec![
            (
                "night_apr14.txt",
                "Night shift, April 14. 04:12: a sound like a chord, everywhere at once, then nothing. Everyone awake is gone. The sedated are still here: Okafor in 104, Reyes in 106. Whatever it was, it coupled to waking brain activity; delta-wave sleep sat below it. If 104 wakes she will need what is in the annex. My pass is in my locker. R.A.".into(),
            ),
            (
                "relay_codes.txt",
                "Facilities memo. The surgery, medication and archive locks are on the controller's relay panel: releasing one asks for the rolling code the panel shows. It is only shown for a moment, so read it before you type. DISP-1 is on the plain relay, no code.".into(),
            ),
            ("stock.txt", "Antibiotics, adrenaline and the suture kits went to the annex in March. The dispensary keeps the rest.".into()),
        ];
        Self { hosts }
    }

    pub fn save(&self) -> NetSave {
        NetSave { found: self.hosts.iter().map(|h| h.found).collect(), owned: self.hosts.iter().map(|h| h.owned).collect() }
    }

    pub fn restore(&mut self, s: &NetSave) {
        for (h, f) in self.hosts.iter_mut().zip(&s.found) {
            h.found = *f;
        }
        for (h, o) in self.hosts.iter_mut().zip(&s.owned) {
            h.owned = *o;
        }
    }

    /// A host by name (once found) or by address.
    pub fn find(&self, what: &str) -> Option<usize> {
        let what = what.trim();
        self.hosts.iter().position(|h| (h.found && h.name.eq_ignore_ascii_case(what)) || h.addr() == what || what.parse::<u8>().is_ok_and(|n| n == h.ip))
    }

    /// What a paper note found in the building says: the passwords, written down.
    pub fn note_text(&self, kind: ItemKind) -> Option<String> {
        let pw = |i: usize| self.hosts[i].account.as_ref().map(|a| a.password.clone()).unwrap_or_default();
        Some(match kind {
            ItemKind::StickyNote => format!("A sticky note, in biro: 'ws-reception  lmoreno / {}'. It was under the keyboard, of course.", pw(RECEPTION)),
            ItemKind::ItMemo => format!("IT memo to Dr. R. Ahmed: 'Your workstation password has been reset to {}. Please change it at first login. Ticket #4398.' He never did.", pw(AHMED)),
            _ => return None,
        })
    }
}

// ---------------------------------------------------------------- the shell

/// Which screen the terminal shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Screen {
    #[default]
    Broadcast,
    Net,
}

/// Something the shell is waiting on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JobKind {
    Scan,
    Ports(usize),
    Connect(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Job {
    pub kind: JobKind,
    pub start_ms: u64,
    pub end_ms: u64,
}

impl Job {
    pub fn label(&self) -> &'static str {
        match self.kind {
            JobKind::Scan => "scanning 10.7.0.0/24",
            JobKind::Ports(_) => "probing ports",
            JobKind::Connect(_) => "connecting",
        }
    }

    pub fn progress(&self, now_ms: u64) -> f64 {
        ((now_ms.saturating_sub(self.start_ms)) as f64 / (self.end_ms - self.start_ms).max(1) as f64).clamp(0.0, 1.0)
    }
}

/// The override: a cursor sweeps back and forth; press when it's in the window. Each hit
/// moves the window and speeds the sweep up; three misses and the host locks you out.
#[derive(Debug, Clone, PartialEq)]
pub struct Timing {
    pub host: usize,
    pub start_ms: u64,
    /// Sweeps per second (there and back is two).
    pub speed: f64,
    /// Centre of the window, 0..1, and its full width.
    pub target: f64,
    pub width: f64,
    pub hits: u8,
    pub need: u8,
    pub misses: u8,
    /// The last press: a hit or not, and when, for the frontend to flash.
    pub last: Option<(bool, u64)>,
}

impl Timing {
    /// Where the cursor is, 0..1.
    pub fn cursor(&self, now_ms: u64) -> f64 {
        let t = now_ms.saturating_sub(self.start_ms) as f64 / 1000.0 * self.speed;
        let phase = t.rem_euclid(2.0);
        if phase <= 1.0 { phase } else { 2.0 - phase }
    }

    pub fn in_window(&self, now_ms: u64) -> bool {
        (self.cursor(now_ms) - self.target).abs() <= self.width / 2.0
    }

    /// When the cursor will next be at the centre of the window: for tests, and the
    /// frontend's own curiosity.
    pub fn next_centre_ms(&self, now_ms: u64) -> u64 {
        let mut t = now_ms;
        while (self.cursor(t) - self.target).abs() > 0.005 {
            t += 1;
        }
        t
    }
}

/// The relay code: shown for a moment, then typed back. Two rounds release the lock.
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub door_id: &'static str,
    pub code: String,
    pub shown_until_ms: u64,
    pub round: u8,
    pub rounds: u8,
    pub misses: u8,
}

impl Sequence {
    pub fn showing(&self, now_ms: u64) -> bool {
        now_ms < self.shown_until_ms
    }
}

/// A skill test in progress on the NET screen. New kinds of puzzle go here.
#[derive(Debug, Clone, PartialEq)]
pub enum Minigame {
    Timing(Timing),
    Sequence(Sequence),
}

/// The NET screen: a command line with a scrollback.
#[derive(Debug, Clone, Default)]
pub struct Shell {
    pub lines: VecDeque<String>,
    pub input: String,
    pub history: Vec<String>,
    /// Where ↑ has got to in the history, if anywhere.
    pub recall: Option<usize>,
    /// The host you're logged into.
    pub session: Option<usize>,
    pub job: Option<Job>,
    pub game: Option<Minigame>,
    /// Lines scrolled back from the bottom.
    pub scroll: usize,
    /// Shown once, the first time the screen opens.
    pub greeted: bool,
}

impl Shell {
    pub fn print(&mut self, line: impl Into<String>) {
        self.lines.push_back(line.into());
        while self.lines.len() > SCROLLBACK {
            self.lines.pop_front();
        }
        self.scroll = 0;
    }

    pub fn busy(&self) -> bool {
        self.job.is_some() || self.game.is_some()
    }

    pub fn type_char(&mut self, c: char) {
        if !c.is_control() && self.input.chars().count() < INPUT_MAX {
            self.input.push(c);
        }
    }

    pub fn backspace(&mut self) {
        self.input.pop();
    }

    /// ↑ (`-1`) and ↓ (`1`) through what was typed before.
    pub fn recall(&mut self, dir: i8) {
        if self.history.is_empty() {
            return;
        }
        let i = match (self.recall, dir) {
            (None, d) if d < 0 => self.history.len() - 1,
            (None, _) => return,
            (Some(i), d) if d < 0 => i.saturating_sub(1),
            (Some(i), _) if i + 1 >= self.history.len() => {
                self.recall = None;
                self.input.clear();
                return;
            }
            (Some(i), _) => i + 1,
        };
        self.recall = Some(i);
        self.input = self.history[i].clone();
    }

    pub fn page(&mut self, dir: i8, page: usize) {
        if dir < 0 {
            self.scroll = (self.scroll + page).min(self.lines.len().saturating_sub(1));
        } else {
            self.scroll = self.scroll.saturating_sub(page);
        }
    }
}

const HELP: [&str; 12] = [
    "scan                       find the machines on the hospital network",
    "hosts                      list what the scan found",
    "ports <host>               see what a machine answers on",
    "connect <host>             open its front page, or start a session you already have",
    "login <host> <user> <pw>   log in (the staff wrote their passwords down)",
    "override <host>            time your way past a login: Enter when the cursor is in the window",
    "files / read <file>        what's on the machine you're logged into",
    "status                     on doorctl: the electronic locks",
    "unlock <LOCK>              on doorctl: release one (card locks ask for the relay code)",
    "logout                     close the session",
    "clear                      clear the screen",
    "Esc leaves a test; Tab is the broadcast screen; ↑↓ recall; PgUp/PgDn scroll",
];

impl World {
    /// The prompt in front of the input line.
    pub fn net_prompt(&self) -> String {
        match self.laptop.shell.session {
            Some(i) => {
                let h = &self.net.hosts[i];
                format!("{}@{}$ ", h.account.as_ref().map_or("guest", |a| a.user), h.name)
            }
            None => "net> ".into(),
        }
    }

    /// The first time the NET screen opens.
    pub(crate) fn greet_net(&mut self) {
        if self.laptop.shell.greeted {
            return;
        }
        self.laptop.shell.greeted = true;
        let sh = &mut self.laptop.shell;
        sh.print("FANFARE EMERGENCY TERMINAL // NET  (building network, Wi-Fi: SCMH-STAFF, joined)");
        sh.print("The hospital's machines are still up. Type help for the commands, scan to begin.");
    }

    /// Enter on the NET screen: a press in a timing test, a code in a memory test, or a
    /// command.
    pub(crate) fn net_enter(&mut self) {
        let now = self.now_ms;
        match self.laptop.shell.game.take() {
            Some(Minigame::Timing(t)) => self.timing_press(t, now),
            Some(Minigame::Sequence(s)) => {
                let typed = std::mem::take(&mut self.laptop.shell.input);
                self.sequence_enter(s, typed, now);
            }
            None => {
                let line = std::mem::take(&mut self.laptop.shell.input);
                self.laptop.shell.recall = None;
                self.net_command(&line);
            }
        }
    }

    /// Esc on the NET screen with a test running: give it up. Returns whether there was one.
    pub(crate) fn net_abort(&mut self) -> bool {
        if self.laptop.shell.game.take().is_some() {
            self.laptop.shell.input.clear();
            self.laptop.shell.print("[test abandoned]");
            return true;
        }
        false
    }

    /// Runs one line as typed. Public so a frontend (or a test) can drive the shell.
    pub fn net_command(&mut self, line: &str) {
        let now = self.now_ms;
        let prompt = self.net_prompt();
        let sh = &mut self.laptop.shell;
        let line = line.trim();
        sh.print(format!("{prompt}{line}"));
        if line.is_empty() {
            return;
        }
        if sh.history.last().is_none_or(|l| l != line) {
            sh.history.push(line.to_string());
            if sh.history.len() > HISTORY {
                sh.history.remove(0);
            }
        }
        if sh.job.is_some() {
            sh.print("[busy: wait for it to finish]");
            return;
        }
        let mut words = line.split_whitespace();
        let cmd = words.next().unwrap_or("").to_ascii_lowercase();
        let args: Vec<&str> = words.collect();
        match cmd.as_str() {
            "help" | "?" => {
                for l in HELP {
                    self.laptop.shell.print(l);
                }
            }
            "clear" => self.laptop.shell.lines.clear(),
            "scan" => self.laptop.shell.job = Some(Job { kind: JobKind::Scan, start_ms: now, end_ms: now + SCAN_MS }),
            "hosts" => self.list_hosts(),
            "ports" => {
                if let Some(i) = self.host_arg(args.first().copied()) {
                    self.laptop.shell.job = Some(Job { kind: JobKind::Ports(i), start_ms: now, end_ms: now + PORTS_MS });
                }
            }
            "connect" | "open" => {
                if let Some(i) = self.host_arg(args.first().copied()) {
                    self.laptop.shell.job = Some(Job { kind: JobKind::Connect(i), start_ms: now, end_ms: now + CONNECT_MS });
                }
            }
            "login" => self.login(&args),
            "override" => self.start_override(args.first().copied()),
            "files" | "ls" => self.list_files(),
            "read" | "cat" => self.read_file(args.first().copied()),
            "status" | "locks" => self.lock_status(),
            "unlock" | "release" => self.unlock(args.first().copied()),
            "logout" | "exit" => {
                if self.laptop.shell.session.take().is_some() {
                    self.laptop.shell.print("Connection closed.");
                } else {
                    self.laptop.shell.print("Not logged in anywhere.");
                }
            }
            other => self.laptop.shell.print(format!("{other}: unknown command. Try help.")),
        }
    }

    fn host_arg(&mut self, arg: Option<&str>) -> Option<usize> {
        let Some(what) = arg else {
            self.laptop.shell.print("Which host? A name from the scan, or an address.");
            return None;
        };
        let found = self.net.find(what);
        if found.is_none() {
            self.laptop.shell.print(format!("{what}: no such host (scan first; names come from the scan)."));
        }
        found
    }

    fn list_hosts(&mut self) {
        let found: Vec<String> = self
            .net
            .hosts
            .iter()
            .filter(|h| h.found)
            .map(|h| format!("  {:<11} {:<14} {}{}", h.addr(), h.name, h.role, if h.owned { "  [session]" } else { "" }))
            .collect();
        if found.is_empty() {
            self.laptop.shell.print("Nothing found yet. Run scan.");
        }
        for l in found {
            self.laptop.shell.print(l);
        }
    }

    fn login(&mut self, args: &[&str]) {
        let [host, user, pw, ..] = args else {
            self.laptop.shell.print("usage: login <host> <user> <password>");
            return;
        };
        let Some(i) = self.host_arg(Some(host)) else { return };
        let now = self.now_ms;
        let h = &self.net.hosts[i];
        if h.lockout_until_ms > now {
            let left = (h.lockout_until_ms - now).div_ceil(1000);
            self.laptop.shell.print(format!("{}: connection refused (too many failed attempts; {left} s).", h.name));
            return;
        }
        let Some(acc) = &h.account else {
            self.laptop.shell.print(format!("{}: no login service here. Try connect.", h.name));
            return;
        };
        if acc.user.eq_ignore_ascii_case(user) && acc.password == *pw {
            self.open_session(i);
        } else {
            self.laptop.shell.print("Permission denied. (The staff wrote their passwords down. Look around.)");
        }
    }

    fn open_session(&mut self, i: usize) {
        let first = !self.net.hosts[i].owned;
        self.net.hosts[i].owned = true;
        self.net.hosts[i].found = true;
        self.laptop.shell.session = Some(i);
        let h = &self.net.hosts[i];
        let mut lines = vec![format!("Welcome to {} ({}). {}", h.name, h.role, if first { "First login." } else { "" })];
        if i == DOORCTL {
            lines.push("Facilities controller. status lists the locks; unlock <LOCK> releases one.".into());
        } else if !h.files.is_empty() {
            lines.push(format!("{} files here. files to list them, read <file> to read.", h.files.len()));
        }
        for l in lines {
            self.laptop.shell.print(l);
        }
        if first {
            self.say(format!("You're in: {}.", self.net.hosts[i].name));
        }
    }

    fn start_override(&mut self, arg: Option<&str>) {
        let Some(i) = self.host_arg(arg) else { return };
        let now = self.now_ms;
        let h = &self.net.hosts[i];
        if h.owned {
            self.open_session(i);
            return;
        }
        if h.lockout_until_ms > now {
            let left = (h.lockout_until_ms - now).div_ceil(1000);
            self.laptop.shell.print(format!("{}: connection refused (too many failed attempts; {left} s).", h.name));
            return;
        }
        let Some((need, width)) = h.bypass else {
            self.laptop.shell.print(format!("{}: nothing to override here.", h.name));
            return;
        };
        let target = 0.2 + self.rng.range(0, 60) as f64 / 100.0;
        self.laptop.shell.game = Some(Minigame::Timing(Timing { host: i, start_ms: now, speed: 0.9, target, width, hits: 0, need, misses: 0, last: None }));
        self.laptop.shell.print(format!("Override on {}: press Enter when the cursor is in the window, {need} times. Three misses and it shuts you out.", h.name));
    }

    fn timing_press(&mut self, mut t: Timing, now: u64) {
        let hit = t.in_window(now);
        t.last = Some((hit, now));
        if hit {
            t.hits += 1;
            if t.hits >= t.need {
                self.laptop.shell.print(format!("[{}/{}] Timing accepted. Session open.", t.hits, t.need));
                self.open_session(t.host);
                return;
            }
            t.speed += 0.25;
            t.width = (t.width * 0.9).max(0.05);
            t.target = 0.15 + self.rng.range(0, 70) as f64 / 100.0;
            self.laptop.shell.print(format!("[{}/{}] hit", t.hits, t.need));
        } else {
            t.misses += 1;
            if t.misses >= OVERRIDE_MISSES {
                let name = self.net.hosts[t.host].name;
                self.net.hosts[t.host].lockout_until_ms = now + LOCKOUT_MS;
                self.laptop.shell.print(format!("{name}: too many failed attempts. Connections refused for {} s.", LOCKOUT_MS / 1000));
                return;
            }
            self.laptop.shell.print(format!("miss ({} left)", OVERRIDE_MISSES - t.misses));
        }
        self.laptop.shell.game = Some(Minigame::Timing(t));
    }

    fn list_files(&mut self) {
        let Some(i) = self.laptop.shell.session else {
            self.laptop.shell.print("Not logged in. login or override a host first.");
            return;
        };
        let names: Vec<String> = self.net.hosts[i].files.iter().map(|(n, _)| format!("  {n}")).collect();
        if names.is_empty() {
            self.laptop.shell.print("No files here.");
        }
        for n in names {
            self.laptop.shell.print(n);
        }
    }

    fn read_file(&mut self, name: Option<&str>) {
        let Some(i) = self.laptop.shell.session else {
            self.laptop.shell.print("Not logged in. login or override a host first.");
            return;
        };
        let Some(name) = name else {
            self.laptop.shell.print("usage: read <file>");
            return;
        };
        match self.net.hosts[i].files.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)) {
            Some((_, text)) => {
                let text = text.clone();
                self.laptop.shell.print(format!("--- {name}"));
                self.laptop.shell.print(text);
            }
            None => self.laptop.shell.print(format!("{name}: no such file (files lists them).")),
        }
    }

    /// The locks wired to the controller, in the order of the floor plan.
    fn controller_locks(&self) -> Vec<(&'static str, usize, usize, Option<Lock>)> {
        let mut v: Vec<_> = self.doors.iter().filter_map(|d| d.net_id.map(|id| (id, d.tx, d.ty, d.lock))).collect();
        v.sort_by_key(|(id, ..)| *id);
        v
    }

    fn on_controller(&mut self) -> bool {
        if self.laptop.shell.session == Some(DOORCTL) {
            return true;
        }
        self.laptop.shell.print("That's a doorctl command: log into the facilities controller first.");
        false
    }

    fn lock_status(&mut self) {
        if !self.on_controller() {
            return;
        }
        let names = |id: &str| match id {
            "SURG-1" => "Surgery sector door",
            "OR-1" => "Operating room",
            "MED-1" => "Medication room",
            "ARCH-1" => "Clinical archive",
            "DISP-1" => "Dispensary",
            _ => "Door",
        };
        let rows: Vec<String> = self
            .controller_locks()
            .iter()
            .map(|(id, _, _, lock)| {
                let state = match lock {
                    None => "RELEASED",
                    Some(Lock::Card(_)) => "LOCKED  (card reader; relay code to release)",
                    Some(Lock::Remote) => "LOCKED  (no reader; plain relay)",
                };
                format!("  {id:<7} {:<22} {state}", names(id))
            })
            .collect();
        for r in rows {
            self.laptop.shell.print(r);
        }
    }

    fn unlock(&mut self, id: Option<&str>) {
        if !self.on_controller() {
            return;
        }
        let Some(id) = id else {
            self.laptop.shell.print("usage: unlock <LOCK> (status lists them)");
            return;
        };
        let id = id.to_ascii_uppercase();
        let Some(&(id, _, _, lock)) = self.controller_locks().iter().find(|(l, ..)| *l == id) else {
            self.laptop.shell.print(format!("{id}: no such lock on this controller."));
            return;
        };
        match lock {
            None => self.laptop.shell.print(format!("{id} is already released.")),
            Some(Lock::Remote) => self.release_lock(id),
            Some(Lock::Card(_)) => {
                let now = self.now_ms;
                let seq = self.new_code(id, 1, 2, now);
                self.laptop.shell.print(format!("{id} is a card lock: the relay panel shows a code. Read it, then type it and press Enter. Two codes release it."));
                self.laptop.shell.game = Some(Minigame::Sequence(seq));
            }
        }
    }

    fn new_code(&mut self, door_id: &'static str, round: u8, rounds: u8, now: u64) -> Sequence {
        let len = 3 + round as usize;
        let code: String = (0..len).map(|_| char::from(b'0' + self.rng.range(0, 9) as u8)).collect();
        Sequence { door_id, code, shown_until_ms: now + 1_400 + 350 * len as u64, round, rounds, misses: 0 }
    }

    fn sequence_enter(&mut self, mut s: Sequence, typed: String, now: u64) {
        if s.showing(now) {
            // pressing while it's still up just hides it early: the test is reading it
            s.shown_until_ms = now;
            self.laptop.shell.input = typed;
            self.laptop.shell.game = Some(Minigame::Sequence(s));
            return;
        }
        if typed.trim() == s.code {
            if s.round >= s.rounds {
                self.laptop.shell.print(format!("Relay code accepted ({}/{}).", s.round, s.rounds));
                self.release_lock(s.door_id);
                return;
            }
            self.laptop.shell.print(format!("Relay code accepted ({}/{}). Next code.", s.round, s.rounds));
            let next = self.new_code(s.door_id, s.round + 1, s.rounds, now);
            self.laptop.shell.game = Some(Minigame::Sequence(next));
        } else {
            s.misses += 1;
            if s.misses >= RELAY_MISSES {
                self.laptop.shell.print("Wrong code. The relay panel times out; run unlock again.");
                return;
            }
            self.laptop.shell.print("Wrong code. The panel shows another.");
            let mut next = self.new_code(s.door_id, s.round, s.rounds, now);
            next.misses = s.misses;
            self.laptop.shell.game = Some(Minigame::Sequence(next));
        }
    }

    /// The controller lets a lock go: the door opens like any other from now on.
    pub(crate) fn release_lock(&mut self, id: &'static str) {
        let Some(d) = self.doors.iter_mut().find(|d| d.net_id == Some(id)) else { return };
        d.lock = None;
        self.laptop.shell.print(format!("{id}: relay released. The lock is open."));
        self.say(format!("Somewhere in the building a lock clicks open ({id})."));
    }

    /// Advances what the shell is waiting on. Called every update.
    pub(crate) fn tick_net(&mut self, now: u64) {
        let Some(job) = self.laptop.shell.job else { return };
        if now < job.end_ms {
            return;
        }
        self.laptop.shell.job = None;
        match job.kind {
            JobKind::Scan => {
                let new = self.net.hosts.iter().filter(|h| !h.found).count();
                for h in &mut self.net.hosts {
                    h.found = true;
                }
                self.laptop.shell.print(format!("{} hosts up ({new} new):", self.net.hosts.len()));
                self.list_hosts();
                if new > 0 {
                    self.say("The terminal finds the hospital's machines, still running.");
                }
            }
            JobKind::Ports(i) => {
                let h = &self.net.hosts[i];
                let rows: Vec<String> = h.ports.iter().map(|(p, s)| format!("  {p:<5}/tcp  open   {s}")).collect();
                let head = format!("{} ({}):", h.name, h.addr());
                self.laptop.shell.print(head);
                for r in rows {
                    self.laptop.shell.print(r);
                }
            }
            JobKind::Connect(i) => self.connect(i, now),
        }
    }

    fn connect(&mut self, i: usize, now: u64) {
        if self.net.hosts[i].owned {
            self.open_session(i);
            return;
        }
        let lines: Vec<String> = match i {
            GATEWAY => vec!["hospital-edge: management interface. Access restricted to the IT VLAN. Nothing for you here.".into()],
            INTRANET => vec![
                "ST. CYPRIAN METROPOLITAN HOSPITAL // STAFF INTRANET".into(),
                "NOTICES".into(),
                " · IT: quarterly password rotation complete. Passwords are NOT to be kept on sticky notes under the reception keyboard. Again.".into(),
                " · Facilities: locks SURG-1, OR-1, MED-1, ARCH-1 and DISP-1 are now on the facilities controller doorctl (10.7.0.40). Card readers stay as backup; the dispensary has none.".into(),
                " · Pharmacy: dispensary stock moved. See L. Moreno's handover on the reception workstation.".into(),
                "DIRECTORY".into(),
                "  rahmed      Dr. R. Ahmed     Consultant, ICU          ws-ahmed".into(),
                "  lmoreno     L. Moreno        Pharmacy / Reception     ws-reception".into(),
                "  svc-doors   (service)        Facilities controller    doorctl".into(),
                "  helpdesk    IT Helpdesk      ext. 4411".into(),
            ],
            _ => {
                let h = &self.net.hosts[i];
                if h.lockout_until_ms > now {
                    vec![format!("{}: connection refused (too many failed attempts; {} s).", h.name, (h.lockout_until_ms - now).div_ceil(1000))]
                } else {
                    vec![format!("{} ({}): login required. login {} <user> <password>, or override {}.", h.name, h.role, h.name, h.name)]
                }
            }
        };
        for l in lines {
            self.laptop.shell.print(l);
        }
    }
}
