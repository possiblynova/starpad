use enigo::{Enigo, Key, KeyboardControllable};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
#[serde(try_from = "String", into = "String")]
pub struct KeySequence(Vec<(Key, Action)>);

impl KeySequence {
    pub fn parse(src: &str) -> Result<Self, &'static str> {
        struct State {
            pub actions: Vec<(Key, Action)>,

            pub alt: bool,
            pub ctrl: bool,
            pub shift: bool,

            pub next: Action,
        }

        impl State {
            pub fn push(&mut self, key: Key) {
                match self.next {
                    Action::Press => {
                        if self.alt {
                            self.actions.push((Key::Alt, Action::Press));
                        }
                        if self.ctrl {
                            self.actions.push((Key::Control, Action::Press));
                        }
                        if self.shift {
                            self.actions.push((Key::Shift, Action::Press));
                        }

                        self.actions.push((key, Action::Press));
                    }
                    Action::Release => {
                        self.actions.push((key, Action::Release));

                        if self.alt {
                            self.actions.push((Key::Alt, Action::Release));
                        }
                        if self.ctrl {
                            self.actions.push((Key::Control, Action::Release));
                        }
                        if self.shift {
                            self.actions.push((Key::Shift, Action::Release));
                        }
                    }
                    Action::Tap => {
                        if self.alt {
                            self.actions.push((Key::Alt, Action::Press));
                        }
                        if self.ctrl {
                            self.actions.push((Key::Control, Action::Press));
                        }
                        if self.shift {
                            self.actions.push((Key::Shift, Action::Press));
                        }

                        self.actions.push((key, self.next));

                        if self.alt {
                            self.actions.push((Key::Alt, Action::Release));
                        }
                        if self.ctrl {
                            self.actions.push((Key::Control, Action::Release));
                        }
                        if self.shift {
                            self.actions.push((Key::Shift, Action::Release));
                        }
                    }
                }

                self.alt = false;
                self.ctrl = false;
                self.shift = false;

                self.next = Action::Tap;
            }
        }

        let mut state =
            State { actions: Vec::new(), alt: false, ctrl: false, shift: false, next: Action::Tap };

        let mut iter = src.chars().peekable();

        while let Some(char) = iter.next() {
            match char {
                '!' => state.alt = true,
                '^' => state.ctrl = true,
                '+' => state.shift = true,

                '>' => state.next = Action::Press,
                '<' => state.next = Action::Release,

                '(' => {
                    let mut cc = String::new();

                    loop {
                        let ch = iter.next().ok_or("missing terminator in control code")?;

                        if ch == ')' {
                            break;
                        } else {
                            cc.push(ch);
                        }
                    }

                    state.push(match &*cc {
                        "alt" => Key::Alt,
                        "ctrl" => Key::Control,
                        "shift" => Key::Shift,

                        "enter" => Key::Return,
                        "backspace" => Key::Backspace,
                        "tab" => Key::Tab,
                        "caps" => Key::CapsLock,
                        "space" => Key::Space,
                        "menu" => Key::Apps, // ???

                        "#0" => Key::Numpad0,
                        "#1" => Key::Numpad1,
                        "#2" => Key::Numpad2,
                        "#3" => Key::Numpad3,
                        "#4" => Key::Numpad4,
                        "#5" => Key::Numpad5,
                        "#6" => Key::Numpad6,
                        "#8" => Key::Numpad8,
                        "#9" => Key::Numpad9,

                        "f1" => Key::F1,
                        "f2" => Key::F2,
                        "f3" => Key::F3,
                        "f4" => Key::F4,
                        "f5" => Key::F5,
                        "f6" => Key::F6,
                        "f8" => Key::F8,
                        "f9" => Key::F9,
                        "f10" => Key::F10,
                        "f11" => Key::F11,
                        "f12" => Key::F12,
                        "f13" => Key::F13,
                        "f14" => Key::F14,
                        "f15" => Key::F15,
                        "f16" => Key::F16,
                        "f17" => Key::F17,
                        "f18" => Key::F18,
                        "f19" => Key::F19,
                        "f20" => Key::F20,
                        "f21" => Key::F21,
                        "f22" => Key::F22,
                        "f23" => Key::F23,
                        "f24" => Key::F24,

                        _ => return Err("invalid control code"),
                    });

                    state.alt = false;
                    state.ctrl = false;
                    state.shift = false;

                    state.next = Action::Tap;
                }

                _ => {
                    state.push(Key::Layout(char));

                    state.alt = false;
                    state.ctrl = false;
                    state.shift = false;

                    state.next = Action::Tap;
                }
            }
        }

        Ok(Self(state.actions))
    }

    pub fn exec(&self, enigo: &mut Enigo) {
        for action in self.0.iter() {
            match action.1 {
                Action::Press => enigo.key_down(action.0),
                Action::Release => enigo.key_up(action.0),
                Action::Tap => enigo.key_click(action.0),
            }
        }
    }
}

impl TryFrom<String> for KeySequence {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> { KeySequence::parse(&value) }
}

impl From<KeySequence> for String {
    fn from(seq: KeySequence) -> Self { format!("{seq:?}") }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Press,
    Release,
    Tap,
}
