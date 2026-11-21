use horizon_mod_sdk::{
    Mod, Registration, game,
    render::{self, FillShape, Rect, Shape},
};

// How strong the colours are, 0 to 100. Lower it for a lighter look.
const STRENGTH: u32 = 100;
// How fast moods fade in and out. Higher is faster (0.01 to 0.1).
const FADE: f32 = 0.02;
// How far to look for ground and trees to read the mood from, in blocks.
const LOOK: f32 = 48.0;

type Rgba = (f32, f32, f32, f32);

/// A colour mood for the kind of ground or trees you are looking at.
fn mood_for_block(block: &str) -> Option<Rgba> {
    let b = block.strip_prefix("minecraft:").unwrap_or(block);
    let has = |s: &str| b.contains(s);
    if has("sand") || has("terracotta") {
        Some((240.0, 185.0, 95.0, 20.0)) // desert and badlands: warm dust
    } else if has("snow") || has("ice") {
        Some((170.0, 210.0, 255.0, 20.0)) // cold
    } else if has("mud") || has("mangrove") {
        Some((50.0, 105.0, 85.0, 24.0)) // swamp: murky green
    } else if has("jungle") || has("bamboo") {
        Some((60.0, 165.0, 75.0, 18.0)) // jungle: lush green
    } else if has("cherry") {
        Some((255.0, 170.0, 200.0, 18.0)) // cherry grove: pink
    } else if has("spruce") || has("podzol") || has("coarse_dirt") {
        Some((75.0, 115.0, 105.0, 16.0)) // taiga: cool green-grey
    } else if has("mycelium") {
        Some((175.0, 100.0, 205.0, 20.0)) // mushroom fields: purple
    } else {
        None
    }
}

fn approach(from: f32, to: f32) -> f32 {
    from + (to - from) * FADE
}

struct Atmosphere {
    shown: bool,
    last: [u32; 6],
    mood: Rgba,
    target: Rgba,
    plain_frames: u32,
}

impl Mod for Atmosphere {
    fn init() -> Result<(Self, Registration), String> {
        Ok((
            Atmosphere {
                shown: false,
                last: [0; 6],
                mood: (0.0, 0.0, 0.0, 0.0),
                target: (0.0, 0.0, 0.0, 0.0),
                plain_frames: 0,
            },
            Registration::new().on_frame(),
        ))
    }

    fn frame(&mut self, _seconds: f32) {
        // Nothing to tint on the title screen.
        let Some(player) = game::get_player() else {
            if self.shown {
                render::set_hud(&[]);
                self.shown = false;
            }
            return;
        };
        let strength = STRENGTH.min(100);

        let dim = player.dimension.as_str();
        let mut next: Option<Rgba> = None;
        if dim.ends_with("the_nether") {
            next = Some((200.0, 60.0, 30.0, 34.0)); // hot red haze
        } else if dim.ends_with("the_end") {
            next = Some((130.0, 90.0, 200.0, 30.0)); // violet
        } else if player.y < 40.0 {
            next = Some((20.0, 25.0, 55.0, 28.0)); // deep and cool
        } else if player.y > 150.0 {
            next = Some((200.0, 225.0, 255.0, 10.0)); // thin, bright air
        } else if let Some(hit) = game::raycast(LOOK) {
            match mood_for_block(&hit.block) {
                Some(m) => {
                    self.plain_frames = 0;
                    next = Some(m);
                }
                None => {
                    // Plain ground for a while: let the mood fade away.
                    self.plain_frames += 1;
                    if self.plain_frames > 240 {
                        next = Some((0.0, 0.0, 0.0, 0.0));
                    }
                }
            }
        }
        if let Some(t) = next {
            self.target = t;
        }
        self.mood = (
            approach(self.mood.0, self.target.0),
            approach(self.mood.1, self.target.1),
            approach(self.mood.2, self.target.2),
            approach(self.mood.3, self.target.3),
        );

        let er = self.mood.0.round() as u32;
        let eg = self.mood.1.round() as u32;
        let eb = self.mood.2.round() as u32;
        let ea = (self.mood.3.round() as u32) * strength / 100;
        if ea == 0 {
            if self.shown {
                render::set_hud(&[]);
                self.shown = false;
            }
            return;
        }
        let (w, h) = render::gui_size();
        let key = [er, eg, eb, ea, w as u32, h as u32];
        if self.shown && key == self.last {
            return;
        }
        self.last = key;
        self.shown = true;

        let color = (ea << 24) | (er << 16) | (eg << 8) | eb;
        render::set_hud(&[Shape::Fill(FillShape {
            rect: Rect { x: 0.0, y: 0.0, w, h },
            color,
        })]);
    }
}

horizon_mod_sdk::export_mod!(Atmosphere);
