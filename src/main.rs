use macroquad::prelude::*;

const W: usize = 32;
const H: usize = 24;
const D: usize = 32;

struct World {
    b: Vec<u8>,
}

impl World {
    fn idx(x: usize, y: usize, z: usize) -> usize {
        (y * D + z) * W + x
    }
    fn inb(x: i32, y: i32, z: i32) -> bool {
        x >= 0 && z >= 0 && y >= 0 && x < W as i32 && z < D as i32 && y < H as i32
    }
    fn get(&self, x: i32, y: i32, z: i32) -> u8 {
        if !Self::inb(x, y, z) {
            return 0;
        }
        self.b[Self::idx(x as usize, y as usize, z as usize)]
    }
    fn set(&mut self, x: i32, y: i32, z: i32, t: u8) {
        if Self::inb(x, y, z) {
            let i = Self::idx(x as usize, y as usize, z as usize);
            self.b[i] = t;
        }
    }
    fn solid(&self, x: i32, y: i32, z: i32) -> bool {
        if y < 0 || x < 0 || z < 0 || x >= W as i32 || z >= D as i32 {
            return true;
        }
        self.get(x, y, z) != 0
    }
}

fn generate() -> World {
    let mut w = World { b: vec![0; W * H * D] };
    for x in 0..W {
        for z in 0..D {
            let fx = x as f32;
            let fz = z as f32;
            let h = (8.0
                + (fx * 0.21).sin() * 2.5
                + (fz * 0.17).cos() * 2.5
                + ((fx + fz) * 0.09).sin() * 2.0) as usize;
            let h = h.min(H - 8);
            for y in 0..=h {
                let t = if y == h {
                    1
                } else if y + 3 > h {
                    2
                } else {
                    3
                };
                w.set(x as i32, y as i32, z as i32, t);
            }
        }
    }
    // trees
    for x in 3..W - 3 {
        for z in 3..D - 3 {
            if (x * 7 + z * 13) % 41 == 0 {
                let mut h = 0;
                for y in 0..H {
                    if w.get(x as i32, y as i32, z as i32) != 0 {
                        h = y;
                    }
                }
                if h + 7 >= H {
                    continue;
                }
                for y in h + 1..=h + 4 {
                    w.set(x as i32, y as i32, z as i32, 4);
                }
                for dx in -2i32..=2 {
                    for dz in -2i32..=2 {
                        for dy in 4..=5 {
                            let (lx, ly, lz) = (x as i32 + dx, (h + dy) as i32, z as i32 + dz);
                            if w.get(lx, ly, lz) == 0 {
                                w.set(lx, ly, lz, 5);
                            }
                        }
                    }
                }
            }
        }
    }
    w
}

fn block_color(t: u8) -> (f32, f32, f32) {
    match t {
        1 => (0.35, 0.68, 0.24),
        2 => (0.52, 0.36, 0.20),
        3 => (0.50, 0.50, 0.52),
        4 => (0.63, 0.47, 0.27),
        _ => (0.18, 0.50, 0.16),
    }
}

type Face = ((i32, i32, i32), [[f32; 3]; 4], f32);

fn build(w: &World) -> Vec<Mesh> {
    let faces: [Face; 6] = [
        ((1, 0, 0), [[1., 0., 0.], [1., 1., 0.], [1., 1., 1.], [1., 0., 1.]], 0.8),
        ((-1, 0, 0), [[0., 0., 0.], [0., 0., 1.], [0., 1., 1.], [0., 1., 0.]], 0.8),
        ((0, 1, 0), [[0., 1., 0.], [0., 1., 1.], [1., 1., 1.], [1., 1., 0.]], 1.0),
        ((0, -1, 0), [[0., 0., 0.], [1., 0., 0.], [1., 0., 1.], [0., 0., 1.]], 0.5),
        ((0, 0, 1), [[0., 0., 1.], [1., 0., 1.], [1., 1., 1.], [0., 1., 1.]], 0.65),
        ((0, 0, -1), [[0., 0., 0.], [0., 1., 0.], [1., 1., 0.], [1., 0., 0.]], 0.65),
    ];
    let mut out: Vec<Mesh> = Vec::new();
    let mut verts: Vec<Vertex> = Vec::new();
    let mut inds: Vec<u16> = Vec::new();

    for y in 0..H as i32 {
        for z in 0..D as i32 {
            for x in 0..W as i32 {
                let t = w.get(x, y, z);
                if t == 0 {
                    continue;
                }
                let (r, g, b) = block_color(t);
                let var = 1.0 - ((x * 31 + y * 17 + z * 13) % 5) as f32 * 0.025;
                for (n, c, shade) in faces.iter() {
                    if w.solid(x + n.0, y + n.1, z + n.2) {
                        continue;
                    }
                    if verts.len() + 4 > 8000 {
                        out.push(Mesh {
                            vertices: std::mem::take(&mut verts),
                            indices: std::mem::take(&mut inds),
                            texture: None,
                        });
                    }
                    let s = shade * var;
                    let col = Color::new(r * s, g * s, b * s, 1.0);
                    let base = verts.len() as u16;
                    for p in c.iter() {
                        verts.push(Vertex::new(
                            x as f32 + p[0],
                            y as f32 + p[1],
                            z as f32 + p[2],
                            0.0,
                            0.0,
                            col,
                        ));
                    }
                    inds.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
                }
            }
        }
    }
    if !verts.is_empty() {
        out.push(Mesh { vertices: verts, indices: inds, texture: None });
    }
    out
}

fn collides(w: &World, p: Vec3) -> bool {
    for dx in [-0.3f32, 0.3] {
        for dz in [-0.3f32, 0.3] {
            for dy in [0.0f32, 0.9, 1.7] {
                if w.solid(
                    (p.x + dx).floor() as i32,
                    (p.y + dy).floor() as i32,
                    (p.z + dz).floor() as i32,
                ) {
                    return true;
                }
            }
        }
    }
    false
}

fn raycast(w: &World, o: Vec3, d: Vec3) -> Option<((i32, i32, i32), (i32, i32, i32))> {
    let cell = |p: Vec3| (p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
    let mut prev = cell(o);
    let mut t = 0.0;
    while t < 6.0 {
        let c = cell(o + d * t);
        if w.get(c.0, c.1, c.2) != 0 {
            return Some((c, prev));
        }
        prev = c;
        t += 0.04;
    }
    None
}

fn conf() -> Conf {
    Conf {
        window_title: "BlockCraft".to_owned(),
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let mut world = generate();
    let mut meshes = build(&world);

    let mut pos = vec3(16.0, 22.0, 16.0);
    let (mut yaw, mut pitch) = (0.8f32, -0.2f32);
    let mut vy = 0.0f32;
    let mut grounded = false;
    let mut place_mode = false;
    let mut sel: u8 = 1;

    let mut stick: Option<(u64, Vec2, Vec2)> = None;
    let mut look: Option<(u64, Vec2, Vec2, f64, bool)> = None;

    loop {
        let sw = screen_width();
        let sh = screen_height();
        let u = sh / 600.0;
        let jump_r = Rect::new(sw - 130.0 * u, sh - 130.0 * u, 110.0 * u, 110.0 * u);
        let mode_r = Rect::new(sw - 130.0 * u, sh - 260.0 * u, 110.0 * u, 110.0 * u);
        let slot = |i: usize| {
            Rect::new(
                sw / 2.0 - 175.0 * u + i as f32 * 70.0 * u,
                10.0 * u,
                62.0 * u,
                62.0 * u,
            )
        };

        let mut jump = false;
        let mut act = false;

        for t in touches() {
            match t.phase {
                TouchPhase::Started => {
                    if jump_r.contains(t.position) {
                        jump = true;
                        continue;
                    }
                    if mode_r.contains(t.position) {
                        place_mode = !place_mode;
                        continue;
                    }
                    let mut hit = false;
                    for i in 0..5 {
                        if slot(i).contains(t.position) {
                            sel = i as u8 + 1;
                            hit = true;
                        }
                    }
                    if hit {
                        continue;
                    }
                    if t.position.x < sw * 0.4 && stick.is_none() {
                        stick = Some((t.id, t.position, t.position));
                    } else if look.is_none() {
                        look = Some((t.id, t.position, t.position, get_time(), false));
                    }
                }
                TouchPhase::Moved | TouchPhase::Stretched => {
                    if let Some(s) = stick.as_mut() {
                        if s.0 == t.id {
                            s.2 = t.position;
                        }
                    }
                    if let Some(l) = look.as_mut() {
                        if l.0 == t.id {
                            let d = t.position - l.1;
                            let k = 0.005 / u;
                            yaw += d.x * k;
                            pitch = (pitch - d.y * k).clamp(-1.5, 1.5);
                            l.1 = t.position;
                            if (t.position - l.2).length() > 20.0 * u {
                                l.4 = true;
                            }
                        }
                    }
                }
                _ => {
                    if let Some(s) = stick {
                        if s.0 == t.id {
                            stick = None;
                        }
                    }
                    if let Some(l) = look {
                        if l.0 == t.id {
                            if !l.4
                                && get_time() - l.3 < 0.35
                                && matches!(t.phase, TouchPhase::Ended)
                            {
                                act = true;
                            }
                            look = None;
                        }
                    }
                }
            }
        }

        let dt = get_frame_time().min(0.05);
        let mut mv = Vec2::ZERO;
        if let Some(s) = stick {
            mv = ((s.2 - s.1) / (60.0 * u)).clamp_length_max(1.0);
        }
        let fwd = vec3(yaw.cos(), 0.0, yaw.sin());
        let right = vec3(-yaw.sin(), 0.0, yaw.cos());
        let wish = (fwd * -mv.y + right * mv.x) * 4.5 * dt;

        let mut np = pos;
        np.x += wish.x;
        if !collides(&world, np) {
            pos = np;
        }
        np = pos;
        np.z += wish.z;
        if !collides(&world, np) {
            pos = np;
        }

        vy -= 22.0 * dt;
        if jump && grounded {
            vy = 7.5;
        }
        np = pos;
        np.y += vy * dt;
        if collides(&world, np) {
            if vy < 0.0 {
                grounded = true;
            }
            vy = 0.0;
        } else {
            pos = np;
            grounded = false;
        }
        if pos.y < -10.0 {
            pos = vec3(16.0, 22.0, 16.0);
            vy = 0.0;
        }

        let eye = pos + vec3(0.0, 1.6, 0.0);
        let front = vec3(
            pitch.cos() * yaw.cos(),
            pitch.sin(),
            pitch.cos() * yaw.sin(),
        );
        let cam = Camera3D {
            position: eye,
            target: eye + front,
            up: vec3(0.0, 1.0, 0.0),
            fovy: 1.2,
            ..Default::default()
        };

        let target = raycast(&world, eye, front);

        if act {
            if let Some((hit, prev)) = target {
                if place_mode {
                    if world.get(prev.0, prev.1, prev.2) == 0 {
                        world.set(prev.0, prev.1, prev.2, sel);
                        if collides(&world, pos) {
                            world.set(prev.0, prev.1, prev.2, 0);
                        } else {
                            meshes = build(&world);
                        }
                    }
                } else if hit.1 > 0 {
                    world.set(hit.0, hit.1, hit.2, 0);
                    meshes = build(&world);
                }
            }
        }

        clear_background(Color::new(0.53, 0.81, 0.92, 1.0));
        set_camera(&cam);
        for m in &meshes {
            draw_mesh(m);
        }
        if let Some((h, _)) = target {
            draw_cube_wires(
                vec3(h.0 as f32 + 0.5, h.1 as f32 + 0.5, h.2 as f32 + 0.5),
                vec3(1.01, 1.01, 1.01),
                BLACK,
            );
        }

        set_default_camera();
        draw_line(sw / 2.0 - 10.0 * u, sh / 2.0, sw / 2.0 + 10.0 * u, sh / 2.0, 2.0, WHITE);
        draw_line(sw / 2.0, sh / 2.0 - 10.0 * u, sw / 2.0, sh / 2.0 + 10.0 * u, 2.0, WHITE);

        for i in 0..5 {
            let r = slot(i);
            let (cr, cg, cb) = block_color(i as u8 + 1);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(cr, cg, cb, 1.0));
            let c = if sel == i as u8 + 1 { YELLOW } else { BLACK };
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 4.0 * u, c);
        }

        draw_rectangle(jump_r.x, jump_r.y, jump_r.w, jump_r.h, Color::new(1.0, 1.0, 1.0, 0.3));
        draw_text("JUMP", jump_r.x + 14.0 * u, jump_r.y + 62.0 * u, 34.0 * u, BLACK);
        let mc = if place_mode {
            Color::new(0.2, 0.8, 0.2, 0.5)
        } else {
            Color::new(0.9, 0.2, 0.2, 0.5)
        };
        draw_rectangle(mode_r.x, mode_r.y, mode_r.w, mode_r.h, mc);
        draw_text(
            if place_mode { "PLACE" } else { "BREAK" },
            mode_r.x + 8.0 * u,
            mode_r.y + 62.0 * u,
            34.0 * u,
            BLACK,
        );

        if let Some(s) = stick {
            draw_circle(s.1.x, s.1.y, 60.0 * u, Color::new(1.0, 1.0, 1.0, 0.2));
            let d = (s.2 - s.1).clamp_length_max(60.0 * u);
            draw_circle(s.1.x + d.x, s.1.y + d.y, 28.0 * u, Color::new(1.0, 1.0, 1.0, 0.5));
        }

        next_frame().await;
    }
}
