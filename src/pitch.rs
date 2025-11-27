//! Pitch classes and related calculations for 12-tone equal temperament.

use std::fmt::{self, Display, Formatter};

/// A 12-tone equal temperament pitch class.
///
/// This includes the seven natural pitch classes (A through G) as well as
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
    /// Get the canonical pitch class that is enharmonically equivalent to this one.
    ///
    /// "Canonical" here means one of the seven natural pitch classes, or
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

    /// Get the canonical pitch class `offset` semitones above C in an octave.
    ///
    /// See [`PitchClass::canonical`] for an explanation of "canonical" in this context.
    pub const fn from_offset_in_octave(offset: u8) -> Self {
        match offset % 12 {
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

    /// Get the canonical pitch class of the note `semitones` semitones above a note of this pitch class.
    ///
    /// See [`PitchClass::canonical`] for an explanation of "canonical" in this context.
    pub const fn semitones_above(self, semitones: u8) -> Self {
        Self::from_offset_in_octave(self.offset_in_octave() + (semitones % 12))
    }

    /// Get the canonical pitch class of the note `semitones` semitones below a note of this pitch class.
    ///
    /// See [`PitchClass::canonical`] for an explanation of "canonical" in this context.
    pub const fn semitones_below(self, semitones: u8) -> Self {
        Self::from_offset_in_octave(self.offset_in_octave() + 12 - (semitones % 12))
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

    /// Raise this pitch class a semitone.
    ///
    /// This method will prefer enharmonics that refer to `self` over (perhaps
    /// more canonical) enharmonics. For instance, `PitchClass::B.raised()
    /// == PitchClass::Bs`, and `PitchClass::Cf.raised() == PitchClass::C`.
    pub const fn raised(self) -> Self {
        match self {
            Self::Bs | Self::C => Self::Cs,
            Self::Cs | Self::Df => Self::D,
            Self::D => Self::Ds,
            Self::Ds | Self::Ef => Self::E,
            Self::E => Self::Es,
            Self::Ff => Self::F,
            Self::Es | Self::F => Self::Fs,
            Self::Fs | Self::Gf => Self::G,
            Self::G => Self::Gs,
            Self::Gs | Self::Af => Self::A,
            Self::A => Self::As,
            Self::As | Self::Bf => Self::B,
            Self::B => Self::Bs,
            Self::Cf => Self::C,
        }
    }

    /// Lower this pitch class a semitone.
    ///
    /// This method will prefer enharmonics that refer to `self` over (perhaps
    /// more canonical) enharmonics. For instance, `PitchClass::C.lowered() ==
    /// PitchClass::Cf`, and `PitchClass::Bs.lowered() == PitchClass::B`.
    pub const fn lowered(self) -> Self {
        match self {
            Self::C => Self::Cf,
            Self::Cs | Self::Df => Self::C,
            Self::D => Self::Df,
            Self::Ds | Self::Ef => Self::D,
            Self::E | Self::Ff => Self::Ef,
            Self::Es => Self::E,
            Self::F => Self::Ff,
            Self::Fs | Self::Gf => Self::F,
            Self::G => Self::Gf,
            Self::Gs | Self::Af => Self::G,
            Self::A => Self::Af,
            Self::As | Self::Bf => Self::A,
            Self::B | Self::Cf => Self::Bf,
            Self::Bs => Self::B,
        }
    }

    /// Get the enharmonic of this pitch class that is a flat natural note, or otherwise not a sharp.
    ///
    /// For sharp natural notes that have flat enharmonic equivalents, this returns that equivalent:
    ///
    /// ```
    /// # use rfermata::pitch::*;
    /// use PitchClass::*;
    /// assert_eq!(Cs.flat_enharmonic(), Df);
    /// assert_eq!(Gs.flat_enharmonic(), Af);
    /// ```
    ///
    /// For natural pitch classes, and flat natural pitch classes, this returns the pitch class unchanged:
    ///
    /// ```
    /// # use rfermata::pitch::*;
    /// use PitchClass::*;
    /// assert_eq!(C.flat_enharmonic(), C);
    /// assert_eq!(B.flat_enharmonic(), B);
    /// assert_eq!(Bf.flat_enharmonic(), Bf);
    /// assert_eq!(Df.flat_enharmonic(), Df);
    /// ```
    ///
    /// For E♯ and B♯, which are enharmonically equivalent to the natural notes
    /// F and C respectively, this returns the corresponding natural notes:
    ///
    /// ```
    /// # use rfermata::pitch::*;
    /// use PitchClass::*;
    /// assert_eq!(Es.flat_enharmonic(), F);
    /// assert_eq!(Bs.flat_enharmonic(), C);
    /// ```
    ///
    /// However, F♭ and C♭, which are enharmonically equivalent to E and B
    /// respectively, are unchanged, since they are already flat natural notes:
    ///
    /// ```
    /// # use rfermata::pitch::*;
    /// use PitchClass::*;
    /// assert_eq!(Ff.flat_enharmonic(), Ff);
    /// assert_eq!(Cf.flat_enharmonic(), Cf);
    /// ```
    pub const fn flat_enharmonic(self) -> Self {
        self.raised().lowered()
    }

    /// Get the enharmonic of this pitch class that is a sharp natural note, or otherwise not a flat.
    ///
    /// For flat natural notes that have sharp enharmonic equivalents, this returns that equivalent:
    ///
    /// ```
    /// # use rfermata::pitch::*;
    /// use PitchClass::*;
    /// assert_eq!(Df.sharp_enharmonic(), Cs);
    /// assert_eq!(Af.sharp_enharmonic(), Gs);
    /// ```
    ///
    /// For natural pitch classes, and sharp natural pitch classes, this returns the pitch class unchanged:
    ///
    /// ```
    /// # use rfermata::pitch::*;
    /// use PitchClass::*;
    /// assert_eq!(C.sharp_enharmonic(), C);
    /// assert_eq!(B.sharp_enharmonic(), B);
    /// assert_eq!(As.sharp_enharmonic(), As);
    /// assert_eq!(Ds.sharp_enharmonic(), Ds);
    /// ```
    ///
    /// For F♭ and C♭, which are enharmonically equivalent to the natural notes
    /// E and B respectively, this returns the corresponding natural notes:
    ///
    /// ```
    /// # use rfermata::pitch::*;
    /// use PitchClass::*;
    /// assert_eq!(Ff.sharp_enharmonic(), E);
    /// assert_eq!(Cf.sharp_enharmonic(), B);
    /// ```
    ///
    /// However, E♯ and B♯, which are enharmonically equivalent to F and C
    /// respectively, are unchanged, since they are already sharp natural notes:
    ///
    /// ```
    /// # use rfermata::pitch::*;
    /// use PitchClass::*;
    /// assert_eq!(Es.sharp_enharmonic(), Es);
    /// assert_eq!(Bs.sharp_enharmonic(), Bs);
    /// ```
    pub const fn sharp_enharmonic(self) -> Self {
        self.lowered().raised()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn flat_enharmonics() {
        fn test_pair(base: PitchClass, expected: PitchClass) {
            let got = base.flat_enharmonic();
            assert_eq!(
                expected, got,
                "Unexpected pitch class: flat enharmonic of {base} expected to be {expected}, got {got}",
            );
        }

        use PitchClass::*;
        let pairs = [
            (Cs, Df),
            (Df, Df),
            (Es, F),
            (Ff, Ff),
            (Bs, C),
            (B, B),
            (C, C),
            (Cf, Cf),
        ];

        for (base, expected) in pairs {
            test_pair(base, expected);
        }
    }

    #[test]
    fn sharp_enharmonics() {
        fn test_pair(base: PitchClass, expected: PitchClass) {
            let got = base.sharp_enharmonic();
            assert_eq!(
                expected, got,
                "Unexpected pitch class: sharp enharmonic of {base} expected to be {expected}, got {got}",
            );
        }

        use PitchClass::*;
        let pairs = [
            (Cs, Cs),
            (Df, Cs),
            (Fs, Fs),
            (Gf, Fs),
            (Ff, E),
            (Cf, B),
            (Es, Es),
            (Bs, Bs),
            (E, E),
            (F, F),
        ];

        for (base, expected) in pairs {
            test_pair(base, expected);
        }
    }

    #[test]
    fn semitones_above() {
        fn test(base: PitchClass, offset: u8, expected: PitchClass) {
            let got = base.semitones_above(offset);
            assert_eq!(
                expected, got,
                "Pitch class {offset} semitones above {base}: expected {expected}, got {got}",
            );
        }

        use PitchClass::*;
        let cases = [
            (C, 1, Cs),
            (C, 7, G),
            (C, 11, B),
            (C, 12, C),
            (D, 12, D),
            (B, 7, Fs),
            (Fs, 7, Cs),
            (G, 4, B),
            (G, 3, As),
            (C, 6, Fs),
        ];

        for (base, offset, expected) in cases {
            test(base, offset, expected);
        }
    }

    #[test]
    fn semitones_below() {
        fn test(base: PitchClass, offset: u8, expected: PitchClass) {
            let got = base.semitones_below(offset);
            assert_eq!(
                expected, got,
                "Pitch class {offset} semitones below {base}: expected {expected}, got {got}",
            );
        }

        use PitchClass::*;
        let cases = [
            (C, 7, F),
            (D, 7, G),
            (C, 1, B),
            (C, 11, Cs),
            (G, 7, C),
            (G, 6, Cs),
            (B, 4, G),
            (B, 3, Gs),
        ];

        for (base, offset, expected) in cases {
            test(base, offset, expected);
        }
    }
}
