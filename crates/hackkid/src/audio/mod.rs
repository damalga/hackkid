//! Sound: a synthesized drone that follows the player's state, and the album tracks
//! the emergency terminal plays.
//!
//! The tracks are read from disk if they're there: `$HACKKID_MUSIC`, `./music`, or
//! `music/` in the user's data folder, named the way the album files are
//! (`01_april_14.flac`, `07_fanfare.mp3`... anything starting with the two-digit
//! number, as FLAC, Ogg Vorbis, MP3 or WAV).

pub mod drone;
pub mod state;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use hackkid_core::game::tracks::TRACKS;
use hackkid_core::game::world::World;
use rodio::source::EmptyCallback;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

pub use state::AudioState;

const EXTENSIONS: [&str; 4] = ["flac", "ogg", "mp3", "wav"];
const NONE: usize = usize::MAX;

pub struct Audio {
    sink: MixerDeviceSink,
    _drone: Player,
    pub state: Arc<AudioState>,
    /// One player per track: dropping it stops the track at once.
    music: Option<(usize, Player)>,
    /// Index of the track that just played to its end, or [`NONE`].
    finished: Arc<AtomicUsize>,
}

impl Audio {
    /// None when there's no sound device; the game runs silently then.
    pub fn start() -> Option<Self> {
        let mut sink = DeviceSinkBuilder::open_default_sink().ok()?;
        // rodio would print to stderr when the sink closes, over the restored terminal
        sink.log_on_drop(false);
        let state = Arc::new(AudioState::default());
        let drone = Player::connect_new(sink.mixer());
        drone.append(drone::DroneSource::new(state.clone(), 44100));
        drone.play();
        Some(Self { sink, _drone: drone, state, music: None, finished: Arc::new(AtomicUsize::new(NONE)) })
    }

    pub fn toggle_mute(&self) -> bool {
        let muted = self.state.toggle_mute();
        if let Some((_, p)) = &self.music {
            p.set_volume(if muted { 0.0 } else { 1.0 });
        }
        muted
    }

    /// Feeds the drone, follows what the world wants to hear, and reports finished tracks.
    pub fn sync(&mut self, world: &mut World) {
        let s = &self.state;
        let st = &world.player.stats;
        s.hour_x100.store((world.hour * 100.0).clamp(0.0, 2400.0) as u32, Ordering::Relaxed);
        s.indoor.store(!world.player_outdoors(), Ordering::Relaxed);
        s.body.store(st.body.clamp(0.0, 100.0) as u32, Ordering::Relaxed);
        s.thirst.store(st.thirst.clamp(0.0, 100.0) as u32, Ordering::Relaxed);
        s.sleep.store(st.sleep.clamp(0.0, 100.0) as u32, Ordering::Relaxed);
        s.dialog.store(world.message.is_some(), Ordering::Relaxed);

        let done = self.finished.swap(NONE, Ordering::Relaxed);
        if done != NONE {
            if self.music.as_ref().is_some_and(|(i, _)| *i == done) {
                self.music = None;
            }
            world.music_finished(done);
        }

        let want = world.music();
        if want != self.music.as_ref().map(|(i, _)| *i) {
            self.music = None;
            if let Some(idx) = want {
                match self.play(idx) {
                    Ok(player) => self.music = Some((idx, player)),
                    Err(why) => world.music_unavailable(idx, &why),
                }
            }
        }
        s.music.store(self.music.is_some(), Ordering::Relaxed);
    }

    fn play(&self, idx: usize) -> Result<Player, String> {
        let track = &TRACKS[idx];
        let path = find_track_file(track.n).ok_or_else(|| {
            format!(
                "No audio for {:02} {}. Put the album files ({}.flac, ...) in {} or set HACKKID_MUSIC.",
                track.n,
                track.name,
                track.file,
                music_dirs().last().map_or_else(|| "./music".into(), |d| d.display().to_string()),
            )
        })?;
        let file = File::open(&path).map_err(|e| format!("Can't open {}: {e}", path.display()))?;
        let decoder = Decoder::try_from(file).map_err(|e| format!("Can't decode {}: {e}", path.display()))?;
        let player = Player::connect_new(self.sink.mixer());
        if self.state.mute.load(Ordering::Relaxed) {
            player.set_volume(0.0);
        }
        player.append(decoder);
        let finished = self.finished.clone();
        player.append(EmptyCallback::new(Box::new(move || finished.store(idx, Ordering::Relaxed))));
        player.play();
        Ok(player)
    }
}

/// Where to look for the album, in order.
fn music_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(d) = std::env::var_os("HACKKID_MUSIC") {
        dirs.push(PathBuf::from(d));
    }
    dirs.push(PathBuf::from("music"));
    if let Some(d) = dirs::data_dir() {
        dirs.push(d.join("hackkid").join("music"));
    }
    dirs
}

fn find_track_file(n: u8) -> Option<PathBuf> {
    let prefix = format!("{n:02}");
    music_dirs().iter().find_map(|dir| find_in(dir, &prefix))
}

fn find_in(dir: &Path, prefix: &str) -> Option<PathBuf> {
    let mut hits: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
            // "07_fanfare.flac" or "07 - fanfare.mp3", but not "070.flac"
            name.starts_with(prefix)
                && !name[prefix.len()..].starts_with(|c: char| c.is_ascii_digit())
                && EXTENSIONS.contains(&ext.as_str())
        })
        .collect();
    hits.sort();
    hits.into_iter().next()
}
