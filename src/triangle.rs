use glm::Vec3;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Estructura para representar un triángulo 3D.
#[derive(Clone, Debug)]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
    pub color: Color,
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, color: Color) -> Self {
        Self { v0, v1, v2, color }
    }

    /// Dibuja el triángulo como *wireframe* (solo sus aristas)
    pub fn draw_wireframe(&self, fb: &Framebuffer, canvas: &mut Canvas<Window>, scale: f32, offset: Vec3) {
        let p0 = to_screen_coords(self.v0, scale, offset);
        let p1 = to_screen_coords(self.v1, scale, offset);
        let p2 = to_screen_coords(self.v2, scale, offset);

        canvas.set_draw_color(self.color.to_sdl());
        let _ = canvas.draw_line(p0, p1);
        let _ = canvas.draw_line(p1, p2);
        let _ = canvas.draw_line(p2, p0);
    }
}

/// Convierte coordenadas 3D en 2D para dibujar en pantalla
fn to_screen_coords(v: Vec3, scale: f32, offset: Vec3) -> sdl2::rect::Point {
    sdl2::rect::Point::new(
        (v.x * scale + offset.x) as i32,
        (v.y * -scale + offset.y) as i32,
    )
}
