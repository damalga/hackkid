use crate::engine::camera::{CROUCH_EYE_H, EYE_H, SEATED_EYE_H};
use crate::engine::renderer::{self, Body, Camera, FeetLook, Scene, Torch, Viewport};
use crate::game::player::Garment;
use crate::engine::sky::Env;
use crate::game::sprites::collect_sprites;
use crate::game::world::{GameMode, Weather, World};

impl World {
    fn camera(&self) -> Camera {
        let p = &self.player;
        let eye = match self.mode {
            GameMode::Sitting => SEATED_EYE_H,
            _ if p.hidden => CROUCH_EYE_H,
            _ => EYE_H,
        };
        Camera { x: p.x, y: p.y, dir_x: p.dir_x, dir_y: p.dir_y, plane_x: p.plane_x, plane_y: p.plane_y, eye, pitch: p.pitch }
    }

    fn env(&self, now_ms: u64) -> Env {
        Env { hour: self.hour, cloudy: self.weather == Weather::Cloudy, t_ms: now_ms }
    }

    /// Updates the light for this moment into the viewport's scratch space.
    fn light_up(&self, vp: &mut Viewport, now_ms: u64) {
        let env = self.env(now_ms);
        let tubes = &self.fluorescents;
        let power = self.power_level(now_ms);
        self.lightmap().field(&mut vp.light, &env, power, |i| tubes[i].brightness(now_ms));
        vp.panels.rebuild(self.map.width, self.map.height, tubes, now_ms, power);
    }

    /// Your feet show when you look down, standing.
    fn body(&self) -> Option<Body> {
        if !matches!(self.mode, GameMode::Exploring | GameMode::Inventory) {
            return None;
        }
        let feet = match self.player.wardrobe.feet {
            Some(Garment::Sneakers) => FeetLook::Sneakers,
            Some(Garment::Boots) => FeetLook::Boots,
            _ => FeetLook::Socks,
        };
        Some(Body { feet, stride: self.player.stride })
    }

    fn torch(&self) -> Option<Torch> {
        let strength = self.flashlight_beam() as f32;
        (strength > 0.0).then(|| {
            let p = &self.player;
            Torch { x: p.x, y: p.y, dir_x: p.dir_x, dir_y: p.dir_y, strength }
        })
    }

    /// The first-person view from where the player stands, into `vp` (resize it to the
    /// frame size first). `now_ms` drives flickering lights and drifting clouds.
    pub fn render_view(&self, vp: &mut Viewport, now_ms: u64) {
        self.light_up(vp, now_ms);
        let boxes = self.boxes();
        let sprites = collect_sprites(self);
        let light = std::mem::take(&mut vp.light);
        let panels = std::mem::take(&mut vp.panels);
        let scene = Scene {
            map: &self.map,
            doors: &self.doors,
            light: &light,
            panels: &panels,
            decals: &self.decals,
            env: self.env(now_ms),
            boxes: &boxes,
            sprites: &sprites,
            torch: self.torch(),
            body: self.body(),
        };
        renderer::render(vp, &self.camera(), &scene);
        vp.light = light;
        vp.panels = panels;
    }

    /// Lying on your back: the ceiling overhead.
    pub fn render_ceiling(&self, vp: &mut Viewport, now_ms: u64) {
        self.light_up(vp, now_ms);
        let light = std::mem::take(&mut vp.light);
        let panels = std::mem::take(&mut vp.panels);
        let scene = Scene {
            map: &self.map,
            doors: &self.doors,
            light: &light,
            panels: &panels,
            decals: &self.decals,
            env: self.env(now_ms),
            boxes: &[],
            sprites: &[],
            torch: None,
            body: None,
        };
        renderer::render_ceiling(vp, &self.camera(), &scene);
        vp.light = light;
        vp.panels = panels;
    }
}
