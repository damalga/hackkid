use crate::engine::camera::{CROUCH_EYE_H, EYE_H, SEATED_EYE_H};
use crate::engine::renderer::{self, Camera, Scene, Viewport};
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
        Camera { x: p.x, y: p.y, dir_x: p.dir_x, dir_y: p.dir_y, plane_x: p.plane_x, plane_y: p.plane_y, eye }
    }

    fn env(&self, now_ms: u64) -> Env {
        Env { hour: self.hour, cloudy: self.weather == Weather::Cloudy, t_ms: now_ms }
    }

    /// Updates the light for this moment into the viewport's scratch space.
    fn light_up(&self, vp: &mut Viewport, now_ms: u64) {
        let env = self.env(now_ms);
        let tubes = &self.fluorescents;
        self.lightmap().field(&mut vp.light, &env, |i| tubes[i].brightness(now_ms));
        vp.panels.rebuild(self.map.width, self.map.height, tubes, now_ms);
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
        };
        renderer::render_ceiling(vp, &self.camera(), &scene);
        vp.light = light;
        vp.panels = panels;
    }
}
