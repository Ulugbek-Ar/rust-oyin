// NEON HOKKEY — 2 players on one screen (or vs bot). Rust + macroquad
use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

const VH: f32 = 720.0;
const RW: f32 = 1160.0; // rink size in world units
const RH: f32 = 600.0;
const GOAL: f32 = 210.0;
const PR: f32 = 22.0; // puck radius
const MR: f32 = 42.0; // mallet radius
const WIN: u32 = 7;

fn sc() -> f32 { screen_height() / VH }
fn vw() -> f32 { screen_width() / sc() }
fn k() -> f32 { (screen_width() / (RW + 90.0)).min(screen_height() / (RH + 90.0)) }
fn to_world(p: Vec2) -> Vec2 { (p - vec2(screen_width(), screen_height()) / 2.0) / k() + vec2(RW, RH) / 2.0 }
fn blue() -> Color { Color::new(0.25, 0.85, 1.0, 1.0) }
fn pink() -> Color { Color::new(1.0, 0.3, 0.7, 1.0) }

#[derive(PartialEq, Clone, Copy)]
enum State { Menu, Play, Win }
struct Part { p: Vec2, v: Vec2, life: f32, max: f32, col: Color }
struct Sfx { hit: Sound, wall: Sound, goal: Sound, win: Sound }
fn play(s: &Sound, v: f32) { play_sound(s, PlaySoundParams { looped: false, volume: v.clamp(0.0, 1.0) }); }

struct Mallet { p: Vec2, v: Vec2, target: Vec2 }
struct Game {
    state: State, bot: bool, m: [Mallet; 2], puck: Vec2, pv: Vec2, score: [u32; 2], pause: f32, last_goal: usize,
    parts: Vec<Part>, trail: Vec<Vec2>, shake: f32, t: f32, flash: f32, mouse_on: bool, last_mouse: Vec2, wall_cd: f32,
}

impl Game {
    fn new() -> Game {
        Game { state: State::Menu, bot: true, m: [Mallet { p: vec2(150.0, RH / 2.0), v: Vec2::ZERO, target: vec2(150.0, RH / 2.0) },
            Mallet { p: vec2(RW - 150.0, RH / 2.0), v: Vec2::ZERO, target: vec2(RW - 150.0, RH / 2.0) }],
            puck: vec2(RW / 2.0, RH / 2.0), pv: vec2(160.0, 90.0), score: [0, 0], pause: 0.0, last_goal: 0, parts: vec![], trail: vec![],
            shake: 0.0, t: 0.0, flash: 0.0, mouse_on: false, last_mouse: Vec2::ZERO, wall_cd: 0.0 }
    }
    fn start(&mut self, bot: bool) {
        let keep_mouse = self.mouse_on;
        *self = Game::new();
        self.bot = bot; self.state = State::Play; self.mouse_on = keep_mouse;
        self.reset_puck(if rand::gen_range(0, 2) == 0 { 0 } else { 1 });
    }
    fn reset_puck(&mut self, toward: usize) {
        self.puck = vec2(if toward == 0 { RW * 0.35 } else { RW * 0.65 }, RH / 2.0);
        self.pv = Vec2::ZERO; self.pause = 1.0; self.trail.clear();
    }
    fn burst(&mut self, p: Vec2, col: Color, n: usize, spd: f32) {
        for _ in 0..n {
            let life = rand::gen_range(0.3, 0.9);
            self.parts.push(Part { p, v: Vec2::from_angle(rand::gen_range(0.0, TAU)) * rand::gen_range(0.2, 1.0) * spd, life, max: life, col });
        }
    }

    fn input(&mut self, dt: f32) {
        // keyboard: P1 WASD, P2 arrows
        let keys = [[KeyCode::W, KeyCode::S, KeyCode::A, KeyCode::D], [KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right]];
        for (i, ks) in keys.iter().enumerate() {
            if i == 1 && self.bot { continue; }
            let mut d = Vec2::ZERO;
            if is_key_down(ks[0]) { d.y -= 1.0 } if is_key_down(ks[1]) { d.y += 1.0 }
            if is_key_down(ks[2]) { d.x -= 1.0 } if is_key_down(ks[3]) { d.x += 1.0 }
            if d != Vec2::ZERO { self.m[i].target = self.m[i].p + d.normalize() * 1100.0 * dt; self.mouse_on = self.mouse_on && i != 0; }
        }
        // mouse drives player 1
        let mp = Vec2::from(mouse_position());
        if mp.distance(self.last_mouse) > 1.0 { self.mouse_on = true; self.last_mouse = mp; }
        if self.mouse_on && touches().is_empty() { self.m[0].target = to_world(mp); }
        // touches: each half moves its own mallet
        for t in touches() {
            if t.phase == TouchPhase::Ended || t.phase == TouchPhase::Cancelled { continue; }
            let w = to_world(t.position);
            let side = if w.x < RW / 2.0 { 0 } else { 1 };
            if side == 1 && self.bot { continue; }
            self.m[side].target = w;
            self.mouse_on = false;
        }
        if self.bot { self.ai(); }
    }

    fn ai(&mut self) {
        let lvl = (self.score[0] as f32 - self.score[1] as f32).max(0.0);
        let (p, v) = (self.puck, self.pv);
        let t = if p.x > RW / 2.0 - 40.0 && (v.x > -250.0 || p.x > RW * 0.7) {
            // get behind the puck and push toward the left goal
            let aim = (vec2(0.0, RH / 2.0) - p).normalize_or(vec2(-1.0, 0.0));
            if self.m[1].p.x < p.x + 10.0 { p + vec2(MR + PR + 25.0, (self.m[1].p.y - p.y).signum() * 50.0) } else { p - aim * (PR + MR * 0.5) }
        } else {
            vec2(RW - 120.0, (p.y + v.y * 0.25).clamp(RH / 2.0 - GOAL / 2.0, RH / 2.0 + GOAL / 2.0))
        };
        let m = &mut self.m[1];
        let step = (650.0 + lvl * 90.0) * get_frame_time();
        m.target = m.p + (t - m.p).clamp_length_max(step * 1.4);
    }

    fn update(&mut self, dt: f32, sfx: &Sfx) {
        self.t += dt;
        self.shake = (self.shake - dt * 30.0).max(0.0);
        self.flash = (self.flash - dt * 1.5).max(0.0);
        self.wall_cd -= dt;
        for p in &mut self.parts { p.p += p.v * dt; p.v *= 1.0 - dt * 3.0; p.life -= dt; }
        self.parts.retain(|p| p.life > 0.0);
        if self.state != State::Play { return; }
        self.input(dt);

        // mallets follow their target, limited speed, stay in own half
        for (i, m) in self.m.iter_mut().enumerate() {
            let (x0, x1) = if i == 0 { (MR, RW / 2.0 - MR) } else { (RW / 2.0 + MR, RW - MR) };
            let tg = vec2(m.target.x.clamp(x0, x1), m.target.y.clamp(MR, RH - MR));
            let np = m.p + (tg - m.p).clamp_length_max(2600.0 * dt);
            m.v = (np - m.p) / dt.max(0.001);
            m.p = np;
        }

        if self.pause > 0.0 { self.pause -= dt; return; }

        // puck physics (sub-steps for fast hits)
        let steps = 4;
        let h = dt / steps as f32;
        for _ in 0..steps {
            self.puck += self.pv * h;
            for i in 0..2 {
                let d = self.puck - self.m[i].p;
                let dist = d.length();
                if dist < PR + MR && dist > 0.01 {
                    let n = d / dist;
                    self.puck = self.m[i].p + n * (PR + MR + 0.5);
                    let rel = self.pv - self.m[i].v;
                    let vn = rel.dot(n);
                    if vn < 0.0 {
                        self.pv -= n * vn * 1.9;
                        self.pv += self.m[i].v * 0.15;
                        let pow = (-vn / 1600.0).clamp(0.1, 1.0);
                        play(&sfx.hit, 0.3 + pow * 0.7);
                        let col = if i == 0 { blue() } else { pink() };
                        let pp = self.puck - n * PR;
                        self.burst(pp, col, (6.0 + pow * 16.0) as usize, 200.0 + pow * 300.0);
                        self.shake = self.shake.max(pow * 6.0);
                    }
                }
            }
            if self.pv.length() > 2000.0 { self.pv = self.pv.normalize() * 2000.0; }
            // walls
            let in_mouth = (self.puck.y - RH / 2.0).abs() < GOAL / 2.0 - PR * 0.3;
            let mut bounced = false;
            if self.puck.y < PR { self.puck.y = PR; self.pv.y = self.pv.y.abs() * 0.9; bounced = true; }
            if self.puck.y > RH - PR { self.puck.y = RH - PR; self.pv.y = -self.pv.y.abs() * 0.9; bounced = true; }
            if !in_mouth {
                if self.puck.x < PR { self.puck.x = PR; self.pv.x = self.pv.x.abs() * 0.9; bounced = true; }
                if self.puck.x > RW - PR { self.puck.x = RW - PR; self.pv.x = -self.pv.x.abs() * 0.9; bounced = true; }
            }
            if bounced && self.wall_cd <= 0.0 && self.pv.length() > 150.0 { play(&sfx.wall, (self.pv.length() / 1500.0).min(0.6)); self.wall_cd = 0.06; }
        }
        self.pv *= 1.0 - dt * 0.25;
        self.trail.push(self.puck);
        if self.trail.len() > 14 { self.trail.remove(0); }

        // goals
        let scorer = if self.puck.x < -PR { Some(1) } else if self.puck.x > RW + PR { Some(0) } else { None };
        if let Some(s) = scorer {
            self.score[s] += 1; self.last_goal = s; self.flash = 1.0; self.shake = 18.0;
            let col = if s == 0 { blue() } else { pink() };
            let gp = vec2(if s == 0 { RW } else { 0.0 }, RH / 2.0);
            self.burst(gp, col, 90, 700.0); self.burst(gp, WHITE, 30, 400.0);
            if self.score[s] >= WIN { self.state = State::Win; play(&sfx.win, 0.8); } else { play(&sfx.goal, 0.8); }
            self.reset_puck(1 - s);
        }
    }

    fn draw(&self) {
        let kk = k();
        let sh = vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)) * self.shake;
        set_camera(&Camera2D { target: vec2(RW, RH) / 2.0 + sh, zoom: vec2(2.0 * kk / screen_width(), 2.0 * kk / screen_height()), ..Default::default() });
        let pulse = 0.5 + 0.5 * (self.t * 2.0).sin();
        // rink
        draw_rectangle(-30.0, -30.0, RW + 60.0, RH + 60.0, Color::new(0.03, 0.03, 0.08, 1.0));
        draw_rectangle(0.0, 0.0, RW, RH, Color::new(0.05, 0.06, 0.13, 1.0));
        for i in 0..24 { let x = i as f32 * RW / 24.0; draw_line(x, 0.0, x, RH, 1.0, Color::new(0.2, 0.25, 0.5, 0.12)); }
        let glow_line = |a: Vec2, b: Vec2, c: Color, w: f32| {
            for (t, al) in [(w * 5.0, 0.06), (w * 2.5, 0.15), (w, 1.0)] { draw_line(a.x, a.y, b.x, b.y, t, Color { a: al * c.a, ..c }); }
        };
        let glow_circle = |p: Vec2, r: f32, c: Color, w: f32| {
            for (t, al) in [(w * 5.0, 0.06), (w * 2.5, 0.15), (w, 1.0)] { draw_circle_lines(p.x, p.y, r, t, Color { a: al * c.a, ..c }); }
        };
        let dim = |c: Color, a: f32| Color { a, ..c };
        let gy0 = RH / 2.0 - GOAL / 2.0; let gy1 = RH / 2.0 + GOAL / 2.0;
        // walls (blue left half, pink right half), goal mouths left open
        glow_line(vec2(0.0, 0.0), vec2(RW / 2.0, 0.0), blue(), 4.0);
        glow_line(vec2(0.0, RH), vec2(RW / 2.0, RH), blue(), 4.0);
        glow_line(vec2(RW / 2.0, 0.0), vec2(RW, 0.0), pink(), 4.0);
        glow_line(vec2(RW / 2.0, RH), vec2(RW, RH), pink(), 4.0);
        glow_line(vec2(0.0, 0.0), vec2(0.0, gy0), blue(), 4.0); glow_line(vec2(0.0, gy1), vec2(0.0, RH), blue(), 4.0);
        glow_line(vec2(RW, 0.0), vec2(RW, gy0), pink(), 4.0); glow_line(vec2(RW, gy1), vec2(RW, RH), pink(), 4.0);
        // goals
        draw_rectangle(-26.0, gy0, 26.0, GOAL, dim(blue(), 0.15 + 0.1 * pulse));
        draw_rectangle(RW, gy0, 26.0, GOAL, dim(pink(), 0.15 + 0.1 * pulse));
        // centre markings
        let c = vec2(RW, RH) / 2.0;
        glow_line(vec2(RW / 2.0, 0.0), vec2(RW / 2.0, RH), Color::new(0.8, 0.8, 1.0, 0.5), 2.0);
        glow_circle(c, 90.0, Color::new(0.8, 0.8, 1.0, 0.5), 2.0);
        glow_circle(vec2(0.0, RH / 2.0), 130.0, dim(blue(), 0.5), 2.0);
        glow_circle(vec2(RW, RH / 2.0), 130.0, dim(pink(), 0.5), 2.0);
        // big faded scores on the ice
        let s0 = format!("{}", self.score[0]); let s1 = format!("{}", self.score[1]);
        wtxt(&s0, RW * 0.25, RH / 2.0 + 60.0, 180.0, dim(blue(), 0.12), 0.0);
        wtxt(&s1, RW * 0.75, RH / 2.0 + 60.0, 180.0, dim(pink(), 0.12), 0.0);

        // puck trail + puck
        for (i, p) in self.trail.iter().enumerate() {
            let a = i as f32 / self.trail.len().max(1) as f32;
            draw_circle(p.x, p.y, PR * a, Color::new(1.0, 0.95, 0.6, a * 0.25));
        }
        for i in (1..5).rev() { draw_circle(self.puck.x, self.puck.y, PR * (1.0 + i as f32 * 0.35), Color::new(1.0, 0.9, 0.5, 0.05)); }
        draw_circle(self.puck.x, self.puck.y, PR, Color::new(1.0, 0.95, 0.75, 1.0));
        draw_circle(self.puck.x, self.puck.y, PR * 0.55, Color::new(1.0, 0.8, 0.3, 1.0));
        // mallets
        for (i, m) in self.m.iter().enumerate() {
            let col = if i == 0 { blue() } else { pink() };
            for j in (1..5).rev() { draw_circle(m.p.x, m.p.y, MR * (1.0 + j as f32 * 0.3), dim(col, 0.06)); }
            draw_circle(m.p.x, m.p.y, MR, Color::new(0.05, 0.05, 0.1, 1.0));
            draw_circle_lines(m.p.x, m.p.y, MR - 3.0, 6.0, col);
            draw_circle(m.p.x, m.p.y, MR * 0.4, col);
            draw_circle(m.p.x - 5.0, m.p.y - 5.0, MR * 0.15, Color::new(1.0, 1.0, 1.0, 0.6));
        }
        for p in &self.parts { let a = p.life / p.max; draw_rectangle(p.p.x - 3.0, p.p.y - 3.0, 6.0 * a, 6.0 * a, Color { a, ..p.col }); }
        if self.pause > 0.0 && self.state == State::Play {
            let n = format!("{}", (self.pause * 3.0).ceil() as i32);
            wtxt(&n, self.puck.x, self.puck.y - 40.0, 48.0, Color::new(1.0, 1.0, 1.0, 0.8), 0.0);
        }
        if self.flash > 0.0 && self.state == State::Play {
            let col = if self.last_goal == 0 { blue() } else { pink() };
            let a = self.flash.min(1.0);
            // each player reads the text from their own side
            wtxt("GOL!", RW * 0.25, RH / 2.0 - 120.0, 110.0 * (1.0 + (1.0 - a) * 0.3), dim(col, a), PI / 2.0);
            wtxt("GOL!", RW * 0.75, RH / 2.0 - 120.0, 110.0 * (1.0 + (1.0 - a) * 0.3), dim(col, a), -PI / 2.0);
        }
    }
}

// text in world space (rink camera), centred, optional rotation
fn wtxt(t: &str, x: f32, y: f32, size: f32, color: Color, rot: f32) {
    let kk = k();
    let fs = (size * kk).max(8.0) as u16;
    let dims = measure_text(t, None, fs, 1.0 / kk);
    if rot == 0.0 {
        draw_text_ex(t, x - dims.width / 2.0, y, TextParams { font_size: fs, font_scale: 1.0 / kk, color, ..Default::default() });
    } else {
        // rotate around (x, y): shift start point along the rotated baseline
        let dir = Vec2::from_angle(rot);
        let start = vec2(x, y) - dir * dims.width / 2.0;
        draw_text_ex(t, start.x, start.y, TextParams { font_size: fs, font_scale: 1.0 / kk, color, rotation: rot, ..Default::default() });
    }
}
fn ui_camera() { set_camera(&Camera2D { target: vec2(vw(), VH) / 2.0, zoom: vec2(2.0 / vw(), 2.0 / VH), ..Default::default() }); }
fn txt(t: &str, x: f32, y: f32, size: f32, color: Color) {
    let s = sc();
    draw_text_ex(t, x, y, TextParams { font_size: (size * s) as u16, font_scale: 1.0 / s, color, ..Default::default() });
}
fn tw(t: &str, size: f32) -> f32 { let s = sc(); measure_text(t, None, (size * s) as u16, 1.0 / s).width }
fn centered(t: &str, y: f32, size: f32, col: Color) { txt(t, vw() / 2.0 - tw(t, size) / 2.0, y, size, col); }
fn tap() -> Option<Vec2> {
    if is_mouse_button_pressed(MouseButton::Left) { return Some(Vec2::from(mouse_position()) / sc()); }
    touches().iter().find(|t| t.phase == TouchPhase::Started).map(|t| t.position / sc())
}
fn button(r: Rect, label: &str, sub: &str, col: Color, t: Option<Vec2>) -> bool {
    let hover = r.contains(Vec2::from(mouse_position()) / sc());
    for i in (1..4).rev() { draw_rectangle_lines(r.x - i as f32 * 3.0, r.y - i as f32 * 3.0, r.w + i as f32 * 6.0, r.h + i as f32 * 6.0, 2.0, Color { a: 0.12, ..col }); }
    draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.04, 0.04, 0.1, 0.9));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, if hover { 5.0 } else { 3.0 }, col);
    txt(label, r.x + r.w / 2.0 - tw(label, 38.0) / 2.0, r.y + 58.0, 38.0, WHITE);
    txt(sub, r.x + r.w / 2.0 - tw(sub, 20.0) / 2.0, r.y + 92.0, 20.0, Color { a: 0.8, ..col });
    t.map_or(false, |p| r.contains(p))
}

// ---------------- audio ----------------
const SR: f32 = 22050.0;
fn wav(dur: f32, f: impl Fn(f32) -> f32) -> Vec<u8> {
    let n = (dur * SR) as u32;
    let mut b = Vec::with_capacity(44 + n as usize * 2);
    let u32le = |b: &mut Vec<u8>, v: u32| b.extend_from_slice(&v.to_le_bytes());
    b.extend_from_slice(b"RIFF"); u32le(&mut b, 36 + n * 2); b.extend_from_slice(b"WAVEfmt ");
    u32le(&mut b, 16); b.extend_from_slice(&[1, 0, 1, 0]); u32le(&mut b, SR as u32); u32le(&mut b, SR as u32 * 2);
    b.extend_from_slice(&[2, 0, 16, 0]); b.extend_from_slice(b"data"); u32le(&mut b, n * 2);
    for i in 0..n { b.extend_from_slice(&((f(i as f32 / SR).clamp(-1.0, 1.0) * 32000.0) as i16).to_le_bytes()); }
    b
}
fn sine(f: f32, t: f32) -> f32 { (TAU * f * t).sin() }
fn sq(f: f32, t: f32) -> f32 { if sine(f, t) >= 0.0 { 1.0 } else { -1.0 } }
async fn snd(dur: f32, f: impl Fn(f32) -> f32) -> Sound { load_sound_from_bytes(&wav(dur, f)).await.unwrap() }
async fn load_audio() -> (Sfx, Sound) {
    let sfx = Sfx {
        hit: snd(0.12, |t| (sine(520.0 - t * 1500.0, t) * 0.6 + rand::gen_range(-1.0, 1.0) * 0.4 * (-t * 60.0).exp()) * (-t * 30.0).exp()).await,
        wall: snd(0.08, |t| sine(260.0, t) * (-t * 50.0).exp() * 0.6).await,
        goal: snd(0.9, |t| { let f = [392.0, 523.25, 659.25, 783.99][((t / 0.12) as usize).min(3)]; (sq(f, t) * 0.15 + sine(f, t) * 0.4) * (1.0 - t / 0.9) }).await,
        win: snd(1.6, |t| { let f = [523.25, 659.25, 783.99, 1046.5, 783.99, 1046.5][((t / 0.16) as usize).min(5)]; (sq(f, t) * 0.12 + sine(f, t) * 0.4) * (1.0 - t / 1.6) }).await,
    };
    let music = snd(8.0, |t| {
        let roots = [130.81, 110.0, 87.31, 98.0];
        let f = roots[((t / 2.0) as usize).min(3)];
        let step = (t * 8.0) as usize;
        let env = (-(t % 0.125) * 18.0).exp();
        let arp = [1.0, 1.5, 2.0, 3.0][step % 4] * f * 2.0;
        let bass = sq(f / 2.0, t) * 0.06 * (-(t % 0.25) * 6.0).exp();
        let kick = if step % 4 == 0 { sine(60.0 - (t % 0.125) * 200.0, t) * (-(t % 0.125) * 25.0).exp() * 0.3 } else { 0.0 };
        (sine(arp, t) * env * 0.08 + bass + kick) * (t * 20.0).min(1.0) * ((8.0 - t) * 20.0).min(1.0)
    }).await;
    (sfx, music)
}

fn conf() -> Conf { Conf { window_title: "NEON HOKKEY".into(), window_width: 1280, window_height: 720, high_dpi: true, sample_count: 4, ..Default::default() } }

#[macroquad::main(conf)]
async fn main() {
    std::panic::set_hook(Box::new(|info| { macroquad::logging::error!("ZQ PANIC: {}", info); }));
    simulate_mouse_with_touch(false);
    rand::srand(macroquad::miniquad::date::now() as u64);
    let (sfx, music) = load_audio().await;
    play_sound(&music, PlaySoundParams { looped: true, volume: 0.35 });
    let mut g = Game::new();
    let mut frames = 0u32;
    loop {
        frames += 1;
        if frames == 1 || frames == 60 { macroquad::logging::info!("ZQ frame {}", frames); }
        let dt = get_frame_time().min(1.0 / 30.0);
        clear_background(Color::new(0.02, 0.02, 0.05, 1.0));
        if g.state == State::Menu {
            // attract mode: puck drifts around
            g.t += dt;
            g.puck += g.pv * dt;
            if g.puck.x < PR || g.puck.x > RW - PR { g.pv.x = -g.pv.x; }
            if g.puck.y < PR || g.puck.y > RH - PR { g.pv.y = -g.pv.y; }
        } else { g.update(dt, &sfx); }
        g.draw();
        ui_camera();
        let t = tap();
        match g.state {
            State::Menu => {
                draw_rectangle(0.0, 0.0, vw(), VH, Color::new(0.0, 0.0, 0.05, 0.6));
                let p = 0.8 + 0.2 * (g.t * 3.0).sin();
                centered("NEON HOKKEY", VH * 0.27, 100.0, Color::new(1.0, 1.0, 1.0, p));
                centered("Bitta ekranda ikki kishi yoki botga qarshi", VH * 0.27 + 44.0, 24.0, Color::new(0.8, 0.85, 1.0, 0.9));
                let cx = vw() / 2.0;
                if button(Rect::new(cx - 330.0, VH * 0.45, 300.0, 120.0), "1 O'YINCHI", "botga qarshi", blue(), t) || is_key_pressed(KeyCode::Key1) { g.start(true); }
                if button(Rect::new(cx + 30.0, VH * 0.45, 300.0, 120.0), "2 O'YINCHI", "bitta ekranda", pink(), t) || is_key_pressed(KeyCode::Key2) { g.start(false); }
                centered("Telefon: o'z yarmingizda barmoq bilan suring   Kompyuter: WASD / strelkalar / sichqoncha", VH * 0.82, 20.0, Color::new(1.0, 1.0, 1.0, 0.7));
                centered(&format!("{} golgacha o'ynaladi", WIN), VH * 0.87, 20.0, Color::new(1.0, 1.0, 1.0, 0.5));
            }
            State::Play => {
                if is_key_pressed(KeyCode::Escape) { g.state = State::Menu; }
            }
            State::Win => {
                draw_rectangle(0.0, 0.0, vw(), VH, Color::new(0.0, 0.0, 0.05, 0.55));
                let w = if g.score[0] >= WIN { 0 } else { 1 };
                let col = if w == 0 { blue() } else { pink() };
                let name = if w == 0 { "KO'K YUTDI!" } else if g.bot { "BOT YUTDI!" } else { "PUSHTI YUTDI!" };
                centered(name, VH * 0.32, 90.0, col);
                centered(&format!("{} : {}", g.score[0], g.score[1]), VH * 0.45, 60.0, WHITE);
                let cx = vw() / 2.0;
                let bot = g.bot;
                if button(Rect::new(cx - 330.0, VH * 0.55, 300.0, 120.0), "QAYTA", "yana bir o'yin", col, t) || is_key_pressed(KeyCode::Enter) { g.start(bot); }
                if button(Rect::new(cx + 30.0, VH * 0.55, 300.0, 120.0), "MENYU", "rejimni tanlash", Color::new(0.8, 0.8, 1.0, 1.0), t) { let m = g.mouse_on; g = Game::new(); g.mouse_on = m; }
            }
        }
        next_frame().await;
    }
}
