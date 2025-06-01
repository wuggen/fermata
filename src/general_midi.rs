//! Definitions for the General MIDI sound maps.

macro_rules! mk_instruments {
    (
        $($Name:ident $(= $num:expr)?),* $(,)?
    ) => {
        /// An instrument as defined by the General MIDI sound set.
        ///
        /// Use `as u8` to convert this enum to a program number for use in MIDI
        /// Program Change messages.
        ///
        /// Note that in MIDI specification documents, program numbers are
        /// written 1-based, whereas they are transmitted over the wire
        /// 0-based. The values of this enum are the 0-based forms; e.g.
        /// `Instrument::Clavi as u8` yields 7, corresponding to General MIDI
        /// program number 8.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum Instrument {
            $(
                $Name $(= $num)?,
            )*
        }

        impl From<Instrument> for u8 {
            fn from(val: Instrument) -> u8 {
                val as u8
            }
        }

        impl Instrument {
            /// Get the program number of this General MIDI instrument.
            ///
            /// This is equivalent to `self as u8`.
            pub const fn program_number(self) -> u8 {
                self as u8
            }

            /// Convert a MIDI program number to a General MIDI instrument.
            ///
            /// This will succeed so long as the given byte is a valid MIDI data
            /// byte (i.e. has its high bit cleared or, equivalently, is less
            /// than 128). Returns None otherwise.
            pub const fn from_program_number(num: u8) -> Option<Self> {
                #![allow(non_upper_case_globals)]
                $(
                    const $Name: u8 = Instrument::$Name as u8;
                )*
                match num {
                    $(
                        $Name => Some(Self::$Name),
                    )*
                    _ => None,
                }
            }
        }

        #[cfg(test)]
        mod instrument_test {
            use super::*;

            #[test]
            fn instruments() {
                let instruments = [
                    $(
                        Instrument::$Name,
                    )*
                ];

                for (i, instr) in instruments.into_iter().enumerate() {
                    assert_eq!(
                        i as u8,
                        instr as u8,
                        "Instrument number mismatch: {i} {instr:?} (got {})",
                        instr as u8,
                    );
                }
            }
        }
    };
}

mk_instruments! {
    AcousticGrandPiano = 0,
    BrightAcousticPiano,
    ElectricGrandPiano,
    HonkyTonkPiano,
    ElectricPiano1,
    ElectricPiano2,
    Harpsichord,
    Clavi,
    Celesta,
    Glockenspiel,
    MusicBox,
    Vibraphone,
    Marimba,
    Xylophone,
    TubularBells,
    Dulcimer,
    DrawbarOrgan,
    PercussiveOrgan,
    RockOrgan,
    ChurchOrgan,
    ReedOrgan,
    Accordion,
    Harmonica,
    TangoAccordion,
    AcousticGuitarNylon,
    AcousticGuitarSteel,
    ElectricGuitarJazz,
    ElectricGuitarClean,
    ElectricGuitarMuted,
    OverdrivenGuitar,
    DistortionGuitar,
    GuitarHarmonics = 31,
    AcousticBass = 32,
    ElectricBassFinger,
    ElectricBassPick,
    FretlessBass,
    SlapBass1,
    SlapBass2,
    SynthBass1,
    SynthBass2,
    Violin,
    Viola,
    Cello,
    Contrabass,
    TremoloStrings,
    PizzicatoStrings,
    OrchestralHarp,
    Timpani,
    StringEnsemble1,
    StringEnsemble2,
    SynthStrings1,
    SynthStrings2,
    ChoirAahs,
    VoiceOohs,
    SynthVoice,
    OrchestraHit,
    Trumpet,
    Trombone,
    Tuba,
    MutedTrumpet,
    FrenchHorn,
    BrassSection,
    SynthBrass1,
    SynthBrass2 = 63,
    SopranoSax = 64,
    AltoSax,
    TenorSax,
    BaritoneSax,
    Oboe,
    EnglishHorn,
    Bassoon,
    Clarinet,
    Piccolo,
    Flute,
    Recorder,
    PanFlute,
    BlownBottle,
    Shakuhachi,
    Whistle,
    Ocarina,
    Lead1Square,
    Lead2Sawtooth,
    Lead3Calliope,
    Lead4Chiff,
    Lead5Charang,
    Lead6Voice,
    Lead7Fifths,
    Lead8BassLead,
    Pad1NewAge,
    Pad2Warm,
    Pad3Polysynth,
    Pad4Choir,
    Pad5Bowed,
    Pad6Metallic,
    Pad7Halo,
    Pad8Sweep = 95,
    Fx1Rain = 96,
    Fx2Soundtrack,
    Fx3Crystal,
    Fx4Atmosphere,
    Fx5Brightness,
    Fx6Goblins,
    Fx7Echoes,
    Fx8SciFi,
    Sitar,
    Banjo,
    Shamisen,
    Koto,
    Kalimba,
    BagPipe,
    Fiddle,
    Shanai,
    TinkleBell,
    Agogo,
    SteelDrums,
    Woodblock,
    TaikoDrum,
    MelodicTom,
    SynthDrum,
    ReverseCymbal,
    GuitarFretNoise,
    BreathNoise,
    Seashore,
    BirdTweet,
    TelephoneRing,
    Helicopter,
    Applause,
    Gunshot = 127,
}

macro_rules! mk_percussion {
    (
        $($Name:ident $(= $num:expr)?),* $(,)?
    ) => {
        /// A percussion sound as defined by the General MIDI percussion map.
        ///
        /// In a General MIDI system, non-chromatic percussion is sent on MIDI
        /// channel 10 (1-based, transmitted over the wire in 0-based form
        /// as 9), and particular percussion sounds correspond to particular
        /// MIDI note numbers on that channel. This enum encodes those note
        /// numbers; use `as u8` to convert from an instance of this enum to the
        /// corresponding MIDI note number.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum Percussion {
            $(
                $Name $(= $num)?,
            )*
        }

        impl From<Percussion> for u8 {
            fn from(val: Percussion) -> u8 {
                val as u8
            }
        }

        impl Percussion {
            /// Get the MIDI note number for this percussion sound.
            ///
            /// This is equivalent to `self as u8`.
            pub const fn note_number(self) -> u8 {
                self as u8
            }

            /// Convert a MIDI note number to a General MIDI percussion sound.
            ///
            /// Returns None if the given note number does not correspond to a
            /// General MIDI percussion sound. Percussion sounds are mapped to
            /// MIDI notes 35 through 81, inclusive.
            pub const fn from_note_number(num: u8) -> Option<Self> {
                #![allow(non_upper_case_globals)]
                $(
                    const $Name: u8 = Percussion::$Name as u8;
                )*
                match num {
                    $(
                        $Name => Some(Percussion::$Name),
                    )*
                    _ => None,
                }
            }
        }

        #[cfg(test)]
        mod percussion_test {
            use super::*;

            #[test]
            fn percussion() {
                let percussion = [
                    $(
                        Percussion::$Name,
                    )*
                ];

                for (i, perc) in percussion.into_iter().enumerate() {
                    assert_eq!(
                        i as u8 + MIN_PERCUSSION,
                        perc as u8,
                        "Percussion note number mismatch: {} {perc:?} (got {})",
                        i as u8 + MIN_PERCUSSION,
                        perc as u8,
                    );
                }
            }
        }
    };
}

const MIN_PERCUSSION: u8 = 35;

mk_percussion! {
    AcousticBassDrum = MIN_PERCUSSION,
    BassDrum1,
    SideStick,
    AcousticSnare,
    HandClap,
    ElectricSnare,
    LowFloorTom,
    ClosedHiHat,
    HighFloorTom,
    PedalHiHat,
    LowTom,
    OpenHiHat,
    LowMidTom,
    HiMidTom,
    CrashCymbal1,
    HighTom = 50,
    RideCymbal1 = 51,
    ChineseCymbal,
    RideBell,
    Tambourine,
    SplashCymbal,
    Cowbell,
    CrashCymbal2,
    Vibraslap,
    RideCymbal2,
    HiBongo,
    LowBongo,
    MuteHiConga,
    OpenHiConga,
    LowConga,
    HighTimbale,
    LowTimbale = 66,
    HighAgogo = 67,
    LowAgogo,
    Cabasa,
    Mracas,
    ShortWhistle,
    LongWhistle,
    ShortGuiro,
    LongGuiro,
    Claves,
    HiWoodBlock,
    LowWoodBlock,
    MuteCuica,
    OpenCuica,
    MuteTriangle,
    OpenTriangle = 81,
}
