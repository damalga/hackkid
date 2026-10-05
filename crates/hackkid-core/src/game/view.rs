use crate::engine::{raycaster, renderer::Renderer};
use crate::game::sprites::collect_sprites;
use crate::game::world::{Weather, World};

impl World {
    /// The first-person view from where the player stands, as RGBA pixels of the
    /// renderer's size. `now_ms` drives the flickering lights.
    pub fn render_view(&self, renderer: &Renderer, now_ms: u64) -> Vec<u8> {
        let p = &self.player;
        let rays = raycaster::cast(&self.map, p.x, p.y, p.dir_x, p.dir_y, p.plane_x, p.plane_y, renderer.screen_width);
        let cloudy = self.weather == Weather::Cloudy;
        let mut frame = renderer.build_frame(&rays, &self.doors, cloudy, &self.map, p.x, p.y);
        renderer.draw_ceiling_lights(&mut frame, &rays, &self.map, p.x, p.y, &self.fluorescents, now_ms);
        renderer.draw_sprites(&mut frame, &rays, p.x, p.y, p.dir_x, p.dir_y, p.plane_x, p.plane_y, &collect_sprites(self));
        frame
    }
}
