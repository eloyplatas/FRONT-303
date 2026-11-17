//! The panel's drawing: materials, knobs, buttons, LEDs and displays, all painted from
//! vectors and procedural textures so they stay sharp at any size.
//!
//! Every function takes design coordinates (the panel is designed at 1280 × 860) through a
//! [`Pen`], which scales and places them in the window.

use egui::epaint::{Mesh, PathShape, PathStroke};
use egui::{
    Align2, Color32, ColorImage, Context, CornerRadius, FontFamily, FontId, Painter, Pos2, Rect,
    Shape, Stroke, StrokeKind, TextureHandle, TextureOptions, Vec2, pos2, vec2,
};

/// Design coordinates to window coordinates.
#[derive(Clone)]
pub struct Pen {
    pub painter: Painter,
    pub scale: f32,
    pub origin: Pos2,
}

pub fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

pub fn rgba(hex: u32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, a)
}

/// `a` towards `b` by `t`.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        f(a.r(), b.r()),
        f(a.g(), b.g()),
        f(a.b(), b.b()),
        f(a.a(), b.a()),
    )
}

pub const BOLD: &str = "bold";
pub const MEDIUM: &str = "medium";

impl Pen {
    pub fn p(&self, x: f32, y: f32) -> Pos2 {
        self.origin + vec2(x, y) * self.scale
    }

    pub fn r(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(self.p(x, y), vec2(w, h) * self.scale)
    }

    /// A window rectangle back in design coordinates.
    pub fn design(&self, r: Rect) -> (f32, f32, f32, f32) {
        let min = (r.min - self.origin) / self.scale;
        (
            min.x,
            min.y,
            r.width() / self.scale,
            r.height() / self.scale,
        )
    }

    pub fn s(&self, x: f32) -> f32 {
        x * self.scale
    }

    pub fn font(&self, family: &str, size: f32) -> FontId {
        FontId::new(size * self.scale, FontFamily::Name(family.into()))
    }

    pub fn text(
        &self,
        x: f32,
        y: f32,
        align: Align2,
        text: &str,
        family: &str,
        size: f32,
        color: Color32,
    ) {
        self.painter
            .text(self.p(x, y), align, text, self.font(family, size), color);
    }

    /// Text pressed into a metal panel: a light edge below, the letters above it.
    pub fn engraved(
        &self,
        x: f32,
        y: f32,
        align: Align2,
        text: &str,
        size: f32,
        color: Color32,
        light: Color32,
    ) {
        self.text(x, y + 0.8, align, text, BOLD, size, light);
        self.text(x, y, align, text, BOLD, size, color);
    }

    pub fn fill(&self, r: Rect, radius: f32, color: Color32) {
        self.painter
            .rect_filled(r, CornerRadius::same(self.s(radius).round() as u8), color);
    }

    pub fn stroke(&self, r: Rect, radius: f32, width: f32, color: Color32) {
        self.painter.rect_stroke(
            r,
            CornerRadius::same(self.s(radius).round() as u8),
            Stroke::new(self.s(width), color),
            StrokeKind::Inside,
        );
    }

    pub fn line(&self, a: Pos2, b: Pos2, width: f32, color: Color32) {
        self.painter
            .line_segment([a, b], Stroke::new(self.s(width), color));
    }

    pub fn circle(&self, c: Pos2, r: f32, color: Color32) {
        self.painter.circle_filled(c, self.s(r), color);
    }

    /// A rectangle shaded from `top` to `bottom`.
    pub fn vgradient(&self, r: Rect, top: Color32, bottom: Color32) {
        let mut m = Mesh::default();
        m.colored_vertex(r.left_top(), top);
        m.colored_vertex(r.right_top(), top);
        m.colored_vertex(r.right_bottom(), bottom);
        m.colored_vertex(r.left_bottom(), bottom);
        m.add_triangle(0, 1, 2);
        m.add_triangle(0, 2, 3);
        self.painter.add(Shape::mesh(m));
    }

    /// A rectangle shaded through several colours from top to bottom.
    pub fn vstops(&self, r: Rect, stops: &[(f32, Color32)]) {
        let mut m = Mesh::default();
        for (i, &(t, c)) in stops.iter().enumerate() {
            let y = r.top() + r.height() * t;
            m.colored_vertex(pos2(r.left(), y), c);
            m.colored_vertex(pos2(r.right(), y), c);
            if i > 0 {
                let k = (2 * i) as u32;
                m.add_triangle(k - 2, k - 1, k + 1);
                m.add_triangle(k - 2, k + 1, k);
            }
        }
        self.painter.add(Shape::mesh(m));
    }

    /// A disc shaded from `inner` at `centre + offset` to `outer` at its rim.
    pub fn radial(&self, c: Pos2, r: f32, offset: Vec2, inner: Color32, outer: Color32) {
        let r = self.s(r);
        let n = 48;
        let mut m = Mesh::default();
        m.colored_vertex(c + offset * self.scale, inner);
        for i in 0..n {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            m.colored_vertex(c + vec2(a.cos(), a.sin()) * r, outer);
        }
        for i in 0..n {
            m.add_triangle(0, 1 + i, 1 + (i + 1) % n);
        }
        self.painter.add(Shape::mesh(m));
    }

    /// A soft shadow: rings of falling opacity round a rounded rectangle.
    pub fn shadow(&self, r: Rect, radius: f32, spread: f32, alpha: u8) {
        let steps = 6;
        for i in (1..=steps).rev() {
            let grow = self.s(spread) * i as f32 / steps as f32;
            let a =
                (alpha as f32 * (1.0 - i as f32 / (steps as f32 + 1.0)) / steps as f32 * 2.0) as u8;
            self.painter.rect_filled(
                r.expand(grow).translate(vec2(0.0, grow * 0.5)),
                CornerRadius::same((self.s(radius) + grow).round().min(255.0) as u8),
                Color32::from_black_alpha(a),
            );
        }
    }

    pub fn texture(&self, tex: &TextureHandle, r: Rect, uv: Rect, tint: Color32) {
        self.painter.image(tex.id(), r, uv, tint);
    }
}

/// The procedural textures, made once per editor.
pub struct Textures {
    pub brushed: TextureHandle,
    pub wood: TextureHandle,
    pub grain: TextureHandle,
}

/// Value noise in 0..1 from a lattice of hashed points.
fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8DA6_B343)
        ^ (y as u32).wrapping_mul(0xD816_3841)
        ^ seed.wrapping_mul(0xCB1A_B31F);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5BD1_E995);
    h ^= h >> 15;
    (h & 0xFFFF) as f32 / 65535.0
}

fn smooth_noise(x: f32, y: f32, seed: u32) -> f32 {
    let (x0, y0) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let (sx, sy) = (s(fx), s(fy));
    let a = hash(x0, y0, seed) + (hash(x0 + 1, y0, seed) - hash(x0, y0, seed)) * sx;
    let b = hash(x0, y0 + 1, seed) + (hash(x0 + 1, y0 + 1, seed) - hash(x0, y0 + 1, seed)) * sx;
    a + (b - a) * sy
}

impl Textures {
    pub fn new(ctx: &Context) -> Self {
        // brushed aluminium: fine horizontal streaks, in grey levels round the middle
        let (w, h) = (1024, 256);
        let mut brushed = ColorImage::filled([w, h], Color32::GRAY);
        for y in 0..h {
            let row = hash(0, y as i32, 7) * 0.5 + hash(1, y as i32 / 2, 8) * 0.5;
            for x in 0..w {
                let streak = smooth_noise(x as f32 / 90.0, y as f32 * 1.7, 3);
                let fine = hash(x as i32, y as i32, 9);
                let v = 0.5 + (row - 0.5) * 0.55 + (streak - 0.5) * 0.35 + (fine - 0.5) * 0.12;
                let g = (128.0 + (v - 0.5) * 120.0).clamp(0.0, 255.0) as u8;
                brushed[(x, y)] = Color32::from_gray(g);
            }
        }
        // wood: dark walnut with long grain and a few rings, along the vertical
        let (w, h) = (128, 1024);
        let mut wood = ColorImage::filled([w, h], Color32::BLACK);
        for y in 0..h {
            for x in 0..w {
                let warp = smooth_noise(x as f32 / 20.0, y as f32 / 140.0, 11) * 6.0;
                let grain = ((x as f32 / 3.1 + warp * 2.2).sin() * 0.5 + 0.5).powf(1.6);
                let fibre = smooth_noise(x as f32 / 1.6, y as f32 / 40.0, 12);
                let knot = smooth_noise(x as f32 / 30.0, y as f32 / 60.0, 13);
                let v = 0.35 + grain * 0.3 + fibre * 0.22 + knot * 0.13;
                let base = rgb(0x4a2416);
                let light = rgb(0x9a5a33);
                wood[(x, y)] = mix(base, light, v.clamp(0.0, 1.0) * 0.9);
            }
        }
        // fine grain for the dark painted panels
        let (w, h) = (256, 256);
        let mut grain = ColorImage::filled([w, h], Color32::GRAY);
        for y in 0..h {
            for x in 0..w {
                let n = hash(x as i32, y as i32, 21) * 0.6
                    + smooth_noise(x as f32 / 8.0, y as f32 / 8.0, 22) * 0.4;
                let g = (128.0 + (n - 0.5) * 40.0) as u8;
                grain[(x, y)] = Color32::from_gray(g);
            }
        }
        Textures {
            brushed: ctx.load_texture("front303-brushed", brushed, TextureOptions::LINEAR_REPEAT),
            wood: ctx.load_texture("front303-wood", wood, TextureOptions::LINEAR_REPEAT),
            grain: ctx.load_texture("front303-grain", grain, TextureOptions::LINEAR_REPEAT),
        }
    }
}

/// A brushed aluminium plate with bevelled edges.
pub fn metal_plate(pen: &Pen, t: &Textures, r: Rect, tint: Color32) {
    pen.shadow(r, 3.0, 6.0, 140);
    pen.fill(r, 3.0, tint);
    let (_, _, w, h) = pen.design(r);
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(w / 900.0, h / 220.0));
    pen.texture(&t.brushed, r.shrink(pen.s(1.0)), uv, rgba(0xffffff, 120));
    // light falls from above: a sheen across the top, darker towards the bottom
    pen.vstops(
        r.shrink(pen.s(1.0)),
        &[
            (0.0, rgba(0xffffff, 70)),
            (0.12, rgba(0xffffff, 18)),
            (0.6, rgba(0x000000, 0)),
            (1.0, rgba(0x000000, 40)),
        ],
    );
    pen.stroke(r, 3.0, 1.0, rgba(0x000000, 160));
    pen.line(
        r.left_top() + vec2(pen.s(2.0), pen.s(1.5)),
        r.right_top() + vec2(-pen.s(2.0), pen.s(1.5)),
        1.0,
        rgba(0xffffff, 150),
    );
}

/// A dark painted module with a fine grain.
pub fn dark_plate(pen: &Pen, t: &Textures, r: Rect, top: Color32, bottom: Color32) {
    pen.shadow(r, 4.0, 6.0, 150);
    pen.vgradient(r, top, bottom);
    let (_, _, w, h) = pen.design(r);
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(w / 256.0, h / 256.0));
    pen.texture(&t.grain, r, uv, rgba(0xffffff, 26));
    pen.stroke(r, 4.0, 1.0, rgba(0x000000, 200));
    pen.line(
        r.left_top() + vec2(pen.s(3.0), pen.s(1.0)),
        r.right_top() + vec2(-pen.s(3.0), pen.s(1.0)),
        1.0,
        rgba(0xffffff, 40),
    );
}

/// A recessed area: darker, with an inner shadow along its top edge.
pub fn well(pen: &Pen, r: Rect, color: Color32, radius: f32) {
    pen.fill(r, radius, color);
    pen.vstops(
        r,
        &[
            (0.0, rgba(0x000000, 120)),
            (0.15, rgba(0x000000, 0)),
            (1.0, rgba(0x000000, 0)),
        ],
    );
    pen.stroke(r, radius, 1.0, rgba(0x000000, 180));
    pen.line(
        r.left_bottom() + vec2(pen.s(radius), 0.0),
        r.right_bottom() - vec2(pen.s(radius), 0.0),
        1.0,
        rgba(0xffffff, 50),
    );
}

pub fn screw(pen: &Pen, c: Pos2) {
    pen.circle(c + vec2(0.0, pen.s(0.8)), 4.6, rgba(0x000000, 120));
    pen.radial(c, 4.0, vec2(-1.2, -1.4), rgb(0xe8e8e8), rgb(0x6a6a6a));
    let a = 0.6f32;
    let d = vec2(a.cos(), a.sin()) * pen.s(3.0);
    pen.line(c - d, c + d, 1.2, rgb(0x333333));
}

/// Wood end cheek.
pub fn cheek(pen: &Pen, t: &Textures, r: Rect, flip: bool) {
    pen.fill(r, 2.0, rgb(0x2a140c));
    let uv = if flip {
        Rect::from_min_max(pos2(1.0, 0.0), pos2(0.6, 0.85))
    } else {
        Rect::from_min_max(pos2(0.0, 0.0), pos2(0.4, 0.85))
    };
    pen.texture(&t.wood, r, uv, Color32::WHITE);
    // a rounded edge: light on the outer face, shade on the inner
    let edge = pen.s(5.0);
    let (outer, inner) = if flip {
        (r.right(), r.left())
    } else {
        (r.left(), r.right())
    };
    let o = Rect::from_x_y_ranges(
        outer.min(outer + if flip { -edge } else { edge })
            ..=outer.max(outer + if flip { -edge } else { edge }),
        r.y_range(),
    );
    pen.fill(o, 0.0, rgba(0xffffff, 18));
    let i = Rect::from_x_y_ranges(
        inner.min(inner + if flip { edge } else { -edge })
            ..=inner.max(inner + if flip { edge } else { -edge }),
        r.y_range(),
    );
    pen.fill(i, 0.0, rgba(0x000000, 90));
    pen.vstops(
        r,
        &[
            (0.0, rgba(0xffffff, 30)),
            (0.05, rgba(0, 0)),
            (0.9, rgba(0, 0)),
            (1.0, rgba(0x000000, 90)),
        ],
    );
}

/// The angle of a knob at `t` (0..1): seven o'clock to five o'clock.
pub fn knob_angle(t: f32) -> f32 {
    let start = 0.75 * std::f32::consts::PI;
    start + t.clamp(0.0, 1.0) * 1.5 * std::f32::consts::PI
}

fn dir(a: f32) -> Vec2 {
    vec2(a.cos(), a.sin())
}

/// The scale printed round a knob.
pub fn knob_scale(pen: &Pen, c: Pos2, r: f32, ticks: usize, color: Color32) {
    for i in 0..ticks {
        let t = i as f32 / (ticks - 1) as f32;
        let d = dir(knob_angle(t));
        let long = i == 0 || i == ticks - 1 || i == ticks / 2;
        let (a, b) = (r + 4.0, r + if long { 9.0 } else { 7.0 });
        pen.line(
            c + d * pen.s(a),
            c + d * pen.s(b),
            if long { 1.6 } else { 1.1 },
            color,
        );
    }
}

/// The TB-303's knob: a black ribbed skirt with a metal cap and a white line.
pub fn knob(pen: &Pen, c: Pos2, r: f32, t: f32, accent: Color32, lit: bool) {
    // shadow cast down and to the right
    for i in 0..4 {
        let k = i as f32;
        pen.circle(
            c + vec2(pen.s(1.0 + k), pen.s(2.0 + k * 1.2)),
            r + 1.0 + k * 0.6,
            rgba(0x000000, 50 - i * 10),
        );
    }
    // the skirt
    pen.radial(c, r, vec2(-r * 0.3, -r * 0.4), rgb(0x3c3c3e), rgb(0x0b0b0c));
    let ribs = 28;
    for i in 0..ribs {
        let a = i as f32 / ribs as f32 * std::f32::consts::TAU + knob_angle(t);
        let d = dir(a);
        let light = (0.5 + 0.5 * (a + 2.3).cos()).powf(2.0);
        let col = mix(rgba(0x000000, 160), rgba(0x8a8a8e, 150), light);
        pen.line(c + d * pen.s(r * 0.80), c + d * pen.s(r * 0.98), 1.1, col);
    }
    pen.painter
        .circle_stroke(c, pen.s(r), Stroke::new(pen.s(1.0), rgba(0x000000, 220)));
    // the top: a dark dome with the metal cap
    let top = r * 0.74;
    pen.radial(
        c,
        top,
        vec2(-top * 0.35, -top * 0.45),
        rgb(0x2e2e30),
        rgb(0x0d0d0e),
    );
    let cap = r * 0.46;
    pen.radial(
        c,
        cap,
        vec2(-cap * 0.4, -cap * 0.5),
        rgb(0xf4f4f4),
        rgb(0x7e8084),
    );
    // concentric turning marks on the cap
    for k in [0.35, 0.6, 0.85] {
        pen.painter.circle_stroke(
            c,
            pen.s(cap * k),
            Stroke::new(pen.s(0.5), rgba(0x000000, 28)),
        );
    }
    pen.painter
        .circle_stroke(c, pen.s(cap), Stroke::new(pen.s(0.8), rgba(0x000000, 150)));
    // the indicator: a groove across the cap, and a white line on the dome beyond it
    let d = dir(knob_angle(t));
    let line = if lit {
        mix(Color32::WHITE, accent, 0.35)
    } else {
        Color32::WHITE
    };
    pen.line(
        c + d * pen.s(cap * 0.15),
        c + d * pen.s(cap * 0.95),
        2.2,
        rgba(0x1a1a1c, 230),
    );
    pen.line(
        c + d * pen.s(cap * 0.15) + vec2(0.0, pen.s(0.7)),
        c + d * pen.s(cap * 0.95) + vec2(0.0, pen.s(0.7)),
        0.8,
        rgba(0xffffff, 120),
    );
    pen.line(
        c + d * pen.s(cap * 1.05),
        c + d * pen.s(top * 0.98),
        3.0,
        rgba(0x000000, 140),
    );
    pen.line(
        c + d * pen.s(cap * 1.05),
        c + d * pen.s(top * 0.96),
        2.2,
        line,
    );
}

/// A small knob for the dark modules: black with a coloured pointer.
pub fn small_knob(pen: &Pen, c: Pos2, r: f32, t: f32, accent: Color32) {
    for i in 0..3 {
        let k = i as f32;
        pen.circle(
            c + vec2(pen.s(0.8 + k), pen.s(1.6 + k)),
            r + 0.8 + k * 0.5,
            rgba(0x000000, 60 - i * 15),
        );
    }
    pen.radial(
        c,
        r,
        vec2(-r * 0.35, -r * 0.45),
        rgb(0x55575c),
        rgb(0x141416),
    );
    pen.radial(
        c,
        r * 0.72,
        vec2(-r * 0.25, -r * 0.35),
        rgb(0x3a3b3f),
        rgb(0x1a1b1d),
    );
    pen.painter
        .circle_stroke(c, pen.s(r), Stroke::new(pen.s(0.9), rgba(0x000000, 220)));
    let d = dir(knob_angle(t));
    pen.line(
        c + d * pen.s(r * 0.25),
        c + d * pen.s(r * 0.92),
        2.0,
        accent,
    );
}

/// An arc round a knob showing its value, from its minimum or from its centre.
pub fn value_arc(pen: &Pen, c: Pos2, r: f32, t: f32, bipolar: bool, color: Color32) {
    let from = if bipolar { 0.5 } else { 0.0 };
    let (a, b) = if t >= from { (from, t) } else { (t, from) };
    let n = ((b - a) * 40.0).ceil().max(1.0) as usize;
    let pts: Vec<Pos2> = (0..=n)
        .map(|i| c + dir(knob_angle(a + (b - a) * i as f32 / n as f32)) * pen.s(r))
        .collect();
    pen.painter.add(Shape::Path(PathShape::line(
        pts,
        PathStroke::new(pen.s(2.2), color),
    )));
}

/// A light: off it is a dark lens; on it glows.
pub fn led(pen: &Pen, c: Pos2, r: f32, color: Color32, level: f32) {
    let level = level.clamp(0.0, 1.0);
    pen.circle(c, r + 1.2, rgba(0x000000, 200));
    let off = mix(rgb(0x1a1a1a), color, 0.18);
    let on = mix(color, Color32::WHITE, 0.25);
    pen.radial(
        c,
        r,
        vec2(-r * 0.3, -r * 0.35),
        mix(mix(off, Color32::WHITE, 0.15), on, level),
        mix(mix(off, Color32::BLACK, 0.3), color, level),
    );
    if level > 0.05 {
        for i in 1..=4 {
            let k = i as f32;
            let a = (level * 55.0 / k) as u8;
            pen.circle(
                c,
                r + k * 1.8,
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a),
            );
        }
    }
    pen.circle(
        c + vec2(-pen.s(r * 0.35), -pen.s(r * 0.4)),
        r * 0.28,
        rgba(0xffffff, 110),
    );
}

/// A push button: raised cap, pressed or lit.
pub fn button(
    pen: &Pen,
    r: Rect,
    label: &str,
    size: f32,
    cap: Color32,
    lit: Option<Color32>,
    pressed: bool,
) {
    let depth = if pressed { 0.5 } else { 2.0 };
    pen.fill(
        r.translate(vec2(0.0, pen.s(depth))),
        3.0,
        rgba(0x000000, 170),
    );
    let face = r.translate(vec2(0.0, if pressed { pen.s(1.2) } else { 0.0 }));
    let base = match lit {
        Some(l) => mix(cap, l, 0.85),
        None => cap,
    };
    pen.fill(face, 3.0, base);
    pen.vstops(
        face,
        &[
            (0.0, rgba(0xffffff, 70)),
            (0.45, rgba(0xffffff, 10)),
            (1.0, rgba(0x000000, 60)),
        ],
    );
    if let Some(l) = lit {
        pen.painter.rect_filled(
            face.expand(pen.s(2.0)),
            CornerRadius::same(pen.s(5.0) as u8),
            Color32::from_rgba_unmultiplied(l.r(), l.g(), l.b(), 40),
        );
    }
    pen.stroke(face, 3.0, 1.0, rgba(0x000000, 190));
    let text = if lit.is_some() {
        rgb(0x1a1205)
    } else {
        rgb(0xe8e8e8)
    };
    let lum = base.r() as u32 + base.g() as u32 + base.b() as u32;
    let text = if lit.is_none() && lum > 400 {
        rgb(0x202020)
    } else {
        text
    };
    pen.painter.text(
        face.center(),
        Align2::CENTER_CENTER,
        label,
        pen.font(BOLD, size),
        text,
    );
}

/// The segments of a seven-segment digit lit for `ch`: bits a..g.
fn segments(ch: char) -> u8 {
    match ch {
        '0' => 0b0111111,
        '1' => 0b0000110,
        '2' => 0b1011011,
        '3' => 0b1001111,
        '4' => 0b1100110,
        '5' => 0b1101101,
        '6' => 0b1111101,
        '7' => 0b0000111,
        '8' => 0b1111111,
        '9' => 0b1101111,
        '-' => 0b1000000,
        _ => 0,
    }
}

/// A seven-segment display of `text` (digits, '-', ' ' and '.') in `r`.
pub fn seven_segment(pen: &Pen, r: Rect, text: &str, color: Color32) {
    well(pen, r, rgb(0x120606), 3.0);
    let chars: Vec<char> = text.chars().filter(|&c| c != '.').collect();
    let dots: Vec<usize> = {
        let mut out = Vec::new();
        let mut i = 0;
        for c in text.chars() {
            if c == '.' {
                out.push(i.max(1) - 1);
            } else {
                i += 1;
            }
        }
        out
    };
    let n = chars.len().max(1) as f32;
    let inner = r.shrink2(vec2(r.width() * 0.08, r.height() * 0.16));
    let cw = inner.width() / n;
    let h = inner.height();
    let w = cw * 0.62;
    let th = h * 0.11;
    let dim = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 28);
    for (i, &ch) in chars.iter().enumerate() {
        let x0 = inner.left() + cw * i as f32 + (cw - w) * 0.5;
        let y0 = inner.top();
        let skew = |p: Pos2| pos2(p.x + (y0 + h - p.y) * 0.08, p.y);
        let horiz = |x: f32, y: f32| {
            vec![
                skew(pos2(x + th * 0.6, y)),
                skew(pos2(x + th * 1.1, y - th * 0.5)),
                skew(pos2(x + w - th * 1.1, y - th * 0.5)),
                skew(pos2(x + w - th * 0.6, y)),
                skew(pos2(x + w - th * 1.1, y + th * 0.5)),
                skew(pos2(x + th * 1.1, y + th * 0.5)),
            ]
        };
        let vert = |x: f32, y: f32, len: f32| {
            vec![
                skew(pos2(x, y + th * 0.6)),
                skew(pos2(x + th * 0.5, y + th * 1.1)),
                skew(pos2(x + th * 0.5, y + len - th * 1.1)),
                skew(pos2(x, y + len - th * 0.6)),
                skew(pos2(x - th * 0.5, y + len - th * 1.1)),
                skew(pos2(x - th * 0.5, y + th * 1.1)),
            ]
        };
        let half = h / 2.0;
        let segs = [
            horiz(x0, y0 + th * 0.5),
            vert(x0 + w - th * 0.5, y0, half),
            vert(x0 + w - th * 0.5, y0 + half, half),
            horiz(x0, y0 + h - th * 0.5),
            vert(x0 + th * 0.5, y0 + half, half),
            vert(x0 + th * 0.5, y0, half),
            horiz(x0, y0 + half),
        ];
        let on = segments(ch);
        for (k, seg) in segs.into_iter().enumerate() {
            let lit = on & (1 << k) != 0;
            if lit {
                pen.painter.add(Shape::convex_polygon(
                    seg.iter().map(|p| *p + vec2(0.0, 0.0)).collect(),
                    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 60),
                    Stroke::new(
                        pen.s(2.5),
                        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 40),
                    ),
                ));
                pen.painter
                    .add(Shape::convex_polygon(seg, color, Stroke::NONE));
            } else {
                pen.painter
                    .add(Shape::convex_polygon(seg, dim, Stroke::NONE));
            }
        }
        if dots.contains(&i) {
            pen.painter
                .circle_filled(pos2(x0 + w + cw * 0.12, y0 + h - th * 0.4), th * 0.6, color);
        }
    }
    // glass
    pen.vstops(
        r.shrink(pen.s(1.5)),
        &[
            (0.0, rgba(0xffffff, 22)),
            (0.45, rgba(0xffffff, 6)),
            (0.5, rgba(0, 0)),
            (1.0, rgba(0, 0)),
        ],
    );
}

/// A small LCD window with text.
pub fn lcd(pen: &Pen, r: Rect, text: &str, size: f32, color: Color32) {
    well(pen, r, rgb(0x0d1410), 2.5);
    pen.painter.text(
        r.center() + vec2(0.0, pen.s(0.5)),
        Align2::CENTER_CENTER,
        text,
        pen.font(BOLD, size),
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 50),
    );
    pen.painter.text(
        r.center(),
        Align2::CENTER_CENTER,
        text,
        pen.font(BOLD, size),
        color,
    );
    pen.vstops(
        r.shrink(pen.s(1.0)),
        &[
            (0.0, rgba(0xffffff, 18)),
            (0.5, rgba(0xffffff, 4)),
            (0.52, rgba(0, 0)),
            (1.0, rgba(0, 0)),
        ],
    );
}

/// A vertical LED meter of `segs` segments at `level` (0..1, 1 is full scale).
/// A segmented meter, vertical, or horizontal when `r` is wider than tall.
pub fn meter(pen: &Pen, r: Rect, level: f32, segs: usize) {
    well(pen, r, rgb(0x0a0a0a), 2.0);
    let inner = r.shrink(pen.s(2.0));
    let horizontal = inner.width() > inner.height();
    let h = if horizontal {
        inner.width()
    } else {
        inner.height()
    } / segs as f32;
    // -36 dB to 0 dB across the segments
    let db = 20.0 * level.max(1e-6).log10();
    let lit = (((db + 36.0) / 36.0) * segs as f32).clamp(0.0, segs as f32);
    for i in 0..segs {
        let seg = if horizontal {
            let x = inner.left() + h * i as f32;
            Rect::from_min_size(pos2(x + 0.6, inner.top()), vec2(h - 1.2, inner.height()))
        } else {
            let y = inner.bottom() - h * (i as f32 + 1.0);
            Rect::from_min_size(pos2(inner.left(), y + 0.6), vec2(inner.width(), h - 1.2))
        };
        let col = if i + 1 == segs {
            rgb(0xff3020)
        } else if i + 3 >= segs {
            rgb(0xffb020)
        } else {
            rgb(0x40e050)
        };
        let on = (lit - i as f32).clamp(0.0, 1.0);
        pen.painter.rect_filled(
            seg,
            CornerRadius::same(1),
            mix(mix(rgb(0x141414), col, 0.12), col, on),
        );
    }
}
