use std::error::Error;
use std::io::{self, Write};
use std::time::Duration;

use midir::{MidiInput, MidiOutput};
use pitch::PitchClass;

use crate::pitch::Letter;
use crate::scale::diatonic::{AeolianMode, IonianMode};
use crate::scale::{Scale, ScaleKind};

pub mod general_midi;
pub mod midi;
pub mod pitch;
pub mod scale;
pub mod stream;

pub fn run() -> Result<(), Box<dyn Error>> {
    fn print_scale<K: ScaleKind>(root: PitchClass) {
        let scale = Scale::<K>::new(root);
        for pitch in scale.scale_degrees() {
            print!("{pitch} ");
        }
        println!();
    }

    print!("E Ionian: ");
    print_scale::<IonianMode>(PitchClass::natural(Letter::E));

    print!("D Aeolian: ");
    print_scale::<AeolianMode>(PitchClass::natural(Letter::D));

    let midi_in = MidiInput::new("Fermata")?;
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input ports found".into()),
        1 => {
            println!("Found one MIDI input port: {}", midi_in.port_name(&in_ports[0]).unwrap());
            &in_ports[0]
        }
        _ => {
            println!("Available input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{i}: {}", midi_in.port_name(p).unwrap());
            }

            let mut input = String::new();
            loop {
                print!("> ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut input)?;

                if let Ok(val) = input.trim().parse::<usize>() {
                    if val < in_ports.len() {
                        break &in_ports[val];
                    }
                }

                println!("Invalid port number");
                input.clear();
            }
        }
    };

    println!("Opening connection...");
    let in_stream = stream::MidiInputStream::new(":in", midi_in, in_port);
    println!("Connection open, recording messages");

    for (stamp, msg) in in_stream {
        println!("{stamp:05} - {msg:?}");
    }

    // let midi_out = MidiOutput::new("RFermata")?;

    // let out_ports = midi_out.ports();

    // let out_port = match out_ports.len() {
    //     0 => return Err("no output ports found".into()),
    //     1 => {
    //         println!(
    //             "Found one MIDI port: {}",
    //             midi_out.port_name(&out_ports[0]).unwrap()
    //         );
    //         &out_ports[0]
    //     }
    //     _ => {
    //         println!("Available output ports:");
    //         for (i, p) in out_ports.iter().enumerate() {
    //             println!("{i}: {}", midi_out.port_name(p).unwrap());
    //         }

    //         let mut input = String::new();
    //         loop {
    //             input.clear();
    //             print!("> ");
    //             io::stdout().flush().unwrap();
    //             io::stdin().read_line(&mut input)?;

    //             if let Ok(val) = input.trim().parse::<usize>() {
    //                 if val < out_ports.len() {
    //                     break &out_ports[val];
    //                 }
    //             }

    //             println!("Invalid port number");
    //         }
    //     }
    // };

    // println!("Opening connection...");
    // let mut conn_out = midi_out.connect(out_port, ":out")?;
    // println!("Connection open, playing");

    // let mut running = None;
    // let mut send_msg = |msg: midi::Message| {
    //     let encode_status = Some(msg.status()) != running;
    //     running = Some(msg.status());
    //     let _ = conn_out.send(&msg.encode(encode_status));
    // };

    // let mut play_note = |note: (PitchClass, u8), duration: u64| {
    //     const VEL: u8 = 0x64;

    //     let note = note.0.midi_note(note.1).unwrap();

    //     send_msg(midi::Message::NoteOn {
    //         channel: 0,
    //         note,
    //         velocity: VEL,
    //     });

    //     std::thread::sleep(Duration::from_millis(duration * 150));

    //     send_msg(midi::Message::NoteOn {
    //         channel: 0,
    //         note,
    //         velocity: 0,
    //     });
    // };

    // use PitchClass::*;

    // play_note((Fs, 4), 4);
    // play_note((Es, 4), 3);
    // play_note((Ds, 4), 1);
    // play_note((Cs, 4), 6);
    // play_note((B, 3), 2);
    // play_note((As, 3), 4);
    // play_note((Gs, 3), 4);
    // play_note((Fs, 3), 4);

    Ok(())
}
