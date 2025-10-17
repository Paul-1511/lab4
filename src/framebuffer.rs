use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color as SdlColor;
use crate::color::Color;

pub struct Framebuffer {
    width: u32,
    height: u32,
    bg_color: Color,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, bg_color: Color) -> Self {
        Self { width, height, bg_color }
    }

    pub fn clear(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(self.bg_color.to_sdl());
        canvas.clear();
    }

    pub fn draw_pixel(&self, canvas: &mut Canvas<Window>, x: i32, y: i32, color: Color) {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            canvas.set_draw_color(color.to_sdl());
            let _ = canvas.draw_point(sdl2::rect::Point::new(x, y));
        }
    }

    /// Fill triangle using barycentric coordinates (simple, not super-optimized)
    pub fn fill_triangle(&self, canvas: &mut Canvas<Window>, p0: sdl2::rect::Point, p1: sdl2::rect::Point, p2: sdl2::rect::Point, color: Color) {
        // Convert to floats
        let x0 = p0.x as f32; let y0 = p0.y as f32;
        let x1 = p1.x as f32; let y1 = p1.y as f32;
        let x2 = p2.x as f32; let y2 = p2.y as f32;

        // Bounding box
        let minx = x0.min(x1).min(x2).floor() as i32;
        let maxx = x0.max(x1).max(x2).ceil() as i32;
        let miny = y0.min(y1).min(y2).floor() as i32;
        let maxy = y0.max(y1).max(y2).ceil() as i32;

        // Precompute edge functions denom
        let denom = (y1 - y2)*(x0 - x2) + (x2 - x1)*(y0 - y2);
        if denom == 0.0 { return; }

        for y in miny..=maxy {
            for x in minx..=maxx {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                let w0 = ((y1 - y2)*(px - x2) + (x2 - x1)*(py - y2)) / denom;
                let w1 = ((y2 - y0)*(px - x2) + (x0 - x2)*(py - y2)) / denom;
                let w2 = 1.0 - w0 - w1;

                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    self.draw_pixel(canvas, x, y, color);
                }
            }
        }
    }
}
