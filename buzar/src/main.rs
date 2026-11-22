// QAL'A BUZAR — catapult physics puzzle (Rust + macroquad + rapier2d)
use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;
use rapier2d::prelude::*;
use std::f32::consts::{PI, TAU};

const VH: f32 = 720.0;
const WW: f32 = 32.0; // visible world width in metres
const ORIGIN: (f32, f32) = (3.4, 3.1);
const MAXPULL: f32 = 2.6;
const POWER: f32 = 8.8;

fn sc() -> f32 { screen_height() / VH }
fn vw() -> f32 { screen_width() / sc() }
fn rgb(r: u8, g: u8, b: u8) -> Color { Color::from_rgba(r, g, b, 255) }
fn ppm() -> f32 { screen_width() / WW }
fn wh() -> f32 { screen_height() / ppm() }
fn to_world(p: Vec2) -> Vec2 { let k = ppm(); vec2(p.x / k, wh() - p.y / k - 0.0) + vec2(0.0, -1.2) }
fn world_cam() -> Camera2D {
    let h = wh();
    Camera2D { target: vec2(WW / 2.0, h / 2.0 - 1.2), zoom: vec2(2.0 / WW, -2.0 / h), ..Default::default() }
}

#[derive(Clone, Copy, PartialEq)]
enum K { Wood, Stone, Glass, Dev, Ball }
struct Obj { kind: K, hp: f32, w: f32, h: f32, body: RigidBodyHandle, alive: bool, hit: f32 }
struct Part { p: Vec2, v: Vec2, life: f32, max: f32, col: Color, s: f32, rot: f32 }
struct FText { p: Vec2, s: String, life: f32, col: Color }

struct Phys {
    bodies: RigidBodySet, colliders: ColliderSet, pipeline: PhysicsPipeline, islands: IslandManager, broad: DefaultBroadPhase,
    narrow: NarrowPhase, joints: ImpulseJointSet, multi: MultibodyJointSet, ccd: CCDSolver, params: IntegrationParameters,
}
impl Phys {
    fn new() -> Phys {
        Phys { bodies: RigidBodySet::new(), colliders: ColliderSet::new(), pipeline: PhysicsPipeline::new(), islands: IslandManager::new(),
            broad: DefaultBroadPhase::new(), narrow: NarrowPhase::new(), joints: ImpulseJointSet::new(), multi: MultibodyJointSet::new(),
            ccd: CCDSolver::new(), params: IntegrationParameters::default() }
    }
    fn step(&mut self, dt: f32) {
        self.params.dt = dt;
        let g = vector![0.0, -9.81];
        self.pipeline.step(&g, &self.params, &mut self.islands, &mut self.broad, &mut self.narrow, &mut self.bodies, &mut self.colliders,
            &mut self.joints, &mut self.multi, &mut self.ccd, None, &(), &());
    }
    fn remove(&mut self, h: RigidBodyHandle) {
        self.bodies.remove(h, &mut self.islands, &mut self.colliders, &mut self.joints, &mut self.multi, true);
    }
}

// ---------------- levels ----------------
type Piece = (K, f32, f32, f32, f32); // kind, centre x, centre y, w, h
struct Level { shots: u32, pieces: Vec<Piece>, devs: Vec<(f32, f32)> }
fn floor(v: &mut Vec<Piece>, x: f32, y: f32, k: K, pk: K, span: f32) -> f32 {
    v.push((k, x - span / 2.0, y + 0.8, 0.35, 1.6));
    v.push((k, x + span / 2.0, y + 0.8, 0.35, 1.6));
    v.push((pk, x, y + 1.75, span + 0.6, 0.3));
    y + 1.9
}
fn level(n: usize) -> Level {
    let mut p: Vec<Piece> = vec![];
    let mut d = vec![];
    let shots;
    match n {
        0 => { let y = floor(&mut p, 21.0, 0.0, K::Wood, K::Wood, 2.0); let y2 = floor(&mut p, 21.0, y, K::Wood, K::Wood, 2.0); d.push((21.0, 0.4)); d.push((21.0, y2 + 0.4)); shots = 3; }
        1 => {
            for i in 0..4 { p.push((K::Glass, 18.0, 0.5 + i as f32, 0.5, 1.0)); }
            let y = floor(&mut p, 23.0, 0.0, K::Wood, K::Wood, 2.4); floor(&mut p, 23.0, y, K::Glass, K::Wood, 2.4);
            d.push((23.0, 0.4)); d.push((23.0, y + 0.4)); shots = 3;
        }
        2 => {
            let a = floor(&mut p, 19.0, 0.0, K::Stone, K::Wood, 1.8); let b = floor(&mut p, 25.0, 0.0, K::Stone, K::Wood, 1.8);
            p.push((K::Wood, 22.0, a + 0.15, 7.6, 0.3));
            d.push((19.0, 0.4)); d.push((25.0, 0.4)); d.push((22.0, a.max(b) + 0.75)); shots = 4;
        }
        3 => {
            for row in 0..4 { for i in 0..(4 - row) { let x = 19.5 + i as f32 * 1.6 + row as f32 * 0.8; p.push((if row % 2 == 0 { K::Wood } else { K::Stone }, x, 0.6 + row as f32 * 1.2, 1.4, 1.2)); } }
            d.push((22.0, 5.4)); d.push((26.8, 0.4)); shots = 3;
        }
        4 => {
            let mut y = 0.0;
            for i in 0..4 { y = floor(&mut p, 22.0, y, if i < 2 { K::Stone } else { K::Wood }, if i % 2 == 0 { K::Wood } else { K::Glass }, 2.2); d.push((22.0, y - 1.5)); }
            d.truncate(3); d.push((22.0, y + 0.4)); shots = 4;
        }
        5 => {
            for i in 0..5 { p.push((K::Stone, 18.5, 0.5 + i as f32, 0.6, 1.0)); p.push((K::Stone, 26.5, 0.5 + i as f32, 0.6, 1.0)); }
            p.push((K::Wood, 22.5, 5.15, 9.0, 0.3));
            let y = floor(&mut p, 22.5, 0.0, K::Glass, K::Wood, 2.6);
            for i in 0..3 { p.push((K::Wood, 20.6 + i as f32 * 1.9, 5.8, 0.4, 1.0)); }
            d.push((22.5, 0.4)); d.push((22.5, y + 0.4)); d.push((20.0, 5.75)); shots = 4;
        }
        6 => {
            for x in [19.0, 22.0, 25.0] { p.push((K::Stone, x, 1.5, 0.6, 3.0)); }
            p.push((K::Wood, 22.0, 3.15, 7.5, 0.3));
            for i in 0..5 { p.push((K::Glass, 19.0 + i as f32 * 1.5, 3.8, 0.4, 1.0)); }
            p.push((K::Glass, 22.0, 4.45, 7.0, 0.3));
            d.push((20.5, 0.4)); d.push((23.5, 0.4)); d.push((20.2, 3.75)); d.push((23.8, 3.75)); shots = 5;
        }
        _ => {
            let a = { let mut y = 0.0; for i in 0..3 { y = floor(&mut p, 18.5, y, K::Stone, if i == 2 { K::Stone } else { K::Wood }, 1.8); } y };
            let _ = { let mut y = 0.0; for i in 0..3 { y = floor(&mut p, 27.0, y, K::Stone, if i == 2 { K::Stone } else { K::Wood }, 1.8); } y };
            p.push((K::Wood, 22.75, 3.95, 7.0, 0.3));
            let m = floor(&mut p, 22.75, 0.0, K::Glass, K::Wood, 2.4);
            d.push((18.5, 0.4)); d.push((27.0, 0.4)); d.push((22.75, 0.4)); d.push((22.75, m + 0.4)); d.push((18.5, a + 0.4)); d.push((27.0, a + 0.4)); shots = 5;
        }
    }
    Level { shots, pieces: p, devs: d }
}
const LEVELS: usize = 8;

// ---------------- game ----------------
#[derive(Clone, Copy, PartialEq)]
enum State { Menu, Play, Won, Lost }
struct Sfx { launch: Sound, wood: Sound, stone: Sound, glass: Sound, pop: Sound, win: Sound, lose: Sound, stretch: Sound }
fn play(s: &Sound, v: f32) { play_sound(s, PlaySoundParams { looped: false, volume: v.clamp(0.0, 1.0) }); }

struct Game {
    state: State, lvl: usize, ph: Phys, objs: Vec<Obj>, shots: u32, ball: Option<usize>, aiming: bool, pull: Vec2, shot_t: f32,
    score: u32, parts: Vec<Part>, texts: Vec<FText>, shake: f32, t: f32, grace: f32, end_t: f32, stars: [u32; LEVELS], unlocked: usize,
    snd_cd: f32, clouds: Vec<Vec2>,
}

impl Game {
    fn new() -> Game {
        let clouds = (0..6).map(|i| vec2(i as f32 * 6.0 + rand::gen_range(0.0, 3.0), rand::gen_range(9.0, 14.0))).collect();
        Game { state: State::Menu, lvl: 0, ph: Phys::new(), objs: vec![], shots: 0, ball: None, aiming: false, pull: Vec2::ZERO, shot_t: 0.0,
            score: 0, parts: vec![], texts: vec![], shake: 0.0, t: 0.0, grace: 0.0, end_t: 0.0, stars: [0; LEVELS], unlocked: 1, snd_cd: 0.0, clouds }
    }
    fn load(&mut self, n: usize) {
        self.lvl = n; self.ph = Phys::new(); self.objs.clear(); self.parts.clear(); self.texts.clear();
        self.ball = None; self.aiming = false; self.score = 0; self.grace = 1.2; self.end_t = 0.0; self.state = State::Play;
        let ground = ColliderBuilder::cuboid(200.0, 1.0).translation(vector![0.0, -1.0]).friction(0.9).build();
        self.ph.colliders.insert(ground);
        let hill = ColliderBuilder::cuboid(3.0, 1.0).translation(vector![2.0, 1.0]).build();
        self.ph.colliders.insert(hill);
        let lv = level(n);
        self.shots = lv.shots;
        for (k, x, y, w, h) in lv.pieces {
            let (dens, hp) = match k { K::Wood => (0.7, 9.0), K::Stone => (2.4, 26.0), K::Glass => (0.9, 3.0), _ => (1.0, 1.0) };
            let rb = self.ph.bodies.insert(RigidBodyBuilder::dynamic().translation(vector![x, y]).build());
            let col = ColliderBuilder::cuboid(w / 2.0, h / 2.0).density(dens).friction(0.8).restitution(0.05).build();
            self.ph.colliders.insert_with_parent(col, rb, &mut self.ph.bodies);
            self.objs.push(Obj { kind: k, hp, w, h, body: rb, alive: true, hit: 0.0 });
        }
        for (x, y) in lv.devs {
            let rb = self.ph.bodies.insert(RigidBodyBuilder::dynamic().translation(vector![x, y]).build());
            let col = ColliderBuilder::ball(0.4).density(0.8).friction(0.7).restitution(0.2).build();
            self.ph.colliders.insert_with_parent(col, rb, &mut self.ph.bodies);
            self.objs.push(Obj { kind: K::Dev, hp: 3.5, w: 0.8, h: 0.8, body: rb, alive: true, hit: 0.0 });
        }
    }
    fn devs_left(&self) -> usize { self.objs.iter().filter(|o| o.alive && o.kind == K::Dev).count() }
    fn pos(&self, o: &Obj) -> (Vec2, f32) { let b = &self.ph.bodies[o.body]; (vec2(b.translation().x, b.translation().y), b.rotation().angle()) }
    fn burst(&mut self, p: Vec2, col: Color, n: usize, spd: f32, size: f32) {
        for _ in 0..n {
            let life = rand::gen_range(0.5, 1.2);
            self.parts.push(Part { p, v: Vec2::from_angle(rand::gen_range(0.0, TAU)) * rand::gen_range(0.3, 1.0) * spd + vec2(0.0, 2.0), life, max: life, col, s: size * rand::gen_range(0.5, 1.2), rot: rand::gen_range(0.0, TAU) });
        }
    }
    fn settled(&self) -> bool {
        self.objs.iter().filter(|o| o.alive).all(|o| { let b = &self.ph.bodies[o.body]; b.is_sleeping() || b.linvel().norm() < 0.25 })
    }

    fn fire(&mut self, sfx: &Sfx) {
        let v = self.pull * POWER;
        let rb = self.ph.bodies.insert(RigidBodyBuilder::dynamic().translation(vector![ORIGIN.0, ORIGIN.1]).linvel(vector![v.x, v.y]).ccd_enabled(true).build());
        let col = ColliderBuilder::ball(0.36).density(3.2).friction(0.6).restitution(0.25).build();
        self.ph.colliders.insert_with_parent(col, rb, &mut self.ph.bodies);
        self.objs.push(Obj { kind: K::Ball, hp: 1e9, w: 0.72, h: 0.72, body: rb, alive: true, hit: 0.0 });
        self.ball = Some(self.objs.len() - 1);
        self.shots -= 1; self.shot_t = 0.0;
        play(&sfx.launch, 0.7);
    }

    fn update(&mut self, dt: f32, sfx: &Sfx) {
        self.t += dt;
        self.shake = (self.shake - dt * 20.0).max(0.0);
        self.grace -= dt; self.snd_cd -= dt;
        for c in self.clouds.iter_mut() { c.x += dt * 0.4; if c.x > WW + 4.0 { c.x = -4.0; } }
        for p in self.parts.iter_mut() { p.p += p.v * dt; p.v.y -= 12.0 * dt; p.life -= dt; p.rot += dt * 5.0; }
        self.parts.retain(|p| p.life > 0.0);
        for t in self.texts.iter_mut() { t.p.y += dt * 1.2; t.life -= dt; }
        self.texts.retain(|t| t.life > 0.0);
        if self.state != State::Play && self.state != State::Won && self.state != State::Lost { return; }

        // aiming input (mouse or touch, macroquad maps touch to mouse)
        let mp = to_world(Vec2::from(mouse_position()));
        let o = vec2(ORIGIN.0, ORIGIN.1);
        let ready = self.state == State::Play && self.shots > 0 && self.ball.is_none();
        if ready && is_mouse_button_pressed(MouseButton::Left) && mp.distance(o) < 3.2 { self.aiming = true; play(&sfx.stretch, 0.4); }
        if self.aiming {
            self.pull = (o - mp).clamp_length_max(MAXPULL);
            if !is_mouse_button_down(MouseButton::Left) {
                self.aiming = false;
                if self.pull.length() > 0.5 { self.fire(sfx); }
            }
        }

        // physics (fixed sub-steps)
        let steps = 2;
        for _ in 0..steps { self.ph.step(dt / steps as f32); }

        // damage from contact impulses
        if self.grace <= 0.0 {
            let mut dmg: Vec<(RigidBodyHandle, f32)> = vec![];
            for pair in self.ph.narrow.contact_pairs() {
                let mut imp = 0.0;
                for m in &pair.manifolds { for pt in &m.points { imp += pt.data.impulse; } }
                if imp < 0.9 { continue; }
                for ch in [pair.collider1, pair.collider2] {
                    if let Some(parent) = self.ph.colliders.get(ch).and_then(|c| c.parent()) { dmg.push((parent, imp)); }
                }
            }
            let mut broken = vec![];
            for (h, imp) in dmg {
                if let Some(o) = self.objs.iter_mut().find(|o| o.alive && o.body == h) {
                    if o.kind == K::Ball { continue; }
                    let mult = if o.kind == K::Dev { 1.0 } else { 0.8 };
                    o.hp -= (imp - 0.9) * mult; o.hit = 0.15;
                    if o.hp <= 0.0 { o.alive = false; broken.push((o.kind, o.body, o.w, o.h)); }
                    else if imp > 4.0 && self.snd_cd <= 0.0 { self.snd_cd = 0.08; play(match o.kind { K::Stone => &sfx.stone, K::Glass => &sfx.glass, _ => &sfx.wood }, (imp / 20.0).min(0.6)); }
                }
            }
            for (k, h, w, hh) in broken {
                let b = &self.ph.bodies[h];
                let p = vec2(b.translation().x, b.translation().y);
                self.ph.remove(h);
                let (col, pts, s) = match k { K::Wood => (rgb(196, 140, 80), 100, &sfx.wood), K::Stone => (rgb(150, 150, 160), 200, &sfx.stone), K::Glass => (rgb(170, 225, 255), 50, &sfx.glass), _ => (rgb(110, 200, 90), 1000, &sfx.pop) };
                let n = ((w * hh) * 14.0) as usize + 6;
                self.burst(p, col, n.min(30), 4.0, if k == K::Dev { 0.18 } else { 0.14 });
                if k == K::Dev { self.burst(p, WHITE, 10, 3.0, 0.12); self.shake = self.shake.max(6.0); }
                self.score += pts;
                self.texts.push(FText { p, s: format!("+{}", pts), life: 1.0, col: if k == K::Dev { rgb(140, 255, 120) } else { WHITE } });
                play(s, if k == K::Dev { 0.8 } else { 0.5 });
            }
        }
        // remove bodies that fell off the world
        for o in self.objs.iter_mut() { if o.alive && self.ph.bodies[o.body].translation().y < -6.0 { o.alive = false; } }

        // turn flow
        if let Some(bi) = self.ball {
            self.shot_t += dt;
            let b = &self.ph.bodies[self.objs[bi].body];
            let gone = b.translation().x > WW + 6.0 || b.translation().x < -4.0;
            if gone || (self.shot_t > 1.5 && self.settled()) || self.shot_t > 7.0 {
                let h = self.objs[bi].body;
                self.objs[bi].alive = false; self.ph.remove(h); self.ball = None;
            }
        }
        if self.state == State::Play && self.ball.is_none() {
            if self.devs_left() == 0 {
                self.end_t += dt;
                if self.end_t > 1.0 {
                    let bonus = self.shots * 2000; self.score += bonus;
                    let st = 1 + self.shots.min(2);
                    self.stars[self.lvl] = self.stars[self.lvl].max(st);
                    self.unlocked = self.unlocked.max((self.lvl + 2).min(LEVELS));
                    self.state = State::Won; self.end_t = 0.0; play(&sfx.win, 0.8);
                }
            } else if self.shots == 0 && self.settled() {
                self.end_t += dt;
                if self.end_t > 1.5 { self.state = State::Lost; self.end_t = 0.0; play(&sfx.lose, 0.7); }
            } else { self.end_t = 0.0; }
        }
        if self.state != State::Play { self.end_t += dt; }
    }

    fn draw(&self) {
        // sky
        set_default_camera();
        let (w, h) = (screen_width(), screen_height());
        for i in 0..30 { let t = i as f32 / 29.0; let c = vec3(0.42, 0.68, 0.95).lerp(vec3(0.85, 0.93, 1.0), t); draw_rectangle(0.0, t * h, w, h / 29.0 + 1.0, Color::new(c.x, c.y, c.z, 1.0)); }
        let mut cam = world_cam();
        cam.target += vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)) * self.shake * 0.02;
        set_camera(&cam);
        draw_circle(27.0, 13.0, 1.4, Color::new(1.0, 0.95, 0.7, 0.4)); draw_circle(27.0, 13.0, 1.0, rgb(255, 240, 180));
        for c in &self.clouds { for (dx, r) in [(0.0, 0.9), (0.9, 1.2), (1.9, 0.8)] { draw_circle(c.x + dx, c.y, r, Color::new(1.0, 1.0, 1.0, 0.85)); } }
        // distant hills
        for i in 0..8 { let x = i as f32 * 5.0 - 2.0; draw_circle(x, -2.0, 5.5 + (i % 3) as f32, rgb(150, 190, 140)); }
        // ground
        draw_rectangle(-5.0, -8.0, WW + 10.0, 8.0, rgb(110, 80, 50));
        draw_rectangle(-5.0, -0.35, WW + 10.0, 0.35, rgb(90, 160, 70));
        // catapult hill
        draw_rectangle(-1.0, 0.0, 6.0, 2.0, rgb(110, 80, 50)); draw_rectangle(-1.0, 1.7, 6.0, 0.3, rgb(90, 160, 70));
        // catapult
        let o = vec2(ORIGIN.0, ORIGIN.1);
        let wood = rgb(130, 85, 45); let dark = rgb(80, 50, 25);
        draw_line(2.6, 2.0, 3.0, 3.3, 0.22, dark); draw_line(4.2, 2.0, 3.8, 3.3, 0.22, dark);
        draw_rectangle(2.4, 2.0, 2.0, 0.25, wood);
        let ball_at = if self.aiming { o - self.pull } else { o };
        let band = rgb(90, 40, 30);
        draw_line(3.0, 3.3, ball_at.x, ball_at.y, 0.09, band);
        if self.state == State::Play && self.ball.is_none() && self.shots > 0 { draw_circle(ball_at.x, ball_at.y, 0.36, rgb(120, 120, 130)); draw_circle(ball_at.x - 0.1, ball_at.y + 0.1, 0.12, rgb(190, 190, 200)); }
        draw_line(3.8, 3.3, ball_at.x, ball_at.y, 0.09, band);
        // trajectory preview
        if self.aiming {
            let v = self.pull * POWER;
            for i in 1..22 {
                let t = i as f32 * 0.07;
                let p = o + v * t + vec2(0.0, -0.5 * 9.81 * t * t);
                draw_circle(p.x, p.y, 0.09 - i as f32 * 0.002, Color::new(1.0, 1.0, 1.0, 0.9 - i as f32 * 0.035));
            }
        }
        // objects
        for ob in self.objs.iter().filter(|o| o.alive) {
            let (p, a) = self.pos(ob);
            match ob.kind {
                K::Ball => { draw_circle(p.x, p.y, 0.36, rgb(110, 110, 120)); draw_circle(p.x - 0.1, p.y + 0.1, 0.12, rgb(190, 190, 200)); }
                K::Dev => {
                    let flash = ob.hit > 0.0;
                    draw_circle(p.x, p.y, 0.44, rgb(40, 70, 30));
                    draw_circle(p.x, p.y, 0.4, if flash { rgb(200, 255, 170) } else { rgb(110, 190, 80) });
                    for s in [-1.0f32, 1.0] {
                        draw_triangle(vec2(p.x + 0.2 * s, p.y + 0.3), vec2(p.x + 0.38 * s, p.y + 0.25), vec2(p.x + 0.42 * s, p.y + 0.62), rgb(240, 230, 200));
                        draw_circle(p.x + 0.15 * s, p.y + 0.08, 0.11, WHITE);
                        draw_circle(p.x + 0.15 * s - 0.04, p.y + 0.07, 0.05, BLACK);
                    }
                    let hurt = ob.hp < 2.0;
                    draw_line(p.x - 0.15, p.y - 0.17, p.x + 0.15, p.y - if hurt { 0.22 } else { 0.12 }, 0.05, rgb(40, 60, 30));
                }
                _ => {
                    let (c, outline) = match ob.kind { K::Wood => (rgb(196, 140, 80), rgb(110, 70, 35)), K::Stone => (rgb(150, 150, 162), rgb(80, 80, 92)), _ => (Color::new(0.7, 0.9, 1.0, 0.55), Color::new(0.9, 0.97, 1.0, 0.9)) };
                    let c = if ob.hit > 0.0 { Color::new((c.r + 0.25).min(1.0), (c.g + 0.25).min(1.0), (c.b + 0.25).min(1.0), c.a) } else { c };
                    let r = Vec2::from_angle(a);
                    let tr = |x: f32, y: f32| p + vec2(x * r.x - y * r.y, x * r.y + y * r.x);
                    let (hw, hh) = (ob.w / 2.0, ob.h / 2.0);
                    let (q0, q1, q2, q3) = (tr(-hw, -hh), tr(hw, -hh), tr(hw, hh), tr(-hw, hh));
                    draw_triangle(q0, q1, q2, c); draw_triangle(q0, q2, q3, c);
                    for (s, e) in [(q0, q1), (q1, q2), (q2, q3), (q3, q0)] { draw_line(s.x, s.y, e.x, e.y, 0.06, outline); }
                    // material detail
                    let maxhp = match ob.kind { K::Wood => 9.0, K::Stone => 26.0, _ => 3.0 };
                    if ob.kind == K::Wood { let along = hw > hh; for i in 1..3 { let f = i as f32 / 3.0 * 2.0 - 1.0; let (s, e) = if along { (tr(-hw * 0.9, hh * f * 0.6), tr(hw * 0.9, hh * f * 0.6)) } else { (tr(hw * f * 0.6, -hh * 0.9), tr(hw * f * 0.6, hh * 0.9)) }; draw_line(s.x, s.y, e.x, e.y, 0.03, outline); } }
                    if ob.kind == K::Stone { let s = tr(-hw, 0.0); let e = tr(hw, 0.0); draw_line(s.x, s.y, e.x, e.y, 0.03, outline); }
                    if ob.kind == K::Glass { let s = tr(-hw * 0.6, hh * 0.7); let e = tr(-hw * 0.1, hh * 0.7); draw_line(s.x, s.y, e.x, e.y, 0.05, WHITE); }
                    if ob.hp < maxhp * 0.5 { let s = tr(-hw * 0.5, -hh * 0.4); let m = tr(0.0, 0.1 * hh); let e = tr(hw * 0.3, hh * 0.5); draw_line(s.x, s.y, m.x, m.y, 0.04, Color::new(0.1, 0.05, 0.0, 0.7)); draw_line(m.x, m.y, e.x, e.y, 0.04, Color::new(0.1, 0.05, 0.0, 0.7)); }
                }
            }
        }
        for p in &self.parts {
            let a = (p.life / p.max).min(1.0);
            let r = Vec2::from_angle(p.rot) * p.s;
            draw_triangle(p.p + r, p.p + r.perp(), p.p - r, Color { a, ..p.col });
        }
        set_camera(&Camera2D { target: vec2(vw(), VH) / 2.0, zoom: vec2(2.0 / vw(), 2.0 / VH), ..Default::default() });
        for t in &self.texts {
            let k = ppm() / sc();
            let sp = vec2(t.p.x * k, VH - (t.p.y + 1.2) * k);
            txt(&t.s, sp.x - tw(&t.s, 30.0) / 2.0, sp.y, 30.0, Color { a: t.life.min(1.0), ..t.col });
        }
    }

    fn hud(&self, tapped: Option<Vec2>) -> Option<&'static str> {
        let w = vw();
        txt(&format!("BOSQICH {}", self.lvl + 1), 24.0, 44.0, 32.0, rgb(60, 40, 20));
        for i in 0..self.shots { draw_circle(36.0 + i as f32 * 34.0, 72.0, 12.0, rgb(110, 110, 120)); draw_circle(32.0 + i as f32 * 34.0, 68.0, 4.0, rgb(190, 190, 200)); }
        let s = format!("{}", self.score);
        txt(&s, w / 2.0 - tw(&s, 40.0) / 2.0, 48.0, 40.0, WHITE);
        let dl = format!("Devlar: {}", self.devs_left());
        txt(&dl, w / 2.0 - tw(&dl, 22.0) / 2.0, 76.0, 22.0, rgb(40, 90, 30));
        let mut act = None;
        for (i, (l, id)) in [("QAYTA", "retry"), ("MENYU", "menu")].iter().enumerate() {
            let r = Rect::new(w - 150.0 - i as f32 * 140.0, 20.0, 128.0, 52.0);
            btn(r, l, rgb(240, 200, 90));
            if tapped.map_or(false, |p| r.contains(p)) { act = Some(*id); }
        }
        if self.state == State::Play && self.ball.is_none() && self.shots > 0 && !self.aiming && self.t < 6.0 && self.lvl == 0 {
            txt("Toshni ushlab orqaga torting va qo'yib yuboring", 120.0, VH * 0.42, 24.0, Color::new(0.2, 0.1, 0.0, 0.8));
        }
        act
    }
}

fn btn(r: Rect, l: &str, col: Color) {
    draw_rectangle(r.x, r.y + 4.0, r.w, r.h, Color::new(col.r * 0.55, col.g * 0.55, col.b * 0.55, 1.0));
    draw_rectangle(r.x, r.y, r.w, r.h, col);
    txt(l, r.x + r.w / 2.0 - tw(l, 24.0) / 2.0, r.y + r.h / 2.0 + 8.0, 24.0, rgb(50, 30, 10));
}
fn txt(t: &str, x: f32, y: f32, size: f32, color: Color) {
    let s = sc();
    draw_text_ex(t, x, y, TextParams { font_size: (size * s) as u16, font_scale: 1.0 / s, color, ..Default::default() });
}
fn tw(t: &str, size: f32) -> f32 { let s = sc(); measure_text(t, None, (size * s) as u16, 1.0 / s).width }
fn centered(t: &str, y: f32, size: f32, col: Color) { txt(t, vw() / 2.0 - tw(t, size) / 2.0, y, size, col); }
fn star(c: Vec2, r: f32, on: bool) {
    let col = if on { rgb(255, 210, 60) } else { Color::new(0.3, 0.25, 0.2, 0.5) };
    let pts: Vec<Vec2> = (0..10).map(|i| { let a = -PI / 2.0 + i as f32 * PI / 5.0; c + Vec2::from_angle(a) * if i % 2 == 0 { r } else { r * 0.45 } }).collect();
    for i in 0..10 { draw_triangle(c, pts[i], pts[(i + 1) % 10], col); }
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
fn noise() -> f32 { rand::gen_range(-1.0, 1.0) }
fn sine(f: f32, t: f32) -> f32 { (TAU * f * t).sin() }
fn sq(f: f32, t: f32) -> f32 { if sine(f, t) >= 0.0 { 1.0 } else { -1.0 } }
async fn snd(dur: f32, f: impl Fn(f32) -> f32) -> Sound { load_sound_from_bytes(&wav(dur, f)).await.unwrap() }
async fn load_audio() -> (Sfx, Sound) {
    let sfx = Sfx {
        launch: snd(0.4, |t| (noise() * 0.5 + sine(180.0 - t * 200.0, t) * 0.4) * (-t * 7.0).exp()).await,
        wood: snd(0.25, |t| (noise() * 0.6 + sine(320.0 - t * 400.0, t) * 0.5) * (-t * 18.0).exp()).await,
        stone: snd(0.3, |t| (noise() * 0.5 + sine(110.0, t) * 0.7) * (-t * 14.0).exp()).await,
        glass: snd(0.4, |t| (sine(2400.0 + noise() * 400.0, t) * 0.25 + sine(3300.0, t) * 0.15 + noise() * 0.25) * (-t * 9.0).exp()).await,
        pop: snd(0.35, |t| (sine(500.0 + t * 900.0, t) * 0.5 + sq(250.0, t) * 0.1) * (-t * 9.0).exp()).await,
        win: snd(1.6, |t| { let f = [523.25, 659.25, 783.99, 1046.5, 783.99, 1046.5][((t / 0.16) as usize).min(5)]; (sq(f, t) * 0.1 + sine(f, t) * 0.4) * (1.0 - t / 1.6) }).await,
        lose: snd(1.2, |t| (sine(300.0 * (1.0 - t * 0.4), t) * 0.4) * (1.0 - t / 1.2)).await,
        stretch: snd(0.3, |t| sine(200.0 + t * 400.0, t) * 0.25 * (1.0 - t / 0.3)).await,
    };
    let music = snd(8.0, |t| {
        let roots = [261.63, 196.0, 220.0, 174.61];
        let f = roots[((t / 2.0) as usize).min(3)];
        let step = (t * 4.0) as usize;
        let mel = [1.0, 1.25, 1.5, 2.0, 1.5, 1.25, 1.0, 0.75][step % 8] * f;
        let env = (-(t % 0.25) * 6.0).exp();
        (sine(mel, t) * env * 0.08 + sq(mel, t) * env * 0.02 + sine(f / 2.0, t) * 0.08 * (0.6 + 0.4 * (-(t % 0.5) * 4.0).exp())) * (t * 20.0).min(1.0) * ((8.0 - t) * 20.0).min(1.0)
    }).await;
    (sfx, music)
}

fn conf() -> Conf { Conf { window_title: "QAL'A BUZAR".into(), window_width: 1280, window_height: 720, high_dpi: true, sample_count: 1, ..Default::default() } }

#[macroquad::main(conf)]
async fn main() {
    std::panic::set_hook(Box::new(|info| { macroquad::logging::error!("ZQ PANIC: {}", info); }));
    rand::srand(macroquad::miniquad::date::now() as u64);
    let (sfx, music) = load_audio().await;
    play_sound(&music, PlaySoundParams { looped: true, volume: 0.3 });
    let mut g = Game::new();
    g.load(0); g.state = State::Menu;
    let mut frames = 0u32;
    loop {
        frames += 1;
        if frames == 1 || frames == 60 { macroquad::logging::info!("ZQ frame {}", frames); }
        let dt = get_frame_time().min(1.0 / 30.0);
        clear_background(BLACK);
        let tap = if is_mouse_button_pressed(MouseButton::Left) { Some(Vec2::from(mouse_position()) / sc()) } else { None };
        g.update(dt, &sfx);
        g.draw();
        match g.state {
            State::Menu => {
                draw_rectangle(0.0, 0.0, vw(), VH, Color::new(0.1, 0.06, 0.02, 0.55));
                centered("QAL'A BUZAR", VH * 0.2, 96.0, rgb(255, 215, 90));
                centered("Katapult bilan devlar qal'asini buzing", VH * 0.2 + 42.0, 24.0, WHITE);
                let cols = 4; let (bw, bh) = (150.0, 110.0);
                let x0 = vw() / 2.0 - (cols as f32 * (bw + 20.0) - 20.0) / 2.0;
                for i in 0..LEVELS {
                    let r = Rect::new(x0 + (i % cols) as f32 * (bw + 20.0), VH * 0.36 + (i / cols) as f32 * (bh + 24.0), bw, bh);
                    let open = i < g.unlocked;
                    btn(r, "", if open { rgb(240, 200, 90) } else { rgb(120, 110, 100) });
                    let n = format!("{}", i + 1);
                    txt(&n, r.x + r.w / 2.0 - tw(&n, 44.0) / 2.0, r.y + 52.0, 44.0, rgb(60, 35, 10));
                    for s in 0..3 { star(vec2(r.x + r.w / 2.0 - 30.0 + s as f32 * 30.0, r.y + 82.0), 11.0, g.stars[i] > s as u32); }
                    if open && tap.map_or(false, |p| r.contains(p)) { g.load(i); }
                }
                centered("Toshni orqaga torting, nishonga oling va qo'yib yuboring", VH * 0.9, 22.0, Color::new(1.0, 1.0, 1.0, 0.8));
            }
            State::Play => {
                match g.hud(tap) { Some("retry") => { let l = g.lvl; g.load(l); } Some("menu") => { g.state = State::Menu; } _ => {} }
            }
            State::Won | State::Lost => {
                draw_rectangle(0.0, 0.0, vw(), VH, Color::new(0.1, 0.06, 0.02, (g.end_t * 0.8).min(0.55)));
                let won = g.state == State::Won;
                centered(if won { "QAL'A QULADI!" } else { "DEVLAR OMON QOLDI" }, VH * 0.3, 76.0, if won { rgb(255, 215, 90) } else { rgb(255, 120, 100) });
                if won { for s in 0..3 { let on = g.stars[g.lvl] > s as u32 && g.end_t > 0.3 + s as f32 * 0.3; star(vec2(vw() / 2.0 - 80.0 + s as f32 * 80.0, VH * 0.42), 32.0, on); } }
                centered(&format!("Ochko: {}", g.score), VH * 0.54, 36.0, WHITE);
                let cx = vw() / 2.0;
                let (r1, r2, r3) = (Rect::new(cx - 330.0, VH * 0.64, 200.0, 70.0), Rect::new(cx - 100.0, VH * 0.64, 200.0, 70.0), Rect::new(cx + 130.0, VH * 0.64, 200.0, 70.0));
                btn(r1, "MENYU", rgb(200, 190, 170)); btn(r2, "QAYTA", rgb(240, 200, 90));
                let has_next = won && g.lvl + 1 < LEVELS;
                if has_next { btn(r3, "KEYINGI", rgb(130, 220, 110)); }
                if g.end_t > 0.5 {
                    if let Some(p) = tap {
                        if r1.contains(p) { g.state = State::Menu; }
                        else if r2.contains(p) { let l = g.lvl; g.load(l); }
                        else if has_next && r3.contains(p) { let l = g.lvl + 1; g.load(l); }
                    }
                }
            }
        }
        next_frame().await;
    }
}
