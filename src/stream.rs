//! Parsing and serialization wrappers for MIDI streams.

use std::sync::mpsc::{self, Receiver};

use midir::{MidiInput, MidiInputConnection, MidiInputPort};

use crate::midi;

pub struct MidiInputStream {
    _conn: MidiInputConnection<Option<u8>>,
    recv: Receiver<(u64, midi::Message)>,
}

impl MidiInputStream {
    pub fn new(name: &str, input: MidiInput, port: &MidiInputPort) -> Self {
        let (send, recv) = mpsc::sync_channel::<(u64, midi::Message)>(1);

        let conn = input
            .connect(
                port,
                name,
                move |stamp, bytes, status| match midi::Message::decode(bytes, *status) {
                    Ok(msg) => {
                        if let midi::Message::SystemExclusive { id } = msg {
                            eprint!("WARN: received sysex message (id {id:?}): ");
                            for b in &bytes[msg.encoded_len()..] {
                                eprint!("{b:02x}");
                            }
                            eprintln!();
                        }

                        send.send((stamp, msg))
                            .expect("Failed to send message over MPSC");
                    }

                    Err(err) => {
                        eprintln!("ERROR: message decode error: {err}");
                    }
                },
                None::<u8>,
            )
            .expect("Failed to open MIDI connection");

        MidiInputStream { _conn: conn, recv }
    }
}

impl Iterator for MidiInputStream {
    type Item = (u64, midi::Message);

    fn next(&mut self) -> Option<Self::Item> {
        self.recv.recv().ok()
    }
}
