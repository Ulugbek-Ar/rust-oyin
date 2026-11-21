use macroquad::prelude::*;

const ARENA: f32 = 1400.0;

#[derive(PartialEq)]
enum State { Title, Play, LevelUp, Over }

struct Particle { p: Vec2, v: Vec2, life: f32, max: f32, size: f32, col: Color }
struct Bullet { p: Vec2, v: Vec2, life: f32, dmg: f32 }
struct Orb { p: Vec2, val: f32 }

#[derive(Clone, Copy, PartialEq)]
enum Kind { Chaser, Dasher, Tank, Splitter }

struct Enemy { p: Vec2, v: Vec2, hp: f32, kind: Kind, r: f32, flash: f32, timer: f32 }

struct Upgrade { name: &'static str, desc: &'static str }
const UPGRADES: [Upgrade; 6] = [
    Upgrade { name: "Tez o'q", desc: "Otish tezligi +25%" },
    Upgrade { name: "Kuchli nur", desc: "O'q zarari +30%" },
    Upgrade { name: "Uch tarmoq", desc: "+1 qo'shimcha o'q" },
    Upgrade { name: "Shamol", desc: "Harakat tezligi +15%" },
    Upgrade { name: "Hayot", desc: "Maks. HP +1 va to'liq tiklash" },
    Upgrade { name: "Magnit", desc: "Tajriba yig'ish radiusi +50%" },
];

struct Game {
    state: State,
    pos: Vec2, vel: Vec2, hp: i32, max_hp: i32, inv: f32,
    dash_cd: f32, dash_t: f32, dash_dir: Vec2,
    fire_cd: f32, fire_rate: f32, dmg: f32, shots: i32, speed: f32, magnet: f32,
    xp: f32, xp_need: f32, level: i32, choices: [usize; 3],
    bullets: Vec<Bullet>, enemies: Vec<Enemy>, parts: Vec<Particle>, orbs: Vec<Orb>,
    t: f32, spawn_t: f32, score: u32, best: u32, kills: u32,
    shake: f32, hitstop: f32, cam: Vec2,
}

impl Game {
    fn new(best: u32) -> Self {
        Game {
            state: State::Title, pos: Vec2::ZERO, vel: Vec2::ZERO, hp: 5, max_hp: 5, inv: 0.0,
            dash_cd: 0.0, dash_t: 0.0, dash_dir: Vec2::X,
            fire_cd: 0.0, fire_rate: 5.0, dmg: 1.0, shots: 1, speed: 320.0, magnet: 90.0,
            xp: 0.0, xp_need: 5.0, level: 1, choices: [0, 1, 2],
            bullets: vec![], enemies: vec![], parts: vec![], orbs: vec![],
            t: 0.0, spawn_t: 0.0, score: 0, best, kills: 0, shake: 0.0, hitstop: 0.0, cam: Vec2::ZERO,
        }
    }

    fn burst(&mut self, p: Vec2, col: Color, n: usize, spd: f32) {
        for _ in 0..n {
            let a = rand::gen_range(0.0, std::f32::consts::TAU);
            let s = rand::gen_range(0.2, 1.0) * spd;
            let life = rand::gen_range(0.3, 0.8);
            self.parts.push(Particle { p, v: Vec2::from_angle(a) * s, life, max: life, size: rand::gen_range(2.0, 5.0), col });
        }
    }

    fn spawn_enemy(&mut self) {
        let a = rand::gen_range(0.0, std::f32::consts::TAU);
        let p = self.pos + Vec2::from_angle(a) * rand::gen_range(650.0, 800.0);
        let p = p.clamp(Vec2::splat(-ARENA), Vec2::splat(ARENA));
        let diff = 1.0 + self.t / 40.0;
        let roll = rand::gen_range(0.0, 1.0);
        let kind = if self.t > 60.0 && roll < 0.12 { Kind::Tank }
            else if self.t > 35.0 && roll < 0.30 { Kind::Splitter }
            else if self.t > 15.0 && roll < 0.50 { Kind::Dasher }
            else { Kind::Chaser };
        let (hp, r) = match kind { Kind::Chaser => (2.0, 14.0), Kind::Dasher => (2.0, 12.0), Kind::Tank => (14.0, 30.0), Kind::Splitter => (5.0, 20.0) };
        self.enemies.push(Enemy { p, v: Vec2::ZERO, hp: hp * diff.sqrt(), kind, r, flash: 0.0, timer: rand::gen_range(0.0, 2.0) });
    }

    fn roll_choices(&mut self) {
        let mut ids: Vec<usize> = (0..UPGRADES.len()).collect();
        for i in 0..3 { let j = rand::gen_range(i, ids.len()); ids.swap(i, j); self.choices[i] = ids[i]; }
    }

    fn apply(&mut self, id: usize) {
        match id {
            0 => self.fire_rate *= 1.25, 1 => self.dmg *= 1.3, 2 => self.shots += 1,
            3 => self.speed *= 1.15, 4 => { self.max_hp += 1; self.hp = self.max_hp; }
            _ => self.magnet *= 1.5,
        }
    }

    fn update(&mut self, dt: f32) {
        self.t += dt;
        self.inv -= dt; self.dash_cd -= dt; self.dash_t -= dt; self.fire_cd -= dt;
        self.shake = (self.shake - dt * 30.0).max(0.0);

        // movement
        let mut dir = Vec2::ZERO;
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { dir.y -= 1.0 }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { dir.y += 1.0 }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { dir.x -= 1.0 }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { dir.x += 1.0 }
        let dir = dir.normalize_or_zero();
        if dir != Vec2::ZERO { self.dash_dir = dir; }
        if (is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Right)) && self.dash_cd <= 0.0 {
            self.dash_t = 0.15; self.dash_cd = 0.9; self.inv = self.inv.max(0.25); self.shake = 4.0;
        }
        if self.dash_t > 0.0 {
            self.vel = self.dash_dir * 1300.0;
            let p = self.pos;
            self.parts.push(Particle { p, v: Vec2::ZERO, life: 0.3, max: 0.3, size: 12.0, col: SKYBLUE });
        } else {
            self.vel = self.vel.lerp(dir * self.speed, (dt * 14.0).min(1.0));
        }
        self.pos = (self.pos + self.vel * dt).clamp(Vec2::splat(-ARENA), Vec2::splat(ARENA));

        // aim & fire
        let m: Vec2 = mouse_position().into();
        let world_m = m - vec2(screen_width(), screen_height()) / 2.0 + self.cam;
        let aim = (world_m - self.pos).normalize_or(Vec2::X);
        if self.fire_cd <= 0.0 {
            self.fire_cd = 1.0 / self.fire_rate;
            let spread = 0.12;
            for i in 0..self.shots {
                let off = (i as f32 - (self.shots - 1) as f32 / 2.0) * spread;
                let v = Vec2::from_angle(aim.to_angle() + off) * 900.0;
                self.bullets.push(Bullet { p: self.pos + aim * 16.0, v, life: 0.9, dmg: self.dmg });
            }
        }

        // spawn
        self.spawn_t -= dt;
        if self.spawn_t <= 0.0 {
            self.spawn_t = (1.1 - self.t / 120.0).max(0.18);
            let n = 1 + (self.t / 30.0) as usize;
            for _ in 0..n { self.spawn_enemy(); }
        }

        // bullets
        for b in &mut self.bullets { b.p += b.v * dt; b.life -= dt; }

        // enemies
        let ppos = self.pos;
        for e in &mut self.enemies {
            e.flash -= dt; e.timer += dt;
            let to = (ppos - e.p).normalize_or_zero();
            match e.kind {
                Kind::Chaser => e.v = e.v.lerp(to * 170.0, dt * 3.0),
                Kind::Tank => e.v = e.v.lerp(to * 80.0, dt * 2.0),
                Kind::Splitter => e.v = e.v.lerp((to + to.perp() * (e.timer * 3.0).sin() * 0.8) * 140.0, dt * 3.0),
                Kind::Dasher => {
                    if e.timer % 2.0 < 1.4 { e.v *= 1.0 - dt * 4.0; } else if e.v.length() < 200.0 { e.v = to * 650.0; }
                }
            }
            e.p += e.v * dt;
        }
        // enemy separation
        for i in 0..self.enemies.len() {
            for j in (i + 1)..self.enemies.len() {
                let d = self.enemies[j].p - self.enemies[i].p;
                let min = self.enemies[i].r + self.enemies[j].r;
                let l = d.length();
                if l < min && l > 0.01 {
                    let push = d / l * (min - l) * 0.5;
                    self.enemies[i].p -= push; self.enemies[j].p += push;
                }
            }
        }

        // bullet vs enemy
        let mut hits = vec![];
        for (bi, b) in self.bullets.iter_mut().enumerate() {
            for (ei, e) in self.enemies.iter_mut().enumerate() {
                if b.life > 0.0 && e.hp > 0.0 && b.p.distance(e.p) < e.r + 4.0 {
                    e.hp -= b.dmg; e.flash = 0.08; e.v += b.v.normalize() * 120.0; b.life = 0.0;
                    hits.push((bi, ei, b.p));
                }
            }
        }
        for (_, _, p) in &hits { self.burst(*p, ORANGE, 4, 200.0); }

        // deaths
        let mut dead = vec![];
        self.enemies.retain(|e| { if e.hp <= 0.0 { dead.push((e.p, e.kind, e.r)); false } else { true } });
        for (p, k, r) in dead {
            self.kills += 1;
            self.score += match k { Kind::Tank => 50, Kind::Splitter => 20, _ => 10 };
            let col = enemy_col(k);
            self.burst(p, col, 18, 380.0);
            self.burst(p, WHITE, 6, 150.0);
            self.shake = self.shake.max(r * 0.25);
            if k == Kind::Tank { self.hitstop = 0.06; }
            let val = match k { Kind::Tank => 5.0, Kind::Splitter => 2.0, _ => 1.0 };
            self.orbs.push(Orb { p, val });
            if k == Kind::Splitter {
                for s in [-1.0, 1.0] {
                    self.enemies.push(Enemy { p: p + vec2(s * 15.0, 0.0), v: vec2(s * 300.0, 0.0), hp: 1.5, kind: Kind::Chaser, r: 10.0, flash: 0.0, timer: 0.0 });
                }
            }
        }

        // enemy vs player
        if self.inv <= 0.0 {
            if let Some(e) = self.enemies.iter().find(|e| e.p.distance(self.pos) < e.r + 12.0) {
                let ep = e.p;
                self.hp -= 1; self.inv = 1.2; self.shake = 14.0; self.hitstop = 0.1;
                self.vel = (self.pos - ep).normalize_or(Vec2::X) * 700.0;
                let p = self.pos; self.burst(p, RED, 30, 450.0);
                if self.hp <= 0 {
                    self.burst(p, SKYBLUE, 80, 700.0);
                    self.best = self.best.max(self.score);
                    self.state = State::Over;
                }
            }
        }

        // orbs
        let mut gained = 0.0;
        let (pos, mag) = (self.pos, self.magnet);
        self.orbs.retain_mut(|o| {
            let d = pos.distance(o.p);
            if d < mag { o.p = o.p.lerp(pos, (dt * 10.0).min(1.0)); }
            if d < 16.0 { gained += o.val; false } else { true }
        });
        self.xp += gained;
        if self.xp >= self.xp_need {
            self.xp -= self.xp_need; self.xp_need *= 1.35; self.level += 1;
            self.roll_choices(); self.state = State::LevelUp;
        }

        // particles
        for p in &mut self.parts { p.p += p.v * dt; p.v *= 1.0 - dt * 3.0; p.life -= dt; }
        self.parts.retain(|p| p.life > 0.0);
        self.bullets.retain(|b| b.life > 0.0);

        self.cam = self.cam.lerp(self.pos, (dt * 6.0).min(1.0));
    }

    fn draw_world(&self) {
        let sh = vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)) * self.shake;
        let c = self.cam + sh;
        set_camera(&Camera2D { target: c, zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()), ..Default::default() });

        // grid with pulse
        let pulse = 0.5 + 0.5 * (self.t * 2.0).sin();
        let step = 80.0;
        let (x0, x1) = (((c.x - screen_width()) / step).floor() as i32, ((c.x + screen_width()) / step).ceil() as i32);
        let (y0, y1) = (((c.y - screen_height()) / step).floor() as i32, ((c.y + screen_height()) / step).ceil() as i32);
        for i in x0..=x1 {
            let x = i as f32 * step; let d = ((x - self.pos.x).abs() / 600.0).min(1.0);
            draw_line(x, y0 as f32 * step, x, y1 as f32 * step, 1.0, Color::new(0.2, 0.3, 0.7, 0.25 * (1.0 - d) + 0.04 * pulse));
        }
        for j in y0..=y1 {
            let y = j as f32 * step; let d = ((y - self.pos.y).abs() / 600.0).min(1.0);
            draw_line(x0 as f32 * step, y, x1 as f32 * step, y, 1.0, Color::new(0.2, 0.3, 0.7, 0.25 * (1.0 - d) + 0.04 * pulse));
        }
        draw_rectangle_lines(-ARENA, -ARENA, ARENA * 2.0, ARENA * 2.0, 4.0, Color::new(1.0, 0.2, 0.5, 0.6 + 0.4 * pulse));

        for o in &self.orbs { glow(o.p, 4.0 + o.val, GREEN); }
        for p in &self.parts {
            let k = p.life / p.max;
            draw_circle(p.p.x, p.p.y, p.size * k, Color { a: k * p.col.a, ..p.col });
        }
        for e in &self.enemies {
            let col = if e.flash > 0.0 { WHITE } else { enemy_col(e.kind) };
            glow(e.p, e.r * 0.6, Color { a: 0.5, ..col });
            let sides = match e.kind { Kind::Chaser => 3, Kind::Dasher => 4, Kind::Tank => 6, Kind::Splitter => 5 };
            let rot = e.v.to_angle().to_degrees() + if e.kind == Kind::Splitter { self.t * 120.0 } else { 0.0 };
            draw_poly(e.p.x, e.p.y, sides, e.r, rot, Color::new(0.02, 0.02, 0.05, 1.0));
            draw_poly_lines(e.p.x, e.p.y, sides, e.r, rot, 3.0, col);
            if e.kind == Kind::Dasher && e.timer % 2.0 > 1.1 && e.timer % 2.0 < 1.4 {
                let to = (self.pos - e.p).normalize_or_zero();
                draw_line(e.p.x, e.p.y, e.p.x + to.x * 120.0, e.p.y + to.y * 120.0, 2.0, Color::new(1.0, 0.9, 0.2, 0.6));
            }
        }
        for b in &self.bullets {
            let tail = b.p - b.v.normalize() * 18.0;
            draw_line(tail.x, tail.y, b.p.x, b.p.y, 4.0, Color::new(0.5, 0.9, 1.0, 0.5));
            draw_circle(b.p.x, b.p.y, 3.0, WHITE);
        }
        if self.state != State::Over && !(self.inv > 0.0 && (self.t * 20.0) as i32 % 2 == 0) {
            glow(self.pos, 12.0, SKYBLUE);
            draw_circle(self.pos.x, self.pos.y, 9.0, WHITE);
        }
        set_default_camera();
    }

    fn draw_hud(&self) {
        let w = screen_width();
        for i in 0..self.max_hp {
            let col = if i < self.hp { Color::new(1.0, 0.3, 0.4, 1.0) } else { Color::new(0.3, 0.1, 0.15, 1.0) };
            draw_poly(30.0 + i as f32 * 28.0, 30.0, 4, 10.0, 45.0, col);
        }
        draw_rectangle(20.0, 50.0, 220.0, 8.0, Color::new(0.1, 0.2, 0.1, 1.0));
        draw_rectangle(20.0, 50.0, 220.0 * (self.xp / self.xp_need).min(1.0), 8.0, GREEN);
        draw_text(&format!("LVL {}", self.level), 250.0, 59.0, 22.0, GREEN);
        let dash = if self.dash_cd <= 0.0 { "DASH  TAYYOR" } else { "DASH  ..." };
        draw_text(dash, 20.0, 82.0, 20.0, SKYBLUE);
        let s = format!("{:06}", self.score);
        let d = measure_text(&s, None, 40, 1.0);
        draw_text(&s, w / 2.0 - d.width / 2.0, 42.0, 40.0, WHITE);
        draw_text(&format!("{:02}:{:02}", self.t as u32 / 60, self.t as u32 % 60), w - 90.0, 36.0, 28.0, GRAY);
    }
}

fn enemy_col(k: Kind) -> Color {
    match k { Kind::Chaser => Color::new(1.0, 0.25, 0.4, 1.0), Kind::Dasher => Color::new(1.0, 0.85, 0.2, 1.0),
              Kind::Tank => Color::new(0.7, 0.3, 1.0, 1.0), Kind::Splitter => Color::new(1.0, 0.5, 0.1, 1.0) }
}

fn glow(p: Vec2, r: f32, c: Color) {
    for i in (1..=5).rev() {
        let k = i as f32;
        draw_circle(p.x, p.y, r * (1.0 + k * 0.6), Color { a: c.a * 0.06 * (6.0 - k), ..c });
    }
}

fn centered(text: &str, y: f32, size: u16, col: Color) {
    let d = measure_text(text, None, size, 1.0);
    draw_text(text, screen_width() / 2.0 - d.width / 2.0, y, size as f32, col);
}

fn conf() -> Conf {
    Conf { window_title: "ZULMAT".into(), window_width: 1280, window_height: 720, high_dpi: true, sample_count: 4, ..Default::default() }
}

#[macroquad::main(conf)]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as u64);
    let mut g = Game::new(0);
    loop {
        let dt = get_frame_time().min(1.0 / 30.0);
        clear_background(Color::new(0.02, 0.02, 0.06, 1.0));
        let (cx, h) = (screen_width() / 2.0, screen_height());
        match g.state {
            State::Title => {
                g.t += dt;
                g.draw_world();
                let pulse = 0.7 + 0.3 * (g.t * 3.0).sin();
                centered("ZULMAT", h * 0.38, 120, Color::new(0.5, 0.9, 1.0, pulse));
                centered("Zulmatda yorug'lik bo'l", h * 0.38 + 50.0, 28, GRAY);
                centered("WASD - harakat   SICHQONCHA - nishon   SPACE - dash", h * 0.62, 24, WHITE);
                centered("[ ENTER ] boshlash", h * 0.72, 32, Color::new(1.0, 1.0, 1.0, pulse));
                if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                    let best = g.best; g = Game::new(best); g.state = State::Play;
                }
            }
            State::Play => {
                if g.hitstop > 0.0 { g.hitstop -= dt; } else { g.update(dt); }
                g.draw_world(); g.draw_hud();
                if is_key_pressed(KeyCode::Escape) { g.state = State::Title; }
            }
            State::LevelUp => {
                g.draw_world(); g.draw_hud();
                draw_rectangle(0.0, 0.0, screen_width(), h, Color::new(0.0, 0.0, 0.0, 0.6));
                centered("DARAJA OSHDI!", h * 0.25, 60, GREEN);
                let mut pick = None;
                for i in 0..3 {
                    let u = &UPGRADES[g.choices[i]];
                    let r = Rect::new(cx - 420.0 + i as f32 * 290.0, h * 0.38, 260.0, 180.0);
                    let hover = r.contains(mouse_position().into());
                    draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.05, 0.08, 0.15, 0.95));
                    draw_rectangle_lines(r.x, r.y, r.w, r.h, if hover { 4.0 } else { 2.0 }, if hover { SKYBLUE } else { GRAY });
                    draw_text(&format!("[{}]", i + 1), r.x + 15.0, r.y + 35.0, 28.0, SKYBLUE);
                    draw_text(u.name, r.x + 15.0, r.y + 85.0, 34.0, WHITE);
                    draw_text(u.desc, r.x + 15.0, r.y + 130.0, 20.0, LIGHTGRAY);
                    let key = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3][i];
                    if is_key_pressed(key) || (hover && is_mouse_button_pressed(MouseButton::Left)) { pick = Some(g.choices[i]); }
                }
                if let Some(id) = pick { g.apply(id); g.state = State::Play; }
            }
            State::Over => {
                g.t += dt;
                for p in &mut g.parts { p.p += p.v * dt; p.v *= 1.0 - dt * 3.0; p.life -= dt; }
                g.parts.retain(|p| p.life > 0.0);
                g.draw_world();
                draw_rectangle(0.0, 0.0, screen_width(), h, Color::new(0.0, 0.0, 0.0, 0.5));
                centered("NUR SO'NDI", h * 0.35, 90, Color::new(1.0, 0.3, 0.4, 1.0));
                centered(&format!("Ochko: {}   Eng yaxshi: {}", g.score, g.best), h * 0.48, 36, WHITE);
                centered(&format!("O'ldirilgan: {}   Vaqt: {:.0}s   Daraja: {}", g.kills, g.t, g.level), h * 0.55, 26, GRAY);
                centered("[ ENTER ] qaytadan", h * 0.7, 32, SKYBLUE);
                if is_key_pressed(KeyCode::Enter) { let best = g.best; g = Game::new(best); g.state = State::Play; }
            }
        }
        next_frame().await;
    }
}
