use {
    crate::{
        dsl::KeySequence,
        interface::{Color, Led, Pad, PadInterface},
    },
    enigo::Enigo,
    std::collections::HashMap,
};

fn default_inactive_color() -> Color { Color::RED }
fn default_active_color() -> Color { Color::GREEN }

fn default_threshold() -> u8 { 0 }

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Bind {
    #[serde(default)]
    pub press: KeySequence,
    #[serde(default)]
    pub release: KeySequence,

    #[serde(default = "default_inactive_color")]
    pub inactive: Color,
    #[serde(default = "default_active_color")]
    pub active: Color,

    #[serde(default = "default_threshold")]
    pub threshold: u8,

    #[serde(skip)]
    pressed: bool,
}

pub struct Binds {
    binds: HashMap<Pad, Bind>,
    led: Led,
    enigo: Enigo,
}

impl Binds {
    pub fn new(binds: HashMap<Pad, Bind>, led: Led) -> Self {
        Self { binds, led, enigo: Enigo::new() }
    }
}

impl PadInterface for Binds {
    fn push(&mut self, pad: Pad, vel: u8) {
        if let Some(bind) = self.binds.get_mut(&pad) {
            if vel >= bind.threshold {
                bind.press.exec(&mut self.enigo);
                self.led.set(pad, bind.active);
                bind.pressed = true;
            }
        }
    }

    fn aftertouch(&mut self, pad: Pad, vel: u8) {
        if let Some(bind) = self.binds.get_mut(&pad) {
            if vel < bind.threshold && bind.pressed {
                self.release(pad);
            }
        }
    }

    fn release(&mut self, pad: Pad) {
        if let Some(bind) = self.binds.get_mut(&pad) {
            if bind.pressed {
                bind.release.exec(&mut self.enigo);
                self.led.set(pad, bind.inactive);
                bind.pressed = false;
            }
        }
    }
}
