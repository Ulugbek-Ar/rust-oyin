// QAL'A MUDOFAASI — pixel tower defense (Rust + macroquad)
use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

const VH: f32 = 720.0;
const T: f32 = 16.0;
const MC: i32 = 20;
const MR: i32 = 11;
const WAVES: u32 = 15;

fn sc() -> f32 { screen_height() / VH }
fn vw() -> f32 { screen_width() / sc() }
fn rgb(r: u8, g: u8, b: u8) -> Color { Color::from_rgba(r, g, b, 255) }
fn hash(x: u32, y: u32, s: u32) -> u32 {
    let mut h = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263) ^ s.wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

// ---------------- pixel art (shared look with Qal'a Ritsari) ----------------
const HERO: [&str; 16] = [
    "......kyk.......", ".....kyyyk......", "....kkhhhkk.....", "...khhhhhhhk....", "...khHhhhHhk....", "...kHkkkkkHk....", "...kHsesesHk....", "...kHHsssHHk....",
    "....kkHHHkk.....", "..kbbbbybbbbk...", ".ksbbbbybbbbsk..", ".ksBbbbybbbBsk..", "..kkBlllllBkk...", "...kBBBkBBBk....", "...kllk.kllk....", "...kkkk.kkkk....",
];
const SLIME: [&str; 14] = ["", "", "", "", "......kkkk......", "....kkggggkk....", "...kggggggggk...", "..kggwwggwwggk..", "..kggwkggwkggk..", ".kggggggggggggk.", ".kgGggggggggGgk.", ".kgGGggggggGGgk.", ".kGGGGGGGGGGGGk.", "..kkkkkkkkkkkk.."];
const BAT: [&str; 12] = ["", "", "", "", "k......kk......k", "kk....kppk....kk", "kpk..kprprk..kpk", "kppkkppppppkkppk", "kppppppppppppppk", ".kppppkppkppppk.", "..kpk.kppk.kpk..", "...k...kk...k..."];
const BOSS: [&str; 16] = [
    "......kkkk......", "....kkddddkk....", "...kddddddddk...", "..kddddddddddk..", "..kddkkkkkkddk..", "..kdkrrkkrrkdk..", "..kdkkkkkkkkdk..", "..kddkkkkkkddk..",
    ".kddddddddddddk.", ".kdpddddddddpdk.", "kddpddddddddpddk", "kdppddddddddppdk", "kdpddddddddddpdk", "kddddpdddpddddk.", ".kdddpdddpdddk..", "..kkkkkkkkkkkk..",
];
const TREE: [&str; 13] = ["", "", "", "......kkkk......", "....kkGGGGkk....", "...kGGgGGgGGk...", "..kGgGGGGGGgGk..", "..kGGGgGGGGGGk..", "..kGgGGGGgGGGk..", "...kGGGGGGGGk...", "....kkGlGGkk....", "......kllk......", ".....kkkkkk....."];
const COIN: [&str; 8] = ["..kkkk..", ".kyyyyk.", "kyywyyyk", "kyywyyyk", "kyyyyyyk", "kyyyyyyk", ".kyyyyk.", "..kkkk.."];
const HEART: [&str; 8] = [".kk.kk..", "krrkrrk.", "krrrrrk.", "krrrrrk.", ".krrrk..", "..krk...", "...k....", "........"];
const KNIGHT_SWAP: [(char, char); 7] = [('b', 'R'), ('B', 'D'), ('h', 'M'), ('H', 'N'), ('y', 'r'), ('s', 'k'), ('e', 'r')];

fn pal(c: char) -> Option<Color> {
    Some(match c {
        'k' => rgb(20, 12, 28), 'y' => rgb(255, 215, 90), 'h' => rgb(200, 208, 222), 'H' => rgb(120, 128, 150), 's' => rgb(242, 199, 154),
        'e' => rgb(30, 20, 40), 'b' => rgb(58, 108, 216), 'B' => rgb(37, 65, 138), 'l' => rgb(122, 74, 42), 'g' => rgb(100, 210, 100),
        'G' => rgb(40, 140, 70), 'p' => rgb(150, 80, 200), 'r' => rgb(240, 60, 60), 'w' => rgb(236, 232, 220), 'd' => rgb(72, 44, 104),
        'R' => rgb(200, 50, 50), 'D' => rgb(120, 25, 35), 'M' => rgb(90, 90, 105), 'N' => rgb(55, 55, 68), _ => return None,
    })
}
fn sprite(rows: &[&str], size: u16, swap: &[(char, char)]) -> Texture2D {
    let mut img = Image::gen_image_color(size, size, BLANK);
    for (y, r) in rows.iter().enumerate().take(size as usize) {
        for (x, c) in r.chars().enumerate().take(size as usize) {
            let c = swap.iter().find(|s| s.0 == c).map_or(c, |s| s.1);
            if let Some(col) = pal(c) { img.set_pixel(x as u32, y as u32, col); }
        }
    }
    let t = Texture2D::from_image(&img); t.set_filter(FilterMode::Nearest); t
}
fn noise_tex(seed: u32, base: [u8; 3], lo: [u8; 3], hi: [u8; 3], pebbles: bool) -> Texture2D {
    let mut img = Image::gen_image_color(16, 16, rgb(base[0], base[1], base[2]));
    for y in 0..16u32 {
        for x in 0..16u32 {
            let n = hash(x, y, seed) % 100;
            if n < 8 { img.set_pixel(x, y, rgb(hi[0], hi[1], hi[2])); } else if n < 16 { img.set_pixel(x, y, rgb(lo[0], lo[1], lo[2])); }
            else if pebbles && n == 99 { img.set_pixel(x, y, rgb(150, 130, 110)); }
        }
    }
    let t = Texture2D::from_image(&img); t.set_filter(FilterMode::Nearest); t
}
struct Art { slime: Texture2D, bat: Texture2D, knight: Texture2D, boss: Texture2D, tree: Texture2D, coin: Texture2D, heart: Texture2D, grass: [Texture2D; 3], dirt: [Texture2D; 2] }
impl Art {
    fn load() -> Art {
        Art {
            slime: sprite(&SLIME, 16, &[]), bat: sprite(&BAT, 16, &[]), knight: sprite(&HERO, 16, &KNIGHT_SWAP), boss: sprite(&BOSS, 16, &[]),
            tree: sprite(&TREE, 16, &[]), coin: sprite(&COIN, 8, &[]), heart: sprite(&HEART, 8, &[]),
            grass: [0, 1, 2].map(|s| noise_tex(s, [74, 148, 70], [60, 124, 60], [96, 172, 84], false)),
            dirt: [3, 4].map(|s| noise_tex(s, [176, 132, 86], [150, 110, 70], [196, 156, 106], true)),
        }
    }
}
fn spr(t: &Texture2D, x: f32, y: f32, w: f32, tint: Color, flip: bool) {
    draw_texture_ex(t, x, y, tint, DrawTextureParams { dest_size: Some(vec2(w, w)), flip_x: flip, ..Default::default() });
}

// ---------------- data ----------------
#[derive(Clone, Copy, PartialEq)]
enum TK { Archer, Mage, Cannon }
const TKS: [TK; 3] = [TK::Archer, TK::Mage, TK::Cannon];
impl TK {
    fn name(self) -> &'static str { match self { TK::Archer => "Kamonchi", TK::Mage => "Sehrgar", TK::Cannon => "To'p" } }
    fn cost(self, lvl: u32) -> u32 { match (self, lvl) { (TK::Archer, 0) => 50, (TK::Archer, 1) => 60, (TK::Archer, _) => 90,
        (TK::Mage, 0) => 80, (TK::Mage, 1) => 90, (TK::Mage, _) => 120, (TK::Cannon, 0) => 110, (TK::Cannon, 1) => 120, (TK::Cannon, _) => 160 } }
    fn range(self, lvl: u32) -> f32 { T * match self { TK::Archer => 3.0, TK::Mage => 2.6, TK::Cannon => 3.0 } + T * 0.35 * (lvl - 1) as f32 }
    fn dmg(self, lvl: u32) -> f32 { (match self { TK::Archer => 9.0, TK::Mage => 6.0, TK::Cannon => 24.0 }) * 1.6f32.powi(lvl as i32 - 1) }
    fn cd(self, lvl: u32) -> f32 { (match self { TK::Archer => 0.6, TK::Mage => 0.9, TK::Cannon => 1.6 }) - 0.08 * (lvl - 1) as f32 }
    fn color(self) -> Color { match self { TK::Archer => rgb(120, 200, 90), TK::Mage => rgb(170, 110, 255), TK::Cannon => rgb(255, 150, 60) } }
    fn desc(self) -> &'static str { match self { TK::Archer => "Tez, havoni ham uradi", TK::Mage => "Sekinlashtiradi", TK::Cannon => "Portlash, faqat yer" } }
}
#[derive(Clone, Copy, PartialEq)]
enum EK { Slime, Bat, Knight, Boss }
struct Tower { kind: TK, tx: i32, ty: i32, lvl: u32, cd: f32, aim: f32, flash: f32, spent: u32 }
struct Enemy { id: u32, kind: EK, d: f32, hp: f32, max: f32, speed: f32, slow: f32, flash: f32, t: f32 }
struct Shot { kind: TK, pos: Vec2, target: u32, last: Vec2, dmg: f32, from: Vec2, t: f32, dur: f32, lvl: u32 }
struct Part { p: Vec2, v: Vec2, life: f32, max: f32, col: Color, s: f32 }
struct FText { p: Vec2, s: String, col: Color, life: f32 }
#[derive(Clone, Copy, PartialEq)]
enum Sel { None, Tile(i32, i32), Tower(usize) }
#[derive(Clone, Copy, PartialEq)]
enum State { Title, Play, Won, Lost }
struct Sfx { arrow: Sound, zap: Sound, boom: Sound, die: Sound, coin: Sound, horn: Sound, hurt: Sound, win: Sound, lose: Sound, build: Sound }
fn play(s: &Sound, v: f32) { play_sound(s, PlaySoundParams { looped: false, volume: v }); }

const PATH: [(i32, i32); 8] = [(-1, 2), (4, 2), (4, 8), (9, 8), (9, 2), (14, 2), (14, 8), (18, 8)];
fn tc(x: i32, y: i32) -> Vec2 { vec2(x as f32 * T + 8.0, y as f32 * T + 8.0) }

struct Game {
    state: State, towers: Vec<Tower>, enemies: Vec<Enemy>, shots: Vec<Shot>, parts: Vec<Part>, texts: Vec<FText>,
    gold: u32, lives: i32, wave: u32, queue: Vec<(f32, EK)>, spawn_t: f32, next_id: u32, sel: Sel, speed2: bool,
    path: Vec<Vec2>, seg: Vec<f32>, total: f32, path_cells: Vec<(i32, i32)>, t: f32, shake: f32, end_t: f32, kills: u32, bonus_paid: bool,
}

impl Game {
    fn new() -> Game {
        let path: Vec<Vec2> = PATH.iter().map(|&(x, y)| tc(x, y)).collect();
        let seg: Vec<f32> = path.windows(2).map(|w| w[0].distance(w[1])).collect();
        let total = seg.iter().sum();
        let mut cells = vec![];
        for w in PATH.windows(2) {
            let (a, b) = (w[0], w[1]);
            let (dx, dy) = ((b.0 - a.0).signum(), (b.1 - a.1).signum());
            let (mut x, mut y) = a;
            loop { if x >= 0 { cells.push((x, y)); } if (x, y) == b { break; } x += dx; y += dy; }
        }
        for y in 7..=9 { cells.push((19, y)); cells.push((18, y)); }
        Game { state: State::Title, towers: vec![], enemies: vec![], shots: vec![], parts: vec![], texts: vec![], gold: 140, lives: 20, wave: 0,
            queue: vec![], spawn_t: 0.0, next_id: 1, sel: Sel::None, speed2: false, path, seg, total, path_cells: cells, t: 0.0, shake: 0.0, end_t: 0.0, kills: 0, bonus_paid: true }
    }
    fn path_pos(&self, d: f32) -> Vec2 {
        let mut d = d.max(0.0);
        for (i, l) in self.seg.iter().enumerate() {
            if d <= *l { return self.path[i].lerp(self.path[i + 1], d / l); }
            d -= l;
        }
        *self.path.last().unwrap()
    }
    fn blocked(&self, x: i32, y: i32) -> bool {
        x < 0 || y < 1 || x >= MC || y >= MR || self.path_cells.contains(&(x, y)) || self.towers.iter().any(|t| t.tx == x && t.ty == y)
    }
    fn tree_at(&self, x: i32, y: i32) -> bool { hash(x as u32, y as u32, 77) % 9 == 0 && !self.blocked(x, y) && y > 0 }
    fn wave_busy(&self) -> bool { !self.queue.is_empty() || !self.enemies.is_empty() }

    fn start_wave(&mut self, sfx: &Sfx) {
        if self.wave_busy() || self.wave >= WAVES { return; }
        self.wave += 1;
        self.bonus_paid = false;
        let n = self.wave;
        let count = (6 + n * 2).min(30);
        self.queue.clear();
        for i in 0..count {
            let r = rand::gen_range(0.0, 1.0);
            let k = if n >= 5 && r < 0.25 { EK::Knight } else if n >= 3 && r < 0.55 { EK::Bat } else { EK::Slime };
            self.queue.push((if k == EK::Bat { 0.5 } else { 0.85 }, k));
            if n >= 8 && i % 6 == 5 { self.queue.push((0.4, EK::Knight)); }
        }
        if n % 5 == 0 { self.queue.push((1.5, EK::Boss)); if n == WAVES { self.queue.push((2.0, EK::Boss)); } }
        self.queue.reverse();
        self.spawn_t = 0.5;
        play(&sfx.horn, 0.6);
        let p = vec2(MC as f32 * T / 2.0, MR as f32 * T / 2.0);
        self.text(p, format!("{}-TO'LQIN", n), rgb(255, 220, 120));
    }
    fn text(&mut self, p: Vec2, s: String, col: Color) { self.texts.push(FText { p, s, col, life: 1.0 }); }
    fn burst(&mut self, p: Vec2, col: Color, n: usize, spd: f32) {
        for _ in 0..n {
            let life = rand::gen_range(0.3, 0.7);
            self.parts.push(Part { p, v: Vec2::from_angle(rand::gen_range(0.0, TAU)) * rand::gen_range(0.2, 1.0) * spd, life, max: life, col, s: rand::gen_range(1.0, 2.5) });
        }
    }
    fn spawn(&mut self, k: EK) {
        let n = self.wave as f32;
        let scale = 1.0 + 0.3 * (n - 1.0) + 0.02 * (n - 1.0).powi(2);
        let (hp, spd) = match k { EK::Slime => (30.0, 22.0), EK::Bat => (18.0, 40.0), EK::Knight => (90.0, 16.0), EK::Boss => (700.0, 11.0) };
        self.enemies.push(Enemy { id: self.next_id, kind: k, d: 0.0, hp: hp * scale, max: hp * scale, speed: spd, slow: 0.0, flash: 0.0, t: rand::gen_range(0.0, 3.0) });
        self.next_id += 1;
    }
    fn reward(k: EK) -> u32 { match k { EK::Slime => 5, EK::Bat => 4, EK::Knight => 10, EK::Boss => 80 } }
    fn leak(k: EK) -> i32 { match k { EK::Boss => 10, EK::Knight => 2, _ => 1 } }

    fn damage(&mut self, id: u32, dmg: f32, slow: f32) {
        if let Some(e) = self.enemies.iter_mut().find(|e| e.id == id) {
            e.hp -= dmg; e.flash = 0.1;
            if slow > 0.0 { e.slow = e.slow.max(slow); }
        }
    }

    fn update(&mut self, dt: f32, sfx: &Sfx) {
        self.t += dt;
        self.shake = (self.shake - dt * 20.0).max(0.0);
        // spawning
        if let Some(&(delay, k)) = self.queue.last() {
            self.spawn_t -= dt;
            if self.spawn_t <= 0.0 { self.queue.pop(); self.spawn(k); self.spawn_t = delay; }
        }
        // enemies
        let mut leaked = vec![];
        for e in self.enemies.iter_mut() {
            e.t += dt; e.flash -= dt;
            let sl = if e.slow > 0.0 { 0.5 } else { 1.0 };
            e.slow = (e.slow - dt).max(0.0);
            e.d += e.speed * sl * dt;
            if e.d >= self.total { leaked.push((e.id, e.kind)); }
        }
        for (id, k) in leaked {
            self.enemies.retain(|e| e.id != id);
            self.lives -= Self::leak(k); self.shake = 4.0;
            play(&sfx.hurt, 0.6);
            let p = tc(18, 8);
            self.burst(p, rgb(240, 60, 60), 12, 60.0);
            if self.lives <= 0 { self.lives = 0; self.state = State::Lost; self.end_t = 0.0; play(&sfx.lose, 0.8); }
        }
        // towers
        let mut new_shots = vec![];
        let epos: Vec<(u32, Vec2, f32, bool)> = self.enemies.iter().map(|e| (e.id, self.path_pos(e.d), e.d, e.kind == EK::Bat)).collect();
        for tw in self.towers.iter_mut() {
            tw.cd -= dt; tw.flash -= dt;
            let p = tc(tw.tx, tw.ty);
            let r = tw.kind.range(tw.lvl);
            let target = epos.iter().filter(|e| e.1.distance(p) <= r && !(tw.kind == TK::Cannon && e.3)).max_by(|a, b| a.2.total_cmp(&b.2));
            if let Some(&(id, ep, _, _)) = target {
                tw.aim = (ep - p).to_angle();
                if tw.cd <= 0.0 {
                    tw.cd = tw.kind.cd(tw.lvl); tw.flash = 0.12;
                    let src = p - vec2(0.0, 7.0);
                    let dur = if tw.kind == TK::Cannon { 0.6 } else { 0.0 };
                    new_shots.push(Shot { kind: tw.kind, pos: src, target: id, last: ep, dmg: tw.kind.dmg(tw.lvl), from: src, t: 0.0, dur, lvl: tw.lvl });
                    play(match tw.kind { TK::Archer => &sfx.arrow, TK::Mage => &sfx.zap, TK::Cannon => &sfx.boom }, match tw.kind { TK::Cannon => 0.35, _ => 0.18 });
                }
            }
        }
        self.shots.extend(new_shots);
        // shots
        let mut hits: Vec<(TK, u32, Vec2, f32, u32)> = vec![];
        for s in self.shots.iter_mut() {
            if let Some(e) = epos.iter().find(|e| e.0 == s.target) { s.last = e.1 - vec2(0.0, 5.0); }
            s.t += dt;
            match s.kind {
                TK::Cannon => {
                    let k = (s.t / s.dur).min(1.0);
                    s.pos = s.from.lerp(s.last, k) - vec2(0.0, (k * PI).sin() * 20.0);
                    if k >= 1.0 { hits.push((s.kind, s.target, s.last, s.dmg, s.lvl)); s.dur = -1.0; }
                }
                _ => {
                    let spd = if s.kind == TK::Archer { 240.0 } else { 150.0 };
                    let d = s.last - s.pos;
                    if d.length() < 4.0 || s.t > 2.0 { hits.push((s.kind, s.target, s.last, s.dmg, s.lvl)); s.dur = -1.0; }
                    else { s.pos += d.normalize() * spd * dt; }
                }
            }
        }
        self.shots.retain(|s| s.dur >= 0.0);
        for (k, id, p, dmg, lvl) in hits {
            match k {
                TK::Archer => { self.damage(id, dmg, 0.0); self.burst(p, rgb(230, 220, 180), 3, 30.0); }
                TK::Mage => { self.damage(id, dmg, 1.6); self.burst(p, rgb(180, 120, 255), 8, 40.0); }
                TK::Cannon => {
                    let r = 18.0 + 4.0 * lvl as f32;
                    let ids: Vec<u32> = epos.iter().filter(|e| !e.3 && (e.1 - vec2(0.0, 5.0)).distance(p) < r).map(|e| e.0).collect();
                    for i in ids { self.damage(i, dmg, 0.0); }
                    self.burst(p, rgb(255, 160, 60), 18, 70.0); self.burst(p, rgb(90, 80, 80), 8, 30.0);
                    self.shake = self.shake.max(1.5);
                }
            }
        }
        // deaths
        let mut dead = vec![];
        self.enemies.retain(|e| if e.hp <= 0.0 { dead.push((e.kind, e.d)); false } else { true });
        for (k, d) in dead {
            let p = self.path_pos(d);
            let g = Self::reward(k);
            self.gold += g; self.kills += 1;
            self.text(p - vec2(0.0, 12.0), format!("+{}", g), rgb(255, 215, 90));
            let col = match k { EK::Slime => rgb(100, 210, 100), EK::Bat => rgb(150, 80, 200), EK::Knight => rgb(200, 50, 50), EK::Boss => rgb(150, 80, 200) };
            self.burst(p - vec2(0.0, 5.0), col, if k == EK::Boss { 60 } else { 12 }, if k == EK::Boss { 120.0 } else { 60.0 });
            play(&sfx.die, 0.35);
            if k == EK::Boss { self.shake = 8.0; }
        }
        // wave cleared
        if self.wave > 0 && !self.wave_busy() && !self.bonus_paid && self.state == State::Play {
            self.bonus_paid = true;
            if self.wave >= WAVES { self.state = State::Won; self.end_t = 0.0; play(&sfx.win, 0.8); }
            else {
                let b = 20 + 5 * self.wave;
                self.gold += b;
                let p = vec2(MC as f32 * T / 2.0, MR as f32 * T / 2.0);
                self.text(p, format!("To'lqin tugadi! +{}", b), rgb(140, 255, 160));
                play(&sfx.coin, 0.5);
            }
        }
        for p in &mut self.parts { p.p += p.v * dt; p.v *= 1.0 - dt * 4.0; p.life -= dt; }
        self.parts.retain(|p| p.life > 0.0);
        for t in &mut self.texts { t.p.y -= dt * 14.0; t.life -= dt * 0.8; }
        self.texts.retain(|t| t.life > 0.0);
    }

    // ---------------- view helpers ----------------
    fn view() -> (f32, Vec2) {
        let (mw, mh) = (MC as f32 * T, MR as f32 * T);
        let k = ((screen_width() - 20.0 * sc()) / mw).min((screen_height() - 110.0 * sc()) / mh).floor().max(1.0);
        (k, vec2(mw, mh) / 2.0 - vec2(0.0, 30.0 * sc() / k))
    }
    fn w2ui(p: Vec2) -> Vec2 { let (k, c) = Self::view(); ((p - c) * k + vec2(screen_width(), screen_height()) / 2.0) / sc() }
    fn ui2w(p: Vec2) -> Vec2 { let (k, c) = Self::view(); (p * sc() - vec2(screen_width(), screen_height()) / 2.0) / k + c }

    fn draw_tower(&self, tw: &Tower) {
        let p = tc(tw.tx, tw.ty);
        let (x, y) = (p.x - 8.0, p.y - 8.0);
        let r = |a: f32, b: f32, w: f32, h: f32, c: Color| draw_rectangle(x + a, y + b, w, h, c);
        draw_ellipse(p.x, p.y + 6.0, 7.0, 2.5, 0.0, Color::new(0.0, 0.0, 0.0, 0.3));
        let stone = rgb(150, 146, 160); let dark = rgb(96, 92, 110); let k = rgb(20, 12, 28);
        match tw.kind {
            TK::Archer => {
                r(3.0, 3.0, 10.0, 12.0, k); r(4.0, 4.0, 8.0, 10.0, stone); r(4.0, 7.0, 8.0, 1.0, dark); r(4.0, 11.0, 8.0, 1.0, dark);
                r(2.0, 1.0, 12.0, 4.0, k); r(3.0, 2.0, 10.0, 2.0, rgb(140, 90, 50));
                for i in 0..3 { r(3.0 + i as f32 * 4.0, -1.0, 2.0, 2.0, rgb(140, 90, 50)); }
                r(7.0, -3.0, 2.0, 2.0, rgb(242, 199, 154));
            }
            TK::Mage => {
                r(4.0, 6.0, 8.0, 9.0, k); r(5.0, 7.0, 6.0, 7.0, stone); r(7.0, 10.0, 2.0, 4.0, rgb(60, 40, 90));
                draw_triangle(vec2(x + 2.0, y + 7.0), vec2(x + 14.0, y + 7.0), vec2(x + 8.0, y - 5.0), rgb(110, 60, 200));
                draw_triangle(vec2(x + 4.0, y + 7.0), vec2(x + 8.0, y + 7.0), vec2(x + 8.0, y - 3.0), rgb(150, 90, 240));
                let glow = 2.0 + (self.t * 4.0).sin() * 0.5 + if tw.flash > 0.0 { 1.5 } else { 0.0 };
                draw_circle(x + 8.0, y - 6.0, glow + 1.5, Color::new(0.7, 0.5, 1.0, 0.35)); draw_circle(x + 8.0, y - 6.0, glow * 0.6, rgb(230, 200, 255));
            }
            TK::Cannon => {
                r(2.0, 8.0, 12.0, 7.0, k); r(3.0, 9.0, 10.0, 5.0, stone); r(3.0, 11.0, 10.0, 1.0, dark);
                let dir = Vec2::from_angle(tw.aim);
                let c = p + vec2(0.0, -1.0);
                let rec = if tw.flash > 0.0 { 2.0 } else { 0.0 };
                let tip = c + dir * (8.0 - rec);
                draw_line(c.x, c.y, tip.x, tip.y, 5.0, k); draw_line(c.x, c.y, tip.x, tip.y, 3.0, rgb(60, 60, 72));
                draw_circle(c.x, c.y, 3.0, rgb(40, 40, 50));
                if tw.flash > 0.0 { draw_circle(tip.x, tip.y, 3.0, rgb(255, 200, 80)); }
            }
        }
        for i in 0..tw.lvl { draw_rectangle(x + 4.0 + i as f32 * 3.0, y + 15.5, 2.0, 2.0, rgb(255, 215, 90)); }
    }

    fn draw_world(&self, art: &Art) {
        let (k, c) = Self::view();
        let sh = vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)) * self.shake * 0.3;
        set_camera(&Camera2D { target: c + sh, zoom: vec2(2.0 * k / screen_width(), 2.0 * k / screen_height()), ..Default::default() });
        let fs = T + 0.05;
        for y in 0..MR {
            for x in 0..MC {
                let (px, py) = (x as f32 * T, y as f32 * T);
                let h = hash(x as u32, y as u32, 5);
                if self.path_cells.contains(&(x, y)) && !(x >= 18 && (y == 7 || y == 9)) { spr(&art.dirt[(h % 2) as usize], px, py, fs, WHITE, false); }
                else { spr(&art.grass[(h % 3) as usize], px, py, fs, WHITE, false); }
            }
        }
        // build hints
        if let Sel::Tile(x, y) = self.sel { draw_rectangle_lines(x as f32 * T, y as f32 * T, T, T, 1.0, Color::new(1.0, 1.0, 1.0, 0.9)); }
        for y in 0..MR { for x in 0..MC { if self.tree_at(x, y) { spr(&art.tree, x as f32 * T, y as f32 * T - 4.0, 16.0, WHITE, false); } } }
        // castle
        let cp = vec2(18.0 * T, 6.5 * T);
        let castle = |a: f32, b: f32, w: f32, h: f32, col: Color| draw_rectangle(cp.x + a, cp.y + b, w, h, col);
        castle(0.0, 6.0, 32.0, 34.0, rgb(20, 12, 28)); castle(1.0, 7.0, 30.0, 32.0, rgb(160, 156, 172));
        for i in 0..4 { castle(1.0 + i as f32 * 8.0, 2.0, 5.0, 6.0, rgb(160, 156, 172)); castle(1.0 + i as f32 * 8.0, 2.0, 5.0, 1.0, rgb(200, 196, 210)); }
        castle(11.0, 22.0, 10.0, 17.0, rgb(70, 40, 30)); castle(12.0, 23.0, 8.0, 16.0, rgb(110, 70, 45));
        castle(5.0, 14.0, 4.0, 5.0, rgb(40, 30, 50)); castle(23.0, 14.0, 4.0, 5.0, rgb(40, 30, 50));
        castle(15.0, -8.0, 1.0, 10.0, rgb(90, 60, 40));
        let wave = (self.t * 5.0).sin();
        draw_triangle(cp + vec2(16.0, -8.0), cp + vec2(16.0, -3.0), cp + vec2(24.0 + wave, -5.5), rgb(58, 108, 216));
        // range of selected tower
        if let Sel::Tower(i) = self.sel {
            if let Some(tw) = self.towers.get(i) {
                let p = tc(tw.tx, tw.ty); let r = tw.kind.range(tw.lvl);
                draw_circle(p.x, p.y, r, Color::new(1.0, 1.0, 1.0, 0.1)); draw_circle_lines(p.x, p.y, r, 0.8, Color::new(1.0, 1.0, 1.0, 0.6));
            }
        }
        // towers & enemies, sorted by y
        let mut items: Vec<(f32, bool, usize)> = self.towers.iter().enumerate().map(|(i, t)| (tc(t.tx, t.ty).y, true, i)).collect();
        items.extend(self.enemies.iter().enumerate().map(|(i, e)| (self.path_pos(e.d).y, false, i)));
        items.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, is_t, i) in items {
            if is_t { self.draw_tower(&self.towers[i]); continue; }
            let e = &self.enemies[i];
            let p = self.path_pos(e.d);
            let big = e.kind == EK::Boss;
            let sz = if big { 32.0 } else { 16.0 };
            draw_ellipse(p.x, p.y + 6.0, sz * 0.3, sz * 0.1, 0.0, Color::new(0.0, 0.0, 0.0, 0.3));
            let (tex, bob) = match e.kind {
                EK::Slime => (&art.slime, -((e.t * 4.0).sin().abs()) * 2.0), EK::Bat => (&art.bat, (e.t * 10.0).sin() * 2.0 - 8.0),
                EK::Knight => (&art.knight, -((e.t * 6.0).sin().abs())), EK::Boss => (&art.boss, (e.t * 2.0).sin()),
            };
            let tint = if e.flash > 0.0 { Color::new(1.0, 0.5, 0.5, 1.0) } else if e.slow > 0.0 { Color::new(0.75, 0.7, 1.0, 1.0) } else { WHITE };
            let next = self.path_pos(e.d + 2.0);
            spr(tex, p.x - sz / 2.0, p.y - sz + 7.0 + bob, sz, tint, next.x < p.x);
            if e.hp < e.max {
                let w = if big { 24.0 } else { 12.0 };
                let yb = p.y - sz + 4.0 + bob;
                draw_rectangle(p.x - w / 2.0, yb, w, 1.5, rgb(40, 10, 15));
                draw_rectangle(p.x - w / 2.0, yb, w * (e.hp / e.max).max(0.0), 1.5, rgb(240, 60, 60));
            }
        }
        for s in &self.shots {
            match s.kind {
                TK::Archer => { let d = (s.last - s.pos).normalize_or_zero() * 4.0; draw_line(s.pos.x - d.x, s.pos.y - d.y, s.pos.x, s.pos.y, 1.0, rgb(240, 230, 200)); }
                TK::Mage => { draw_circle(s.pos.x, s.pos.y, 2.5, Color::new(0.7, 0.5, 1.0, 0.4)); draw_circle(s.pos.x, s.pos.y, 1.3, rgb(235, 210, 255)); }
                TK::Cannon => { draw_circle(s.pos.x, s.pos.y, 2.2, rgb(30, 30, 40)); }
            }
        }
        for p in &self.parts { let a = p.life / p.max; draw_rectangle(p.p.x, p.p.y, p.s, p.s, Color { a, ..p.col }); }
        set_camera(&Camera2D { target: vec2(vw(), VH) / 2.0, zoom: vec2(2.0 / vw(), 2.0 / VH), ..Default::default() });
    }

    // returns rects of popup buttons for hit testing
    fn popup(&self) -> Vec<(Rect, usize)> {
        let mut out = vec![];
        match self.sel {
            Sel::Tile(x, y) => {
                let p = Self::w2ui(tc(x, y));
                let (w, h) = (150.0, 120.0);
                let x0 = (p.x - (w * 3.0 + 20.0) / 2.0).clamp(10.0, vw() - w * 3.0 - 30.0);
                let y0 = if p.y > VH * 0.55 { p.y - h - 50.0 } else { p.y + 40.0 };
                for i in 0..3 { out.push((Rect::new(x0 + i as f32 * (w + 10.0), y0, w, h), i)); }
            }
            Sel::Tower(i) => {
                if let Some(tw) = self.towers.get(i) {
                    let p = Self::w2ui(tc(tw.tx, tw.ty));
                    let (w, h) = (170.0, 100.0);
                    let x0 = (p.x - w - 5.0).clamp(10.0, vw() - w * 2.0 - 20.0);
                    let y0 = if p.y > VH * 0.55 { p.y - h - 50.0 } else { p.y + 40.0 };
                    out.push((Rect::new(x0, y0, w, h), 10));
                    out.push((Rect::new(x0 + w + 10.0, y0, w, h), 11));
                }
            }
            Sel::None => {}
        }
        out
    }
    fn buttons() -> [Rect; 2] { let w = vw(); [Rect::new(w - 250.0, VH - 86.0, 150.0, 66.0), Rect::new(w - 90.0, VH - 86.0, 70.0, 66.0)] }

    fn click(&mut self, p: Vec2, sfx: &Sfx) {
        let [start, speed] = Self::buttons();
        if start.contains(p) { self.start_wave(sfx); self.sel = Sel::None; return; }
        if speed.contains(p) { self.speed2 = !self.speed2; return; }
        for (r, id) in self.popup() {
            if !r.contains(p) { continue; }
            match (self.sel, id) {
                (Sel::Tile(x, y), i) if i < 3 => {
                    let k = TKS[i];
                    if self.gold >= k.cost(0) {
                        self.gold -= k.cost(0);
                        self.towers.push(Tower { kind: k, tx: x, ty: y, lvl: 1, cd: 0.3, aim: 0.0, flash: 0.0, spent: k.cost(0) });
                        self.burst(tc(x, y), rgb(220, 220, 200), 12, 50.0); play(&sfx.build, 0.5);
                        self.sel = Sel::Tower(self.towers.len() - 1);
                    } else { self.text(tc(x, y), "Oltin yetmaydi".into(), rgb(255, 120, 120)); }
                }
                (Sel::Tower(i), 10) => {
                    let (k, l) = (self.towers[i].kind, self.towers[i].lvl);
                    if l < 3 && self.gold >= k.cost(l) {
                        self.gold -= k.cost(l); self.towers[i].lvl += 1; self.towers[i].spent += k.cost(l);
                        let tp = tc(self.towers[i].tx, self.towers[i].ty);
                        self.burst(tp, rgb(255, 215, 90), 16, 60.0); play(&sfx.build, 0.6);
                    } else if l < 3 { let tp = tc(self.towers[i].tx, self.towers[i].ty); self.text(tp, "Oltin yetmaydi".into(), rgb(255, 120, 120)); }
                }
                (Sel::Tower(i), 11) => {
                    let back = self.towers[i].spent * 6 / 10;
                    self.gold += back; let tp = tc(self.towers[i].tx, self.towers[i].ty);
                    self.text(tp, format!("+{}", back), rgb(255, 215, 90)); play(&sfx.coin, 0.5);
                    self.towers.remove(i); self.sel = Sel::None;
                }
                _ => {}
            }
            return;
        }
        let w = Self::ui2w(p);
        let (tx, ty) = ((w.x / T).floor() as i32, (w.y / T).floor() as i32);
        if let Some(i) = self.towers.iter().position(|t| t.tx == tx && t.ty == ty) { self.sel = Sel::Tower(i); }
        else if !self.blocked(tx, ty) { self.sel = if self.sel == Sel::Tile(tx, ty) { Sel::None } else { Sel::Tile(tx, ty) }; }
        else { self.sel = Sel::None; }
    }

    fn draw_ui(&self, art: &Art) {
        let w = vw();
        // top bar
        draw_rectangle(0.0, 0.0, w, 56.0, Color::new(0.06, 0.04, 0.1, 0.85));
        spr(&art.coin, 20.0, 12.0, 30.0, WHITE, false);
        txt(&format!("{}", self.gold), 58.0, 40.0, 30.0, rgb(255, 215, 90));
        spr(&art.heart, 170.0, 12.0, 30.0, WHITE, false);
        txt(&format!("{}", self.lives), 208.0, 40.0, 30.0, rgb(255, 120, 120));
        let wv = format!("To'lqin {}/{}", self.wave, WAVES);
        txt(&wv, w / 2.0 - tw(&wv, 28.0) / 2.0, 38.0, 28.0, WHITE);
        txt(&format!("O'ldirilgan: {}", self.kills), w - 220.0, 38.0, 22.0, Color::new(1.0, 1.0, 1.0, 0.6));
        // bottom buttons
        let [start, speed] = Self::buttons();
        let busy = self.wave_busy();
        let can = !busy && self.wave < WAVES;
        button(start, if can { if self.wave == 0 { "BOSHLASH" } else { "KEYINGI" } } else { "JANG..." }, if can { rgb(90, 200, 110) } else { rgb(90, 90, 110) });
        button(speed, if self.speed2 { "x2" } else { "x1" }, if self.speed2 { rgb(255, 180, 60) } else { rgb(140, 140, 170) });
        if self.wave == 0 && self.sel == Sel::None { txt("Maysani bosing: minora qurish", 20.0, VH - 40.0, 22.0, Color::new(1.0, 1.0, 1.0, 0.75)); }
        // floating texts
        for t in &self.texts {
            let p = Self::w2ui(t.p); let a = t.life.min(1.0);
            txt(&t.s, p.x - tw(&t.s, 24.0) / 2.0 + 2.0, p.y + 2.0, 24.0, Color::new(0.0, 0.0, 0.0, a * 0.6));
            txt(&t.s, p.x - tw(&t.s, 24.0) / 2.0, p.y, 24.0, Color { a, ..t.col });
        }
        // popup
        for (r, id) in self.popup() {
            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.08, 0.06, 0.14, 0.95));
            match (self.sel, id) {
                (Sel::Tile(..), i) if i < 3 => {
                    let k = TKS[i]; let ok = self.gold >= k.cost(0);
                    draw_rectangle_lines(r.x, r.y, r.w, r.h, 3.0, if ok { k.color() } else { rgb(90, 80, 100) });
                    txt(k.name(), r.x + 12.0, r.y + 34.0, 28.0, if ok { WHITE } else { GRAY });
                    spr(&art.coin, r.x + 12.0, r.y + 48.0, 20.0, WHITE, false);
                    txt(&format!("{}", k.cost(0)), r.x + 38.0, r.y + 66.0, 24.0, if ok { rgb(255, 215, 90) } else { rgb(200, 90, 90) });
                    txt(k.desc(), r.x + 12.0, r.y + 100.0, 16.0, Color::new(0.85, 0.85, 0.95, 0.85));
                }
                (Sel::Tower(i), 10) => {
                    let t = &self.towers[i];
                    if t.lvl < 3 {
                        let c = t.kind.cost(t.lvl); let ok = self.gold >= c;
                        draw_rectangle_lines(r.x, r.y, r.w, r.h, 3.0, if ok { rgb(255, 215, 90) } else { rgb(90, 80, 100) });
                        txt(&format!("Yangilash {}", t.lvl + 1), r.x + 12.0, r.y + 34.0, 24.0, if ok { WHITE } else { GRAY });
                        spr(&art.coin, r.x + 12.0, r.y + 48.0, 20.0, WHITE, false);
                        txt(&format!("{}", c), r.x + 38.0, r.y + 66.0, 24.0, if ok { rgb(255, 215, 90) } else { rgb(200, 90, 90) });
                        txt(&format!("zarar {:.0} -> {:.0}", t.kind.dmg(t.lvl), t.kind.dmg(t.lvl + 1)), r.x + 12.0, r.y + 90.0, 16.0, Color::new(0.85, 0.85, 0.95, 0.85));
                    } else {
                        draw_rectangle_lines(r.x, r.y, r.w, r.h, 3.0, rgb(255, 215, 90));
                        txt(t.kind.name(), r.x + 12.0, r.y + 40.0, 26.0, WHITE);
                        txt("Maksimal daraja", r.x + 12.0, r.y + 74.0, 18.0, rgb(255, 215, 90));
                    }
                }
                (Sel::Tower(i), 11) => {
                    let back = self.towers[i].spent * 6 / 10;
                    draw_rectangle_lines(r.x, r.y, r.w, r.h, 3.0, rgb(240, 90, 90));
                    txt("Sotish", r.x + 12.0, r.y + 34.0, 26.0, WHITE);
                    spr(&art.coin, r.x + 12.0, r.y + 48.0, 20.0, WHITE, false);
                    txt(&format!("+{}", back), r.x + 38.0, r.y + 66.0, 24.0, rgb(255, 215, 90));
                }
                _ => {}
            }
        }
    }
}

fn button(r: Rect, label: &str, col: Color) {
    draw_rectangle(r.x, r.y + 4.0, r.w, r.h, Color { r: col.r * 0.5, g: col.g * 0.5, b: col.b * 0.5, a: 1.0 });
    draw_rectangle(r.x, r.y, r.w, r.h, col);
    draw_rectangle(r.x, r.y, r.w, 3.0, Color::new(1.0, 1.0, 1.0, 0.3));
    txt(label, r.x + r.w / 2.0 - tw(label, 26.0) / 2.0, r.y + r.h / 2.0 + 9.0, 26.0, rgb(20, 12, 28));
}
fn txt(t: &str, x: f32, y: f32, size: f32, color: Color) {
    let s = sc();
    draw_text_ex(t, x, y, TextParams { font_size: (size * s) as u16, font_scale: 1.0 / s, color, ..Default::default() });
}
fn tw(t: &str, size: f32) -> f32 { let s = sc(); measure_text(t, None, (size * s) as u16, 1.0 / s).width }
fn centered(t: &str, y: f32, size: f32, col: Color) { txt(t, vw() / 2.0 - tw(t, size) / 2.0, y, size, col); }
fn tap() -> Option<Vec2> { if is_mouse_button_pressed(MouseButton::Left) { Some(Vec2::from(mouse_position()) / sc()) } else { None } }

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
        arrow: snd(0.12, |t| (sine(900.0 - t * 4000.0, t) * 0.4 + noise() * 0.2) * (-t * 30.0).exp()).await,
        zap: snd(0.25, |t| (sine(600.0 + (t * 90.0).sin() * 300.0, t) * 0.5) * (1.0 - t / 0.25)).await,
        boom: snd(0.45, |t| (noise() * 0.7 + sine(70.0 - t * 60.0, t)) * (-t * 9.0).exp() * 0.8).await,
        die: snd(0.2, |t| (sq(300.0 - t * 900.0, t) * 0.2 + noise() * 0.3) * (-t * 14.0).exp()).await,
        coin: snd(0.18, |t| sq(if t < 0.06 { 988.0 } else { 1319.0 }, t) * 0.18 * (1.0 - t / 0.18)).await,
        horn: snd(0.9, |t| (sq(196.0, t) * 0.15 + sine(196.0, t) * 0.3 + sine(293.66, t) * 0.2) * (t * 10.0).min(1.0) * (1.0 - t / 0.9)).await,
        hurt: snd(0.3, |t| (sine(200.0 - t * 300.0, t) * 0.6 + noise() * 0.2) * (-t * 8.0).exp()).await,
        win: snd(1.6, |t| { let f = [523.25, 659.25, 783.99, 1046.5, 783.99, 1046.5][((t / 0.16) as usize).min(5)]; (sq(f, t) * 0.12 + sine(f, t) * 0.4) * (1.0 - t / 1.6) }).await,
        lose: snd(1.4, |t| (sine(330.0 * (1.0 - t * 0.4), t) * 0.4 + sq(165.0 * (1.0 - t * 0.4), t) * 0.1) * (1.0 - t / 1.4)).await,
        build: snd(0.25, |t| (sine(220.0, t) * 0.5 + noise() * 0.3 * (-t * 30.0).exp()) * (-t * 12.0).exp() + sine(880.0, t) * 0.15 * (-(t - 0.08).abs() * 40.0).exp()).await,
    };
    let music = snd(8.0, |t| {
        let roots = [146.83, 130.81, 116.54, 130.81];
        let f = roots[((t / 2.0) as usize).min(3)];
        let step = (t * 4.0) as usize;
        let mel = [2.0, 2.4, 3.0, 2.4, 2.0, 1.5, 2.0, 3.0][step % 8] * f;
        let env = (-(t % 0.25) * 7.0).exp();
        let drum = if step % 2 == 0 { noise() * (-(t % 0.25) * 40.0).exp() * 0.12 } else { 0.0 };
        (sq(mel, t) * env * 0.04 + sine(mel, t) * env * 0.06 + sine(f / 2.0, t) * 0.1 + drum) * (t * 20.0).min(1.0) * ((8.0 - t) * 20.0).min(1.0)
    }).await;
    (sfx, music)
}

fn conf() -> Conf { Conf { window_title: "QAL'A MUDOFAASI".into(), window_width: 1280, window_height: 720, high_dpi: true, ..Default::default() } }

#[macroquad::main(conf)]
async fn main() {
    std::panic::set_hook(Box::new(|info| { macroquad::logging::error!("ZQ PANIC: {}", info); }));
    rand::srand(macroquad::miniquad::date::now() as u64);
    let art = Art::load();
    let (sfx, music) = load_audio().await;
    play_sound(&music, PlaySoundParams { looped: true, volume: 0.35 });
    let mut g = Game::new();
    let mut frames = 0u32;
    loop {
        frames += 1;
        if frames == 1 || frames == 60 { macroquad::logging::info!("ZQ frame {}", frames); }
        let dt = get_frame_time().min(1.0 / 30.0);
        clear_background(rgb(14, 10, 20));
        let t = tap();
        match g.state {
            State::Title => {
                g.t += dt;
                g.draw_world(&art);
                draw_rectangle(0.0, 0.0, vw(), VH, Color::new(0.03, 0.02, 0.06, 0.6));
                centered("QAL'A MUDOFAASI", VH * 0.34, 90.0, rgb(255, 215, 90));
                centered("Minoralar qur, 15 to'lqin dushmandan qal'ani saqla", VH * 0.34 + 44.0, 24.0, Color::new(1.0, 1.0, 1.0, 0.85));
                centered("Kamonchi  ·  Sehrgar  ·  To'p", VH * 0.58, 26.0, Color::new(0.8, 0.9, 1.0, 0.9));
                let p = 0.7 + 0.3 * (g.t * 3.0).sin();
                centered("[ Boshlash uchun bosing ]", VH * 0.74, 32.0, Color::new(1.0, 1.0, 1.0, p));
                if t.is_some() || is_key_pressed(KeyCode::Enter) { g = Game::new(); g.state = State::Play; }
            }
            State::Play => {
                if let Some(p) = t { g.click(p, &sfx); }
                if is_key_pressed(KeyCode::Space) { g.start_wave(&sfx); }
                let steps = if g.speed2 { 2 } else { 1 };
                for _ in 0..steps { if g.state == State::Play { g.update(dt, &sfx); } }
                g.draw_world(&art); g.draw_ui(&art);
            }
            State::Won | State::Lost => {
                g.end_t += dt; g.t += dt;
                for p in &mut g.parts { p.p += p.v * dt; p.life -= dt; }
                g.draw_world(&art);
                draw_rectangle(0.0, 0.0, vw(), VH, Color::new(0.02, 0.01, 0.04, (g.end_t * 0.6).min(0.6)));
                if g.state == State::Won {
                    centered("QAL'A SAQLAB QOLINDI!", VH * 0.34, 72.0, rgb(255, 215, 90));
                    let stars = if g.lives >= 18 { 3 } else if g.lives >= 10 { 2 } else { 1 };
                    centered(&format!("{} / 3 yulduz   Qolgan jon: {}", stars, g.lives), VH * 0.46, 32.0, WHITE);
                } else {
                    centered("QAL'A QULADI", VH * 0.34, 80.0, rgb(240, 80, 80));
                    centered(&format!("{}-to'lqingacha yetdingiz", g.wave), VH * 0.46, 32.0, WHITE);
                }
                centered(&format!("O'ldirilgan dushmanlar: {}", g.kills), VH * 0.53, 24.0, Color::new(1.0, 1.0, 1.0, 0.7));
                if g.end_t > 1.2 {
                    centered("[ Qaytadan - bosing ]", VH * 0.68, 30.0, rgb(255, 215, 90));
                    if t.is_some() || is_key_pressed(KeyCode::Enter) { g = Game::new(); g.state = State::Play; }
                }
            }
        }
        next_frame().await;
    }
}
