//! The 13 transmissions on the emergency terminal: the tracks of Antwood's *Fanfare*.
//!
//! `file` matches the album's file names (`07_fanfare.flac` and so on), so a frontend can
//! find the audio; `log` is the in-game text the terminal shows for each one.

pub struct Track {
    pub n: u8,
    pub name: &'static str,
    pub file: &'static str,
    pub log: &'static str,
}

pub const TRACKS: [Track; 13] = [
    Track { n: 1, name: "april_14", file: "01_april_14", log: "All regional oscillators ceased at 04:12 UTC. Automated emergency repeat active on 104.2 MHz." },
    Track { n: 2, name: "frogtavia", file: "02_frogtavia", log: "Subjects vanished mid-step. Footwear and synthetic fibers unperturbed. No biological mass remaining." },
    Track { n: 3, name: "u_c_me_?", file: "03_u_c_me_?", log: "ICU Ward 104 containment intact due to deep sedative delta waves. Monitoring vital signs..." },
    Track { n: 4, name: "business_lounge", file: "04_business_lounge", log: "Acoustic wavefront propagated at Mach 1.2 across metropolitan grid. Harmonic frequency matches human neural resonance." },
    Track { n: 5, name: "infinite_ascent", file: "05_infinite_ascent", log: "Clothing piles registered at every intersection. City census: 0% active population." },
    Track { n: 6, name: "fetch_life", file: "06_fetch_life", log: "Infusion pumps and cardiac monitors ticking endlessly to empty beds." },
    Track { n: 7, name: "fanfare", file: "07_fanfare", log: "THE FANFARE: The tone that unwove humanity. A chord of pure silence." },
    Track { n: 8, name: "scam_season", file: "08_scam_season", log: "Sublevel backup generator fuel at 48%. Emergency relays routing to rooftop array." },
    Track { n: 9, name: "100_year_floodplain", file: "09_100_year_floodplain", log: "Carrier wave established on 104.2 MHz. Seeking secondary receiver nodes in sector 4." },
    Track { n: 10, name: "scherzo", file: "10_scherzo", log: "Resonance amplification sustained. Acoustic barrier thinning around hospital perimeter." },
    Track { n: 11, name: "flute_&_harp", file: "11_flute_&_harp", log: "Static echoes detected from outer districts. Automated responses repeating." },
    Track { n: 12, name: "waltz_no._1", file: "12_waltz_no._1", log: "Rooftop broadcast array operational. Powering up directional horn." },
    Track { n: 13, name: "waltz_no._2", file: "13_waltz_no._2", log: "Transmission complete. Listen for the return frequency." },
];
