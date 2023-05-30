use {
    core::mem::forget,
    inquire::Select,
    midir::{MidiIO, MidiInput, MidiInputPort, MidiOutput, MidiOutputConnection, MidiOutputPort},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, serde::Serialize, serde::Deserialize)]
pub enum Pad {
    // Top (left -> right)
    Up,
    Down,
    Left,
    Right,
    Session,
    Note,
    Device,
    User,

    // Bottom (left -> right)
    RecordArm,
    TrackSelect,
    Mute,
    Solo,
    Volume,
    Pan,
    Sends,
    StopClip,

    // Left (top -> bottom)
    Shift,
    Click,
    Undo,
    Delete,
    Quantise,
    Duplicate,
    Double,
    Record,

    // Right (top -> bottom)
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,

    // Grid (left -> right, bottom -> top, 1-indexed)
    Grid(u8, u8),
}

impl Pad {
    pub fn parse(byte: u8) -> Self {
        match byte {
            91 => Self::Up,
            92 => Self::Down,
            93 => Self::Left,
            94 => Self::Right,
            95 => Self::Session,
            96 => Self::Note,
            97 => Self::Device,
            98 => Self::User,

            1 => Self::RecordArm,
            2 => Self::TrackSelect,
            3 => Self::Mute,
            4 => Self::Solo,
            5 => Self::Volume,
            6 => Self::Pan,
            7 => Self::Sends,
            8 => Self::StopClip,

            80 => Self::Shift,
            70 => Self::Click,
            60 => Self::Undo,
            50 => Self::Delete,
            40 => Self::Quantise,
            30 => Self::Duplicate,
            20 => Self::Double,
            10 => Self::Record,

            89 => Self::A,
            79 => Self::B,
            69 => Self::C,
            59 => Self::D,
            49 => Self::E,
            39 => Self::F,
            29 => Self::G,
            19 => Self::H,

            x => Self::Grid(x % 10, x / 10),
        }
    }

    pub fn byte(self) -> u8 {
        match self {
            Self::Up => 91,
            Self::Down => 92,
            Self::Left => 93,
            Self::Right => 94,
            Self::Session => 95,
            Self::Note => 96,
            Self::Device => 97,
            Self::User => 98,

            Self::RecordArm => 1,
            Self::TrackSelect => 2,
            Self::Mute => 3,
            Self::Solo => 4,
            Self::Volume => 5,
            Self::Pan => 6,
            Self::Sends => 7,
            Self::StopClip => 8,

            Self::Shift => 80,
            Self::Click => 70,
            Self::Undo => 60,
            Self::Delete => 50,
            Self::Quantise => 40,
            Self::Duplicate => 30,
            Self::Double => 20,
            Self::Record => 10,

            Self::A => 89,
            Self::B => 79,
            Self::C => 69,
            Self::D => 59,
            Self::E => 49,
            Self::F => 39,
            Self::G => 29,
            Self::H => 19,

            Self::Grid(x, y) => x + y * 10,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Color {
    Palette(u8),
    Rgb(u8, u8, u8),
}

#[allow(unused)]
impl Color {
    pub const BLUE: Self = Self::Palette(45);
    pub const CYAN: Self = Self::Palette(37);
    pub const GREEN: Self = Self::Palette(21);
    pub const OFF: Self = Self::Palette(0);
    pub const ORANGE: Self = Self::Palette(9);
    pub const PINK: Self = Self::Palette(53);
    pub const PURPLE: Self = Self::Palette(48);
    pub const RED: Self = Self::Palette(5);
    pub const WHITE: Self = Self::Palette(3);
    pub const YELLOW: Self = Self::Palette(13);
}

pub struct Led(MidiOutputConnection, Vec<u8>);

impl Led {
    /// Channel 1 Mask
    const CHANNEL: u8 = 0;
    /// Header used in LED SysEx messages (push the SysEx header first)
    const LED_HEADER: &'static [u8] = &[0x00, 0x20, 0x29, 0x02, 0x10];
    /// Header byte for SysEx messages
    const SYSEX_HEADING: u8 = 0xF0;
    /// Trailing byte for SysEx messages
    const SYSEX_TRAILING: u8 = 0xF7;

    pub fn new(port: MidiOutputPort) -> Self {
        Self(
            MidiOutput::new("StarPad Output")
                .expect("could not create output client")
                .connect(&port, "StarPad")
                .expect("could not connect to output"),
            Vec::with_capacity(32),
        )
    }

    // Stream util

    fn tx(&mut self) {
        self.0.send(&self.1).expect("could not transmit");
        self.1.clear();
    }

    fn tx_sysex(&mut self) {
        self.1.insert(0, Self::SYSEX_HEADING);
        self.1.push(Self::SYSEX_TRAILING);
        self.tx();
    }

    fn write_byte(&mut self, byte: u8) { self.1.push(byte); }

    fn write(&mut self, bytes: &[u8]) { self.1.extend_from_slice(bytes); }

    // Message utils

    fn note(&mut self, pad: Pad, vel: u8) {
        self.write_byte(
            (if matches!(pad, Pad::Grid(..)) { 0b1001_0000 } else { 0b1011_0000 }) | Self::CHANNEL,
        );
        self.write_byte(pad.byte());
        self.write_byte(vel);
        self.tx();
    }

    // Methods

    pub fn clear(&mut self) {
        self.write(Self::LED_HEADER);
        self.write_byte(0x0E);
        self.write_byte(0); // color (palette)
        self.tx_sysex();
    }

    pub fn set(&mut self, pad: Pad, col: Color) {
        match col {
            Color::Palette(vel) => self.note(pad, vel),

            Color::Rgb(r, g, b) => {
                self.write(Self::LED_HEADER);
                self.write_byte(0x0B);
                self.write_byte(pad.byte());
                self.write(&[r, g, b]);
                self.tx_sysex();
            }
        }
    }
}

pub trait PadInterface: Sized + Send + 'static {
    fn push(&mut self, _: Pad, _: u8) {}
    fn aftertouch(&mut self, _: Pad, _: u8) {}
    fn release(&mut self, _: Pad) {}

    fn run(mut self, port: MidiInputPort) {
        let input = MidiInput::new("StarPad Input").expect("could not create input client");
        forget(
            input
                .connect(
                    &port,
                    "StarPad",
                    move |_, data, _| {
                        if data.len() < 3 {
                            return;
                        }

                        let (sig, chan) = (data[0] & 0b1111_0000, data[0] & 0b0000_1111);

                        if chan != 0 {
                            return;
                        }

                        let (pad, vel) = (Pad::parse(data[1]), data[2]);

                        match sig {
                            0b1001_0000 | 0b1011_0000 => {
                                if vel != 0 {
                                    self.push(pad, vel)
                                } else {
                                    self.release(pad);
                                }
                            }
                            0b1000_0000 => self.release(pad),
                            0b1010_0000 => self.aftertouch(pad, vel),
                            _ => (),
                        }
                    },
                    (),
                )
                .expect("could not connect to output"),
        );
    }
}

// CLI

struct Port<T>(String, T);

impl<T> core::fmt::Display for Port<T> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { self.0.fmt(fmt) }
}

impl<T> core::fmt::Debug for Port<T> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { self.0.fmt(fmt) }
}

pub fn prompt<T: MidiIO>(str: &str, cli: T) -> T::Port {
    let ports =
        cli.ports().into_iter().map(|port| Port(cli.port_name(&port).unwrap(), port)).collect();
    let sel = Select::new(str, ports);
    sel.prompt().unwrap().1
}
