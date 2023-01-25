use std::ops::ControlFlow;

use crate::terminal::input::*;

#[derive(Default)]
pub struct Parser {
    events: Vec<Event>,
    sequence: Sequence,
}

#[derive(Default)]
enum Sequence {
    #[default]
    Char,
    Escape,
    Control,
    Mouse(Mouse),
    DeviceControl(DeviceControl),
}

#[derive(Debug)]
pub enum TerminalEvent {
    Name(String),
    TrueColorSupported,
}

#[derive(Debug)]
pub enum Event {
    KeyPress { key: u8 },
    MouseUp { row: usize, col: usize },
    MouseDown { row: usize, col: usize },
    MouseMove { row: usize, col: usize },
    Scroll { delta: isize },
    Terminal(TerminalEvent),
    Exit,
}

pub type ParseControlFlow = ControlFlow<Option<Event>, Option<Event>>;

impl Parser {
    pub fn new() -> Parser {
        Self::default()
    }

    pub fn parse(&mut self, input: &[u8]) -> Vec<Event> {
        let mut sequence = std::mem::take(&mut self.sequence);

        macro_rules! emit {
            ($event:expr) => {{
                self.events.push($event);
                Sequence::Char
            }};
            ($event:expr; continue) => {{
                self.events.push($event);
                continue;
            }};
        }
        macro_rules! parse {
            ($flow:expr) => (
                match $flow {
                    ControlFlow::Break(None) => Sequence::Char,
                    ControlFlow::Break(Some(event)) => emit!(event),
                    ControlFlow::Continue(None) => continue,
                    ControlFlow::Continue(Some(event)) => emit!(event; continue),
                }
            );
        }

        for &key in input {
            sequence = match sequence {
                Sequence::Char => match key {
                    0x1b => Sequence::Escape,
                    0x03 => emit!(Event::Exit),
                    key => emit!(Event::KeyPress { key }),
                },
                Sequence::Escape => match key {
                    b'[' => Sequence::Control,
                    b'P' => Sequence::DeviceControl(DeviceControl::new()),
                    0x1b => emit!(Event::KeyPress { key: 0x1b }; continue),
                    key => {
                        emit!(Event::KeyPress { key: 0x1b });
                        emit!(Event::KeyPress { key })
                    }
                },
                Sequence::Control => match key {
                    b'<' => Sequence::Mouse(Mouse::new()),
                    b'A' => emit!(Event::KeyPress { key: 0x26 }),
                    b'B' => emit!(Event::KeyPress { key: 0x28 }),
                    b'C' => emit!(Event::KeyPress { key: 0x27 }),
                    b'D' => emit!(Event::KeyPress { key: 0x25 }),
                    _ => Sequence::Char,
                },
                Sequence::Mouse(ref mut mouse) => parse!(mouse.parse(key)),
                Sequence::DeviceControl(ref mut dcs) => parse!(dcs.parse(key)),
            }
        }

        self.sequence = sequence;

        std::mem::take(&mut self.events)
    }
}
