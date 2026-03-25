//! Pitch classes and related calculations for 12-tone equal temperament.

use std::fmt::{self, Display, Formatter};

/// A note letter. Equivalently, a note of the diatonic C major scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Letter {
    C = 0,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl Letter {
    /// Get the mod-7 offset of this note letter from C.
    ///
    /// This counts note letters, (or, equivalently, diatonic scale degrees,)
    /// rather than semitones. For instance, D is two semitones up from C, but
    /// only one letter/scale degree, so its letter offset is 1.
    ///
    /// For the semitone offset from C natural, use [`Letter::semitone_offset`].
    pub const fn letter_offset(self) -> u8 {
        self as u8
    }

    /// Get the mod-12 semitone offset of this note letter from C natural.
    ///
    /// This counts semitones between natural notes. For the letter/scale degree
    /// offset from C, use [`Letter::letter_offset`].
    pub const fn semitone_offset(self) -> u8 {
        match self {
            Letter::C => 0,
            Letter::D => 2,
            Letter::E => 4,
            Letter::F => 5,
            Letter::G => 7,
            Letter::A => 9,
            Letter::B => 11,
        }
    }

    /// Get the letter with the given offset from C mod 7.
    pub const fn from_letter_offset(offset: u8) -> Self {
        match offset % 7 {
            0 => Self::C,
            1 => Self::D,
            2 => Self::E,
            3 => Self::F,
            4 => Self::G,
            5 => Self::A,
            6 => Self::B,
            _ => unreachable!(),
        }
    }

    /// Get the letter that is the given offset, mod 7, from this one.
    pub const fn offset_by(self, letter_offset: u8) -> Self {
        Self::from_letter_offset(self.letter_offset() + letter_offset)
    }

    /// Get the mod-7 offset of this letter from another.
    pub const fn offset_from(self, other: Letter) -> u8 {
        ((self.letter_offset() + 7) - other.letter_offset()) % 7
    }
}

impl Display for Letter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Letter::C => write!(f, "C"),
            Letter::D => write!(f, "D"),
            Letter::E => write!(f, "E"),
            Letter::F => write!(f, "F"),
            Letter::G => write!(f, "G"),
            Letter::A => write!(f, "A"),
            Letter::B => write!(f, "B"),
        }
    }
}

/// An accidental, adjusting the pitch of a base note by semitones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Accidental {
    /// Two semitones flat
    DoubleFlat,

    /// One semitone flat
    Flat,

    /// No change
    Natural,

    /// One semitone sharp
    Sharp,

    /// Two semitones sharp
    DoubleSharp,
}

impl Accidental {
    /// Get the semitone offset applied by this accidental.
    pub const fn offset(self) -> i8 {
        match self {
            Accidental::DoubleFlat => -2,
            Accidental::Flat => -1,
            Accidental::Natural => 0,
            Accidental::Sharp => 1,
            Accidental::DoubleSharp => 2,
        }
    }

    /// Get the accidental corresponding to the given semitone offset.
    ///
    /// Returns `None` if `offset` is outside the inclusive range -2 to 2.
    pub const fn from_offset(offset: i8) -> Option<Self> {
        match offset {
            -2 => Some(Self::DoubleFlat),
            -1 => Some(Self::Flat),
            0 => Some(Self::Natural),
            1 => Some(Self::Sharp),
            2 => Some(Self::DoubleSharp),
            _ => None,
        }
    }
}

impl Display for Accidental {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Accidental::DoubleFlat => write!(f, "𝄫"),
            Accidental::Flat => write!(f, "♭"),
            Accidental::Natural => write!(f, "♮"),
            Accidental::Sharp => write!(f, "♯"),
            Accidental::DoubleSharp => write!(f, "𝄪"),
        }
    }
}

impl Default for Accidental {
    fn default() -> Self {
        Self::Natural
    }
}

/// A pitch class in a 12-tone tuning system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PitchClass {
    pub letter: Letter,
    pub accidental: Accidental,
}

impl PitchClass {
    pub const fn new(letter: Letter, accidental: Accidental) -> Self {
        Self {
            letter,
            accidental,
        }
    }

    /// Create a natural pitch class of the given note letter.
    pub const fn natural(base: Letter) -> Self {
        Self::new(base, Accidental::Natural)
    }

    /// Get the mod-12 semitone offset of this pitch class from C natural.
    pub const fn semitone_offset(self) -> u8 {
        ((self.letter.semitone_offset() as i8 + self.accidental.offset()) % 12) as u8
    }

    /// Get the mod-12 semitone offset of this pitch class from another.
    pub const fn semitone_offset_from(self, other: PitchClass) -> u8 {
        ((self.semitone_offset() + 12) - other.semitone_offset()) % 12
    }

    /// Is this pitch class enharmonically equivalent to another?
    pub const fn enharmonic_eq(self, other: Self) -> bool {
        self.semitone_offset() == other.semitone_offset()
    }
}

impl Display for PitchClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.letter)?;
        if self.accidental != Accidental::Natural {
            write!(f, "{}", self.accidental)?;
        }
        Ok(())
    }
}
