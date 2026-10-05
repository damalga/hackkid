//! FANFARE game core: the raycaster, the hospital and the rules of survival.
//!
//! Nothing in here touches a terminal, a file, a clock or a sound card. A frontend
//! passes the elapsed time in ([`game::world::World::update`]), sends [`game::action::Action`]s,
//! renders the RGBA frame from [`engine::renderer`] and reads the world's state back out.
//! What the core can't do itself (writing a save, playing a track, quitting) it asks
//! for through [`game::world::Request`] and [`game::world::World::music`].

pub mod engine;
pub mod equippables;
pub mod game;
pub mod objects;
