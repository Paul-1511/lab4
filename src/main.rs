use sdl2::pixels::Color as SdlColor;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::path::Path;

mod framebuffer;
mod color;
mod triangle;

use glm::{Vec3, vec3};
use tobj;
use crate::framebuffer::Framebuffer;
use crate::color::Color;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() -> Result<(), String> {
    // Inicializar SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
            .window("Software Renderer - 3D Scene", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .resizable()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let fb = Framebuffer::new(SCREEN_WIDTH, SCREEN_HEIGHT, Color::BLACK);
    let mut event_pump = sdl_context.event_pump()?;

    // Limpiar pantalla
        canvas.set_draw_color(SdlColor::RGB(20, 20, 40));
    canvas.clear();

    // --- CARGAR EL MODELO ---
    // Try several likely locations for Nave.obj, then fallback to scene.obj
    let candidate_paths = vec![
        Path::new("Nave.obj"),
        Path::new("src/Nave.obj"),
        Path::new("assets/Nave.obj"),
        Path::new("scene.obj"),
    ];

    let mut models = Vec::new();
    // tobj::load_obj returns (Vec<Model>, Result<Vec<Material>, LoadError>)
    let mut materials: Result<Vec<tobj::Material>, tobj::LoadError> = Ok(Vec::new());
    let mut loaded = false;
    for p in candidate_paths.iter() {
        if p.exists() {
            match tobj::load_obj(p, &tobj::LoadOptions::default()) {
                Ok((m, mat)) => {
                    models = m;
                    materials = mat;
                    println!("Loaded model from {:?}", p);
                    loaded = true;
                    break;
                }
                Err(e) => {
                    println!("Found {:?} but failed to load: {}", p, e);
                }
            }
        }
    }

    if !loaded {
        return Err("Failed to find or load any model file (looked for Nave.obj/scene.obj).".to_string());
    }
    let materials = match materials {
        Ok(m) => Some(m),
        Err(e) => {
            println!("Warning: failed to load materials: {}", e);
            None
        }
    };
    println!("Modelo cargado: {} mallas", models.len());

    // --- Fit model to screen: compute scene AABB, center and scale
    let mut bb_min = vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut bb_max = vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for model in models.iter() {
        let mesh = &model.mesh;
        for vi in 0..(mesh.positions.len() / 3) {
            let x = mesh.positions[3 * vi];
            let y = mesh.positions[3 * vi + 1];
            let z = mesh.positions[3 * vi + 2];
            bb_min.x = bb_min.x.min(x);
            bb_min.y = bb_min.y.min(y);
            bb_min.z = bb_min.z.min(z);
            bb_max.x = bb_max.x.max(x);
            bb_max.y = bb_max.y.max(y);
            bb_max.z = bb_max.z.max(z);
        }
    }

    let center = (bb_min + bb_max) * 0.5;
    let extent = bb_max - bb_min;
    let max_dim = extent.x.max(extent.y).max(extent.z);
    let scale = if max_dim > 0.0 {
        (SCREEN_WIDTH.min(SCREEN_HEIGHT) as f32 * 0.85) / max_dim
    } else {
        1.0
    };
    let offset = vec3(
        SCREEN_WIDTH as f32 * 0.5 - center.x * scale,
        SCREEN_HEIGHT as f32 * 0.5 + center.y * scale,
        0.0,
    );

    // Palette for per-mesh coloring
    let palette = vec![
        SdlColor::RGB(255, 200, 0),   // warm yellow
        SdlColor::RGB(200, 80, 200),  // magenta
        SdlColor::RGB(0, 200, 255),   // cyan
        SdlColor::RGB(120, 220, 120), // light green
        SdlColor::RGB(255, 120, 60),  // orange
        SdlColor::RGB(180, 180, 255), // light blue
        SdlColor::RGB(255, 120, 200), // pink
    ];

    for model in models.iter() {
        let mesh = &model.mesh;
        // choose a color deterministically from the mesh name
        let name_bytes = model.name.as_bytes();
        let mut sum: usize = 0;
        for &b in name_bytes { sum = sum.wrapping_add(b as usize); }
        let color = palette[sum % palette.len()];
        canvas.set_draw_color(color);
        println!("Dibujando mesh '{}' con {} vértices", model.name, mesh.positions.len() / 3);

                let vertex_count = mesh.positions.len() / 3;
                // Normalize faces to Vec<usize>. tobj uses u32 indices.
                let faces: Vec<usize> = if mesh.indices.is_empty() {
                    // If no indices, assume vertices are already in face order
                    (0..vertex_count).map(|i| i as usize).collect()
                } else {
                    mesh.indices.iter().map(|&i| i as usize).collect()
                };

                for i in (0..faces.len()).step_by(3) {
                    if i + 2 >= faces.len() { break; }
                    let i0 = faces[i];
                    let i1 = faces[i + 1];
                    let i2 = faces[i + 2];

                    let raw_v0 = vec3(mesh.positions[3 * i0], mesh.positions[3 * i0 + 1], mesh.positions[3 * i0 + 2]);
                    let raw_v1 = vec3(mesh.positions[3 * i1], mesh.positions[3 * i1 + 1], mesh.positions[3 * i1 + 2]);
                    let raw_v2 = vec3(mesh.positions[3 * i2], mesh.positions[3 * i2 + 1], mesh.positions[3 * i2 + 2]);

                    // Apply a simple view transform (rotation) so camera is up-right
                    let tv0 = view_transform(raw_v0, center);
                    let tv1 = view_transform(raw_v1, center);
                    let tv2 = view_transform(raw_v2, center);

                    // Transformar vértices a espacio de pantalla
                    let p0 = to_screen_coords(tv0, scale, offset);
                    let p1 = to_screen_coords(tv1, scale, offset);
                    let p2 = to_screen_coords(tv2, scale, offset);

                    // Fill triangle using software rasterizer (framebuffer)
                    // Create a slight per-triangle color variation so meshes are more interesting
                    let tri_hash = i0.wrapping_mul(73856093) ^ i1.wrapping_mul(19349663) ^ i2.wrapping_mul(83492791);
                    let shade = (tri_hash % 64) as i32; // 0..63
                    let base_r = color.r as i32; let base_g = color.g as i32; let base_b = color.b as i32;
                    let r = (base_r + shade).clamp(0, 255) as u8;
                    let g = (base_g + (shade / 2)).clamp(0, 255) as u8;
                    let b = (base_b + (64 - shade) / 3).clamp(0, 255) as u8;
                    let col = Color::new(r, g, b);
                    fb.fill_triangle(&mut canvas, p0, p1, p2, col);
                    // Optionally draw outline
                    canvas.set_draw_color(SdlColor::RGB(10, 10, 10));
                    let _ = canvas.draw_line(p0, p1);
                    let _ = canvas.draw_line(p1, p2);
                    let _ = canvas.draw_line(p2, p0);
                }
    }

    canvas.present();

    // --- BUCLE PRINCIPAL ---
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
    }

    Ok(())
}

fn to_screen_coords(v: Vec3, scale: f32, offset: Vec3) -> sdl2::rect::Point {
    sdl2::rect::Point::new(
        (v.x * scale + offset.x) as i32,
        (v.y * -scale + offset.y) as i32,
    )
}

// Simple view transform: move the model so `center` is origin, rotate so camera is up-right.
fn view_transform(v: Vec3, center: Vec3) -> Vec3 {
    let mut p = v - center;
    // Yaw: rotate -45 degrees around Y (move to right)
    let yaw = -45.0_f32.to_radians();
    let cy = yaw.cos();
    let sy = yaw.sin();
    let x = cy * p.x + sy * p.z;
    let z = -sy * p.x + cy * p.z;
    p.x = x; p.z = z;

    // Pitch: rotate 30 degrees around X (move up)
    let pitch = 30.0_f32.to_radians();
    let cp = pitch.cos();
    let sp = pitch.sin();
    let y = cp * p.y - sp * p.z;
    let z2 = sp * p.y + cp * p.z;
    p.y = y; p.z = z2;

    p
}
