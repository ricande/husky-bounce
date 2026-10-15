//! Husky Bounce: a game that mostly plays itself. Balls bounce inside
//! rotating rings; a bounce plays a note and may spawn a new ball. A ball
//! that slips through a ring's gap moves out to the next ring, and through
//! the outermost one it escapes (a point). It starts with one ball and ends,
//! after `LEVEL_SECONDS`, in chaos. See docs/GAME_RULES.md.

use macroquad::color::Color;
use macroquad::math::{vec2, Vec2};

use crate::audio::Sfx;
use crate::game::{Game, RoundResult};
use crate::i18n::Text;
use crate::t;
use crate::ui::{Input, Key};
use crate::view::{with_alpha, View, ACCENT, FIELD_W, NEON, TEXT};

/// Length of a level: a little under 3:00, the longest YouTube Short (and the
/// framework's video limit), so a whole level fits in one Short.
pub const LEVEL_SECONDS: f32 = 179.0;
const CENTRE: Vec2 = Vec2::new(FIELD_W / 2.0, 520.0);
const OUTER_R: f32 = 240.0;
const RING_SPACING: f32 = 48.0;
const BALL_R: f32 = 7.0;
/// More balls than this are not spawned (keeps every frame cheap).
pub const MAX_BALLS: usize = 400;
const GRAVITY: f32 = 380.0;
/// Bounce notes are at least this far apart, so the sound stays music.
const NOTE_GAP: f32 = 0.045;
/// Index of the escape sound in `SOUNDS`, after the eight notes.
const ESCAPE_SOUND: usize = 8;

const TEXTS: &[Text] = &[
    (
        "menu.subtitle",
        "One ball. Then chaos.",
        "En boll. Sedan kaos.",
    ),
    ("bounce.level", "Level {level}", "Nivå {level}"),
    ("bounce.balls", "Balls {count}", "Bollar {count}"),
    ("bounce.escaped", "Escaped {count}", "Ute {count}"),
    (
        "bounce.help",
        "Space or click: one more ball   Esc: pause",
        "Mellanslag eller klick: en boll till   Esc: paus",
    ),
];

/// A tiny deterministic random generator (xorshift), one per game, so a
/// level plays the same every time and tests can run in parallel.
#[derive(Clone)]
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    /// A number in [0, 1).
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next()
    }
}

struct Ring {
    radius: f32,
    /// Direction of the gap's middle, in radians; it turns with `spin`.
    gap_angle: f32,
    spin: f32,
    hue: f32,
}

#[derive(Clone)]
struct Ball {
    pos: Vec2,
    vel: Vec2,
    /// The ring that holds it (index into `rings`, innermost first);
    /// `rings.len()` once it has escaped.
    zone: usize,
    hue: f32,
}

pub struct HuskyBounce {
    level: u32,
    time: f32,
    rings: Vec<Ring>,
    balls: Vec<Ball>,
    escaped: u32,
    rng: Rng,
    notes_played: usize,
    last_note: f32,
    last_escape: f32,
    sounds: Vec<Sfx>,
    done: bool,
}

/// How far the level has come, 0 at the start and 1 at the end.
fn progress(time: f32) -> f32 {
    (time / LEVEL_SECONDS).clamp(0.0, 1.0)
}

/// Half the gap's width: narrow at first, wide at the end, so the swarm
/// pours out in the last minute.
fn gap_half(time: f32) -> f32 {
    0.22 + 0.55 * progress(time).powi(2)
}

/// The smallest difference between two angles.
fn angle_diff(a: f32, b: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    ((a - b + PI).rem_euclid(TAU) - PI).abs()
}

fn hue_color(hue: f32) -> Color {
    let h = hue.rem_euclid(1.0) * 6.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    // Pastel, so it reads well on the dark background.
    Color::new(0.35 + 0.65 * r, 0.35 + 0.65 * g, 0.35 + 0.65 * b, 1.0)
}

impl HuskyBounce {
    fn spawn(&mut self) {
        if self.balls.len() >= MAX_BALLS {
            return;
        }
        let angle = self.rng.range(0.0, std::f32::consts::TAU);
        let speed = self.rng.range(150.0, 320.0);
        let hue = self.rng.next();
        self.balls.push(Ball {
            pos: CENTRE,
            vel: vec2(angle.cos(), angle.sin()) * speed,
            zone: 0,
            hue,
        });
    }

    /// How many balls bounces may grow the field to by now: one at the
    /// start, `MAX_BALLS` at the end, rising with the square of the time, so
    /// the chaos builds steadily all the way to the end.
    fn ball_limit(&self) -> usize {
        1 + ((MAX_BALLS - 1) as f32 * progress(self.time).powi(2)) as usize
    }

    /// Chance that a bounce spawns a ball (below `ball_limit`).
    fn spawn_chance(&self) -> f32 {
        0.3 + 0.7 * progress(self.time)
    }

    fn note(&mut self) {
        if self.time - self.last_note >= NOTE_GAP {
            self.last_note = self.time;
            // Up and down the eight-note scale.
            let i = self.notes_played % 14;
            let n = if i < 8 { i } else { 14 - i };
            self.sounds.push(Sfx::Game(n));
            self.notes_played += 1;
        }
    }

    /// Moves one ball; returns (bounced, escaped now).
    fn step_ball(ball: &mut Ball, rings: &[Ring], gap: f32, dt: f32) -> (bool, bool) {
        ball.vel.y += GRAVITY * dt;
        ball.pos += ball.vel * dt;
        let Some(ring) = rings.get(ball.zone) else {
            return (false, false);
        };
        let d = ball.pos - CENTRE;
        let dist = d.length().max(1e-3);
        let n = d / dist;
        if dist + BALL_R >= ring.radius {
            if angle_diff(d.y.atan2(d.x), ring.gap_angle) < gap {
                ball.zone += 1;
                return (false, ball.zone == rings.len());
            }
            if ball.vel.dot(n) > 0.0 {
                ball.vel -= 2.0 * ball.vel.dot(n) * n;
                ball.pos = CENTRE + n * (ring.radius - BALL_R);
                return (true, false);
            }
        } else if let Some(inner) = ball.zone.checked_sub(1).map(|i| &rings[i]) {
            // Outside a smaller ring: bounce off its outer side.
            let blocked = angle_diff(d.y.atan2(d.x), inner.gap_angle) >= gap;
            if dist - BALL_R <= inner.radius && blocked && ball.vel.dot(n) < 0.0 {
                ball.vel -= 2.0 * ball.vel.dot(n) * n;
                ball.pos = CENTRE + n * (inner.radius + BALL_R);
                return (true, false);
            }
        }
        (false, false)
    }
}

impl Game for HuskyBounce {
    const TEXTS: &'static [Text] = TEXTS;
    const SOUNDS: &'static [&'static [u8]] = &[
        include_bytes!("../assets/note0.wav"),
        include_bytes!("../assets/note1.wav"),
        include_bytes!("../assets/note2.wav"),
        include_bytes!("../assets/note3.wav"),
        include_bytes!("../assets/note4.wav"),
        include_bytes!("../assets/note5.wav"),
        include_bytes!("../assets/note6.wav"),
        include_bytes!("../assets/note7.wav"),
        include_bytes!("../assets/escape.wav"),
    ];

    fn new(level: u32) -> Self {
        let mut rng = Rng::new(u64::from(level));
        // Level 1 has one ring, every level one more, up to four.
        let count = level.clamp(1, 4) as usize;
        let rings = (0..count)
            .map(|i| {
                let from_outside = (count - 1 - i) as f32;
                let direction = if i % 2 == 0 { 1.0 } else { -1.0 };
                Ring {
                    radius: OUTER_R - from_outside * RING_SPACING,
                    gap_angle: rng.range(0.0, std::f32::consts::TAU),
                    spin: direction * rng.range(0.35, 0.6),
                    hue: rng.next(),
                }
            })
            .collect();
        let mut game = HuskyBounce {
            level,
            time: 0.0,
            rings,
            balls: Vec::new(),
            escaped: 0,
            rng,
            notes_played: 0,
            last_note: -1.0,
            last_escape: -1.0,
            sounds: Vec::new(),
            done: false,
        };
        game.spawn();
        game
    }

    fn handle(&mut self, input: &Input) {
        if input.key(Key::Space) || input.left_pressed {
            self.spawn();
        }
    }

    fn update(&mut self, dt: f32) {
        self.time += dt;
        if self.time >= LEVEL_SECONDS {
            self.done = true;
        }
        let speed_up = 1.0 + 3.0 * progress(self.time);
        for ring in &mut self.rings {
            ring.gap_angle += ring.spin * speed_up * dt;
        }
        let gap = gap_half(self.time);
        let (mut bounces, mut escapes) = (0, 0);
        for ball in &mut self.balls {
            let (bounced, escaped) = Self::step_ball(ball, &self.rings, gap, dt);
            bounces += usize::from(bounced);
            escapes += u32::from(escaped);
        }
        // Escaped balls fall off the screen and are gone.
        self.balls
            .retain(|b| b.pos.y < 1000.0 && b.pos.x > -50.0 && b.pos.x < 590.0);
        self.escaped += escapes;
        if escapes > 0 && self.time - self.last_escape >= NOTE_GAP * 2.0 {
            self.last_escape = self.time;
            self.sounds.push(Sfx::Game(ESCAPE_SOUND));
        }
        if bounces > 0 {
            self.note();
        }
        for _ in 0..bounces {
            if self.balls.len() < self.ball_limit() && self.rng.next() < self.spawn_chance() {
                self.spawn();
            }
        }
        if self.balls.is_empty() {
            // Never an empty field: a new ball comes from the middle.
            self.spawn();
        }
    }

    fn draw(&self, v: &View) {
        // Rings, as segments with the gap left out.
        const SEGMENTS: usize = 120;
        let gap = gap_half(self.time);
        for ring in &self.rings {
            let color = hue_color(ring.hue);
            for i in 0..SEGMENTS {
                let a0 = i as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
                let a1 = (i + 1) as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
                if angle_diff((a0 + a1) / 2.0, ring.gap_angle) < gap {
                    continue;
                }
                let p = |a: f32| CENTRE + vec2(a.cos(), a.sin()) * ring.radius;
                v.line(p(a0), p(a1), 5.0, with_alpha(color, 0.25));
                v.line(p(a0), p(a1), 2.5, color);
            }
        }
        for ball in &self.balls {
            v.circle(ball.pos, BALL_R, hue_color(ball.hue));
        }

        let level = t!("bounce.level", level = self.level);
        v.text(&level, 20.0, 46.0, 26.0, NEON);
        let left = (LEVEL_SECONDS - self.time).max(0.0).ceil() as u32;
        let clock = format!("{}:{:02}", left / 60, left % 60);
        v.text_centered(&clock, FIELD_W / 2.0, 150.0, 56.0, ACCENT);
        let balls = t!("bounce.balls", count = self.balls.len());
        v.text(&balls, 20.0, 900.0, 24.0, TEXT);
        let escaped = t!("bounce.escaped", count = self.escaped);
        let w = v.text_width(&escaped, 24.0);
        v.text(&escaped, FIELD_W - 20.0 - w, 900.0, 24.0, ACCENT);
        v.text_centered(
            t!("bounce.help"),
            FIELD_W / 2.0,
            940.0,
            16.0,
            with_alpha(TEXT, 0.6),
        );
    }

    fn take_sounds(&mut self) -> Vec<Sfx> {
        std::mem::take(&mut self.sounds)
    }

    fn finished(&self) -> Option<RoundResult> {
        self.done.then_some(RoundResult {
            score: self.escaped,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::FIXED_DT;

    fn run(game: &mut HuskyBounce, seconds: f32) {
        for _ in 0..(seconds / FIXED_DT).round() as u32 {
            game.update(FIXED_DT);
        }
    }

    #[test]
    fn a_level_ends_after_its_time() {
        let mut g = HuskyBounce::new(1);
        run(&mut g, LEVEL_SECONDS - 1.0);
        assert_eq!(g.finished(), None);
        run(&mut g, 1.5);
        assert!(g.finished().is_some());
    }

    #[test]
    fn it_starts_simple_and_ends_in_chaos() {
        let mut g = HuskyBounce::new(1);
        assert_eq!(g.balls.len(), 1, "one ball");
        run(&mut g, 10.0);
        let early = g.balls.len();
        run(&mut g, LEVEL_SECONDS - 20.0);
        let late = g.balls.len();
        assert!(early < 10, "early: {early} balls");
        assert!(late > 300, "late: {late} balls");
        assert!(late <= MAX_BALLS);
        assert!(g.escaped > 0, "the swarm gets out");
    }

    #[test]
    fn the_same_level_plays_the_same() {
        let (mut a, mut b) = (HuskyBounce::new(3), HuskyBounce::new(3));
        run(&mut a, 60.0);
        run(&mut b, 60.0);
        assert_eq!(a.balls.len(), b.balls.len());
        assert_eq!(a.escaped, b.escaped);
        assert!(a.balls.iter().zip(&b.balls).all(|(x, y)| x.pos == y.pos));
        assert_eq!(a.rings.len(), 3, "level 3 has three rings");
    }

    #[test]
    fn balls_stay_inside_their_ring() {
        let mut g = HuskyBounce::new(2);
        for _ in 0..(30.0 / FIXED_DT) as u32 {
            g.update(FIXED_DT);
            for ball in g.balls.iter().filter(|b| b.zone < g.rings.len()) {
                let r = g.rings[ball.zone].radius;
                assert!((ball.pos - CENTRE).length() <= r + 1.0, "out of its ring");
            }
        }
    }

    #[test]
    fn notes_are_spaced_and_input_adds_a_ball() {
        let mut g = HuskyBounce::new(1);
        run(&mut g, 30.0);
        let sounds = g.take_sounds();
        assert!(!sounds.is_empty());
        assert!(sounds.len() as f32 <= 30.0 / NOTE_GAP + 30.0);
        let before = g.balls.len();
        g.handle(&Input {
            keys: vec![Key::Space],
            ..Default::default()
        });
        assert_eq!(g.balls.len(), before + 1);
    }
}
