// AJDAR PARVOZI — low-poly 3D canyon flight (Rust + macroquad)
use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use macroquad::material::{gl_use_default_material, gl_use_material, load_material, Material, MaterialParams};
use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;
use std::collections::VecDeque;
use std::f32::consts::{PI, TAU};

const VH: f32 = 720.0;
const CH_LEN: f32 = 80.0;
const DZ: f32 = 4.0;
const ROWS: usize = 20;
const COLS: usize = 20;
const VIEW: usize = 6;

fn sc() -> f32 { screen_height() / VH }
fn vw() -> f32 { screen_width() / sc() }

// ---------------- terrain ----------------
fn h2(x: i32, z: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (z as u32).wrapping_mul(668265263);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 0xffff) as f32 / 65535.0
}
fn vnoise(x: f32, z: f32) -> f32 {
    let (xi, zi) = (x.floor(), z.floor());
    let (fx, fz) = (x - xi, z - zi);
    let (sx, sz) = (fx * fx * (3.0 - 2.0 * fx), fz * fz * (3.0 - 2.0 * fz));
    let (xi, zi) = (xi as i32, zi as i32);
    let a = h2(xi, zi) + (h2(xi + 1, zi) - h2(xi, zi)) * sx;
    let b = h2(xi, zi + 1) + (h2(xi + 1, zi + 1) - h2(xi, zi + 1)) * sx;
    a + (b - a) * sz
}
fn smooth(e0: f32, e1: f32, x: f32) -> f32 { let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) }
fn center(z: f32) -> f32 { (z * 0.008).sin() * 22.0 + (z * 0.019).sin() * 9.0 }
fn height(u: f32, z: f32) -> f32 {
    let n = vnoise(u * 0.15 + 7.0, z * 0.05) * 2.0 + vnoise(u * 0.4, z * 0.2) * 0.6;
    let au = u.abs();
    n * 0.6 - 1.0 + smooth(9.0, 20.0, au) * (16.0 + n * 4.0) + smooth(24.0, 40.0, au) * 8.0
}
fn ground(x: f32, z: f32) -> f32 { height(x - center(z), z) }

// ---------------- mesh helpers ----------------
fn light() -> Vec3 { vec3(-0.35, 0.55, 0.75).normalize() }
fn tri(m: &mut Mesh, a: Vec3, b: Vec3, c: Vec3, col: [u8; 3]) {
    let n = (b - a).cross(c - a).normalize_or_zero();
    let s = 0.42 + 0.58 * n.dot(light()).abs();
    let col = [(col[0] as f32 * s) as u8, (col[1] as f32 * s) as u8, (col[2] as f32 * s) as u8, 255];
    let i = m.vertices.len() as u16;
    for p in [a, b, c] { m.vertices.push(Vertex { position: p, uv: Vec2::ZERO, color: col, normal: Vec4::ZERO }); }
    m.indices.extend_from_slice(&[i, i + 1, i + 2]);
}
fn quad(m: &mut Mesh, a: Vec3, b: Vec3, c: Vec3, d: Vec3, col: [u8; 3]) { tri(m, a, b, c, col); tri(m, a, c, d, col); }
fn octa(m: &mut Mesh, p: Vec3, r: f32, h: f32, rot: f32, c1: [u8; 3], c2: [u8; 3]) {
    let (t, b) = (p + vec3(0.0, h, 0.0), p - vec3(0.0, h, 0.0));
    let e: Vec<Vec3> = (0..4).map(|k| { let v = Vec2::from_angle(rot + k as f32 * PI / 2.0) * r; p + vec3(v.x, 0.0, v.y) }).collect();
    for k in 0..4 {
        let (a, c) = (e[k], e[(k + 1) % 4]);
        tri(m, t, a, c, if k % 2 == 0 { c1 } else { c2 });
        tri(m, b, c, a, if k % 2 == 0 { c2 } else { c1 });
    }
}
fn new_mesh() -> Mesh { Mesh { vertices: Vec::with_capacity(4096), indices: Vec::with_capacity(4096), texture: None } }

// ---------------- world chunks ----------------
struct Pillar { p: Vec3, r: f32, top: f32 }
struct Ring { p: Vec3, done: bool }
struct Gem { p: Vec3, got: bool }
struct Mine { p: Vec3, ph: f32, dead: bool }
struct Chunk { z0: f32, mesh: Mesh, pillars: Vec<Pillar>, rings: Vec<Ring>, gems: Vec<Gem>, mines: Vec<Mine> }

fn terrain_col(h: f32, n: Vec3) -> [u8; 3] {
    let steep = 1.0 - n.y.abs();
    if h < 0.3 { [222, 168, 108] } else if h > 19.0 && steep < 0.35 { [214, 128, 86] } else if steep > 0.55 { [170, 82, 60] } else if h > 10.0 { [150, 72, 62] } else { [196, 116, 76] }
}

fn gen_chunk(idx: i32) -> Chunk {
    let z0 = idx as f32 * CH_LEN;
    let mut m = new_mesh();
    let p = |c: usize, r: usize| { let z = z0 + r as f32 * DZ; let u = -40.0 + c as f32 * 4.0; vec3(center(z) + u, height(u, z), z) };
    for r in 0..ROWS {
        for c in 0..COLS {
            let (a, b, cc, d) = (p(c, r), p(c + 1, r), p(c + 1, r + 1), p(c, r + 1));
            for (x, y, w) in [(a, cc, b), (a, d, cc)] {
                let n = (y - x).cross(w - x).normalize_or_zero();
                tri(&mut m, x, y, w, terrain_col((x.y + y.y + w.y) / 3.0, n));
            }
        }
    }
    let mut ch = Chunk { z0, mesh: m, pillars: vec![], rings: vec![], gems: vec![], mines: vec![] };
    if idx < 2 { return ch; }
    let np = (1 + idx / 3).min(6) as usize;
    for _ in 0..np {
        let z = z0 + rand::gen_range(6.0, 74.0);
        let u = rand::gen_range(-8.5, 8.5);
        let base = vec3(center(z) + u, height(u, z) - 1.0, z);
        let r = rand::gen_range(1.2, 2.3);
        let top = base.y + if rand::gen_range(0, 3) == 0 { rand::gen_range(5.0, 8.0) } else { rand::gen_range(14.0, 26.0) };
        let pts: Vec<Vec2> = (0..6).map(|k| Vec2::from_angle(k as f32 * TAU / 6.0 + z) * r).collect();
        for k in 0..6 {
            let (a, b) = (pts[k], pts[(k + 1) % 6]);
            let col = if k % 2 == 0 { [120, 62, 58] } else { [138, 70, 62] };
            quad(&mut ch.mesh, base + vec3(a.x, 0.0, a.y), base + vec3(b.x, 0.0, b.y), vec3(base.x + b.x * 0.8, top, base.z + b.y * 0.8), vec3(base.x + a.x * 0.8, top, base.z + a.y * 0.8), col);
            tri(&mut ch.mesh, vec3(base.x, top + 0.6, base.z), vec3(base.x + a.x * 0.8, top, base.z + a.y * 0.8), vec3(base.x + b.x * 0.8, top, base.z + b.y * 0.8), [176, 96, 72]);
        }
        ch.pillars.push(Pillar { p: base, r, top });
    }
    for _ in 0..rand::gen_range(1, 3) {
        let z = z0 + rand::gen_range(10.0, 70.0);
        let u = rand::gen_range(-6.0, 6.0);
        let c = vec3(center(z) + u, (height(u, z) + 3.5).max(3.5) + rand::gen_range(0.0, 6.0), z);
        for k in 0..14 {
            let (a0, a1) = (k as f32 * TAU / 14.0, (k + 1) as f32 * TAU / 14.0);
            let (i0, o0, i1, o1) = (Vec2::from_angle(a0) * 2.6, Vec2::from_angle(a0) * 3.3, Vec2::from_angle(a1) * 2.6, Vec2::from_angle(a1) * 3.3);
            let v = |q: Vec2| c + q.extend(0.0);
            quad(&mut ch.mesh, v(i0), v(o0), v(o1), v(i1), if k % 2 == 0 { [255, 210, 90] } else { [255, 180, 60] });
        }
        ch.rings.push(Ring { p: c, done: false });
        for g in 1..5 { let gz = z + g as f32 * 5.0; ch.gems.push(Gem { p: vec3(c.x + (g as f32).sin() * 1.5, c.y, gz), got: false }); }
    }
    for _ in 0..rand::gen_range(2, 5) {
        let z = z0 + rand::gen_range(0.0, 80.0);
        let u = rand::gen_range(-7.0, 7.0);
        ch.gems.push(Gem { p: vec3(center(z) + u, height(u, z).max(0.0) + rand::gen_range(2.5, 8.0), z), got: false });
    }
    if idx >= 7 {
        for _ in 0..((idx - 5) / 3).min(4) {
            let z = z0 + rand::gen_range(0.0, 80.0);
            let u = rand::gen_range(-7.0, 7.0);
            ch.mines.push(Mine { p: vec3(center(z) + u, height(u, z).max(0.0) + rand::gen_range(3.0, 9.0), z), ph: rand::gen_range(0.0, TAU), dead: false });
        }
    }
    ch
}

// ---------------- game ----------------
#[derive(PartialEq, Clone, Copy)]
enum State { Title, Play, Over }
struct Part { p: Vec3, v: Vec3, life: f32, max: f32, col: [u8; 3], s: f32 }
struct Sfx { gem: Sound, ring: Sound, hit: Sound, boost: Sound, over: Sound }
fn play(s: &Sound, v: f32) { play_sound(s, PlaySoundParams { looped: false, volume: v }); }

struct Game {
    state: State, chunks: VecDeque<Chunk>, next_idx: i32,
    pos: Vec3, vel: Vec2, roll: f32, pitch: f32, lives: i32, inv: f32, energy: f32, boosting: bool,
    speed: f32, gems: u32, rings: u32, t: f32, over_t: f32, shake: f32, best: u32,
    parts: Vec<Part>, streaks: Vec<Vec3>, cam: Vec3, flash: f32, touch: bool, stick: Option<(u64, Vec2, Vec2)>, boost_touch: Option<u64>,
}

impl Game {
    fn new(best: u32, touch: bool) -> Game {
        let mut g = Game { state: State::Title, chunks: VecDeque::new(), next_idx: 0, pos: vec3(center(0.0), 6.0, 0.0), vel: Vec2::ZERO,
            roll: 0.0, pitch: 0.0, lives: 3, inv: 0.0, energy: 1.0, boosting: false, speed: 32.0, gems: 0, rings: 0, t: 0.0, over_t: 0.0,
            shake: 0.0, best, parts: vec![], streaks: vec![], cam: vec3(center(0.0), 9.0, -10.0), flash: 0.0, touch, stick: None, boost_touch: None };
        for _ in 0..VIEW { let c = gen_chunk(g.next_idx); g.chunks.push_back(c); g.next_idx += 1; }
        for _ in 0..40 { g.streaks.push(vec3(rand::gen_range(-12.0, 12.0), rand::gen_range(-4.0, 8.0), rand::gen_range(0.0, 60.0))); }
        g
    }
    fn score(&self) -> u32 { (self.pos.z / 2.0) as u32 + self.gems * 10 + self.rings * 50 }

    fn burst(&mut self, p: Vec3, col: [u8; 3], n: usize, spd: f32) {
        for _ in 0..n {
            let d = vec3(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)).normalize_or_zero();
            let life = rand::gen_range(0.4, 1.0);
            self.parts.push(Part { p, v: d * spd * rand::gen_range(0.3, 1.0), life, max: life, col, s: rand::gen_range(0.15, 0.4) });
        }
    }

    fn input(&mut self) -> (Vec2, bool) {
        let mut d = Vec2::ZERO;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { d.x -= 1.0 }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { d.x += 1.0 }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { d.y += 1.0 }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { d.y -= 1.0 }
        let mut boost = is_key_down(KeyCode::Space) || is_key_down(KeyCode::LeftShift);
        let bpos = vec2(vw() - 110.0, VH - 110.0);
        for t in touches() {
            let p = t.position / sc();
            match t.phase {
                TouchPhase::Started => {
                    self.touch = true;
                    if p.distance(bpos) < 70.0 { self.boost_touch = Some(t.id); }
                    else if self.stick.is_none() { self.stick = Some((t.id, p, p)); }
                }
                TouchPhase::Moved | TouchPhase::Stationary => { if let Some(s) = &mut self.stick { if s.0 == t.id { s.2 = p; } } }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if self.stick.map_or(false, |s| s.0 == t.id) { self.stick = None; }
                    if self.boost_touch == Some(t.id) { self.boost_touch = None; }
                }
            }
        }
        if let Some((_, o, c)) = self.stick { let v = (c - o) / 70.0; d = vec2(v.x, -v.y).clamp_length_max(1.0); }
        if self.boost_touch.is_some() { boost = true; }
        (d.clamp_length_max(1.0), boost)
    }

    fn update(&mut self, dt: f32, sfx: &Sfx) {
        self.t += dt;
        self.inv -= dt;
        self.shake = (self.shake - dt * 2.5).max(0.0);
        self.flash = (self.flash - dt * 3.0).max(0.0);
        let (mut d, boost_in) = if self.state == State::Play { self.input() } else { (Vec2::ZERO, false) };
        if self.state == State::Title {
            // autopilot for the attract screen
            let u = self.pos.x - center(self.pos.z + 20.0);
            d = vec2((-u * 0.15).clamp(-1.0, 1.0), ((7.0 - self.pos.y) * 0.2).clamp(-1.0, 1.0));
        }
        let was = self.boosting;
        self.boosting = boost_in && self.energy > 0.05;
        if self.boosting { self.energy = (self.energy - dt * 0.35).max(0.0); } else { self.energy = (self.energy + dt * 0.08).min(1.0); }
        if self.boosting && !was { play(&sfx.boost, 0.5); }
        let base = (32.0 + self.pos.z * 0.004).min(70.0);
        let target = base * if self.boosting { 1.6 } else { 1.0 };
        self.speed += (target - self.speed) * (dt * 3.0).min(1.0);

        self.vel = self.vel.lerp(vec2(d.x * 19.0, d.y * 13.0), (dt * 6.0).min(1.0));
        let oz = self.pos.z;
        self.pos.z += self.speed * dt;
        self.pos.x += self.vel.x * dt + (center(self.pos.z) - center(oz)) * 0.6;
        self.pos.y += self.vel.y * dt;
        let u = (self.pos.x - center(self.pos.z)).clamp(-16.0, 16.0);
        self.pos.x = center(self.pos.z) + u;
        self.pos.y = self.pos.y.clamp(1.0, 24.0);
        self.roll += (-self.vel.x * 0.045 - self.roll) * (dt * 6.0).min(1.0);
        self.pitch += (self.vel.y * 0.035 - self.pitch) * (dt * 6.0).min(1.0);

        // chunks
        if self.pos.z > self.chunks[0].z0 + CH_LEN + 30.0 {
            self.chunks.pop_front();
            let c = gen_chunk(self.next_idx); self.chunks.push_back(c); self.next_idx += 1;
        }

        // collisions & pickups
        let p = self.pos;
        let mut crash: Option<Vec3> = None;
        let mut got_gems = 0; let mut boom = vec![];
        if p.y - 0.6 < ground(p.x, p.z) || p.y - 0.6 < ground(p.x, p.z + 1.5) { crash = Some(p); }
        for ch in self.chunks.iter_mut() {
            if p.z < ch.z0 - 4.0 || p.z > ch.z0 + CH_LEN + 4.0 { continue; }
            for pl in &ch.pillars {
                if (pl.p.z - p.z).abs() < pl.r + 1.0 && vec2(pl.p.x - p.x, pl.p.z - p.z).length() < pl.r + 0.8 && p.y < pl.top + 0.5 { crash = Some(p); }
            }
            for g in ch.gems.iter_mut() { if !g.got && g.p.distance(p) < 2.0 { g.got = true; got_gems += 1; } }
            for r in ch.rings.iter_mut() {
                if !r.done && oz < r.p.z && p.z >= r.p.z { r.done = true; if vec2(p.x - r.p.x, p.y - r.p.y).length() < 2.7 { boom.push(r.p); } }
            }
            for mn in ch.mines.iter_mut() {
                let mp = mn.p + vec3(0.0, (self.t * 1.5 + mn.ph).sin() * 2.0, 0.0);
                if !mn.dead && mp.distance(p) < 1.7 { mn.dead = true; crash = Some(mp); }
            }
        }
        if self.state != State::Play { crash = None; }
        if got_gems > 0 { self.gems += got_gems; self.energy = (self.energy + 0.08 * got_gems as f32).min(1.0); play(&sfx.gem, 0.45); let pp = self.pos; self.burst(pp, [90, 240, 200], 8, 6.0); }
        for rp in boom { self.rings += 1; self.flash = 0.6; play(&sfx.ring, 0.6); self.burst(rp, [255, 210, 90], 30, 9.0); }
        if let (Some(cp), true) = (crash, self.inv <= 0.0) {
            self.lives -= 1; self.inv = 1.6; self.shake = 1.0;
            self.burst(cp, [255, 120, 50], 40, 12.0); self.burst(cp, [80, 60, 60], 20, 6.0);
            play(&sfx.hit, 0.8);
            self.pos.y = self.pos.y.max(ground(self.pos.x, self.pos.z) + 4.0);
            self.pos.x += (center(self.pos.z) - self.pos.x) * 0.5;
            if self.lives <= 0 { self.state = State::Over; self.over_t = 0.0; self.best = self.best.max(self.score()); play(&sfx.over, 0.7); }
        }

        // particles & streaks
        for pt in self.parts.iter_mut() { pt.p += pt.v * dt; pt.v *= 1.0 - dt * 1.5; pt.life -= dt; }
        self.parts.retain(|p| p.life > 0.0);
        if (self.t * 40.0) as i32 % 2 == 0 && self.state != State::Over {
            let r = Quat::from_rotation_z(self.roll);
            for s in [-1.0, 1.0] {
                let tip = self.pos + r * vec3(3.0 * s, 0.1, -0.6);
                self.parts.push(Part { p: tip, v: Vec3::ZERO, life: 0.35, max: 0.35, col: if self.boosting { [255, 200, 120] } else { [255, 240, 230] }, s: 0.12 });
            }
        }
        for s in self.streaks.iter_mut() {
            if s.z < self.pos.z - 5.0 { *s = vec3(self.pos.x + rand::gen_range(-14.0, 14.0), self.pos.y + rand::gen_range(-5.0, 8.0), self.pos.z + rand::gen_range(40.0, 70.0)); }
        }

        // camera
        let want = self.pos + vec3(-self.vel.x * 0.08, 2.6, -7.8);
        self.cam = self.cam.lerp(want, (dt * 5.0).min(1.0));
    }

    fn draw(&self, mat: &Material) {
        // sky
        set_default_camera();
        let (w, h) = (screen_width(), screen_height());
        let top = vec3(0.16, 0.08, 0.28); let mid = vec3(0.72, 0.30, 0.42); let hor = vec3(1.0, 0.62, 0.38);
        for i in 0..48 {
            let t = i as f32 / 47.0;
            let c = if t < 0.6 { top.lerp(mid, t / 0.6) } else { mid.lerp(hor, (t - 0.6) / 0.4) };
            draw_rectangle(0.0, t * h * 0.62, w, h * 0.62 / 47.0 + 1.0, Color::new(c.x, c.y, c.z, 1.0));
        }
        draw_rectangle(0.0, h * 0.62, w, h * 0.38, Color::new(hor.x, hor.y, hor.z, 1.0));
        let (sx, sy) = (w * 0.38 - self.roll * w * 0.05, h * 0.47);
        for i in (1..6).rev() { draw_circle(sx, sy, h * 0.06 * (1.0 + i as f32 * 0.6), Color::new(1.0, 0.85, 0.55, 0.06)); }
        draw_circle(sx, sy, h * 0.075, Color::new(1.0, 0.93, 0.7, 1.0));

        // 3D
        let sh = vec3(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0), 0.0) * self.shake * 0.6;
        let fov = 1.0 + (self.speed - 32.0).max(0.0) * 0.006;
        set_camera(&Camera3D {
            position: self.cam + sh, target: self.pos + vec3(0.0, 1.0, 16.0) + sh,
            up: vec3((-self.roll * 0.25).sin(), (-self.roll * 0.25).cos(), 0.0), fovy: fov, ..Default::default()
        });
        mat.set_uniform("FogColor", vec4(hor.x, hor.y, hor.z, 1.0));
        mat.set_uniform("FogRange", vec4(90.0, CH_LEN * (VIEW as f32 - 1.0) - 20.0, 0.0, 0.0));
        gl_use_material(mat);
        for ch in &self.chunks { draw_mesh(&ch.mesh); }

        let mut m = new_mesh();
        for ch in &self.chunks {
            for g in &ch.gems { if !g.got { octa(&mut m, g.p + vec3(0.0, (self.t * 3.0 + g.p.z).sin() * 0.3, 0.0), 0.5, 0.75, self.t * 2.5, [110, 255, 220], [40, 190, 170]); } }
            for mn in &ch.mines {
                if mn.dead { continue; }
                let mp = mn.p + vec3(0.0, (self.t * 1.5 + mn.ph).sin() * 2.0, 0.0);
                octa(&mut m, mp, 1.0, 1.0, self.t, [235, 50, 60], [120, 20, 40]);
                for k in 0..4 { let v = Vec2::from_angle(self.t + k as f32 * PI / 2.0) * 1.6; let d = vec3(v.x, 0.0, v.y); octa(&mut m, mp + d, 0.15, 0.15, 0.0, [255, 200, 80], [200, 120, 40]); }
            }
        }
        for s in &self.streaks {
            let a = *s; let b = a + vec3(0.0, 0.0, 3.0 + self.speed * 0.05);
            quad(&mut m, a, a + vec3(0.05, 0.0, 0.0), b + vec3(0.05, 0.0, 0.0), b, [255, 236, 220]);
        }
        for p in &self.parts { let k = p.life / p.max; octa(&mut m, p.p, p.s * k, p.s * k, p.life * 4.0, p.col, p.col); }
        if self.state != State::Over && !(self.inv > 0.0 && (self.t * 14.0) as i32 % 2 == 0) { self.dragon(&mut m); }
        if m.vertices.len() < 60000 { draw_mesh(&m); }
        gl_use_default_material();
        set_camera(&Camera2D { target: vec2(vw(), VH) / 2.0, zoom: vec2(2.0 / vw(), 2.0 / VH), ..Default::default() });
        if self.flash > 0.0 { draw_rectangle(0.0, 0.0, vw(), VH, Color::new(1.0, 0.85, 0.4, self.flash * 0.25)); }
        if self.inv > 1.2 { draw_rectangle(0.0, 0.0, vw(), VH, Color::new(1.0, 0.2, 0.1, (self.inv - 1.2) * 0.6)); }
    }

    fn dragon(&self, m: &mut Mesh) {
        let rot = Quat::from_rotation_y(self.vel.x * 0.012) * Quat::from_rotation_x(-self.pitch) * Quat::from_rotation_z(self.roll);
        let s = 1.15;
        let w = |v: Vec3| self.pos + rot * (v * s);
        let (red, dark, gold, wing) = ([206, 44, 44], [140, 26, 34], [236, 176, 64], [120, 22, 40]);
        let (f, b, t, d, l, r) = (vec3(0.0, 0.0, 1.7), vec3(0.0, 0.1, -1.2), vec3(0.0, 0.5, 0.0), vec3(0.0, -0.38, 0.0), vec3(-0.52, 0.0, 0.0), vec3(0.52, 0.0, 0.0));
        for (a, bb, c, col) in [(f, t, l, red), (f, r, t, red), (f, l, d, gold), (f, d, r, gold), (b, l, t, dark), (b, t, r, dark), (b, d, l, gold), (b, r, d, gold)] { tri(m, w(a), w(bb), w(c), col); }
        // head & horns
        let hd = vec3(0.0, 0.25, 2.3);
        tri(m, w(f + vec3(-0.25, 0.1, 0.0)), w(hd), w(f + vec3(0.25, 0.1, 0.0)), red);
        tri(m, w(f + vec3(-0.25, 0.1, 0.0)), w(f + vec3(0.0, -0.2, 0.2)), w(hd), dark);
        tri(m, w(f + vec3(0.0, -0.2, 0.2)), w(f + vec3(0.25, 0.1, 0.0)), w(hd), dark);
        for sx in [-1.0f32, 1.0] { tri(m, w(vec3(0.15 * sx, 0.35, 1.5)), w(vec3(0.35 * sx, 0.9, 1.0)), w(vec3(0.1 * sx, 0.3, 1.2)), gold); }
        // tail
        let tip = vec3((self.t * 3.0).sin() * 0.4, 0.25, -3.0);
        tri(m, w(b + vec3(-0.22, 0.0, 0.1)), w(b + vec3(0.22, 0.0, 0.1)), w(tip), dark);
        tri(m, w(tip), w(tip + vec3(-0.35, 0.3, -0.3)), w(tip + vec3(0.35, 0.3, -0.3)), gold);
        // wings
        let flap = (self.t * if self.boosting { 13.0 } else { 7.0 }).sin() * 0.9;
        for sx in [-1.0f32, 1.0] {
            let rf = vec3(0.4 * sx, 0.18, 0.55); let rb = vec3(0.4 * sx, 0.18, -0.6);
            let mid = vec3(2.0 * sx, 0.35 + flap * 0.6, 0.6); let tipw = vec3(3.3 * sx, 0.2 + flap, -0.3);
            tri(m, w(rf), w(mid), w(rb), wing);
            tri(m, w(mid), w(tipw), w(rb), wing);
            tri(m, w(mid), w(mid + vec3(0.0, 0.05, 0.1)), w(tipw), gold);
        }
    }

    fn hud(&self) {
        let w = vw();
        let s = format!("{}", self.score());
        txt(&s, w / 2.0 - tw(&s, 48.0) / 2.0, 54.0, 48.0, WHITE);
        txt(&format!("{} m", (self.pos.z / 2.0) as u32), 24.0, 40.0, 26.0, Color::new(1.0, 0.92, 0.8, 1.0));
        txt(&format!("{} km/soat", (self.speed * 3.6) as u32), 24.0, 68.0, 20.0, Color::new(1.0, 0.85, 0.7, 0.8));
        for i in 0..3 {
            let c = if i < self.lives { Color::new(1.0, 0.3, 0.3, 1.0) } else { Color::new(0.3, 0.1, 0.12, 0.6) };
            draw_poly(w - 40.0 - i as f32 * 34.0, 36.0, 4, 12.0, 45.0, c);
        }
        let gs = format!("Kristall {}   Halqa {}", self.gems, self.rings); txt(&gs, w - 24.0 - tw(&gs, 20.0), 76.0, 20.0, Color::new(0.6, 1.0, 0.9, 1.0));
        // boost bar
        draw_rectangle(24.0, VH - 40.0, 220.0, 12.0, Color::new(0.0, 0.0, 0.0, 0.4));
        draw_rectangle(24.0, VH - 40.0, 220.0 * self.energy, 12.0, if self.boosting { Color::new(1.0, 0.8, 0.3, 1.0) } else { Color::new(0.4, 0.95, 1.0, 1.0) });
        txt("TURBO", 24.0, VH - 48.0, 18.0, WHITE);
        if self.touch {
            if let Some((_, o, c)) = self.stick {
                draw_circle_lines(o.x, o.y, 70.0, 3.0, Color::new(1.0, 1.0, 1.0, 0.35));
                let k = o + (c - o).clamp_length_max(70.0);
                draw_circle(k.x, k.y, 26.0, Color::new(1.0, 1.0, 1.0, 0.3));
            }
            let b = vec2(w - 110.0, VH - 110.0);
            draw_circle(b.x, b.y, 58.0, Color::new(1.0, 0.7, 0.3, if self.boosting { 0.5 } else { 0.25 }));
            draw_circle_lines(b.x, b.y, 58.0, 3.0, Color::new(1.0, 0.9, 0.7, 0.8));
            txt("TURBO", b.x - tw("TURBO", 22.0) / 2.0, b.y + 8.0, 22.0, WHITE);
        }
    }
}

fn txt(t: &str, x: f32, y: f32, size: f32, color: Color) {
    let s = sc();
    draw_text_ex(t, x + 2.0, y + 2.0, TextParams { font_size: (size * s) as u16, font_scale: 1.0 / s, color: Color::new(0.0, 0.0, 0.0, color.a * 0.45), ..Default::default() });
    draw_text_ex(t, x, y, TextParams { font_size: (size * s) as u16, font_scale: 1.0 / s, color, ..Default::default() });
}
fn tw(t: &str, size: f32) -> f32 { let s = sc(); measure_text(t, None, (size * s) as u16, 1.0 / s).width }
fn centered(t: &str, y: f32, size: f32, col: Color) { txt(t, vw() / 2.0 - tw(t, size) / 2.0, y, size, col); }
fn tapped() -> bool { is_mouse_button_pressed(MouseButton::Left) || touches().iter().any(|t| t.phase == TouchPhase::Started) }

// ---------------- audio ----------------
const SR: f32 = 22050.0;
fn wav(dur: f32, mut f: impl FnMut(f32) -> f32) -> Vec<u8> {
    let n = (dur * SR) as u32;
    let mut b = Vec::with_capacity(44 + n as usize * 2);
    let u32le = |b: &mut Vec<u8>, v: u32| b.extend_from_slice(&v.to_le_bytes());
    b.extend_from_slice(b"RIFF"); u32le(&mut b, 36 + n * 2); b.extend_from_slice(b"WAVEfmt ");
    u32le(&mut b, 16); b.extend_from_slice(&[1, 0, 1, 0]); u32le(&mut b, SR as u32); u32le(&mut b, SR as u32 * 2);
    b.extend_from_slice(&[2, 0, 16, 0]); b.extend_from_slice(b"data"); u32le(&mut b, n * 2);
    for i in 0..n { b.extend_from_slice(&((f(i as f32 / SR).clamp(-1.0, 1.0) * 32000.0) as i16).to_le_bytes()); }
    b
}
fn noise() -> f32 { rand::gen_range(-1.0, 1.0) }
fn sine(f: f32, t: f32) -> f32 { (TAU * f * t).sin() }
async fn snd(dur: f32, f: impl FnMut(f32) -> f32) -> Sound { load_sound_from_bytes(&wav(dur, f)).await.unwrap() }
async fn load_audio() -> (Sfx, Sound, Sound) {
    let sfx = Sfx {
        gem: snd(0.35, |t| (sine(1568.0, t) * 0.5 + sine(2093.0, t) * 0.3) * (-t * 9.0).exp() * 0.6).await,
        ring: snd(0.7, |t| { let f = [523.25, 659.25, 783.99, 1046.5][((t / 0.08) as usize).min(3)]; (sine(f, t) * 0.4 + sine(f * 2.0, t) * 0.1) * (1.0 - t / 0.7) }).await,
        hit: snd(0.6, |t| (noise() * 0.7 + sine(70.0 - t * 40.0, t) * 0.8) * (-t * 6.0).exp()).await,
        boost: { let mut y = 0.0; snd(0.6, move |t| { y += (noise() - y) * (0.05 + t * 0.3); y * 1.8 * (PI * t / 0.6).sin() }).await },
        over: snd(1.2, |t| (sine(440.0 * (1.0 - t * 0.5), t) * 0.4 + sine(220.0 * (1.0 - t * 0.5), t) * 0.3) * (1.0 - t / 1.2)).await,
    };
    let wind = { let mut y = 0.0; snd(4.0, move |t| { y += (noise() - y) * 0.02; y * 2.2 * (0.7 + 0.3 * (t * TAU / 4.0).sin()) }).await };
    let music = snd(8.0, |t| {
        let roots = [110.0, 87.31, 130.81, 98.0];
        let i = ((t / 2.0) as usize).min(3);
        let f = roots[i];
        let pad = (sine(f, t) + sine(f * 1.5, t) * 0.6 + sine(f * 2.0, t) * 0.4) * 0.07;
        let arp = [2.0, 3.0, 4.0, 3.0][((t * 4.0) as usize) % 4] * f;
        let pl = sine(arp, t) * (-(t % 0.25) * 8.0).exp() * 0.06;
        let bass = sine(f / 2.0, t) * 0.12 * (0.6 + 0.4 * (-(t % 0.5) * 4.0).exp());
        (pad + pl + bass) * (t * 20.0).min(1.0) * ((8.0 - t) * 20.0).min(1.0)
    }).await;
    (sfx, wind, music)
}

// ---------------- shader (fog) ----------------
const FOG_VS: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
attribute vec4 normal;
varying lowp vec4 color;
varying mediump float fogf;
uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 FogRange;
void main() {
    vec4 p = Projection * Model * vec4(position, 1.0);
    gl_Position = p;
    color = color0 / 255.0;
    fogf = clamp((p.w - FogRange.x) / (FogRange.y - FogRange.x), 0.0, 1.0);
}"#;
const FOG_FS: &str = r#"#version 100
precision mediump float;
varying lowp vec4 color;
varying mediump float fogf;
uniform vec4 FogColor;
void main() {
    gl_FragColor = vec4(mix(color.rgb, FogColor.rgb, fogf * fogf), 1.0);
}"#;

fn conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: Conf { window_title: "AJDAR PARVOZI".into(), window_width: 1280, window_height: 720, high_dpi: true, sample_count: 1, ..Default::default() },
        draw_call_vertex_capacity: 60000,
        draw_call_index_capacity: 60000,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    std::panic::set_hook(Box::new(|info| { macroquad::logging::error!("ZQ PANIC: {}", info); }));
    rand::srand(macroquad::miniquad::date::now() as u64);
    let mat = load_material(
        ShaderSource::Glsl { vertex: FOG_VS, fragment: FOG_FS },
        MaterialParams {
            pipeline_params: PipelineParams { depth_test: Comparison::LessOrEqual, depth_write: true, ..Default::default() },
            uniforms: vec![UniformDesc::new("FogColor", UniformType::Float4), UniformDesc::new("FogRange", UniformType::Float4)],
            ..Default::default()
        },
    ).unwrap();
    macroquad::logging::info!("ZQ material ok");
    let (sfx, wind, music) = load_audio().await;
    play_sound(&music, PlaySoundParams { looped: true, volume: 0.4 });
    play_sound(&wind, PlaySoundParams { looped: true, volume: 0.25 });
    macroquad::logging::info!("ZQ audio ok");
    let mut g = Game::new(0, cfg!(target_os = "android"));
    let mut frames = 0u32;
    loop {
        frames += 1;
        if frames == 1 || frames == 60 { macroquad::logging::info!("ZQ frame {}", frames); }
        let dt = get_frame_time().min(1.0 / 30.0);
        clear_background(BLACK);
        g.update(dt, &sfx);
        g.draw(&mat);
        match g.state {
            State::Title => {
                let p = 0.7 + 0.3 * (g.t * 3.0).sin();
                centered("AJDAR PARVOZI", VH * 0.3, 92.0, Color::new(1.0, 0.95, 0.85, 1.0));
                centered("Kanyon bo'ylab uch, kristallarni yig', qoyalarga urilma", VH * 0.3 + 44.0, 24.0, Color::new(1.0, 0.9, 0.8, 0.9));
                let help = if g.touch { "Barmoq bilan suring: boshqarish   TURBO: tezlanish" } else { "WASD / strelkalar: boshqarish   Space: turbo" };
                centered(help, VH * 0.72, 22.0, WHITE);
                centered(if g.touch { "[ Boshlash uchun bosing ]" } else { "[ ENTER yoki bosing ]" }, VH * 0.82, 32.0, Color::new(1.0, 1.0, 1.0, p));
                if g.best > 0 { centered(&format!("Rekord: {}", g.best), VH * 0.92, 22.0, Color::new(1.0, 0.9, 0.7, 0.8)); }
                if is_key_pressed(KeyCode::Enter) || tapped() {
                    let tm = g.touch || !touches().is_empty();
                    g = Game::new(g.best, tm);
                    g.state = State::Play;
                }
            }
            State::Play => { g.hud(); if is_key_pressed(KeyCode::Escape) { g.state = State::Title; } }
            State::Over => {
                g.over_t += dt;
                draw_rectangle(0.0, 0.0, vw(), VH, Color::new(0.1, 0.02, 0.05, (g.over_t * 0.5).min(0.5)));
                centered("AJDAR QULADI", VH * 0.34, 80.0, Color::new(1.0, 0.45, 0.35, 1.0));
                centered(&format!("Ochko: {}    Rekord: {}", g.score(), g.best), VH * 0.48, 34.0, WHITE);
                centered(&format!("Masofa: {} m   Kristall: {}   Halqa: {}", (g.pos.z / 2.0) as u32, g.gems, g.rings), VH * 0.56, 24.0, Color::new(1.0, 0.9, 0.8, 0.9));
                if g.over_t > 1.2 {
                    centered(if g.touch { "[ Qaytadan - bosing ]" } else { "[ ENTER - qaytadan ]" }, VH * 0.7, 30.0, Color::new(1.0, 0.85, 0.5, 1.0));
                    if is_key_pressed(KeyCode::Enter) || tapped() { let (b, tm) = (g.best, g.touch); g = Game::new(b, tm); g.state = State::Play; }
                }
            }
        }
        next_frame().await;
    }
}
