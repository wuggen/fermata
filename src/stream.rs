//! Parsing and serialization wrappers for MIDI streams.

use std::sync::mpsc::{self, Receiver};

use midir::{
    MidiInput, MidiInputConnection, MidiInputPort, MidiOutput, MidiOutputConnection, MidiOutputPort,
};

use crate::midi;

/// A MIDI input stream.
///
/// This is a thin wrapper around a [`midir::MidiInputConnection`] which
/// automatically parses incoming messages into [`midi::Message`]s. Its
/// interface is based on [`Iterator`] rather than using callbacks.
pub struct MidiInputStream {
    _conn: MidiInputConnection<Option<u8>>,
    recv: Receiver<(u64, midi::Message)>,
}

impl MidiInputStream {
    /// Create a new MIDI input stream, opening a connection on the given port
    /// of the given input device.
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
            .expect("Failed to open MIDI input connection");

        MidiInputStream { _conn: conn, recv }
    }
}

impl Iterator for MidiInputStream {
    type Item = (u64, midi::Message);

    fn next(&mut self) -> Option<Self::Item> {
        self.recv.recv().ok()
    }
}

pub struct MidiOutputStream {
    conn: MidiOutputConnection,
}

impl MidiOutputStream {
    pub fn new(name: &str, output: MidiOutput, port: &MidiOutputPort) -> Self {
        let conn = output
            .connect(port, name)
            .expect("Failed to open MIDI output connection");
        Self { conn }
    }

    pub fn send(&mut self, msg: midi::Message) {
        let bytes = msg.encode(true);
        self.conn.send(&bytes).expect("Failed to send MIDI message");
    }
}
