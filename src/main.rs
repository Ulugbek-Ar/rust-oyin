// QAL'A RITSARI — pixel action RPG (Rust + macroquad)
use macroquad::audio::{load_sound_from_bytes, play_sound, set_sound_volume, PlaySoundParams, Sound};
use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

const VH: f32 = 720.0;
const T: f32 = 16.0;
const RW: usize = 19;
const RH: usize = 11;
const GRID: i32 = 7;
const DIRS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

fn sc() -> f32 { screen_height() / VH }
fn vw() -> f32 { screen_width() / sc() }
fn rgb(r: u8, g: u8, b: u8) -> Color { Color::from_rgba(r, g, b, 255) }
fn hash(x: u32, y: u32, s: u32) -> u32 {
    let mut h = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263) ^ s.wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

// ---------------- pixel art ----------------
const HERO: [&str; 16] = [
    "......kyk.......", ".....kyyyk......", "....kkhhhkk.....", "...khhhhhhhk....",
    "...khHhhhHhk....", "...kHkkkkkHk....", "...kHsesesHk....", "...kHHsssHHk....",
    "....kkHHHkk.....", "..kbbbbybbbbk...", ".ksbbbbybbbbsk..", ".ksBbbbybbbBsk..",
    "..kkBlllllBkk...", "...kBBBkBBBk....", "...kllk.kllk....", "...kkkk.kkkk....",
];
const SLIME: [&str; 14] = [
    "", "", "", "", "......kkkk......", "....kkggggkk....", "...kggggggggk...", "..kggwwggwwggk..",
    "..kggwkggwkggk..", ".kggggggggggggk.", ".kgGggggggggGgk.", ".kgGGggggggGGgk.", ".kGGGGGGGGGGGGk.", "..kkkkkkkkkkkk..",
];
const BAT: [&str; 12] = [
    "", "", "", "", "k......kk......k", "kk....kppk....kk", "kpk..kprprk..kpk", "kppkkppppppkkppk",
    "kppppppppppppppk", ".kppppkppkppppk.", "..kpk.kppk.kpk..", "...k...kk...k...",
];
const SKEL: [&str; 16] = [
    "", "......kkkk......", ".....kwwwwk.....", "....kwwwwwwk....", "....kwkwwkwk....", "....kwwwwwwk....",
    ".....kwkwkk.....", "......kwwk......", "....kkwwwwkk.l..", "...kwkwkkwkwkl..", "...kwkwwwwkwkl..",
    "....kkwkkwkk.l..", ".....kwkkwk..l..", ".....kwk.kwk....", ".....kwk..kwk...", "....kkk...kkk...",
];
const BOSS: [&str; 16] = [
    "......kkkk......", "....kkddddkk....", "...kddddddddk...", "..kddddddddddk..", "..kddkkkkkkddk..",
    "..kdkrrkkrrkdk..", "..kdkkkkkkkkdk..", "..kddkkkkkkddk..", ".kddddddddddddk.", ".kdpddddddddpdk.",
    "kddpddddddddpddk", "kdppddddddddppdk", "kdpddddddddddpdk", "kddddpdddpddddk.", ".kdddpdddpdddk..", "..kkkkkkkkkkkk..",
];
const COIN: [&str; 8] = ["..kkkk..", ".kyyyyk.", "kyywyyyk", "kyywyyyk", "kyyyyyyk", "kyyyyyyk", ".kyyyyk.", "..kkkk.."];
const POTION: [&str; 8] = ["..kkkk..", "...kk...", "..kwwk..", ".krrrrk.", "krrwrrrk", "krrrrrrk", "krrrrrrk", ".kkkkkk."];
const HEART: [&str; 8] = [".kk.kk..", "krrkrrk.", "krrrrrk.", "krrrrrk.", ".krrrk..", "..krk...", "...k....", "........"];
const CHEST: [&str; 13] = [
    "", "", "", "", "..kkkkkkkkkkkk..", ".kllllllllllllk.", ".klLLLLLLLLLLlk.", ".kkkkkkyykkkkkk.",
    ".kllllkyykllllk.", ".kllllkkkkllllk.", ".kllllllllllllk.", ".kLLLLLLLLLLLLk.", ".kkkkkkkkkkkkkk.",
];

fn pal(c: char) -> Option<Color> {
    Some(match c {
        'k' => rgb(20, 12, 28), 'y' => rgb(255, 215, 90), 'h' => rgb(200, 208, 222), 'H' => rgb(120, 128, 150),
        's' => rgb(242, 199, 154), 'e' => rgb(30, 20, 40), 'b' => rgb(58, 108, 216), 'B' => rgb(37, 65, 138),
        'l' => rgb(122, 74, 42), 'L' => rgb(82, 48, 28), 'g' => rgb(100, 210, 100), 'G' => rgb(40, 140, 70),
        'p' => rgb(150, 80, 200), 'r' => rgb(240, 60, 60), 'w' => rgb(236, 232, 220), 'd' => rgb(72, 44, 104),
        'R' => rgb(200, 50, 50), 'D' => rgb(120, 25, 35), 'M' => rgb(90, 90, 105), 'N' => rgb(55, 55, 68),
        _ => return None,
    })
}
fn sprite_img(rows: &[&str], size: u16, swap: &[(char, char)]) -> Image {
    let mut img = Image::gen_image_color(size, size, BLANK);
    for (y, r) in rows.iter().enumerate().take(size as usize) {
        for (x, c) in r.chars().enumerate().take(size as usize) {
            let c = swap.iter().find(|s| s.0 == c).map_or(c, |s| s.1);
            if let Some(col) = pal(c) { img.set_pixel(x as u32, y as u32, col); }
        }
    }
    img
}
fn img_rows(rows: &[String], size: u16, swap: &[(char, char)], white: bool) -> Image {
    let r: Vec<&str> = rows.iter().map(|x| x.as_str()).collect();
    let mut img = sprite_img(&r, size, swap);
    if white { for y in 0..size as u32 { for x in 0..size as u32 { if img.get_pixel(x, y).a > 0.0 { img.set_pixel(x, y, WHITE); } } } }
    img
}
fn owned(rows: &[&str]) -> Vec<String> { rows.iter().map(|x| x.to_string()).collect() }
const BAT_UP: [&str; 12] = ["", "", "", "k..............k", "kk.....kk.....kk", "kpk...kppk...kpk", "kppk.kprprk.kppk", ".kppkppppppkppk.", "..kpppppppppppk.", "...kppkppkppk...", "....k.kppk.k....", ".......kk......."];
const BAT_DN: [&str; 12] = ["", "", "", "", "", ".......kk.......", "......kppk......", ".....kprprk.....", "..kkkppppppkkk..", ".kpppppppppppppk", "kppk.kppppk.kppk", "kpk...kppk...kpk"];
fn white(rows: &[&str], size: u16, swap: &[(char, char)]) -> Texture2D { tex(&img_rows(&owned(rows), size, swap, true)) }
fn spr2(t: &Texture2D, x: f32, y: f32, w: f32, h: f32, tint: Color, flip: bool) {
    draw_texture_ex(t, x, y, tint, DrawTextureParams { dest_size: Some(vec2(w, h)), flip_x: flip, ..Default::default() });
}
fn region_img(rows: &[&str], b: (u32, u32, u32, u32), swap: &[(char, char)], white: bool) -> Image {
    let (x0, y0, w, h) = b;
    let mut img = Image::gen_image_color(w as u16, h as u16, BLANK);
    for y in 0..h {
        let row = rows.get((y0 + y) as usize).copied().unwrap_or("");
        for x in 0..w {
            if let Some(c) = row.chars().nth((x0 + x) as usize) {
                let c = swap.iter().find(|s| s.0 == c).map_or(c, |s| s.1);
                if let Some(col) = pal(c) { img.set_pixel(x, y, if white { WHITE } else { col }); }
            }
        }
    }
    img
}
// Cut-out rig: head, torso and two legs are cut from the sprite, arms are drawn as limbs.
struct Rig { parts: Vec<(Texture2D, Texture2D, Vec2, Vec2)>, hip: [Vec2; 2], shoulder: [Vec2; 2], sleeve: Color, hand: Color, arm_len: f32, arm_w: f32 }
fn make_rig(rows: &[&str], swap: &[(char, char)], torso_swap: &[(char, char)], boxes: [(u32, u32, u32, u32); 4], hip: [Vec2; 2], shoulder: [Vec2; 2], sleeve: Color, hand: Color, arm_len: f32, arm_w: f32) -> Rig {
    let mut parts = vec![];
    for (i, bx) in boxes.iter().enumerate() {
        let sw: Vec<(char, char)> = if i == 1 { torso_swap.iter().chain(swap.iter()).cloned().collect() } else { swap.to_vec() };
        let t = tex(&region_img(rows, *bx, &sw, false)); let w = tex(&region_img(rows, *bx, &sw, true));
        parts.push((t, w, vec2(bx.0 as f32, bx.1 as f32), vec2(bx.2 as f32, bx.3 as f32)));
    }
    Rig { parts, hip, shoulder, sleeve, hand, arm_len, arm_w }
}
struct Pose { phase: f32, moving: bool, breath: f32, front: Option<Vec2>, back_swing: f32, leg_spread: f32 }
fn rig_point(feet: Vec2, flip: bool, scale: f32, p: Vec2) -> Vec2 { let x = if flip { 16.0 - p.x } else { p.x }; feet - vec2(8.0, 16.0) * scale + vec2(x, p.y) * scale }
fn draw_rig(r: &Rig, feet: Vec2, flip: bool, scale: f32, pose: &Pose, white: bool, tint: Color) {
    let sgn = if flip { -1.0 } else { 1.0 };
    let leg_a = if pose.moving { pose.phase.sin() * 0.6 } else { 0.0 } + pose.leg_spread;
    let bob = if pose.moving { -pose.phase.sin().abs() * 1.3 * scale } else { -pose.breath * scale };
    let part = |i: usize, dy: f32, rot: f32, pivot: Option<Vec2>| {
        let (t, w, at, sz) = &r.parts[i];
        let x = if flip { 16.0 - at.x - sz.x } else { at.x };
        let o = feet - vec2(8.0, 16.0) * scale + vec2(x, at.y) * scale + vec2(0.0, dy);
        draw_texture_ex(if white { w } else { t }, o.x, o.y, tint, DrawTextureParams { dest_size: Some(*sz * scale), flip_x: flip, rotation: rot, pivot, ..Default::default() });
    };
    let arm = |sh: Vec2, dir: Vec2| {
        let s = rig_point(feet, flip, scale, sh) + vec2(0.0, bob);
        let e = s + dir.normalize_or(Vec2::Y) * r.arm_len * scale;
        let (sl, hd) = if white { (WHITE, WHITE) } else { (r.sleeve, r.hand) };
        draw_line(s.x, s.y, e.x, e.y, r.arm_w * scale, Color { a: tint.a, ..sl });
        draw_rectangle(e.x - r.arm_w * scale * 0.6, e.y - r.arm_w * scale * 0.6, r.arm_w * scale * 1.2, r.arm_w * scale * 1.2, Color { a: tint.a, ..hd });
    };
    let down = |a: f32| vec2(a.sin() * sgn, a.cos());
    arm(r.shoulder[0], down(-leg_a * 1.2 + pose.back_swing));
    part(2, 0.0, leg_a * sgn, Some(rig_point(feet, flip, scale, r.hip[0])));
    part(3, 0.0, -leg_a * sgn, Some(rig_point(feet, flip, scale, r.hip[1])));
    part(1, bob, 0.0, None);
    part(0, bob * 1.15 + if pose.moving { 0.0 } else { -pose.breath * scale * 0.3 }, 0.0, None);
    arm(r.shoulder[1], pose.front.unwrap_or(down(leg_a * 1.2 - pose.back_swing)));
}
fn tex(img: &Image) -> Texture2D { let t = Texture2D::from_image(img); t.set_filter(FilterMode::Nearest); t }
fn floor_tex(seed: u32) -> Texture2D {
    let mut img = Image::gen_image_color(16, 16, rgb(36, 32, 48));
    for y in 0..16u32 {
        for x in 0..16u32 {
            let n = hash(x, y, seed) % 100;
            let c = if n < 6 { rgb(46, 41, 60) } else if n < 12 { rgb(29, 26, 39) } else { continue };
            img.set_pixel(x, y, c);
        }
    }
    for i in 0..16 {
        img.set_pixel(i, 0, rgb(42, 38, 56)); img.set_pixel(0, i, rgb(42, 38, 56));
        img.set_pixel(i, 15, rgb(26, 23, 35)); img.set_pixel(15, i, rgb(26, 23, 35));
    }
    if seed == 2 { for i in 3..10 { img.set_pixel(i, 5 + i % 3, rgb(22, 19, 30)); } }
    tex(&img)
}
fn wall_tex() -> Texture2D {
    let mut img = Image::gen_image_color(16, 16, rgb(26, 22, 36));
    for row in 0..4u32 {
        let off = if row % 2 == 0 { 0 } else { 4 };
        for bi in -1..3i32 {
            let x0 = bi * 8 + off;
            let sh = 58 + (hash((bi + 5) as u32, row, 7) % 14) as u8;
            for y in row * 4..row * 4 + 3 {
                for x in x0..x0 + 7 {
                    if (0..16).contains(&x) {
                        let hl = if y == row * 4 { 14 } else { 0 };
                        img.set_pixel(x as u32, y, rgb(sh + hl, sh - 6 + hl, sh + 18 + hl));
                    }
                }
            }
        }
    }
    tex(&img)
}

struct Art { rig_hero: Rig, rig_knight: Rig, rig_skel: Rig, bat_f: [Texture2D; 3],
    knight_w: Texture2D, skel_w: Texture2D, slime_w: Texture2D, bat_w: Texture2D, boss_w: Texture2D,
    merchant: Texture2D, slime: Texture2D, boss: Texture2D,
    coin: Texture2D, potion: Texture2D, heart: Texture2D, chest: Texture2D, floor: [Texture2D; 3], wall: Texture2D }
const KNIGHT_SWAP: [(char, char); 7] = [('b', 'R'), ('B', 'D'), ('h', 'M'), ('H', 'N'), ('y', 'r'), ('s', 'k'), ('e', 'r')];
impl Art {
    fn load() -> Art {
        Art {
            rig_hero: make_rig(&HERO, &[], &[('s', 'b')], [(3, 0, 10, 9), (2, 9, 11, 4), (3, 13, 4, 3), (8, 13, 4, 3)], [vec2(5.0, 13.0), vec2(10.0, 13.0)], [vec2(2.5, 10.0), vec2(12.5, 10.0)], rgb(37, 65, 138), rgb(242, 199, 154), 3.6, 1.7),
            rig_knight: make_rig(&HERO, &KNIGHT_SWAP, &[('s', 'R')], [(3, 0, 10, 9), (2, 9, 11, 4), (3, 13, 4, 3), (8, 13, 4, 3)], [vec2(5.0, 13.0), vec2(10.0, 13.0)], [vec2(2.5, 10.0), vec2(12.5, 10.0)], rgb(120, 25, 35), rgb(55, 55, 68), 3.6, 1.8),
            rig_skel: make_rig(&SKEL, &[], &[], [(4, 1, 8, 7), (4, 8, 8, 5), (4, 13, 4, 3), (9, 13, 4, 3)], [vec2(6.0, 13.0), vec2(11.0, 13.0)], [vec2(4.0, 9.0), vec2(11.5, 9.0)], rgb(236, 232, 220), rgb(236, 232, 220), 3.6, 1.2),
            bat_f: [tex(&sprite_img(&BAT, 16, &[])), tex(&sprite_img(&BAT_UP, 16, &[])), tex(&sprite_img(&BAT_DN, 16, &[]))],
            knight_w: white(&HERO, 16, &KNIGHT_SWAP), skel_w: white(&SKEL, 16, &[]),
            slime_w: white(&SLIME, 16, &[]), bat_w: white(&BAT, 16, &[]), boss_w: white(&BOSS, 16, &[]),
            merchant: tex(&sprite_img(&HERO, 16, &[('b', 'p'), ('B', 'd'), ('h', 'y'), ('H', 'l'), ('y', 'r')])),
            slime: tex(&sprite_img(&SLIME, 16, &[])), boss: tex(&sprite_img(&BOSS, 16, &[])),
            coin: tex(&sprite_img(&COIN, 8, &[])), potion: tex(&sprite_img(&POTION, 8, &[])),
            heart: tex(&sprite_img(&HEART, 8, &[])), chest: tex(&sprite_img(&CHEST, 16, &[])),
            floor: [floor_tex(0), floor_tex(1), floor_tex(2)], wall: wall_tex(),
        }
    }
}
fn spr(t: &Texture2D, x: f32, y: f32, w: f32, tint: Color, flip: bool, rot: f32) {
    draw_texture_ex(t, x, y, tint, DrawTextureParams { dest_size: Some(vec2(w, w)), flip_x: flip, rotation: rot, ..Default::default() });
}

// ---------------- world ----------------
#[derive(Clone, Copy, PartialEq)]
enum RK { Start, Normal, Treasure, Boss, Shop }
struct Room { g: (i32, i32), doors: [bool; 4], kind: RK, cleared: bool, visited: bool, tiles: Vec<u8>, chest: bool, stock: [u8; 4] }
impl Room {
    fn new(g: (i32, i32)) -> Room {
        let mut tiles = vec![0u8; RW * RH];
        for y in 0..RH { for x in 0..RW { if x == 0 || y == 0 || x == RW - 1 || y == RH - 1 { tiles[y * RW + x] = 1; } } }
        let pts: &[(usize, usize)] = match rand::gen_range(0, 5) {
            0 => &[(4, 3), (14, 3), (4, 7), (14, 7)],
            1 => &[(7, 4), (8, 4), (10, 4), (11, 4), (7, 6), (8, 6), (10, 6), (11, 6)],
            2 => &[(5, 2), (5, 3), (13, 7), (13, 8), (5, 8), (13, 2)],
            3 => &[(3, 5), (15, 5), (6, 3), (12, 7)],
            _ => &[],
        };
        for &(x, y) in pts { tiles[y * RW + x] = 2; }
        Room { g, doors: [false; 4], kind: RK::Normal, cleared: false, visited: false, tiles, chest: false, stock: [0; 4] }
    }
    fn set_kind(&mut self, k: RK) {
        self.kind = k;
        if k != RK::Normal { for t in self.tiles.iter_mut() { if *t == 2 { *t = 0; } } }
        self.cleared = k == RK::Start || k == RK::Treasure || k == RK::Shop;
        self.chest = k == RK::Treasure;
        if k == RK::Shop { self.stock = [3, 1, 1, 1]; }
    }
}
fn door_at(tx: i32, ty: i32) -> Option<usize> {
    let (mx, my) = ((RW / 2) as i32, (RH / 2) as i32);
    if ty == 0 && (tx - mx).abs() <= 1 { Some(0) }
    else if tx == RW as i32 - 1 && (ty - my).abs() <= 1 { Some(1) }
    else if ty == RH as i32 - 1 && (tx - mx).abs() <= 1 { Some(2) }
    else if tx == 0 && (ty - my).abs() <= 1 { Some(3) }
    else { None }
}
fn gen_floor(floor: u32) -> Vec<Room> {
    let n = (6 + floor as usize * 2).min(13);
    let mut rooms = vec![Room::new((3, 3))];
    let mut tries = 0;
    while rooms.len() < n && tries < 800 {
        tries += 1;
        let i = rand::gen_range(0, rooms.len());
        let d = rand::gen_range(0, 4);
        let g = (rooms[i].g.0 + DIRS[d].0, rooms[i].g.1 + DIRS[d].1);
        if g.0 < 0 || g.1 < 0 || g.0 >= GRID || g.1 >= GRID || rooms.iter().any(|r| r.g == g) { continue; }
        let nb = DIRS.iter().filter(|dd| rooms.iter().any(|r| r.g == (g.0 + dd.0, g.1 + dd.1))).count();
        if nb > 1 && rand::gen_range(0, 3) != 0 { continue; }
        let mut r = Room::new(g);
        r.doors[(d + 2) % 4] = true;
        rooms[i].doors[d] = true;
        rooms.push(r);
    }
    // BFS distance
    let mut dist = vec![usize::MAX; rooms.len()];
    dist[0] = 0;
    let mut q = vec![0];
    while let Some(i) = q.pop() {
        for d in 0..4 {
            if !rooms[i].doors[d] { continue; }
            let g = (rooms[i].g.0 + DIRS[d].0, rooms[i].g.1 + DIRS[d].1);
            if let Some(j) = rooms.iter().position(|r| r.g == g) { if dist[j] > dist[i] + 1 { dist[j] = dist[i] + 1; q.push(j); } }
        }
    }
    let leaves: Vec<usize> = (1..rooms.len()).filter(|&i| rooms[i].doors.iter().filter(|d| **d).count() == 1).collect();
    let pool: Vec<usize> = if leaves.is_empty() { (1..rooms.len()).collect() } else { leaves };
    let boss = *pool.iter().max_by_key(|&&i| dist[i]).unwrap();
    rooms[0].set_kind(RK::Start);
    rooms[boss].set_kind(RK::Boss);
    let mut tr = usize::MAX;
    if let Some(&t) = pool.iter().filter(|&&i| i != boss).max_by_key(|&&i| dist[i]) { rooms[t].set_kind(RK::Treasure); tr = t; }
    let cands: Vec<usize> = (1..rooms.len()).filter(|&i| i != boss && i != tr && dist[i] >= 1).collect();
    if !cands.is_empty() { let s = cands[rand::gen_range(0, cands.len())]; rooms[s].set_kind(RK::Shop); }
    rooms
}

// ---------------- entities ----------------
#[derive(Clone, Copy, PartialEq)]
enum EK { Slime, Bat, Skel, Knight, Boss }
struct Enemy { kind: EK, pos: Vec2, vel: Vec2, knock: Vec2, hp: f32, max_hp: f32, dmg: f32, r: f32, t: f32, atk: f32,
    state: u8, dir: Vec2, flash: f32, spawn: f32, phase: u32, bv: u8 }
#[derive(Clone, Copy, PartialEq)]
enum SK { Arrow, Orb, Fire }
struct Shot { pos: Vec2, vel: Vec2, dmg: f32, kind: SK, life: f32 }
#[derive(Clone, Copy, PartialEq)]
enum IK { Coin(u32), Potion, Heart }
struct Item { pos: Vec2, vel: Vec2, kind: IK }
struct FText { pos: Vec2, s: String, col: Color, life: f32 }
struct Particle { p: Vec2, v: Vec2, life: f32, max: f32, size: f32, col: Color }
struct Corpse { kind: EK, bv: u8, pos: Vec2, t: f32, flip: bool }

struct Player { pos: Vec2, vel: Vec2, face: Vec2, hp: f32, max_hp: f32, mp: f32, max_mp: f32, mp_regen: f32, atk: f32,
    crit: f32, vamp: f32, speed: f32, level: u32, xp: f32, xp_need: f32, gold: u32, potions: u32, inv: f32,
    roll_t: f32, roll_cd: f32, roll_cd_max: f32, roll_dir: Vec2, atk_cd: f32, swing: f32, combo: usize, combo_t: f32, walk_t: f32 }

struct Perk { name: &'static str, desc: &'static str }
const PERKS: [Perk; 6] = [
    Perk { name: "Kuch", desc: "Qilich zarari +25%" },
    Perk { name: "Temir teri", desc: "Maks. HP +30" },
    Perk { name: "Chaqqonlik", desc: "Tezlik +12%, sakrash tezroq" },
    Perk { name: "Sehrgar", desc: "Mana +30, tez tiklanadi" },
    Perk { name: "Vampir", desc: "Har zarbada +2 HP" },
    Perk { name: "Kritik", desc: "Kritik zarba +12%" },
];

struct Sfx { swing: Sound, hit: Sound, kill: Sound, coin: Sound, hurt: Sound, fire: Sound, level: Sound, door: Sound, potion: Sound, roar: Sound }
static SFX_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
static MUSIC_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
fn sfx_on() -> bool { SFX_ON.load(std::sync::atomic::Ordering::Relaxed) }
fn music_on() -> bool { MUSIC_ON.load(std::sync::atomic::Ordering::Relaxed) }
fn play(s: &Sound, volume: f32) { if sfx_on() { play_sound(s, PlaySoundParams { looped: false, volume }); } }
const BOSS_NAMES: [&str; 3] = ["SOYA LORDI", "SUYAK QIROLI", "BOTQOQ QIROLI"];
const BOSS_SUB: [&str; 3] = ["Zulmat qal'asining qo'riqchisi", "O'liklar qo'shinining sardori", "Zindon botqog'ining ochko'z hukmdori"];
const SHOP: [(&str, u32); 4] = [("Iksir", 30), ("Yurak +20", 60), ("Qilich +20%", 90), ("Mana +20", 50)];
fn shop_slots() -> [Vec2; 4] { let c = room_center() + vec2(0.0, 10.0); [c + vec2(-48.0, 0.0), c + vec2(-16.0, 0.0), c + vec2(16.0, 0.0), c + vec2(48.0, 0.0)] }
fn ui_btn(r: Rect, label: &str, col: Color) {
    draw_rectangle(r.x, r.y + 5.0, r.w, r.h, Color::new(col.r * 0.45, col.g * 0.45, col.b * 0.45, 1.0));
    draw_rectangle(r.x, r.y, r.w, r.h, col);
    draw_rectangle(r.x, r.y, r.w, 3.0, Color::new(1.0, 1.0, 1.0, 0.35));
    txt(label, r.x + r.w / 2.0 - tw(label, 26.0) / 2.0, r.y + r.h / 2.0 + 9.0, 26.0, rgb(20, 12, 28));
}
fn pause_btn() -> Rect { Rect::new(vw() - 20.0 - GRID as f32 * 14.0 - 66.0, 14.0, 52.0, 52.0) }

#[derive(PartialEq)]
enum State { Title, Play, LevelUp, Dead, Pause }

struct Game {
    state: State, floor: u32, rooms: Vec<Room>, cur: usize, p: Player, enemies: Vec<Enemy>, shots: Vec<Shot>,
    items: Vec<Item>, parts: Vec<Particle>, texts: Vec<FText>, t: f32, over_t: f32, shake: f32, hitstop: f32, fade: f32,
    kills: u32, choices: [usize; 3], touch_mode: bool, stick: Option<(u64, Vec2, Vec2)>, atk_touch: Option<u64>,
    btn: [bool; 4], best_floor: u32, boss_t: f32, shop_in: Option<usize>,
    corpses: Vec<Corpse>, lock_anim: f32, chest_t: f32,
}

fn room_center() -> Vec2 { vec2(RW as f32 * T / 2.0, RH as f32 * T / 2.0) }
fn view() -> (f32, Vec2) {
    let k = ((screen_width() - 40.0 * sc()) / (RW as f32 * T)).min((screen_height() - 120.0 * sc()) / (RH as f32 * T)).floor().max(1.0);
    (k, room_center() - vec2(0.0, 26.0 * sc() / k))
}
fn w2ui(p: Vec2) -> Vec2 { let (k, c) = view(); ((p - c) * k + vec2(screen_width(), screen_height()) / 2.0) / sc() }
fn buttons() -> [(Vec2, f32, &'static str); 4] {
    let w = vw();
    [(vec2(w - 120.0, VH - 120.0), 64.0, "HUJUM"), (vec2(w - 255.0, VH - 78.0), 44.0, "SAKRASH"),
     (vec2(w - 120.0, VH - 265.0), 44.0, "OLOV"), (vec2(w - 255.0, VH - 205.0), 36.0, "IKSIR")]
}

impl Game {
    fn new(touch_mode: bool, best_floor: u32) -> Game {
        let p = Player { pos: room_center(), vel: Vec2::ZERO, face: Vec2::X, hp: 100.0, max_hp: 100.0, mp: 60.0, max_mp: 60.0,
            mp_regen: 6.0, atk: 12.0, crit: 0.08, vamp: 0.0, speed: 78.0, level: 1, xp: 0.0, xp_need: 40.0, gold: 0,
            potions: 2, inv: 0.0, roll_t: 0.0, roll_cd: 0.0, roll_cd_max: 0.6, roll_dir: Vec2::X, atk_cd: 0.0, swing: 0.0,
            combo: 0, combo_t: 0.0, walk_t: 0.0 };
        let mut g = Game { state: State::Title, floor: 1, rooms: gen_floor(1), cur: 0, p, enemies: vec![], shots: vec![],
            items: vec![], parts: vec![], texts: vec![], t: 0.0, over_t: 0.0, shake: 0.0, hitstop: 0.0, fade: 0.6, kills: 0,
            choices: [0, 1, 2], touch_mode, stick: None, atk_touch: None, btn: [false; 4], best_floor, boss_t: 0.0, shop_in: None, corpses: vec![], lock_anim: 0.0, chest_t: -1.0 };
        g.rooms[0].visited = true;
        g
    }

    fn solid(&self, tx: i32, ty: i32) -> bool {
        if tx < 0 || ty < 0 || tx >= RW as i32 || ty >= RH as i32 { return false; }
        let r = &self.rooms[self.cur];
        if let Some(d) = door_at(tx, ty) { if r.doors[d] { return !r.cleared; } }
        r.tiles[ty as usize * RW + tx as usize] != 0
    }
    fn solid_at(&self, p: Vec2) -> bool { self.solid((p.x / T).floor() as i32, (p.y / T).floor() as i32) }
    fn hits(&self, p: Vec2, r: f32) -> bool {
        [vec2(-r, -r), vec2(r, -r), vec2(-r, r), vec2(r, r)].iter().any(|o| self.solid_at(p + *o))
    }
    fn move_body(&self, pos: Vec2, d: Vec2, r: f32, fly: bool) -> Vec2 {
        if fly { return (pos + d).clamp(vec2(T + r, T + r), vec2(RW as f32 * T - T - r, RH as f32 * T - T - r)); }
        let mut p = pos;
        let nx = vec2(p.x + d.x, p.y); if !self.hits(nx, r) { p = nx; }
        let ny = vec2(p.x, p.y + d.y); if !self.hits(ny, r) { p = ny; }
        p
    }

    fn burst(&mut self, p: Vec2, col: Color, n: usize, spd: f32) {
        for _ in 0..n {
            let life = rand::gen_range(0.25, 0.6);
            self.parts.push(Particle { p, v: Vec2::from_angle(rand::gen_range(0.0, TAU)) * rand::gen_range(0.2, 1.0) * spd,
                life, max: life, size: rand::gen_range(1.0, 2.5), col });
        }
    }
    fn text(&mut self, pos: Vec2, s: String, col: Color) { self.texts.push(FText { pos, s, col, life: 0.8 }); }

    fn mk_enemy(&self, kind: EK, pos: Vec2) -> Enemy {
        let (hp, dmg, r) = match kind { EK::Slime => (30.0, 8.0, 6.0), EK::Bat => (18.0, 6.0, 5.0), EK::Skel => (35.0, 10.0, 6.0),
            EK::Knight => (60.0, 15.0, 6.0), EK::Boss => (650.0, 16.0, 14.0) };
        let f = (self.floor - 1) as f32;
        let hp = hp * (1.0 + 0.45 * f);
        Enemy { kind, pos, vel: Vec2::ZERO, knock: Vec2::ZERO, hp, max_hp: hp, dmg: dmg * (1.0 + 0.25 * f), r,
            t: rand::gen_range(0.0, 1.0), atk: rand::gen_range(0.8, 1.8), state: 0, dir: Vec2::X, flash: 0.0, spawn: 0.7, phase: 0, bv: if kind == EK::Boss { ((self.floor - 1) % 3) as u8 } else { 0 } }
    }

    fn enter_room(&mut self, sfx: &Sfx) {
        self.rooms[self.cur].visited = true;
        if self.rooms[self.cur].cleared { return; }
        if self.rooms[self.cur].kind == RK::Boss {
            let mut e = self.mk_enemy(EK::Boss, room_center() - vec2(0.0, 20.0));
            e.spawn = 1.8; e.atk = 1.5;
            self.enemies.push(e);
            self.boss_t = 2.5;
            play(&sfx.roar, 0.8);
            return;
        }
        let n = 3 + self.floor as usize + rand::gen_range(0, 3);
        for _ in 0..n {
            for _ in 0..50 {
                let (tx, ty) = (rand::gen_range(2, RW - 2), rand::gen_range(2, RH - 2));
                let p = vec2(tx as f32 * T + 8.0, ty as f32 * T + 8.0);
                if self.rooms[self.cur].tiles[ty * RW + tx] == 0 && p.distance(self.p.pos) > 70.0 {
                    let roll = rand::gen_range(0.0, 1.0);
                    let kind = if self.floor >= 2 && roll < 0.2 { EK::Knight } else if roll < 0.45 { EK::Slime } else if roll < 0.72 { EK::Bat } else { EK::Skel };
                    let e = self.mk_enemy(kind, p);
                    self.enemies.push(e);
                    break;
                }
            }
        }
        play(&sfx.door, 0.6);
    }

    fn next_floor(&mut self, sfx: &Sfx) {
        self.floor += 1;
        self.best_floor = self.best_floor.max(self.floor);
        self.rooms = gen_floor(self.floor);
        self.cur = 0;
        self.p.pos = room_center();
        self.p.hp = (self.p.hp + self.p.max_hp * 0.3).min(self.p.max_hp);
        self.enemies.clear(); self.shots.clear(); self.items.clear();
        self.fade = 0.8;
        let pos = self.p.pos;
        self.text(pos - vec2(0.0, 20.0), format!("QAVAT {}", self.floor), rgb(180, 140, 255));
        self.enter_room(sfx);
        play(&sfx.level, 0.6);
    }

    fn read_touches(&mut self) {
        let bts = buttons();
        for t in touches() {
            let p = t.position / sc();
            match t.phase {
                TouchPhase::Started => {
                    self.touch_mode = true;
                    if let Some(i) = bts.iter().position(|b| p.distance(b.0) < b.1 + 12.0) {
                        if i == 0 { self.atk_touch = Some(t.id); } else { self.btn[i] = true; }
                    } else if p.x < vw() * 0.5 && self.stick.is_none() { self.stick = Some((t.id, p, p)); }
                }
                TouchPhase::Moved | TouchPhase::Stationary => { if let Some(s) = &mut self.stick { if s.0 == t.id { s.2 = p; } } }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if self.stick.map_or(false, |s| s.0 == t.id) { self.stick = None; }
                    if self.atk_touch == Some(t.id) { self.atk_touch = None; }
                }
            }
        }
    }

    fn nearest_enemy(&self, range: f32) -> Option<Vec2> {
        self.enemies.iter().filter(|e| e.spawn <= 0.0 && e.pos.distance(self.p.pos) < range).map(|e| e.pos)
            .min_by(|a, b| a.distance(self.p.pos).total_cmp(&b.distance(self.p.pos)))
    }

    fn hit_enemy(&mut self, i: usize, dmg: f32, kb: Vec2) {
        let crit = rand::gen_range(0.0, 1.0) < self.p.crit;
        let d = (if crit { dmg * 2.0 } else { dmg }) * rand::gen_range(0.9, 1.1);
        let e = &mut self.enemies[i];
        e.hp -= d; e.flash = 0.12;
        e.knock += if e.kind == EK::Boss { kb * 0.12 } else { kb };
        let pos = e.pos;
        self.text(pos - vec2(0.0, 16.0), format!("{}{}", d.round() as i32, if crit { "!" } else { "" }), if crit { rgb(255, 220, 80) } else { WHITE });
        self.burst(pos - vec2(0.0, 6.0), rgb(255, 235, 210), 6, 70.0);
        if self.p.vamp > 0.0 { self.p.hp = (self.p.hp + self.p.vamp).min(self.p.max_hp); }
    }

    fn hurt(&mut self, dmg: f32, from: Vec2, sfx: &Sfx) {
        if self.p.inv > 0.0 || self.p.roll_t > 0.0 || self.state != State::Play { return; }
        self.p.hp -= dmg; self.p.inv = 1.1; self.shake = 6.0; self.hitstop = 0.06;
        self.p.vel += (self.p.pos - from).normalize_or(Vec2::Y) * 160.0;
        let pos = self.p.pos;
        self.text(pos - vec2(0.0, 18.0), format!("-{}", dmg.round() as i32), rgb(255, 80, 80));
        self.burst(pos - vec2(0.0, 6.0), rgb(255, 60, 60), 14, 90.0);
        play(&sfx.hurt, 0.7);
        if self.p.hp <= 0.0 {
            self.p.hp = 0.0;
            self.burst(pos, rgb(120, 170, 255), 50, 140.0);
            self.state = State::Dead; self.over_t = 0.0;
        }
    }

    fn update(&mut self, dt: f32, sfx: &Sfx) {
        self.t += dt;
        let p = &mut self.p;
        p.inv -= dt; p.roll_t -= dt; p.roll_cd -= dt; p.atk_cd -= dt; p.swing -= dt; p.combo_t -= dt;
        p.mp = (p.mp + p.mp_regen * dt).min(p.max_mp);
        self.shake = (self.shake - dt * 30.0).max(0.0);
        self.fade = (self.fade - dt * 2.5).max(0.0);
        self.boss_t -= dt;

        // ---- input ----
        let mut dir = Vec2::ZERO;
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { dir.y -= 1.0 }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { dir.y += 1.0 }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { dir.x -= 1.0 }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { dir.x += 1.0 }
        let mut dir = dir.normalize_or_zero();
        if let Some((_, o, c)) = self.stick { let d = c - o; if d.length() > 8.0 { dir = (d / 60.0).clamp_length_max(1.0); } }
        let mouse_atk = is_mouse_button_down(MouseButton::Left);
        let atk_in = is_key_down(KeyCode::J) || mouse_atk || self.atk_touch.is_some();
        let roll_in = is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::LeftShift) || self.btn[1];
        let fire_in = is_key_pressed(KeyCode::K) || is_mouse_button_pressed(MouseButton::Right) || self.btn[2];
        let pot_in = is_key_pressed(KeyCode::Q) || self.btn[3];
        self.btn = [false; 4];
        let (k, cam) = view();
        let mouse_world = cam + (Vec2::from(mouse_position()) - vec2(screen_width(), screen_height()) / 2.0) / k;
        if dir != Vec2::ZERO && self.p.swing <= 0.0 { self.p.face = dir.normalize(); }

        if pot_in && self.p.potions > 0 && self.p.hp < self.p.max_hp {
            self.p.potions -= 1;
            let heal = self.p.max_hp * 0.5;
            self.p.hp = (self.p.hp + heal).min(self.p.max_hp);
            let pos = self.p.pos;
            self.text(pos - vec2(0.0, 18.0), format!("+{}", heal as i32), rgb(90, 255, 120));
            self.burst(pos - vec2(0.0, 6.0), rgb(90, 255, 120), 16, 60.0);
            play(&sfx.potion, 0.7);
        }
        if roll_in && self.p.roll_cd <= 0.0 {
            self.p.roll_t = 0.28; self.p.roll_cd = self.p.roll_cd_max;
            self.p.roll_dir = if dir != Vec2::ZERO { dir.normalize() } else { self.p.face };
            play(&sfx.swing, 0.3);
        }
        if fire_in && self.p.mp >= 20.0 {
            self.p.mp -= 20.0;
            let aim = if !self.touch_mode && is_mouse_button_pressed(MouseButton::Right) { (mouse_world - self.p.pos).normalize_or(self.p.face) }
                else { self.nearest_enemy(160.0).map_or(self.p.face, |e| (e - self.p.pos).normalize_or(self.p.face)) };
            self.p.face = aim;
            self.shots.push(Shot { pos: self.p.pos - vec2(0.0, 6.0) + aim * 8.0, vel: aim * 200.0, dmg: self.p.atk * 2.2, kind: SK::Fire, life: 1.5 });
            play(&sfx.fire, 0.6);
        }
        if atk_in && self.p.atk_cd <= 0.0 && self.p.roll_t <= 0.0 {
            let p = &mut self.p;
            p.combo = if p.combo_t > 0.0 { (p.combo + 1) % 3 } else { 0 };
            p.combo_t = 0.55; p.atk_cd = if p.combo == 2 { 0.45 } else { 0.26 }; p.swing = 0.18;
            let face = if mouse_atk && !self.touch_mode { (mouse_world - self.p.pos).normalize_or(self.p.face) }
                else { self.nearest_enemy(48.0).map_or(self.p.face, |e| (e - self.p.pos).normalize_or(self.p.face)) };
            self.p.face = face;
            self.p.vel += face * 70.0;
            let mul = [1.0, 1.1, 1.8][self.p.combo];
            let range = if self.p.combo == 2 { 30.0 } else { 26.0 };
            let mut any = false;
            for i in 0..self.enemies.len() {
                let e = &self.enemies[i];
                if e.spawn > 0.0 { continue; }
                let d = e.pos - self.p.pos;
                if d.length() < range + e.r && (d.length() < e.r + 6.0 || face.dot(d.normalize()) > 0.25) {
                    let kb = d.normalize_or(face) * if self.p.combo == 2 { 260.0 } else { 150.0 };
                    let dmg = self.p.atk * mul;
                    self.hit_enemy(i, dmg, kb);
                    any = true;
                }
            }
            play(&sfx.swing, 0.35);
            if any { play(&sfx.hit, 0.6); self.hitstop = if self.p.combo == 2 { 0.07 } else { 0.035 }; self.shake = self.shake.max(2.5); }
        }

        // ---- player movement ----
        if self.p.roll_t > 0.0 {
            self.p.vel = self.p.roll_dir * 210.0;
            if (self.t * 30.0) as i32 % 2 == 0 { let pos = self.p.pos; self.parts.push(Particle { p: pos, v: Vec2::ZERO, life: 0.25, max: 0.25, size: 3.0, col: rgb(120, 140, 200) }); }
        } else {
            let spd = self.p.speed * if self.p.swing > 0.0 { 0.35 } else { 1.0 };
            self.p.vel = self.p.vel.lerp(dir * spd, (dt * 14.0).min(1.0));
        }
        if self.p.vel.length() > 10.0 && self.p.roll_t <= 0.0 {
            let before = (self.p.walk_t * 9.0) as i32;
            self.p.walk_t += dt;
            if (self.p.walk_t * 9.0) as i32 != before && before % 2 == 0 {
                let fp = self.p.pos + vec2(rand::gen_range(-2.0, 2.0), 1.0);
                for _ in 0..2 { self.parts.push(Particle { p: fp, v: vec2(rand::gen_range(-12.0, 12.0), rand::gen_range(-10.0, -2.0)), life: 0.35, max: 0.35, size: 1.5, col: Color::new(0.6, 0.55, 0.65, 0.7) }); }
            }
        }
        let lt = if self.rooms[self.cur].cleared { 0.0 } else { 1.0 };
        self.lock_anim += (lt - self.lock_anim) * (dt * 7.0).min(1.0);
        for c in self.corpses.iter_mut() { c.t += dt; }
        self.corpses.retain(|c| c.t < 0.4);
        if self.chest_t >= 0.0 {
            self.chest_t += dt;
            if self.chest_t < 1.2 { let c = room_center() - vec2(0.0, 6.0); self.parts.push(Particle { p: c + vec2(rand::gen_range(-5.0, 5.0), 0.0), v: vec2(rand::gen_range(-15.0, 15.0), rand::gen_range(-45.0, -20.0)), life: 0.6, max: 0.6, size: 1.5, col: rgb(255, 215, 90) }); }
        }
        self.p.pos = self.move_body(self.p.pos, self.p.vel * dt, 5.0, false);

        // ---- enemies ----
        let pp = self.p.pos;
        let mut es = std::mem::take(&mut self.enemies);
        let mut new_shots = vec![];
        let mut summons = vec![];
        let mut contact: Option<(f32, Vec2)> = None;
        let mut roar = false;
        for e in es.iter_mut() {
            e.t += dt; e.flash -= dt; e.atk -= dt;
            if e.spawn > 0.0 { e.spawn -= dt; continue; }
            let tv = pp - e.pos; let dist = tv.length(); let to = tv.normalize_or_zero();
            let mut fly = false;
            match e.kind {
                EK::Slime => { if e.t > 0.9 { e.t = 0.0; e.vel = to * 150.0; } e.vel *= (-dt * 5.0).exp(); }
                EK::Bat => { fly = true; let want = to * 70.0 + to.perp() * (e.t * 5.0).sin() * 90.0; e.vel = e.vel.lerp(want, (dt * 3.0).min(1.0)); }
                EK::Skel => {
                    let want = if dist < 70.0 { -to } else if dist > 120.0 { to } else { to.perp() * (e.t * 0.7).sin().signum() };
                    e.vel = e.vel.lerp(want * 45.0, (dt * 4.0).min(1.0));
                    if e.atk <= 0.0 && dist < 220.0 {
                        e.atk = rand::gen_range(1.5, 2.3);
                        new_shots.push(Shot { pos: e.pos - vec2(0.0, 7.0), vel: to * 150.0, dmg: e.dmg, kind: SK::Arrow, life: 2.5 });
                    }
                }
                EK::Knight => match e.state {
                    0 => { e.vel = e.vel.lerp(to * 38.0, (dt * 4.0).min(1.0)); if dist < 110.0 && e.atk <= 0.0 { e.state = 1; e.t = 0.0; e.dir = to; } }
                    1 => { e.vel *= 0.8; if e.t > 0.5 { e.state = 2; e.t = 0.0; } }
                    _ => { e.vel = e.dir * 240.0; if e.t > 0.4 { e.state = 0; e.atk = 1.6; } }
                },
                EK::Boss if e.bv == 0 => match e.state {
                    0 => {
                        e.vel = e.vel.lerp(to * 30.0, (dt * 2.0).min(1.0));
                        if e.atk <= 0.0 {
                            e.phase += 1; e.t = 0.0;
                            let src = e.pos - vec2(0.0, 18.0);
                            match e.phase % 4 {
                                1 => { e.state = 1; e.dir = to; }
                                3 => {
                                    for k in -2..=2 { new_shots.push(Shot { pos: src, vel: Vec2::from_angle(to.to_angle() + k as f32 * 0.2) * 130.0, dmg: e.dmg * 0.8, kind: SK::Orb, life: 4.0 }); }
                                    e.atk = 1.6;
                                    if e.hp < e.max_hp * 0.5 { for s in [-1.0, 1.0] { summons.push((EK::Bat, e.pos + vec2(s * 24.0, 0.0))); } }
                                }
                                _ => {
                                    let n = if e.hp < e.max_hp * 0.5 { 18 } else { 12 };
                                    let off = rand::gen_range(0.0, TAU);
                                    for k in 0..n { new_shots.push(Shot { pos: src, vel: Vec2::from_angle(off + k as f32 / n as f32 * TAU) * 85.0, dmg: e.dmg * 0.8, kind: SK::Orb, life: 4.0 }); }
                                    e.atk = 2.0;
                                }
                            }
                        }
                    }
                    1 => { e.vel *= 0.85; if e.t > 0.6 { e.state = 2; e.t = 0.0; roar = true; } }
                    _ => { e.vel = e.dir * 260.0; if e.t > 0.5 { e.state = 0; e.atk = 1.2; } }
                },
                EK::Boss if e.bv == 1 => match e.state {
                    // Suyak Qiroli: arrow fans, raises skeletons, charges
                    0 => {
                        e.vel = e.vel.lerp(to * 28.0, (dt * 2.0).min(1.0));
                        if e.atk <= 0.0 {
                            e.phase += 1; e.t = 0.0;
                            let src = e.pos - vec2(0.0, 14.0);
                            match e.phase % 3 {
                                1 => { let n = if e.hp < e.max_hp * 0.5 { 9 } else { 7 }; for k in 0..n { let a = to.to_angle() + (k as f32 / (n - 1) as f32 - 0.5) * 1.1; new_shots.push(Shot { pos: src, vel: Vec2::from_angle(a) * 135.0, dmg: e.dmg * 0.7, kind: SK::Arrow, life: 3.0 }); } e.atk = 1.7; }
                                2 => { for s in [-1.0, 1.0] { summons.push((EK::Skel, e.pos + vec2(s * 28.0, 6.0))); } e.atk = 2.2; roar = true; }
                                _ => { e.state = 1; e.dir = to; }
                            }
                        }
                    }
                    1 => { e.vel *= 0.85; if e.t > 0.7 { e.state = 2; e.t = 0.0; roar = true; } }
                    _ => { e.vel = e.dir * 240.0; if e.t > 0.55 { e.state = 0; e.atk = 1.3; } }
                },
                EK::Boss => {
                    // Botqoq Qiroli: big hops, shockwave rings on landing, splits when hurt
                    if e.state == 0 {
                        e.vel *= (-dt * 6.0).exp();
                        if e.t > 1.25 { e.t = 0.0; e.state = 1; e.vel = to * (130.0 + (1.0 - e.hp / e.max_hp) * 70.0); }
                    } else if e.t > 0.5 {
                        e.state = 0; e.t = 0.0; e.vel = Vec2::ZERO; roar = true;
                        let n = if e.hp < e.max_hp * 0.5 { 14 } else { 10 };
                        let off = rand::gen_range(0.0, TAU);
                        for k in 0..n { new_shots.push(Shot { pos: e.pos - vec2(0.0, 4.0), vel: Vec2::from_angle(off + k as f32 / n as f32 * TAU) * 75.0, dmg: e.dmg * 0.7, kind: SK::Orb, life: 3.0 }); }
                    }
                    if (e.phase == 0 && e.hp < e.max_hp * 0.6) || (e.phase == 1 && e.hp < e.max_hp * 0.3) {
                        e.phase += 1;
                        for k in 0..3 { summons.push((EK::Slime, e.pos + Vec2::from_angle(k as f32 * TAU / 3.0) * 22.0)); }
                    }
                }
            }
            e.knock *= (-dt * 8.0).exp();
            let r = if e.kind == EK::Boss { 9.0 } else { 5.0 };
            e.pos = self.move_body(e.pos, (e.vel + e.knock) * dt, r, fly);
            if dist < e.r + 5.0 { contact = Some((if e.kind == EK::Boss { e.dmg * 0.7 } else { e.dmg }, e.pos)); }
        }
        // separation
        for i in 0..es.len() {
            for j in (i + 1)..es.len() {
                let d = es[j].pos - es[i].pos; let l = d.length(); let min = es[i].r + es[j].r;
                if l < min && l > 0.01 {
                    let push = d / l * (min - l) * 0.5;
                    es[i].pos = self.move_body(es[i].pos, -push, 5.0, es[i].kind == EK::Bat);
                    es[j].pos = self.move_body(es[j].pos, push, 5.0, es[j].kind == EK::Bat);
                }
            }
        }
        self.enemies = es;
        self.shots.extend(new_shots);
        for (k, s) in summons { let mut e = self.mk_enemy(k, s); if k == EK::Slime { e.hp *= 0.6; e.max_hp = e.hp; } self.enemies.push(e); }
        if roar { play(&sfx.roar, 0.4); self.shake = 4.0; }
        if let Some((d, from)) = contact { self.hurt(d, from, sfx); }

        // ---- shots ----
        for s in &mut self.shots { s.pos += s.vel * dt; s.life -= dt; }
        let walls: Vec<bool> = self.shots.iter().map(|s| self.solid_at(s.pos)).collect();
        let mut booms = vec![];
        let mut player_hits = vec![];
        for (i, s) in self.shots.iter_mut().enumerate() {
            if s.life <= 0.0 { continue; }
            if s.kind == SK::Fire {
                if walls[i] || self.enemies.iter().any(|e| e.spawn <= 0.0 && e.pos.distance(s.pos + vec2(0.0, 6.0)) < e.r + 5.0) { booms.push((s.pos, s.dmg)); s.life = 0.0; }
            } else {
                if walls[i] { s.life = 0.0; continue; }
                if s.pos.distance(pp - vec2(0.0, 6.0)) < 7.0 { player_hits.push((s.dmg, s.pos)); s.life = 0.0; }
            }
        }
        for (d, from) in player_hits { self.hurt(d, from, sfx); }
        for (bp, dmg) in booms {
            self.burst(bp, rgb(255, 140, 40), 26, 120.0);
            self.burst(bp, rgb(255, 230, 120), 10, 60.0);
            self.shake = self.shake.max(4.0);
            play(&sfx.kill, 0.5);
            for i in 0..self.enemies.len() {
                let d = self.enemies[i].pos - (bp + vec2(0.0, 6.0));
                if self.enemies[i].spawn <= 0.0 && d.length() < 32.0 { self.hit_enemy(i, dmg, d.normalize_or(Vec2::X) * 200.0); }
            }
        }
        self.shots.retain(|s| s.life > 0.0);

        // ---- deaths ----
        let mut dead = vec![];
        let px = self.p.pos.x;
        for e in self.enemies.iter().filter(|e| e.hp <= 0.0) { self.corpses.push(Corpse { kind: e.kind, bv: e.bv, pos: e.pos, t: 0.0, flip: px < e.pos.x }); }
        self.enemies.retain(|e| if e.hp <= 0.0 { dead.push((e.kind, e.pos)); false } else { true });
        for (kind, pos) in dead {
            self.kills += 1;
            let col = match kind { EK::Slime => rgb(100, 210, 100), EK::Bat => rgb(150, 80, 200), EK::Skel => rgb(236, 232, 220), EK::Knight => rgb(200, 50, 50), EK::Boss => rgb(150, 80, 200) };
            self.burst(pos - vec2(0.0, 6.0), col, if kind == EK::Boss { 120 } else { 20 }, if kind == EK::Boss { 200.0 } else { 100.0 });
            self.shake = self.shake.max(if kind == EK::Boss { 14.0 } else { 3.0 });
            play(&sfx.kill, 0.55);
            let f = 1.0 + 0.2 * (self.floor - 1) as f32;
            let xp = match kind { EK::Slime => 10.0, EK::Bat => 8.0, EK::Skel => 15.0, EK::Knight => 25.0, EK::Boss => 200.0 } * f;
            self.p.xp += xp;
            let coins = if kind == EK::Boss { 14 } else { rand::gen_range(1, 3 + self.floor as usize) };
            for _ in 0..coins { self.items.push(Item { pos, vel: Vec2::from_angle(rand::gen_range(0.0, TAU)) * rand::gen_range(30.0, 90.0), kind: IK::Coin(self.floor) }); }
            let roll = rand::gen_range(0.0, 1.0);
            if kind == EK::Boss { for _ in 0..2 { self.items.push(Item { pos, vel: Vec2::from_angle(rand::gen_range(0.0, TAU)) * 60.0, kind: IK::Potion }); } }
            else if roll < 0.07 { self.items.push(Item { pos, vel: Vec2::ZERO, kind: IK::Potion }); }
            else if roll < 0.2 { self.items.push(Item { pos, vel: Vec2::ZERO, kind: IK::Heart }); }
        }
        if !self.rooms[self.cur].cleared && self.enemies.is_empty() {
            self.rooms[self.cur].cleared = true;
            play(&sfx.door, 0.8);
            let pos = self.p.pos;
            let msg = if self.rooms[self.cur].kind == RK::Boss { format!("{} YENGILDI!", BOSS_NAMES[((self.floor - 1) % 3) as usize]) } else { "Eshiklar ochildi".to_string() };
            self.text(pos - vec2(0.0, 24.0), msg.to_string(), rgb(180, 200, 255));
        }

        // ---- items ----
        let mut got = vec![];
        for (i, it) in self.items.iter_mut().enumerate() {
            it.vel *= (-dt * 5.0).exp();
            let d = pp - it.pos;
            if d.length() < 40.0 { it.vel = it.vel.lerp(d.normalize_or_zero() * 220.0, (dt * 8.0).min(1.0)); }
            it.pos += it.vel * dt;
            if d.length() < 9.0 { got.push(i); }
        }
        for &i in got.iter().rev() {
            let it = self.items.remove(i);
            match it.kind {
                IK::Coin(v) => { self.p.gold += v; play(&sfx.coin, 0.3); }
                IK::Potion => { self.p.potions += 1; self.text(pp - vec2(0.0, 18.0), "+Iksir".into(), rgb(255, 120, 120)); play(&sfx.potion, 0.5); }
                IK::Heart => { self.p.hp = (self.p.hp + 20.0).min(self.p.max_hp); self.text(pp - vec2(0.0, 18.0), "+20".into(), rgb(90, 255, 120)); play(&sfx.coin, 0.4); }
            }
        }

        // ---- shop ----
        if self.rooms[self.cur].kind == RK::Shop {
            let slot = shop_slots().iter().position(|s| s.distance(pp) < 9.0);
            if slot.is_some() && slot != self.shop_in {
                let i = slot.unwrap();
                let price = (SHOP[i].1 as f32 * (1.0 + 0.25 * (self.floor - 1) as f32)) as u32;
                let sp = shop_slots()[i] - vec2(0.0, 14.0);
                if self.rooms[self.cur].stock[i] == 0 { self.text(sp, "Tugadi".into(), rgb(200, 190, 210)); }
                else if self.p.gold < price { self.text(sp, "Oltin yetmaydi".into(), rgb(255, 120, 120)); }
                else {
                    self.p.gold -= price; self.rooms[self.cur].stock[i] -= 1;
                    match i { 0 => self.p.potions += 1, 1 => { self.p.max_hp += 20.0; self.p.hp = self.p.max_hp; } 2 => self.p.atk *= 1.2, _ => { self.p.max_mp += 20.0; self.p.mp = self.p.max_mp; } }
                    self.text(sp, format!("{}!", SHOP[i].0), rgb(255, 215, 90));
                    self.burst(shop_slots()[i], rgb(255, 215, 90), 18, 60.0);
                    play(&sfx.level, 0.5);
                }
            }
            self.shop_in = slot;
        }

        // ---- chest & portal ----
        let c = room_center();
        if self.rooms[self.cur].chest && pp.distance(c) < 16.0 {
            self.rooms[self.cur].chest = false; self.chest_t = 0.0;
            self.p.atk *= 1.3; self.p.potions += 1;
            for _ in 0..8 { self.items.push(Item { pos: c, vel: Vec2::from_angle(rand::gen_range(0.0, TAU)) * 80.0, kind: IK::Coin(self.floor) }); }
            self.text(c - vec2(0.0, 26.0), "QILICH KUCHAYDI! +30%".into(), rgb(255, 215, 90));
            self.burst(c, rgb(255, 215, 90), 30, 90.0);
            play(&sfx.level, 0.6);
        }
        if self.rooms[self.cur].kind == RK::Boss && self.rooms[self.cur].cleared && pp.distance(c) < 12.0 { self.next_floor(sfx); return; }

        // ---- particles, texts ----
        for p in &mut self.parts { p.p += p.v * dt; p.v *= 1.0 - dt * 4.0; p.life -= dt; }
        self.parts.retain(|p| p.life > 0.0);
        for t in &mut self.texts { t.pos.y -= dt * 22.0; t.life -= dt; }
        self.texts.retain(|t| t.life > 0.0);

        // ---- level up ----
        if self.p.xp >= self.p.xp_need {
            self.p.xp -= self.p.xp_need; self.p.xp_need *= 1.5; self.p.level += 1;
            self.p.max_hp += 8.0; self.p.hp = self.p.max_hp; self.p.atk += 2.0;
            let mut ids: Vec<usize> = (0..PERKS.len()).collect();
            for i in 0..3 { let j = rand::gen_range(i, ids.len()); ids.swap(i, j); self.choices[i] = ids[i]; }
            self.state = State::LevelUp; self.stick = None; self.atk_touch = None;
            play(&sfx.level, 0.7);
        }

        // ---- room exit ----
        let (w, h) = (RW as f32 * T, RH as f32 * T);
        let pos = self.p.pos;
        let d = if pos.y < 4.0 { 0 } else if pos.x > w - 4.0 { 1 } else if pos.y > h - 4.0 { 2 } else if pos.x < 4.0 { 3 } else { return };
        let g = self.rooms[self.cur].g;
        let ng = (g.0 + DIRS[d].0, g.1 + DIRS[d].1);
        if let Some(i) = self.rooms.iter().position(|r| r.g == ng) {
            self.cur = i; self.shots.clear(); self.items.clear();
            self.p.pos = match d { 0 => vec2(pos.x, h - T - 8.0), 1 => vec2(T + 8.0, pos.y), 2 => vec2(pos.x, T + 8.0), _ => vec2(w - T - 8.0, pos.y) };
            self.fade = 0.4;
            self.enter_room(sfx);
        }
    }

    fn apply_perk(&mut self, id: usize) {
        let p = &mut self.p;
        match id {
            0 => p.atk *= 1.25, 1 => { p.max_hp += 30.0; p.hp += 30.0; }
            2 => { p.speed *= 1.12; p.roll_cd_max *= 0.8; } 3 => { p.max_mp += 30.0; p.mp_regen *= 1.4; }
            4 => p.vamp += 2.0, _ => p.crit += 0.12,
        }
    }

    // ---------------- drawing ----------------
    fn draw_world(&self, art: &Art) {
        let (k, cam) = view();
        let sh = vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)) * self.shake * 0.4;
        set_camera(&Camera2D { target: cam + sh, zoom: vec2(2.0 * k / screen_width(), 2.0 * k / screen_height()), ..Default::default() });
        let room = &self.rooms[self.cur];
        let locked = !room.cleared;
        let fs = T + 0.05;
        for ty in 0..RH {
            for tx in 0..RW {
                let (x, y) = (tx as f32 * T, ty as f32 * T);
                let door = door_at(tx as i32, ty as i32).filter(|d| room.doors[*d]);
                let t = room.tiles[ty * RW + tx];
                if t == 0 || door.is_some() {
                    spr(&art.floor[(hash(tx as u32, ty as u32, self.cur as u32 + self.floor * 31) % 3) as usize], x, y, fs, WHITE, false, 0.0);
                    if door.is_some() && (locked || self.lock_anim > 0.02) { let bh = T * self.lock_anim; for i in 0..4 { draw_rectangle(x + 1.0 + i as f32 * 4.0, y, 2.0, bh, rgb(110, 105, 125)); draw_rectangle(x + 1.0 + i as f32 * 4.0, y + bh - 1.5, 2.0, 1.5, rgb(170, 165, 185)); } }
                } else {
                    spr(&art.wall, x, y, fs, if t == 2 { rgb(230, 220, 255) } else { WHITE }, false, 0.0);
                    if t == 2 { draw_rectangle(x, y + T, T, 3.0, Color::new(0.0, 0.0, 0.0, 0.35)); }
                }
            }
        }
        // wall shadow under top wall
        draw_rectangle(T, T, (RW - 2) as f32 * T, 4.0, Color::new(0.0, 0.0, 0.0, 0.35));
        let torches = [vec2(4.5 * T, 0.7 * T), vec2((RW as f32 - 4.5) * T, 0.7 * T)];
        for tp in torches {
            draw_rectangle(tp.x - 1.0, tp.y, 2.0, 6.0, rgb(90, 60, 40));
            draw_rectangle(tp.x - 2.0, tp.y - 0.5, 4.0, 1.5, rgb(70, 50, 40));
            for k in 0..4 {
                let hgt = 2.0 + ((self.t * 19.0 + k as f32 * 1.7 + tp.x).sin() * 0.5 + 0.5) * (4.0 - k as f32 * 0.6);
                let wdt = 4.0 - k as f32;
                let col = [rgb(190, 50, 30), rgb(255, 130, 40), rgb(255, 200, 80), rgb(255, 245, 190)][k];
                draw_rectangle(tp.x - wdt / 2.0 + ((self.t * 7.0 + k as f32).sin() * 0.4), tp.y - hgt, wdt, hgt, col);
            }
            for k in 0..2 { let e = (self.t * 1.3 + tp.x * 0.1 + k as f32 * 0.5).fract(); draw_rectangle(tp.x + (e * 9.0 + tp.x + k as f32).sin() * 2.0, tp.y - 4.0 - e * 10.0, 1.0, 1.0, Color::new(1.0, 0.7, 0.3, 1.0 - e)); }
        }
        let c = room_center();
        if room.kind == RK::Treasure {
            if room.chest {
                let wob = if (self.t * 0.8).fract() > 0.9 { ((self.t * 40.0).sin()) * 0.6 } else { 0.0 };
                spr(&art.chest, c.x - 8.0 + wob, c.y - 12.0, 16.0, WHITE, false, 0.0);
                let g = (self.t * 1.7).fract();
                if g < 0.3 { draw_rectangle(c.x + 3.0, c.y - 8.0 - g * 6.0, 1.0, 1.0, Color::new(1.0, 1.0, 0.9, 1.0 - g * 3.0)); }
            } else {
                spr(&art.chest, c.x - 8.0, c.y - 12.0, 16.0, rgb(150, 135, 140), false, 0.0);
                let lift = (self.chest_t.max(0.0) * 30.0).min(6.0);
                draw_rectangle(c.x - 6.0, c.y - 8.0, 12.0, 2.0, rgb(255, 210, 90));
                draw_rectangle(c.x - 7.0, c.y - 10.0 - lift, 14.0, 3.0, rgb(82, 48, 28));
                draw_rectangle(c.x - 7.0, c.y - 10.0 - lift, 14.0, 1.0, rgb(122, 74, 42));
            }
        }
        if room.kind == RK::Shop {
            draw_rectangle(c.x - 66.0, c.y + 2.0, 132.0, 18.0, rgb(120, 34, 44));
            draw_rectangle_lines(c.x - 66.0, c.y + 2.0, 132.0, 18.0, 1.0, rgb(230, 180, 70));
            spr(&art.merchant, c.x - 8.0, c.y - 40.0 + (self.t * 2.0).sin(), 16.0, WHITE, false, 0.0);
            draw_rectangle(c.x - 14.0, c.y - 26.0, 28.0, 3.0, rgb(150, 100, 60));
            for (i, sp) in shop_slots().iter().enumerate() {
                let a = if room.stock[i] > 0 { 1.0 } else { 0.3 };
                draw_rectangle(sp.x - 5.0, sp.y + 1.0, 10.0, 5.0, Color::new(0.45, 0.42, 0.5, a));
                let bob = (self.t * 3.0 + i as f32).sin() * 1.2 - 6.0;
                let tint = Color::new(1.0, 1.0, 1.0, a);
                match i {
                    0 => spr(&art.potion, sp.x - 4.0, sp.y + bob - 4.0, 8.0, tint, false, 0.0),
                    1 => spr(&art.heart, sp.x - 4.0, sp.y + bob - 4.0, 8.0, tint, false, 0.0),
                    2 => { draw_line(sp.x - 4.0, sp.y + bob + 4.0, sp.x + 4.0, sp.y + bob - 4.0, 2.0, Color::new(0.86, 0.9, 0.95, a)); draw_line(sp.x - 3.0, sp.y + bob + 1.0, sp.x - 1.0, sp.y + bob + 3.0, 2.0, Color::new(1.0, 0.84, 0.35, a)); }
                    _ => { draw_circle(sp.x, sp.y + bob, 3.5, Color::new(0.3, 0.5, 1.0, a)); draw_circle(sp.x - 1.0, sp.y + bob - 1.0, 1.2, Color::new(0.8, 0.9, 1.0, a)); }
                }
            }
        }
        let portal = room.kind == RK::Boss && room.cleared;
        if portal {
            for i in 0..4 { draw_circle_lines(c.x, c.y, 4.0 + i as f32 * 3.0 + (self.t * 4.0 + i as f32).sin(), 1.2, Color::new(0.6, 0.4, 1.0, 0.8 - i as f32 * 0.15)); }
        }
        for it in &self.items {
            let bob = (self.t * 6.0 + it.pos.x).sin() * 1.0;
            match it.kind {
                IK::Coin(_) => { let w = 8.0 * (self.t * 7.0 + it.pos.x).cos().abs().max(0.18); spr2(&art.coin, it.pos.x - w / 2.0, it.pos.y - 8.0 + bob, w, 8.0, WHITE, false); }
                IK::Heart => { let k = 1.0 + ((self.t * 6.0).sin().max(0.0)) * 0.25; spr2(&art.heart, it.pos.x - 4.0 * k, it.pos.y - 4.0 - 4.0 * k + bob, 8.0 * k, 8.0 * k, WHITE, false); }
                IK::Potion => spr(&art.potion, it.pos.x - 4.0, it.pos.y - 8.0 + bob, 8.0, WHITE, false, 0.0),
            }
        }

        // entities sorted by y
        let mut order: Vec<(f32, usize)> = self.enemies.iter().enumerate().map(|(i, e)| (e.pos.y, i)).collect();
        if self.state != State::Dead { order.push((self.p.pos.y, usize::MAX)); }
        order.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, i) in order {
            if i == usize::MAX { self.draw_player(art); } else { self.draw_enemy(&self.enemies[i], art); }
        }
        for s in &self.shots {
            match s.kind {
                SK::Arrow => { let tail = s.pos - s.vel.normalize() * 7.0; draw_line(tail.x, tail.y, s.pos.x, s.pos.y, 1.5, rgb(236, 232, 220)); }
                SK::Orb => draw_circle(s.pos.x, s.pos.y, 3.0, rgb(200, 120, 255)),
                SK::Fire => draw_circle(s.pos.x, s.pos.y, 3.5, rgb(255, 170, 60)),
            }
        }
        for c in &self.corpses {
            let k = c.t / 0.4;
            let big = c.kind == EK::Boss;
            let sz = if big { 48.0 } else { 16.0 } * (1.0 + k * 0.45);
            let t = match (c.kind, c.bv) { (EK::Slime, _) | (EK::Boss, 2) => &art.slime_w, (EK::Bat, _) => &art.bat_w, (EK::Skel, _) | (EK::Boss, 1) => &art.skel_w, (EK::Knight, _) => &art.knight_w, _ => &art.boss_w };
            let h = sz * (1.0 - k * 0.5);
            spr2(t, c.pos.x - sz / 2.0, c.pos.y - h + 2.0 - k * 4.0, sz, h, Color::new(1.0, 1.0, 1.0, 1.0 - k), c.flip);
        }
        for p in &self.parts {
            let a = p.life / p.max;
            draw_rectangle(p.p.x, p.p.y, p.size, p.size, Color { a, ..p.col });
        }

        // darkness around the hero
        let lp = self.p.pos - vec2(0.0, 6.0);
        let base = if room.cleared { 0.45 } else { 0.7 };
        for i in 0..44 {
            let r = 44.0 + i as f32 * 8.0;
            draw_circle_lines(lp.x, lp.y, r, 8.0, Color::new(0.01, 0.0, 0.03, (i as f32 / 14.0).min(1.0) * base));
        }
        // glows on top of darkness
        let glow = |p: Vec2, r: f32, c: Color| { for i in 1..=4 { draw_circle(p.x, p.y, r * i as f32, Color { a: 0.10 / i as f32 * 2.0, ..c }); } };
        for tp in torches { glow(tp, 5.0 + (self.t * 13.0 + tp.x).sin(), rgb(255, 140, 40)); }
        for s in &self.shots {
            match s.kind { SK::Fire => glow(s.pos, 4.0, rgb(255, 140, 40)), SK::Orb => glow(s.pos, 3.5, rgb(180, 90, 255)), _ => {} }
        }
        if portal { glow(c, 8.0, rgb(150, 100, 255)); }
        for e in &self.enemies { if e.kind == EK::Boss && e.bv == 0 { glow(e.pos - vec2(0.0, 30.0), 6.0, rgb(255, 40, 40)); } }
        set_camera(&Camera2D { target: vec2(vw(), VH) / 2.0, zoom: vec2(2.0 / vw(), 2.0 / VH), ..Default::default() });
    }

    fn draw_player(&self, art: &Art) {
        let p = &self.p;
        let moving = p.vel.length() > 10.0 && p.roll_t <= 0.0;
        draw_ellipse(p.pos.x, p.pos.y + 1.0, 5.0, 2.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.4));
        let hurt_flash = p.inv > 0.95;
        if p.inv > 0.0 && !hurt_flash && (self.t * 20.0) as i32 % 2 == 0 { return; }
        let flip = p.face.x < 0.0;
        let lean = if p.swing > 0.0 { p.face * 1.5 } else { Vec2::ZERO };
        let feet = p.pos + vec2(lean.x, 1.0 + lean.y * 0.5);
        let swing_dir = if p.swing > 0.0 {
            let prog = 1.0 - p.swing / 0.18;
            let side = if p.combo % 2 == 0 { 1.0 } else { -1.0 };
            let a0 = p.face.to_angle() - 1.2 * side;
            Some(Vec2::from_angle(a0 + 2.4 * side * prog))
        } else { None };
        if p.roll_t > 0.0 {
            let rot = (1.0 - p.roll_t / 0.28) * TAU * p.roll_dir.x.signum();
            draw_rig(&art.rig_hero, p.pos + vec2(0.0, 1.0), flip, 1.0, &Pose { phase: 0.0, moving: false, breath: 0.0, front: Some(Vec2::from_angle(rot)), back_swing: rot, leg_spread: (rot * 2.0).sin() * 0.9 }, hurt_flash, WHITE);
        } else {
            let pose = Pose { phase: p.walk_t * 10.0, moving, breath: (self.t * 2.6).sin() * 0.5 + 0.5, front: swing_dir, back_swing: (self.t * 2.0).sin() * 0.08, leg_spread: if p.swing > 0.0 { 0.25 } else { 0.0 } };
            draw_rig(&art.rig_hero, feet, flip, 1.0, &pose, hurt_flash, WHITE);
        }
        if p.swing > 0.0 {
            let prog = 1.0 - p.swing / 0.18;
            let side = if p.combo % 2 == 0 { 1.0 } else { -1.0 };
            let base = p.face.to_angle();
            let a = base - 1.2 * side + 2.4 * side * prog;
            let hilt = rig_point(p.pos + vec2(0.0, 1.0), p.face.x < 0.0, 1.0, art.rig_hero.shoulder[1]) + Vec2::from_angle(a) * 3.0;
            let a0 = base - 1.2 * side;
            let steps = 8;
            for s in 0..steps {
                let t0 = a0 + (a - a0) * s as f32 / steps as f32;
                let t1 = a0 + (a - a0) * (s + 1) as f32 / steps as f32;
                let r = if p.combo == 2 { 20.0 } else { 16.0 };
                let (q0, q1) = (hilt + Vec2::from_angle(t0) * r, hilt + Vec2::from_angle(t1) * r);
                draw_line(q0.x, q0.y, q1.x, q1.y, 2.5, Color::new(0.85, 0.92, 1.0, 0.25 + 0.5 * s as f32 / steps as f32));
            }
            let tip = hilt + Vec2::from_angle(a) * 14.0;
            draw_line(hilt.x, hilt.y, tip.x, tip.y, 2.0, rgb(220, 228, 240));
            draw_line(hilt.x, hilt.y, hilt.x + (tip.x - hilt.x) * 0.2, hilt.y + (tip.y - hilt.y) * 0.2, 2.5, rgb(255, 215, 90));
        }
    }

    fn draw_enemy(&self, e: &Enemy, art: &Art) {
        let big = e.kind == EK::Boss;
        let sz = if big { 48.0 } else { 16.0 };
        let sk = if e.spawn > 0.0 { (1.0 - e.spawn / 0.7).clamp(0.12, 1.0) } else { 1.0 };
        draw_ellipse(e.pos.x, e.pos.y + 1.0, (e.r + 1.0) * sk, e.r * 0.45 * sk, 0.0, Color::new(0.0, 0.0, 0.0, 0.4));
        if e.spawn > 0.0 {
            let a = (self.t * 3.0) % TAU;
            for k in 0..6 { let q = e.pos - vec2(0.0, 6.0) + Vec2::from_angle(a + k as f32 * TAU / 6.0) * (6.0 + 8.0 * (1.0 - sk)); draw_rectangle(q.x, q.y, 1.5, 1.5, rgb(200, 120, 255)); }
            draw_circle_lines(e.pos.x, e.pos.y - 6.0, 4.0 + 10.0 * (1.0 - sk), 1.0, Color::new(0.8, 0.5, 1.0, 0.8));
        }
        let moving = e.vel.length() > 6.0;
        let flash = e.flash > 0.0;
        let (mut w, mut h, mut bob) = (sz, sz, 0.0);
        if e.kind == EK::Slime {
            if e.vel.length() > 60.0 { w = sz * 0.72; h = sz * 1.35; bob = -4.0; }
            else if e.t > 0.7 { let k = ((e.t - 0.7) / 0.2).min(1.0); w = sz * (1.0 + 0.35 * k); h = sz * (1.0 - 0.32 * k); }
            else { let b = (self.t * 6.0 + e.pos.x).sin() * 0.12; w = sz * (1.0 + b); h = sz * (1.0 - b); }
        }
        if big && e.bv == 2 {
            if e.state == 1 { bob = -((e.t / 0.5).min(1.0) * PI).sin() * 14.0; w = sz * 0.85; h = sz * 1.2; }
            else if e.t > 1.0 { w = sz * 1.2; h = sz * 0.8; }
            else if e.t < 0.15 { w = sz * 1.28; h = sz * 0.72; }
            else { let b = (self.t * 4.0).sin() * 0.04; w = sz * (1.0 + b); h = sz * (1.0 - b); }
        }
        if big && e.bv == 0 { let pl = 1.0 + (self.t * 3.0).sin() * 0.03; w = sz * pl; h = sz * (2.0 - pl); bob = (self.t * 2.0).sin() * 2.0; }
        let tex: &Texture2D = match e.kind {
            EK::Slime => if flash { &art.slime_w } else { &art.slime },
            EK::Bat => { bob = (e.t * 10.0).sin() * 3.0 - 7.0; w = sz * (1.0 + (e.t * 14.0 * PI / 2.0).sin() * 0.22); if flash { &art.bat_w } else { &art.bat_f[[0, 1, 0, 2][((e.t * 14.0) as usize) % 4]] } }
            EK::Skel | EK::Knight => &art.skel_w,
            EK::Boss => match e.bv { 1 => &art.skel_w, 2 => if flash { &art.slime_w } else { &art.slime }, _ => if flash { &art.boss_w } else { &art.boss } },
        };
        let warn = (e.kind == EK::Knight || big) && e.state == 1 && (self.t * 16.0) as i32 % 2 == 0;
        let tint = if flash { WHITE } else if warn { Color::new(1.0, 0.85, 0.3, 1.0) } else if big { match e.bv { 1 => Color::new(1.0, 0.86, 0.5, 1.0), 2 => Color::new(0.6, 0.9, 0.55, 1.0), _ => WHITE } } else { WHITE };
        let flip = (self.p.pos.x - e.pos.x) < 0.0;
        let (w, h) = (w * sk, h * sk);
        let rig = match (e.kind, e.bv) { (EK::Skel, _) | (EK::Boss, 1) => Some(&art.rig_skel), (EK::Knight, _) => Some(&art.rig_knight), _ => None };
        if let Some(rig) = rig {
            let to_p = (self.p.pos - e.pos).normalize_or(Vec2::X);
            let aiming = (e.kind == EK::Skel && e.atk < 0.45) || (big && e.state == 1);
            let charging = e.state == 2 && (e.kind == EK::Knight || big);
            let front = if aiming || charging { Some(if charging { e.dir } else { to_p }) } else { None };
            let pose = Pose { phase: e.t * 9.0, moving: moving && !aiming, breath: (self.t * 2.2 + e.pos.x).sin() * 0.4 + 0.4, front, back_swing: if charging { -0.9 } else { 0.0 }, leg_spread: if charging { 0.5 } else { 0.0 } };
            let scale = sz / 16.0 * sk;
            draw_rig(rig, e.pos + vec2(0.0, 2.0), flip, scale, &pose, flash, tint);
            if e.kind == EK::Skel || big {
                let hand = rig_point(e.pos + vec2(0.0, 2.0), flip, scale, rig.shoulder[1]) + front.unwrap_or(vec2(0.3 * if flip { -1.0 } else { 1.0 }, 1.0)).normalize() * rig.arm_len * scale;
                let d = front.unwrap_or(vec2(if flip { -1.0 } else { 1.0 }, 0.0)).normalize();
                let pr = d.perp() * 4.0 * scale;
                draw_line(hand.x + pr.x, hand.y + pr.y, hand.x + d.x * 2.0 * scale, hand.y + d.y * 2.0 * scale, 1.0 * scale, rgb(122, 74, 42));
                draw_line(hand.x - pr.x, hand.y - pr.y, hand.x + d.x * 2.0 * scale, hand.y + d.y * 2.0 * scale, 1.0 * scale, rgb(122, 74, 42));
                if aiming { draw_line(hand.x + pr.x, hand.y + pr.y, hand.x - pr.x, hand.y - pr.y, 0.5 * scale, rgb(230, 230, 230)); }
            }
        } else {
            spr2(tex, e.pos.x - w / 2.0, e.pos.y - h + 2.0 + bob, w, h, tint, flip);
        }
        if !big && e.hp < e.max_hp {
            draw_rectangle(e.pos.x - 7.0, e.pos.y - 18.0, 14.0, 2.0, rgb(40, 10, 15));
            draw_rectangle(e.pos.x - 7.0, e.pos.y - 18.0, 14.0 * (e.hp / e.max_hp).max(0.0), 2.0, rgb(240, 60, 60));
        }
    }

    fn draw_ui(&self, art: &Art) {
        let p = &self.p;
        let w = vw();
        let bar = |x: f32, y: f32, bw: f32, bh: f32, frac: f32, c: Color, bg: Color| {
            draw_rectangle(x - 2.0, y - 2.0, bw + 4.0, bh + 4.0, rgb(10, 6, 16));
            draw_rectangle(x, y, bw, bh, bg);
            draw_rectangle(x, y, bw * frac.clamp(0.0, 1.0), bh, c);
            draw_rectangle(x, y, bw * frac.clamp(0.0, 1.0), 2.0, Color { a: 0.35, ..WHITE });
        };
        bar(20.0, 18.0, 260.0, 18.0, p.hp / p.max_hp, rgb(220, 50, 60), rgb(60, 15, 25));
        txt(&format!("{} / {}", p.hp.ceil() as i32, p.max_hp as i32), 28.0, 33.0, 18.0, WHITE);
        bar(20.0, 44.0, 200.0, 10.0, p.mp / p.max_mp, rgb(70, 120, 255), rgb(15, 25, 60));
        bar(20.0, 62.0, 200.0, 5.0, p.xp / p.xp_need, rgb(255, 215, 90), rgb(50, 40, 15));
        txt(&format!("LVL {}", p.level), 230.0, 69.0, 20.0, rgb(255, 215, 90));
        spr(&art.coin, 20.0, 80.0, 22.0, WHITE, false, 0.0);
        txt(&format!("{}", p.gold), 48.0, 98.0, 22.0, rgb(255, 215, 90));
        spr(&art.potion, 110.0, 80.0, 22.0, WHITE, false, 0.0);
        txt(&format!("x{}", p.potions), 136.0, 98.0, 22.0, rgb(255, 130, 130));
        let fl = format!("QAVAT {}", self.floor);
        txt(&fl, w / 2.0 - tw(&fl, 28.0) / 2.0, 34.0, 28.0, rgb(180, 140, 255));

        // minimap
        let cell = 14.0;
        let (ox, oy) = (w - 20.0 - GRID as f32 * cell, 14.0);
        for (i, r) in self.rooms.iter().enumerate() {
            let known = r.visited || DIRS.iter().enumerate().any(|(d, dd)| r.doors[d] && self.rooms.iter().any(|o| o.g == (r.g.0 + dd.0, r.g.1 + dd.1) && o.visited));
            if !known { continue; }
            let (x, y) = (ox + r.g.0 as f32 * cell, oy + r.g.1 as f32 * cell);
            if r.doors[1] { draw_rectangle(x + cell - 2.0, y + cell / 2.0 - 1.0, 4.0, 2.0, rgb(90, 85, 110)); }
            if r.doors[2] { draw_rectangle(x + cell / 2.0 - 1.0, y + cell - 2.0, 2.0, 4.0, rgb(90, 85, 110)); }
            let col = if i == self.cur { WHITE } else if !r.visited { rgb(50, 45, 65) } else {
                match r.kind { RK::Boss => rgb(220, 50, 60), RK::Treasure => rgb(255, 215, 90), RK::Shop => rgb(90, 210, 230), _ => rgb(110, 105, 140) } };
            let col = if !r.visited && r.kind == RK::Boss { rgb(110, 30, 40) } else { col };
            draw_rectangle(x + 1.0, y + 1.0, cell - 2.0, cell - 2.0, col);
        }

        if let Some(b) = self.enemies.iter().find(|e| e.kind == EK::Boss) {
            let bw = 520.0_f32.min(w - 80.0);
            let bn = BOSS_NAMES[b.bv as usize];
            txt(bn, w / 2.0 - tw(bn, 22.0) / 2.0, VH - 52.0, 22.0, rgb(200, 140, 255));
            bar(w / 2.0 - bw / 2.0, VH - 40.0, bw, 12.0, b.hp / b.max_hp, rgb(150, 80, 200), rgb(30, 15, 40));
        }
        if self.boss_t > 0.0 {
            let a = self.boss_t.min(1.0);
            let bv = ((self.floor - 1) % 3) as usize;
            centered(BOSS_NAMES[bv], VH * 0.32, 72.0, Color::new(0.8, 0.5, 1.0, a));
            centered(BOSS_SUB[bv], VH * 0.32 + 40.0, 24.0, Color::new(0.8, 0.8, 0.9, a));
        }
        for t in &self.texts {
            let sp = w2ui(t.pos);
            let a = (t.life / 0.5).min(1.0);
            txt(&t.s, sp.x - tw(&t.s, 22.0) / 2.0 + 2.0, sp.y + 2.0, 22.0, Color::new(0.0, 0.0, 0.0, a * 0.7));
            txt(&t.s, sp.x - tw(&t.s, 22.0) / 2.0, sp.y, 22.0, Color { a, ..t.col });
        }
        if self.touch_mode && self.state == State::Play {
            if let Some((_, o, c)) = self.stick {
                draw_circle_lines(o.x, o.y, 64.0, 3.0, Color::new(1.0, 1.0, 1.0, 0.3));
                let kp = o + (c - o).clamp_length_max(64.0);
                draw_circle(kp.x, kp.y, 26.0, Color::new(1.0, 1.0, 1.0, 0.25));
            }
            let ready = [true, p.roll_cd <= 0.0, p.mp >= 20.0, p.potions > 0];
            for (i, (bp, r, label)) in buttons().iter().enumerate() {
                let a = if ready[i] { 1.0 } else { 0.35 };
                draw_circle(bp.x, bp.y, *r, Color::new(0.1, 0.08, 0.18, 0.55 * a));
                draw_circle_lines(bp.x, bp.y, *r, 3.0, Color::new(0.8, 0.75, 1.0, 0.7 * a));
                let s = if i == 3 { format!("{} {}", label, p.potions) } else { label.to_string() };
                txt(&s, bp.x - tw(&s, 20.0) / 2.0, bp.y + 7.0, 20.0, Color::new(1.0, 1.0, 1.0, a));
            }
        }
        // shop labels
        if self.rooms[self.cur].kind == RK::Shop && self.state == State::Play {
            for (i, sp) in shop_slots().iter().enumerate() {
                let u = w2ui(*sp + vec2(0.0, 12.0));
                let price = (SHOP[i].1 as f32 * (1.0 + 0.25 * (self.floor - 1) as f32)) as u32;
                let ok = self.rooms[self.cur].stock[i] > 0;
                let l = if ok { format!("{}", price) } else { "-".to_string() };
                txt(&l, u.x - tw(&l, 22.0) / 2.0, u.y + 14.0, 22.0, if ok && p.gold >= price { rgb(255, 215, 90) } else { rgb(200, 110, 110) });
                txt(SHOP[i].0, u.x - tw(SHOP[i].0, 16.0) / 2.0, u.y + 34.0, 16.0, Color::new(1.0, 1.0, 1.0, if ok { 0.85 } else { 0.4 }));
            }
            centered("SAVDOGAR: buyum ustidan yuring - sotib olasiz", VH - 30.0, 20.0, Color::new(1.0, 0.92, 0.7, 0.85));
        }
        // first-room tutorial
        if self.floor == 1 && self.cur == 0 && self.state == State::Play {
            let lines: [&str; 4] = if self.touch_mode {
                ["Chap tomon: yurish (joystik)", "HUJUM: qilich, bosib turing - kombo", "SAKRASH: zarbadan qochish, OLOV: sehr", "Eshikdan chiqing - dushmanlar kutmoqda!"]
            } else {
                ["WASD: yurish", "J / sichqoncha: qilich (bosib turing - kombo)", "Space: sakrash (zarar olmaysiz), K: olov, Q: iksir", "Eshikdan chiqing - dushmanlar kutmoqda!"]
            };
            let (pw, ph) = (560.0_f32.min(w - 40.0), 150.0);
            let (px, py) = (w / 2.0 - pw / 2.0, VH * 0.6);
            draw_rectangle(px, py, pw, ph, Color::new(0.05, 0.03, 0.09, 0.82));
            draw_rectangle_lines(px, py, pw, ph, 2.0, rgb(255, 215, 90));
            for (i, l) in lines.iter().enumerate() { txt(l, px + 20.0, py + 34.0 + i as f32 * 32.0, 21.0, if i == 3 { rgb(255, 215, 90) } else { WHITE }); }
        }
        if self.state == State::Play {
            let r = pause_btn();
            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.1, 0.07, 0.16, 0.8));
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, rgb(150, 130, 190));
            draw_rectangle(r.x + 16.0, r.y + 14.0, 7.0, 24.0, WHITE); draw_rectangle(r.x + 29.0, r.y + 14.0, 7.0, 24.0, WHITE);
        }
        if self.fade > 0.0 { draw_rectangle(0.0, 0.0, w, VH, Color::new(0.0, 0.0, 0.0, self.fade.min(1.0))); }
    }
}


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

// ---------------- procedural audio ----------------
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
fn sq(ph: f32) -> f32 { if (ph * TAU).sin() >= 0.0 { 1.0 } else { -1.0 } }
fn saw(ph: f32) -> f32 { 2.0 * ph.fract() - 1.0 }
async fn snd(dur: f32, f: impl Fn(f32) -> f32) -> Sound { load_sound_from_bytes(&wav(dur, f)).await.unwrap() }
async fn load_audio() -> (Sfx, Sound) {
    let sfx = Sfx {
        swing: snd(0.12, |t| noise() * (1.0 - t / 0.12).powi(2) * 0.35 * (t * 80.0).min(1.0)).await,
        hit: snd(0.15, |t| (TAU * (160.0 * t - 400.0 * t * t)).sin() * (-t * 25.0).exp() * 0.9 + noise() * (-t * 40.0).exp() * 0.4).await,
        kill: snd(0.35, |t| (noise() * 0.6 + (TAU * (110.0 * t - 120.0 * t * t)).sin()) * (-t * 10.0).exp() * 0.7).await,
        coin: snd(0.18, |t| sq(if t < 0.06 { 988.0 } else { 1319.0 } * t) * 0.18 * (1.0 - t / 0.18)).await,
        hurt: snd(0.3, |t| saw(380.0 * t - 450.0 * t * t) * (-t * 7.0).exp() * 0.6).await,
        fire: snd(0.4, |t| (noise() * 0.5 + (TAU * (200.0 * t + 300.0 * t * t)).sin() * 0.4) * (PI * t / 0.4).sin() * 0.6).await,
        level: snd(0.6, |t| {
            let f = [523.25, 659.25, 783.99, 1046.5][((t / 0.1) as usize).min(3)];
            ((TAU * f * t).sin() * 0.45 + sq(f * t) * 0.08) * (1.0 - t / 0.6)
        }).await,
        door: snd(0.3, |t| (TAU * 70.0 * t).sin() * (-t * 10.0).exp() * 0.8 + noise() * (-t * 30.0).exp() * 0.3).await,
        potion: snd(0.35, |t| (TAU * ((400.0 + 900.0 * t) * t + (t * 60.0).sin() * 0.8)).sin() * 0.35 * (1.0 - t / 0.35)).await,
        roar: snd(1.2, |t| (saw((55.0 + (t * 30.0).sin() * 6.0) * t) * 0.6 + noise() * 0.3) * (t * 6.0).min(1.0) * (1.0 - t / 1.2) * 0.8).await,
    };
    // dungeon loop: Dm - Bb - C - Am, 8 s
    let music = snd(8.0, |t| {
        let roots = [146.83, 116.54, 130.81, 110.0];
        let i = ((t / 2.0) as usize).min(3);
        let f = roots[i] / 2.0;
        let bass = ((TAU * f * t).sin() * 0.16 + sq(f * t) * 0.03) * (0.6 + 0.4 * (-(t % 0.5) * 3.0).exp());
        let arp = [1.0, 1.5, 2.0, 1.5][((t * 4.0) as usize) % 4] * roots[i] * 2.0;
        let env = (-(t % 0.25) * 10.0).exp();
        let pluck = ((TAU * arp * t).sin() * 0.07 + sq(arp * t) * 0.015) * env;
        let drone = (TAU * f * 0.5 * t).sin() * 0.06;
        (bass + pluck + drone) * (t * 20.0).min(1.0) * ((8.0 - t) * 20.0).min(1.0)
    }).await;
    (sfx, music)
}

fn export_icons() {
    let make = |size: u16| {
        let mut img = Image::gen_image_color(size, size, BLANK);
        let half = size as f32 / 2.0;
        for y in 0..size as u32 {
            for x in 0..size as u32 {
                let d = (vec2(x as f32, y as f32) - vec2(half, half)).length() / half;
                let k = (1.0 - d).clamp(0.0, 1.0);
                img.set_pixel(x, y, Color::new(0.03 + 0.20 * k, 0.02 + 0.10 * k, 0.07 + 0.30 * k, 1.0));
            }
        }
        let hero = sprite_img(&HERO, 16, &[]);
        let f = ((size as u32 * 3 / 4) / 16).max(1);
        let ox = (size as u32 - 16 * f) / 2;
        for y in 0..16u32 {
            for x in 0..16u32 {
                let c = hero.get_pixel(x, y);
                if c.a > 0.0 { for dy in 0..f { for dx in 0..f { img.set_pixel(ox + x * f + dx, size as u32 - 1 - (ox + y * f + dy), c); } } }
            }
        }
        img
    };
    for (name, px) in [("mdpi", 48), ("hdpi", 72), ("xhdpi", 96), ("xxhdpi", 144), ("xxxhdpi", 192)] {
        let _ = std::fs::create_dir_all(format!("android_res/mipmap-{}", name));
        make(px).export_png(&format!("android_res/mipmap-{}/ic_launcher.png", name));
    }
    make(512).export_png("site/icon.png");
}

fn conf() -> Conf {
    Conf { window_title: "QAL'A RITSARI".into(), window_width: 1280, window_height: 720, high_dpi: true, sample_count: 1, ..Default::default() }
}

#[macroquad::main(conf)]
async fn main() {
    if std::env::args().any(|a| a == "--icons") { export_icons(); return; }
    std::panic::set_hook(Box::new(|info| { macroquad::logging::error!("ZQ PANIC: {}", info); }));
    macroquad::logging::info!("ZQ start");
    simulate_mouse_with_touch(false);
    rand::srand(macroquad::miniquad::date::now() as u64);
    let art = Art::load();
    macroquad::logging::info!("ZQ art loaded");
    let (sfx, music) = load_audio().await;
    macroquad::logging::info!("ZQ audio loaded");
    play_sound(&music, PlaySoundParams { looped: true, volume: 0.45 });
    let mut g = Game::new(cfg!(target_os = "android"), 1);
    macroquad::logging::info!("ZQ game created");
    let mut frames = 0u32;
    loop {
        frames += 1;
        if frames == 1 || frames == 60 { macroquad::logging::info!("ZQ frame {}", frames); }
        let dt = get_frame_time().min(1.0 / 30.0);
        clear_background(rgb(6, 4, 10));
        match g.state {
            State::Title => {
                g.t += dt;
                g.draw_world(&art);
                draw_rectangle(0.0, 0.0, vw(), VH, Color::new(0.02, 0.0, 0.05, 0.55));
                let pulse = 0.75 + 0.25 * (g.t * 3.0).sin();
                centered("QAL'A RITSARI", VH * 0.33, 96.0, Color::new(0.75, 0.6, 1.0, 1.0));
                centered("Piksel Action RPG", VH * 0.33 + 44.0, 28.0, rgb(255, 215, 90));
                let help = if g.touch_mode { "Chap: yurish   HUJUM: qilich   SAKRASH   OLOV   IKSIR" }
                    else { "WASD yurish  J/Sichqoncha qilich  Space sakrash  K/O'ng tugma olov  Q iksir" };
                centered(help, VH * 0.62, 22.0, WHITE);
                centered(if g.touch_mode { "[ Boshlash uchun bosing ]" } else { "[ ENTER yoki bosing ]" }, VH * 0.74, 32.0, Color::new(1.0, 1.0, 1.0, pulse));
                if g.best_floor > 1 { centered(&format!("Eng chuqur qavat: {}", g.best_floor), VH * 0.86, 22.0, GRAY); }
                if is_key_pressed(KeyCode::Enter) || tap().is_some() {
                    let tm = g.touch_mode || !touches().is_empty();
                    g = Game::new(tm, g.best_floor);
                    g.state = State::Play;
                    // QA hooks (desktop only): start in the shop or a boss room
                    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
                    {
                        if let Ok(n) = std::env::var("ZQ_BOSS") { let f: u32 = n.parse().unwrap_or(1); g.floor = f; g.rooms = gen_floor(f); if let Some(i) = g.rooms.iter().position(|r| r.kind == RK::Boss) { g.cur = i; g.p.pos = room_center() + vec2(0.0, 50.0); g.enter_room(&sfx); } }
                        if std::env::var("ZQ_SHOP").is_ok() { if let Some(i) = g.rooms.iter().position(|r| r.kind == RK::Shop) { g.cur = i; g.p.gold = 200; g.p.pos = room_center() + vec2(-60.0, 10.0); } }
                    }
                }
            }
            State::Play => {
                g.read_touches();
                if g.hitstop > 0.0 { g.hitstop -= dt; } else { g.update(dt, &sfx); }
                g.draw_world(&art); g.draw_ui(&art);
                let pt = tap();
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) || pt.map_or(false, |p| pause_btn().contains(p)) { g.state = State::Pause; g.stick = None; g.atk_touch = None; }
            }
            State::Pause => {
                g.draw_world(&art); g.draw_ui(&art);
                draw_rectangle(0.0, 0.0, vw(), VH, Color::new(0.02, 0.01, 0.05, 0.7));
                centered("PAUZA", VH * 0.22, 72.0, rgb(255, 215, 90));
                let cx = vw() / 2.0;
                let rs = [Rect::new(cx - 170.0, VH * 0.32, 340.0, 64.0), Rect::new(cx - 170.0, VH * 0.32 + 84.0, 340.0, 64.0), Rect::new(cx - 170.0, VH * 0.32 + 168.0, 340.0, 64.0), Rect::new(cx - 170.0, VH * 0.32 + 252.0, 340.0, 64.0)];
                ui_btn(rs[0], "DAVOM ETISH", rgb(120, 220, 130));
                ui_btn(rs[1], if music_on() { "MUSIQA: YONIQ" } else { "MUSIQA: O'CHIQ" }, rgb(180, 160, 240));
                ui_btn(rs[2], if sfx_on() { "EFFEKTLAR: YONIQ" } else { "EFFEKTLAR: O'CHIQ" }, rgb(180, 160, 240));
                ui_btn(rs[3], "BOSH MENYU", rgb(240, 150, 140));
                let t = tap();
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) || t.map_or(false, |p| rs[0].contains(p)) { g.state = State::Play; }
                if t.map_or(false, |p| rs[1].contains(p)) { MUSIC_ON.store(!music_on(), std::sync::atomic::Ordering::Relaxed); set_sound_volume(&music, if music_on() { 0.45 } else { 0.0 }); }
                if t.map_or(false, |p| rs[2].contains(p)) { SFX_ON.store(!sfx_on(), std::sync::atomic::Ordering::Relaxed); }
                if t.map_or(false, |p| rs[3].contains(p)) { let b = g.best_floor.max(g.floor); g = Game::new(g.touch_mode, b); }
            }
            State::LevelUp => {
                g.draw_world(&art); g.draw_ui(&art);
                draw_rectangle(0.0, 0.0, vw(), VH, Color::new(0.0, 0.0, 0.0, 0.65));
                centered(&format!("DARAJA {}!", g.p.level), VH * 0.22, 64.0, rgb(255, 215, 90));
                centered("Qobiliyat tanlang", VH * 0.22 + 36.0, 24.0, LIGHTGRAY);
                let cx = vw() / 2.0;
                let t = tap();
                let mouse = Vec2::from(mouse_position()) / sc();
                let mut pick = None;
                for i in 0..3 {
                    let pk = &PERKS[g.choices[i]];
                    let r = Rect::new(cx - 420.0 + i as f32 * 290.0, VH * 0.38, 260.0, 190.0);
                    let hover = r.contains(mouse);
                    draw_rectangle(r.x, r.y, r.w, r.h, rgb(22, 16, 36));
                    draw_rectangle_lines(r.x, r.y, r.w, r.h, if hover { 4.0 } else { 2.0 }, if hover { rgb(255, 215, 90) } else { rgb(110, 100, 140) });
                    txt(&format!("[{}]", i + 1), r.x + 16.0, r.y + 36.0, 26.0, rgb(180, 140, 255));
                    txt(pk.name, r.x + 16.0, r.y + 92.0, 34.0, WHITE);
                    txt(pk.desc, r.x + 16.0, r.y + 140.0, 19.0, LIGHTGRAY);
                    let key = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3][i];
                    if is_key_pressed(key) || t.map_or(false, |p| r.contains(p)) { pick = Some(g.choices[i]); }
                }
                if let Some(id) = pick { g.apply_perk(id); g.state = State::Play; }
            }
            State::Dead => {
                g.t += dt; g.over_t += dt;
                for p in &mut g.parts { p.p += p.v * dt; p.v *= 1.0 - dt * 4.0; p.life -= dt; }
                g.parts.retain(|p| p.life > 0.0);
                g.draw_world(&art);
                draw_rectangle(0.0, 0.0, vw(), VH, Color::new(0.1, 0.0, 0.0, (g.over_t * 0.6).min(0.6)));
                centered("SIZ HALOK BO'LDINGIZ", VH * 0.35, 72.0, rgb(240, 70, 80));
                centered(&format!("Qavat: {}   Daraja: {}   Oltin: {}   O'ldirilgan: {}", g.floor, g.p.level, g.p.gold, g.kills), VH * 0.48, 26.0, WHITE);
                if g.over_t > 1.2 {
                    centered(if g.touch_mode { "[ Qaytadan - bosing ]" } else { "[ ENTER - qaytadan ]" }, VH * 0.66, 30.0, rgb(180, 140, 255));
                    if is_key_pressed(KeyCode::Enter) || tap().is_some() {
                        let best = g.best_floor.max(g.floor);
                        g = Game::new(g.touch_mode, best);
                        g.state = State::Play;
                    }
                }
            }
        }
        next_frame().await;
    }
}
