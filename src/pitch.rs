//! Pitch classes and calculations for 12-tone equal temperament.

use std::fmt::{self, Display, Formatter};

/// A 12-tone equal temperament pitch class.
///
/// This includes the seven diatonic pitch classes (A through G) as well as
/// sharps and flats of each. Enharmonically equivalent pitch classes are also
/// included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PitchClass {
    C,
    Cs,
    Df,
    D,
    Ds,
    Ef,
    E,
    Ff,
    Es,
    F,
    Fs,
    Gf,
    G,
    Gs,
    Af,
    A,
    As,
    Bf,
    B,
    Cf,
    Bs,
}

impl Display for PitchClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::C => write!(f, "C"),
            Self::Cs => write!(f, "C♯"),
            Self::Df => write!(f, "D♭"),
            Self::D => write!(f, "D"),
            Self::Ds => write!(f, "D♯"),
            Self::Ef => write!(f, "E♭"),
            Self::E => write!(f, "E"),
            Self::Ff => write!(f, "F♭"),
            Self::Es => write!(f, "E♯"),
            Self::F => write!(f, "F"),
            Self::Fs => write!(f, "F♯"),
            Self::Gf => write!(f, "G♭"),
            Self::G => write!(f, "G"),
            Self::Gs => write!(f, "G♯"),
            Self::Af => write!(f, "A♭"),
            Self::A => write!(f, "A"),
            Self::As => write!(f, "A♯"),
            Self::Bf => write!(f, "B♭"),
            Self::B => write!(f, "B"),
            Self::Cf => write!(f, "C♭"),
            Self::Bs => write!(f, "B♯"),
        }
    }
}

impl PitchClass {
    /// Get the canonical pitch class that is enharminically equivalent to this one.
    ///
    /// "Canonical" here means one of the seven diatonic pitch classes, or
    /// sharps thereof: C, C♯, D, D♯, E, F, F♯, G, G♯, A, A♯, B. These pitch
    /// classes are mapped to themselves, and all other pitch classes are mapped
    /// to their enharmonic equivalent from among this list.
    pub const fn canonical(self) -> Self {
        match self {
            Self::C | Self::Bs => Self::C,
            Self::Cs | Self::Df => Self::Cs,
            Self::Ds | Self::Ef => Self::Ds,
            Self::E | Self::Ff => Self::E,
            Self::Es | Self::F => Self::F,
            Self::Fs | Self::Gf => Self::Fs,
            Self::Gs | Self::Af => Self::Gs,
            Self::As | Self::Bf => Self::As,
            Self::B | Self::Cf => Self::B,
            _ => self,
        }
    }

    /// Get the semitone offset of a note of this pitch class from the next lowest C.
    ///
    /// For C, this will return 0. However, for B♯, this will return 12, despite
    /// B♯ being enharmonically equivalent to C, since B♯ behaves functionally
    /// differently from C.
    pub const fn offset_in_octave(self) -> u8 {
        match self {
            Self::C => 0,
            Self::Cs | Self::Df => 1,
            Self::D => 2,
            Self::Ds | Self::Ef => 3,
            Self::E | Self::Ff => 4,
            Self::Es | Self::F => 5,
            Self::Fs | Self::Gf => 6,
            Self::G => 7,
            Self::Gs | Self::Af => 8,
            Self::A => 9,
            Self::As | Self::Bf => 10,
            Self::B | Self::Cf => 11,
            Self::Bs => 12,
        }
    }

    /// Calculate the MIDI note number of this pitch class in the given octave.
    ///
    /// For instance, `PitchClass::C.midi_note(4)` will return 60, the MIDI note number for C4 or middle C.
    ///
    /// Returns None if the resulting note number would be out of range for MIDI
    /// note numbers. The highest MIDI note number is 127, corresponding to G9.
    ///
    /// Note that this calculation is unable to reach MIDI notes 0 through 11,
    /// as these would be in octave -1 in the usual numbering.
    pub const fn midi_note(self, octave: u8) -> Option<u8> {
        let offset = self.offset_in_octave();

        if octave > 9 || (octave == 9 && offset > 7) {
            None
        } else {
            Some(12 * (octave + 1) + offset)
        }
    }

    /// Get the canonical pitch class of the given MIDI note.
    ///
    /// See [`PitchClass::canonical`] for an explanation of "canonical" in this context.
    ///
    /// Panics if the given note is out of range for MIDI notes (i.e. greater than 127).
    pub const fn of_midi_note(note: u8) -> Self {
        match note % 12 {
            0 => Self::C,
            1 => Self::Cs,
            2 => Self::D,
            3 => Self::Ds,
            4 => Self::E,
            5 => Self::F,
            6 => Self::Fs,
            7 => Self::G,
            8 => Self::Gs,
            9 => Self::A,
            10 => Self::As,
            11 => Self::B,
            _ => unreachable!(),
        }
    }

    /// Are these two pitch classes enharmonically equivalent?
    pub const fn enharmonically_eq(self, other: Self) -> bool {
        self.offset_in_octave() % 12 == other.offset_in_octave() % 12
    }
}
