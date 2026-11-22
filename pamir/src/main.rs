// PAMIR POYGASI — low-poly 3D mountain racing (Rust + macroquad)
use macroquad::audio::{load_sound_from_bytes, play_sound, set_sound_volume, PlaySoundParams, Sound};
use macroquad::material::{gl_use_default_material, gl_use_material, load_material, Material, MaterialParams};
use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;
use std::collections::VecDeque;
use std::f32::consts::{PI, TAU};

const VH: f32 = 720.0;
const CH: f32 = 80.0;
const ROWS: usize = 16;
const COLS: usize = 22;
const VIEW: usize = 6;
const RW: f32 = 7.0; // half road width
const RACE: f32 = 4000.0;
const TOP: f32 = 66.0;

fn sc() -> f32 { screen_height() / VH }
fn vw() -> f32 { screen_width() / sc() }

// ---------------- track ----------------
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
fn center(z: f32) -> f32 { (z * 0.004).sin() * 55.0 + (z * 0.0105).sin() * 20.0 + (z * 0.021).sin() * 5.0 }
fn curv(z: f32) -> f32 { (center(z + 2.0) - 2.0 * center(z) + center(z - 2.0)) / 4.0 }
fn slope_x(z: f32) -> f32 { (center(z + 1.0) - center(z - 1.0)) / 2.0 }
fn elev(z: f32) -> f32 { (z * 0.003).sin() * 22.0 + (z * 0.008).sin() * 7.0 }
fn height(u: f32, z: f32) -> f32 {
    let e = elev(z);
    let au = u.abs();
    if au <= RW + 1.0 { return e; }
    let n = vnoise(u * 0.08 + 3.0, z * 0.03) * 2.0 + vnoise(u * 0.25, z * 0.12) * 0.7;
    if u < 0.0 { e + smooth(RW + 1.0, 30.0, au) * (8.0 + n * 14.0) + smooth(30.0, 75.0, au) * (25.0 + n * 20.0) }
    else { e - smooth(RW + 2.0, 40.0, au) * (18.0 + n * 6.0) + smooth(45.0, 77.0, au) * (40.0 + n * 25.0) }
}

// ---------------- mesh helpers ----------------
fn light() -> Vec3 { vec3(0.4, 0.75, -0.35).normalize() }
fn tri(m: &mut Mesh, a: Vec3, b: Vec3, c: Vec3, col: [u8; 3]) {
    let n = (b - a).cross(c - a).normalize_or_zero();
    let s = 0.5 + 0.5 * n.dot(light()).abs();
    let col = [(col[0] as f32 * s) as u8, (col[1] as f32 * s) as u8, (col[2] as f32 * s) as u8, 255];
    let i = m.vertices.len() as u16;
    for p in [a, b, c] { m.vertices.push(Vertex { position: p, uv: Vec2::ZERO, color: col, normal: Vec4::ZERO }); }
    m.indices.extend_from_slice(&[i, i + 1, i + 2]);
}
fn quad(m: &mut Mesh, a: Vec3, b: Vec3, c: Vec3, d: Vec3, col: [u8; 3]) { tri(m, a, b, c, col); tri(m, a, c, d, col); }
fn new_mesh() -> Mesh { Mesh { vertices: Vec::with_capacity(4096), indices: Vec::with_capacity(4096), texture: None } }
fn bx(m: &mut Mesh, c: Vec3, half: Vec3, rot: Quat, col: [u8; 3]) {
    let p = |x: f32, y: f32, z: f32| c + rot * vec3(x * half.x, y * half.y, z * half.z);
    let v = [p(-1.0, -1.0, -1.0), p(1.0, -1.0, -1.0), p(1.0, 1.0, -1.0), p(-1.0, 1.0, -1.0), p(-1.0, -1.0, 1.0), p(1.0, -1.0, 1.0), p(1.0, 1.0, 1.0), p(-1.0, 1.0, 1.0)];
    for f in [[0, 1, 2, 3], [5, 4, 7, 6], [4, 0, 3, 7], [1, 5, 6, 2], [3, 2, 6, 7], [4, 5, 1, 0]] { quad(m, v[f[0]], v[f[1]], v[f[2]], v[f[3]], col); }
}
fn wpos(s: f32, u: f32) -> Vec3 { vec3(center(s) - u, elev(s), s) }

// ---------------- chunks ----------------
struct Chunk { z0: f32, mesh: Mesh, cans: Vec<(f32, f32, bool)> }
fn tcol(h: f32, rel: f32, n: Vec3) -> [u8; 3] {
    let steep = 1.0 - n.y.abs();
    if h > 34.0 && steep < 0.6 { [236, 240, 246] } else if steep > 0.5 { [128, 116, 108] } else if rel < -8.0 { [92, 130, 70] } else if h > 22.0 { [150, 138, 120] } else { [110, 150, 78] }
}
fn gen_chunk(idx: i32) -> Chunk {
    let z0 = idx as f32 * CH;
    let mut m = new_mesh();
    let dz = CH / ROWS as f32;
    let us: Vec<f32> = (0..=COLS).map(|c| { let t = c as f32 / COLS as f32 * 2.0 - 1.0; t.signum() * (t.abs().powf(1.6) * 78.0) }).collect();
    let p = |c: usize, r: usize| { let z = z0 + r as f32 * dz; let u = us[c]; vec3(center(z) - u, height(u, z) - if u.abs() <= RW + 1.0 { 0.15 } else { 0.0 }, z) };
    for r in 0..ROWS {
        for c in 0..COLS {
            if us[c] >= -RW - 1.0 && us[c + 1] <= RW + 1.0 { continue; }
            let (a, b, cc, d) = (p(c, r), p(c + 1, r), p(c + 1, r + 1), p(c, r + 1));
            for (x, y, w) in [(a, cc, b), (a, d, cc)] {
                let n = (y - x).cross(w - x).normalize_or_zero();
                let h = (x.y + y.y + w.y) / 3.0;
                tri(&mut m, x, y, w, tcol(h, h - elev(x.z), n));
            }
        }
    }
    // road strip
    let steps = 32;
    for i in 0..steps {
        let (s0, s1) = (z0 + i as f32 * CH / steps as f32, z0 + (i + 1) as f32 * CH / steps as f32);
        let q = |s: f32, u: f32, y: f32| wpos(s, u) + vec3(0.0, y, 0.0);
        quad(&mut m, q(s0, -RW - 1.0, 0.0), q(s0, RW + 1.0, 0.0), q(s1, RW + 1.0, 0.0), q(s1, -RW - 1.0, 0.0), [74, 76, 84]);
        for e in [-RW, RW - 0.4] { quad(&mut m, q(s0, e, 0.03), q(s0, e + 0.4, 0.03), q(s1, e + 0.4, 0.03), q(s1, e, 0.03), [235, 235, 235]); }
        if i % 2 == 0 { quad(&mut m, q(s0, -0.15, 0.03), q(s0, 0.15, 0.03), q(s1, 0.15, 0.03), q(s1, -0.15, 0.03), [250, 200, 60]); }
        if idx == (RACE / CH) as i32 && i < 2 {
            for k in 0..8 { let u0 = -RW + k as f32 * RW / 4.0; quad(&mut m, q(s0, u0, 0.04), q(s0, u0 + RW / 4.0, 0.04), q(s1, u0 + RW / 4.0, 0.04), q(s1, u0, 0.04), if (k + i) % 2 == 0 { [250, 250, 250] } else { [20, 20, 20] }); }
        }
    }
    // guard rail on the valley side + posts
    for i in 0..10 {
        let (s0, s1) = (z0 + i as f32 * 8.0, z0 + (i + 1) as f32 * 8.0);
        let (a, b) = (wpos(s0, RW + 1.6), wpos(s1, RW + 1.6));
        quad(&mut m, a + vec3(0.0, 0.5, 0.0), b + vec3(0.0, 0.5, 0.0), b + vec3(0.0, 0.85, 0.0), a + vec3(0.0, 0.85, 0.0), [210, 214, 222]);
        bx(&mut m, a + vec3(0.0, 0.4, 0.0), vec3(0.1, 0.4, 0.1), Quat::IDENTITY, [90, 90, 100]);
    }
    // pine trees on slopes
    for _ in 0..14 {
        let z = z0 + rand::gen_range(0.0, CH);
        let u = if rand::gen_range(0, 2) == 0 { rand::gen_range(-45.0, -11.0) } else { rand::gen_range(12.0, 40.0) };
        let base = vec3(center(z) - u, height(u, z), z);
        if base.y > 30.0 { continue; }
        let hgt = rand::gen_range(3.5, 6.5);
        bx(&mut m, base + vec3(0.0, 0.6, 0.0), vec3(0.2, 0.6, 0.2), Quat::IDENTITY, [100, 70, 50]);
        for lvl in 0..2 {
            let y0 = base.y + 1.0 + lvl as f32 * hgt * 0.35; let r = 1.6 - lvl as f32 * 0.5; let top = vec3(base.x, y0 + hgt * 0.6, base.z);
            for k in 0..5 {
                let (a0, a1) = (k as f32 * TAU / 5.0, (k + 1) as f32 * TAU / 5.0);
                tri(&mut m, top, vec3(base.x + a0.cos() * r, y0, base.z + a0.sin() * r), vec3(base.x + a1.cos() * r, y0, base.z + a1.sin() * r), [40, 100, 60]);
            }
        }
    }
    let mut cans = vec![];
    if idx > 1 { for _ in 0..2 { cans.push((z0 + rand::gen_range(5.0, 75.0), rand::gen_range(-RW + 1.5, RW - 1.5), false)); } }
    Chunk { z0, mesh: m, cans }
}

// ---------------- cars ----------------
struct Car { s: f32, u: f32, h: f32, v: f32, col: [u8; 3], top: f32, lane: f32, roll: f32, finish: Option<f32> }
fn draw_car(m: &mut Mesh, c: &Car, t: f32) {
    let p = wpos(c.s, c.u);
    let pitch = -(elev(c.s + 1.0) - elev(c.s - 1.0)) / 2.0;
    let yaw = slope_x(c.s).atan() - c.h;
    let rot = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch) * Quat::from_rotation_z(c.roll);
    let o = |v: Vec3| p + rot * v;
    bx(m, o(vec3(0.0, 0.55, 0.0)), vec3(0.95, 0.32, 2.0), rot, c.col);
    bx(m, o(vec3(0.0, 1.05, -0.25)), vec3(0.78, 0.22, 0.95), rot, [(c.col[0] / 2) + 40, (c.col[1] / 2) + 40, (c.col[2] / 2) + 60]);
    bx(m, o(vec3(0.0, 1.05, 0.72)), vec3(0.74, 0.18, 0.05), rot, [150, 200, 240]);
    for (x, z) in [(-1.0, 1.3), (1.0, 1.3), (-1.0, -1.3), (1.0, -1.3)] {
        let spin = Quat::from_rotation_x(c.s * 0.8 + t);
        bx(m, o(vec3(x, 0.32, z)), vec3(0.18, 0.32, 0.32), rot * spin, [25, 25, 30]);
    }
    for x in [-0.6, 0.6] { bx(m, o(vec3(x, 0.6, -2.02)), vec3(0.22, 0.08, 0.03), rot, [255, 60, 50]); bx(m, o(vec3(x, 0.6, 2.02)), vec3(0.2, 0.07, 0.03), rot, [255, 250, 210]); }
}

// ---------------- game ----------------
#[derive(PartialEq, Clone, Copy)]
enum State { Title, Count, Race, Done }
struct Sfx { can: Sound, hit: Sound, nitro: Sound, beep: Sound, go: Sound, finish: Sound }
fn play(s: &Sound, v: f32) { play_sound(s, PlaySoundParams { looped: false, volume: v }); }
struct Part { p: Vec3, v: Vec3, life: f32, col: [u8; 3] }

struct Game {
    state: State, chunks: VecDeque<Chunk>, next: i32, me: Car, ai: Vec<Car>, nitro: f32, boosting: bool, t: f32, race_t: f32,
    count: f32, cam: Vec3, shake: f32, parts: Vec<Part>, best: f32, touch: bool, place: usize,
}

impl Game {
    fn new(best: f32) -> Game {
        let mut g = Game { state: State::Title, chunks: VecDeque::new(), next: -1, me: Car { s: 10.0, u: 2.5, h: 0.0, v: 0.0, col: [230, 50, 40], top: TOP, lane: 2.5, roll: 0.0, finish: None },
            ai: vec![], nitro: 0.5, boosting: false, t: 0.0, race_t: 0.0, count: 3.0, cam: vec3(0.0, 5.0, 0.0), shake: 0.0, parts: vec![], best, touch: false, place: 6 };
        let cols = [[40, 110, 230], [250, 200, 40], [60, 190, 90], [240, 240, 240], [150, 70, 200]];
        for (i, c) in cols.iter().enumerate() {
            let lane = if i % 2 == 0 { -2.5 } else { 2.5 };
            g.ai.push(Car { s: 16.0 + i as f32 * 6.0, u: lane, h: 0.0, v: 0.0, col: *c, top: TOP * rand::gen_range(0.9, 1.04), lane, roll: 0.0, finish: None });
        }
        for _ in 0..VIEW { let c = gen_chunk(g.next); g.chunks.push_back(c); g.next += 1; }
        g.cam = wpos(0.0, 2.5) + vec3(0.0, 4.0, -8.0);
        g
    }

    fn input(&mut self) -> (f32, bool, bool) {
        let mut steer: f32 = 0.0;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) { steer -= 1.0 }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { steer += 1.0 }
        let mut brake = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);
        let mut nitro = is_key_down(KeyCode::Space) || is_key_down(KeyCode::LeftShift);
        let b = buttons();
        for t in touches() {
            if t.phase == TouchPhase::Ended || t.phase == TouchPhase::Cancelled { continue; }
            self.touch = true;
            let p = t.position / sc();
            if p.distance(b[0].0) < b[0].1 + 20.0 { steer -= 1.0 }
            else if p.distance(b[1].0) < b[1].1 + 20.0 { steer += 1.0 }
            else if p.distance(b[2].0) < b[2].1 + 15.0 { nitro = true }
            else if p.distance(b[3].0) < b[3].1 + 15.0 { brake = true }
        }
        (steer.clamp(-1.0, 1.0), brake, nitro)
    }

    fn update(&mut self, dt: f32, sfx: &Sfx) {
        self.t += dt;
        self.shake = (self.shake - dt * 3.0).max(0.0);
        let (steer, brake, nitro_in) = match self.state {
            State::Race => self.input(),
            State::Title | State::Done => { let k = curv(self.me.s + 15.0); (((-self.me.u * 0.15) - k * 30.0 - self.me.h * 2.0).clamp(-1.0, 1.0), false, false) }
            State::Count => (0.0, false, false),
        };
        if self.state == State::Count {
            let before = self.count.ceil();
            self.count -= dt;
            if self.count.ceil() != before && self.count > 0.0 { play(&sfx.beep, 0.6); }
            if self.count <= 0.0 { self.state = State::Race; play(&sfx.go, 0.7); }
        }
        let moving = self.state != State::Count;
        // player physics
        let was = self.boosting;
        self.boosting = nitro_in && self.nitro > 0.02 && moving;
        if self.boosting { self.nitro = (self.nitro - dt * 0.3).max(0.0); if !was { play(&sfx.nitro, 0.6); } }
        let me = &mut self.me;
        let off = me.u.abs() > RW + 0.5;
        let top = me.top * if self.boosting { 1.35 } else { 1.0 } * if off { 0.55 } else { 1.0 };
        if moving {
            if brake { me.v -= 60.0 * dt; } else if me.v < top { me.v += (if self.boosting { 30.0 } else { 16.0 }) * dt * (1.0 - me.v / (top + 10.0)).max(0.15); } else { me.v -= 25.0 * dt; }
            me.v = me.v.max(0.0);
        }
        let grip = (1.0 - me.v / (TOP * 1.6)).clamp(0.35, 1.0);
        me.h += steer * 1.6 * grip * dt * (me.v / 20.0).min(1.0);
        me.h *= 1.0 - dt * 1.8;
        me.h = me.h.clamp(-0.6, 0.6);
        let k = curv(me.s);
        me.u += (me.v * me.h.sin() + k * me.v * me.v * 0.9) * dt;
        me.s += me.v * me.h.cos() * dt;
        me.roll += ((-steer * 0.06 - k * me.v * 0.1).clamp(-0.12, 0.12) - me.roll) * (dt * 5.0).min(1.0);
        let drift = (steer.abs() > 0.5 && me.v > 40.0) || (k.abs() * me.v * me.v > 6.0);
        if drift && self.state == State::Race { self.nitro = (self.nitro + dt * 0.07).min(1.0); }
        let mut hit_wall = false;
        if me.u > RW + 1.3 { me.u = RW + 1.3; me.h = -me.h.abs() * 0.5; me.v *= 0.75; hit_wall = true; }
        if me.u < -RW - 4.0 { me.u = -RW - 4.0; me.h = me.h.abs() * 0.5; me.v *= 0.75; hit_wall = true; }
        if hit_wall && me.v > 15.0 { self.shake = 0.8; play(&sfx.hit, 0.6); let p = wpos(me.s, me.u); for _ in 0..15 { self.parts.push(Part { p: p + vec3(0.0, 0.5, 0.0), v: vec3(rand::gen_range(-4.0, 4.0), rand::gen_range(1.0, 6.0), rand::gen_range(-2.0, 6.0)), life: 0.6, col: [255, 220, 120] }); } }
        if off && me.v > 20.0 { self.shake = self.shake.max(0.25); }
        if (drift || off) && me.v > 25.0 && (self.t * 30.0) as i32 % 2 == 0 {
            for x in [-1.0, 1.0] { let p = wpos(me.s - 1.6, me.u + x); self.parts.push(Part { p, v: vec3(0.0, 1.5, 0.0), life: 0.7, col: if off { [150, 120, 80] } else { [220, 220, 225] } }); }
        }
        if self.boosting && (self.t * 30.0) as i32 % 2 == 0 { let p = wpos(me.s - 2.2, me.u) + vec3(0.0, 0.5, 0.0); self.parts.push(Part { p, v: vec3(0.0, 0.5, -6.0), life: 0.3, col: [120, 200, 255] }); }

        // AI cars
        let (ms, mu) = (self.me.s, self.me.u);
        for c in self.ai.iter_mut() {
            if !moving { continue; }
            let ahead = (ms - c.s).abs() < 8.0 && (mu - c.lane).abs() < 2.2 && ms > c.s;
            if ahead { c.lane = if c.lane > 0.0 { -2.5 } else { 2.5 }; }
            let target = c.top * (1.0 - curv(c.s + 20.0).abs() * 6.0).clamp(0.75, 1.0) * if c.finish.is_some() { 0.6 } else { 1.0 };
            c.v += ((target - c.v) * 0.6).clamp(-30.0, 14.0) * dt;
            c.u += (c.lane - c.u) * (dt * 1.5).min(1.0);
            c.h = ((c.lane - c.u) * 0.05).clamp(-0.2, 0.2);
            c.s += c.v * dt;
            c.roll = (-curv(c.s) * c.v * 0.1).clamp(-0.1, 0.1);
            if c.s >= RACE && c.finish.is_none() { c.finish = Some(self.race_t); }
        }
        // car-car collisions
        for c in self.ai.iter_mut() {
            let (ds, du) = (self.me.s - c.s, self.me.u - c.u);
            if ds.abs() < 4.2 && du.abs() < 2.0 {
                let push = (2.0 - du.abs()) * du.signum().max(-1.0) * if du == 0.0 { 1.0 } else { 1.0 };
                self.me.u += push * 0.5; c.u -= push * 0.5;
                if ds > 0.0 { c.v *= 0.97; } else { self.me.v *= 0.95; }
                if self.shake < 0.3 { play(&sfx.hit, 0.35); }
                self.shake = self.shake.max(0.4);
            }
        }
        // nitro cans
        for ch in self.chunks.iter_mut() {
            for can in ch.cans.iter_mut() {
                if !can.2 && (can.0 - self.me.s).abs() < 2.0 && (can.1 - self.me.u).abs() < 1.6 { can.2 = true; self.nitro = (self.nitro + 0.3).min(1.0); play(&sfx.can, 0.5); }
            }
        }
        // chunks
        let first = self.me.s.min(self.ai.iter().map(|c| c.s).fold(f32::MAX, f32::min)).max(self.me.s - 60.0);
        if first > self.chunks[0].z0 + CH + 20.0 { self.chunks.pop_front(); let c = gen_chunk(self.next); self.chunks.push_back(c); self.next += 1; }
        while self.me.s > self.chunks.back().unwrap().z0 - CH * 2.0 { self.chunks.pop_front(); let c = gen_chunk(self.next); self.chunks.push_back(c); self.next += 1; }
        // race state
        if self.state == State::Race {
            self.race_t += dt;
            self.place = 1 + self.ai.iter().filter(|c| c.s > self.me.s).count();
            if self.me.s >= RACE {
                self.me.finish = Some(self.race_t);
                self.state = State::Done;
                if self.place == 1 && (self.best == 0.0 || self.race_t < self.best) { self.best = self.race_t; }
                play(&sfx.finish, 0.8);
            }
        }
        for p in self.parts.iter_mut() { p.p += p.v * dt; p.life -= dt; }
        self.parts.retain(|p| p.life > 0.0);
        // camera
        let me = &self.me;
        let fwd = vec3(slope_x(me.s).atan().sin(), 0.0, slope_x(me.s).atan().cos());
        let want = wpos(me.s, me.u) - fwd * 8.5 + vec3(0.0, 3.6, 0.0);
        self.cam = self.cam.lerp(want, (dt * 6.0).min(1.0));
    }

    fn draw(&self, mat: &Material) {
        set_default_camera();
        let (w, h) = (screen_width(), screen_height());
        let top = vec3(0.24, 0.48, 0.86); let hor = vec3(0.74, 0.86, 0.97);
        for i in 0..40 { let t = i as f32 / 39.0; let c = top.lerp(hor, t); draw_rectangle(0.0, t * h * 0.55, w, h * 0.55 / 39.0 + 1.0, Color::new(c.x, c.y, c.z, 1.0)); }
        draw_rectangle(0.0, h * 0.55, w, h * 0.45, Color::new(hor.x, hor.y, hor.z, 1.0));
        let ang = slope_x(self.me.s).atan() - self.me.h;
        let off = -ang * w * 0.6;
        // far snowy peaks (2D parallax)
        for layer in 0..2 {
            let base = h * (0.5 + layer as f32 * 0.03);
            let col = if layer == 0 { Color::new(0.56, 0.64, 0.78, 1.0) } else { Color::new(0.46, 0.54, 0.68, 1.0) };
            for i in -2..14 {
                let x = (i as f32 * w * 0.13 + off * (0.3 + layer as f32 * 0.2)).rem_euclid(w * 1.82) - w * 0.4;
                let ph = h * (0.12 + h2(i, layer) * 0.14);
                let pw = w * (0.09 + h2(i + 9, layer) * 0.06);
                draw_triangle(vec2(x - pw, base), vec2(x + pw, base), vec2(x, base - ph), col);
                draw_triangle(vec2(x - pw * 0.32, base - ph * 0.68), vec2(x + pw * 0.32, base - ph * 0.68), vec2(x, base - ph), Color::new(0.95, 0.97, 1.0, 1.0));
            }
        }
        let sh = vec3(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0), 0.0) * self.shake * 0.25;
        let fov = 1.0 + (self.me.v / TOP - 0.5).max(0.0) * 0.25 + if self.boosting { 0.1 } else { 0.0 };
        let fwd = vec3(ang.sin(), 0.0, ang.cos());
        let target = wpos(self.me.s, self.me.u) + fwd * 12.0 + vec3(0.0, 1.2, 0.0);
        set_camera(&Camera3D { position: self.cam + sh, target: target + sh, up: vec3(-self.me.roll * 0.4, 1.0, 0.0), fovy: fov, ..Default::default() });
        mat.set_uniform("FogColor", vec4(hor.x, hor.y, hor.z, 1.0));
        mat.set_uniform("FogRange", vec4(80.0, CH * (VIEW as f32 - 1.5), 0.0, 0.0));
        gl_use_material(mat);
        for ch in &self.chunks { draw_mesh(&ch.mesh); }
        let mut m = new_mesh();
        for ch in &self.chunks {
            for can in &ch.cans {
                if can.2 { continue; }
                let p = wpos(can.0, can.1) + vec3(0.0, 0.9 + (self.t * 3.0 + can.0).sin() * 0.2, 0.0);
                bx(&mut m, p, vec3(0.35, 0.55, 0.35), Quat::from_rotation_y(self.t * 2.0), [60, 170, 255]);
                bx(&mut m, p + vec3(0.0, 0.65, 0.0), vec3(0.15, 0.1, 0.15), Quat::IDENTITY, [230, 230, 240]);
            }
        }
        for c in &self.ai { if (c.s - self.me.s).abs() < CH * 4.0 { draw_car(&mut m, c, self.t); } }
        draw_car(&mut m, &self.me, self.t);
        for p in &self.parts { let k = p.life.min(0.6); bx(&mut m, p.p, Vec3::splat(0.12 + (0.6 - k) * 0.4), Quat::IDENTITY, p.col); }
        draw_mesh(&m);
        gl_use_default_material();
        set_camera(&Camera2D { target: vec2(vw(), VH) / 2.0, zoom: vec2(2.0 / vw(), 2.0 / VH), ..Default::default() });
    }

    fn hud(&self) {
        let w = vw();
        let kmh = (self.me.v * 3.6) as i32;
        txt(&format!("{}", kmh), w - 200.0, VH - 140.0, 64.0, WHITE);
        txt("km/soat", w - 200.0, VH - 112.0, 20.0, Color::new(1.0, 1.0, 1.0, 0.8));
        let pl = format!("{}/6", self.place);
        txt(&pl, 24.0, 64.0, 60.0, if self.place == 1 { Color::new(1.0, 0.85, 0.3, 1.0) } else { WHITE });
        txt("O'RIN", 24.0, 88.0, 20.0, Color::new(1.0, 1.0, 1.0, 0.8));
        let left = ((RACE - self.me.s).max(0.0) / 1000.0 * 1.0).max(0.0);
        let tm = format!("{:.1} km qoldi   {:02}:{:04.1}", left, (self.race_t / 60.0) as u32, self.race_t % 60.0);
        txt(&tm, w / 2.0 - tw(&tm, 26.0) / 2.0, 40.0, 26.0, WHITE);
        // progress bar
        let bw = 360.0_f32.min(w - 300.0); let bx0 = w / 2.0 - bw / 2.0;
        draw_rectangle(bx0, 52.0, bw, 6.0, Color::new(0.0, 0.0, 0.0, 0.35));
        for c in &self.ai { draw_circle(bx0 + bw * (c.s / RACE).clamp(0.0, 1.0), 55.0, 5.0, Color::from_rgba(c.col[0], c.col[1], c.col[2], 255)); }
        draw_circle(bx0 + bw * (self.me.s / RACE).clamp(0.0, 1.0), 55.0, 7.0, Color::new(0.9, 0.2, 0.15, 1.0));
        // nitro
        draw_rectangle(w - 200.0, VH - 100.0, 170.0, 12.0, Color::new(0.0, 0.0, 0.0, 0.4));
        draw_rectangle(w - 200.0, VH - 100.0, 170.0 * self.nitro, 12.0, if self.boosting { Color::new(1.0, 0.85, 0.3, 1.0) } else { Color::new(0.35, 0.75, 1.0, 1.0) });
        txt("NITRO", w - 200.0, VH - 68.0, 18.0, WHITE);
        if self.touch {
            for (i, (p, r, l)) in buttons().iter().enumerate() {
                let on = match i { 2 => self.boosting, _ => false };
                draw_circle(p.x, p.y, *r, Color::new(0.0, 0.0, 0.0, if on { 0.5 } else { 0.28 }));
                draw_circle_lines(p.x, p.y, *r, 3.0, Color::new(1.0, 1.0, 1.0, 0.6));
                txt(l, p.x - tw(l, 22.0) / 2.0, p.y + 8.0, 22.0, WHITE);
            }
        }
    }
}

fn buttons() -> [(Vec2, f32, &'static str); 4] {
    let w = vw();
    [(vec2(100.0, VH - 100.0), 62.0, "<"), (vec2(250.0, VH - 100.0), 62.0, ">"), (vec2(w - 330.0, VH - 230.0), 50.0, "NITRO"), (vec2(w - 330.0, VH - 100.0), 50.0, "TORMOZ")]
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
fn saw(f: f32, t: f32) -> f32 { 2.0 * (f * t).fract() - 1.0 }
async fn snd(dur: f32, f: impl FnMut(f32) -> f32) -> Sound { load_sound_from_bytes(&wav(dur, f)).await.unwrap() }
async fn load_audio() -> (Sfx, Sound, Sound) {
    let sfx = Sfx {
        can: snd(0.3, |t| (sine(880.0 + t * 1200.0, t) * 0.4) * (1.0 - t / 0.3)).await,
        hit: snd(0.35, |t| (noise() * 0.8 + sine(80.0, t) * 0.5) * (-t * 12.0).exp()).await,
        nitro: { let mut y = 0.0; snd(0.8, move |t| { y += (noise() - y) * 0.15; (y * 1.5 + saw(90.0 + t * 200.0, t) * 0.2) * (PI * t / 0.8).sin() }).await },
        beep: snd(0.2, |t| sine(660.0, t) * 0.5 * (1.0 - t / 0.2)).await,
        go: snd(0.5, |t| sine(1320.0, t) * 0.5 * (1.0 - t / 0.5)).await,
        finish: snd(1.4, |t| { let f = [523.25, 659.25, 783.99, 1046.5, 1318.5][((t / 0.14) as usize).min(4)]; (sine(f, t) * 0.4 + saw(f, t) * 0.08) * (1.0 - t / 1.4) }).await,
    };
    // engine loop: one period-perfect second so pitch changes do not click
    let engine = snd(1.0, |t| (saw(55.0, t) * 0.35 + saw(110.0, t) * 0.2 + sine(27.5, t) * 0.3) * 0.5).await;
    let music = snd(8.0, |t| {
        let roots = [110.0, 130.81, 146.83, 98.0];
        let f = roots[((t / 2.0) as usize).min(3)];
        let step = (t * 8.0) as usize;
        let env = (-(t % 0.125) * 16.0).exp();
        let arp = [1.0, 2.0, 1.5, 2.0][step % 4] * f * 2.0;
        let kick = if step % 4 == 0 { sine(55.0 - (t % 0.125) * 150.0, t) * (-(t % 0.125) * 25.0).exp() * 0.35 } else { 0.0 };
        let hat = if step % 2 == 1 { noise() * (-(t % 0.125) * 60.0).exp() * 0.08 } else { 0.0 };
        (saw(arp, t) * env * 0.05 + sine(f / 2.0, t) * 0.12 + kick + hat) * (t * 20.0).min(1.0) * ((8.0 - t) * 20.0).min(1.0)
    }).await;
    (sfx, engine, music)
}

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
void main() { gl_FragColor = vec4(mix(color.rgb, FogColor.rgb, fogf * fogf), 1.0); }"#;

fn conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: Conf { window_title: "PAMIR POYGASI".into(), window_width: 1280, window_height: 720, high_dpi: true, sample_count: 1, ..Default::default() },
        draw_call_vertex_capacity: 60000, draw_call_index_capacity: 60000, ..Default::default()
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
    let (sfx, engine, music) = load_audio().await;
    play_sound(&music, PlaySoundParams { looped: true, volume: 0.3 });
    play_sound(&engine, PlaySoundParams { looped: true, volume: 0.0 });
    let mut g = Game::new(0.0);
    let mut frames = 0u32;
    loop {
        frames += 1;
        if frames == 1 || frames == 60 { macroquad::logging::info!("ZQ frame {}", frames); }
        let dt = get_frame_time().min(1.0 / 30.0);
        clear_background(BLACK);
        g.update(dt, &sfx);
        set_sound_volume(&engine, if g.state == State::Title { 0.0 } else { 0.08 + g.me.v / TOP * 0.18 });
        g.draw(&mat);
        match g.state {
            State::Title => {
                let p = 0.7 + 0.3 * (g.t * 3.0).sin();
                centered("PAMIR POYGASI", VH * 0.28, 92.0, WHITE);
                centered("Tog' serpantinida 5 raqibga qarshi 4 km poyga", VH * 0.28 + 44.0, 24.0, Color::new(1.0, 1.0, 1.0, 0.9));
                let help = if g.touch { "< >: rul   NITRO   TORMOZ" } else { "Strelkalar / A D: rul   Space: nitro   S: tormoz" };
                centered(help, VH * 0.72, 22.0, WHITE);
                centered("[ Boshlash uchun bosing ]", VH * 0.82, 32.0, Color::new(1.0, 1.0, 1.0, p));
                if g.best > 0.0 { centered(&format!("Eng yaxshi g'alaba: {:.1} s", g.best), VH * 0.92, 22.0, Color::new(1.0, 0.9, 0.6, 1.0)); }
                if tapped() || is_key_pressed(KeyCode::Enter) { let tm = g.touch || !touches().is_empty(); let b = g.best; g = Game::new(b); g.touch = tm; g.state = State::Count; }
            }
            State::Count => {
                g.hud();
                let n = g.count.ceil() as i32;
                let s = if n > 0 { format!("{}", n) } else { "OLGA!".to_string() };
                centered(&s, VH * 0.45, 140.0, Color::new(1.0, 0.85, 0.3, 1.0));
            }
            State::Race => { g.hud(); if g.race_t < 1.0 { centered("OLGA!", VH * 0.45, 140.0, Color::new(1.0, 0.85, 0.3, 1.0 - g.race_t)); } }
            State::Done => {
                draw_rectangle(0.0, 0.0, vw(), VH, Color::new(0.0, 0.0, 0.0, 0.45));
                let pl = g.ai.iter().filter(|c| c.finish.map_or(false, |f| f < g.me.finish.unwrap_or(f32::MAX))).count() + 1;
                let title = match pl { 1 => "G'ALABA! 1-O'RIN", 2 => "2-O'RIN", 3 => "3-O'RIN", _ => "MARRA" };
                centered(title, VH * 0.34, 84.0, if pl == 1 { Color::new(1.0, 0.85, 0.3, 1.0) } else { WHITE });
                centered(&format!("Vaqt: {:.1} s   O'rin: {}/6", g.me.finish.unwrap_or(0.0), pl), VH * 0.46, 34.0, WHITE);
                if g.best > 0.0 { centered(&format!("Eng yaxshi g'alaba: {:.1} s", g.best), VH * 0.53, 24.0, Color::new(1.0, 0.9, 0.6, 1.0)); }
                centered("[ Qaytadan - bosing ]", VH * 0.68, 30.0, Color::new(1.0, 0.85, 0.5, 1.0));
                if (tapped() || is_key_pressed(KeyCode::Enter)) && g.t > 0.5 { let (b, tm) = (g.best, g.touch); g = Game::new(b); g.touch = tm; g.state = State::Count; }
            }
        }
        next_frame().await;
    }
}
