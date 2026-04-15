//! Definitions and types for diatonic scales and their modes.

use super::*;

/// A diatonic scale.
///
/// As a [`ScaleKind`], this type represents the diatonic natural major scale,
/// with the following letter and semitone offsets:
///
/// | Letter | Semitone | Example (C) |
/// |--------|----------|-------------|
/// | 0      | 0        | C           |
/// | 1      | 2        | D           |
/// | 2      | 4        | E           |
/// | 3      | 5        | F           |
/// | 4      | 7        | G           |
/// | 5      | 9        | A           |
/// | 6      | 11       | B           |
///
/// This is equivalent to [`IonianMode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Diatonic;

const DIATONIC_SCALE_DEGREES: &[ScaleDegree] = &[
    ScaleDegree::new(0, 0),
    ScaleDegree::new(1, 2),
    ScaleDegree::new(2, 4),
    ScaleDegree::new(3, 5),
    ScaleDegree::new(4, 7),
    ScaleDegree::new(5, 9),
    ScaleDegree::new(6, 11),
];

impl ScaleKind for Diatonic {
    fn num_degrees() -> usize {
        DIATONIC_SCALE_DEGREES.len()
    }

    fn scale_degree(n: usize) -> ScaleDegree {
        DIATONIC_SCALE_DEGREES[n % DIATONIC_SCALE_DEGREES.len()]
    }

    fn scale_degrees() -> impl Iterator<Item = ScaleDegree> {
        DIATONIC_SCALE_DEGREES.iter().copied()
    }
}

/// The Ionian (natural major) diatonic mode.
///
/// | Letter offset | Semitone offset | Example (C) |
/// |---------------|-----------------|-------------|
/// | 0             | 0               | C           |
/// | 1             | 2               | D           |
/// | 2             | 4               | E           |
/// | 3             | 5               | F           |
/// | 4             | 7               | G           |
/// | 5             | 9               | A           |
/// | 6             | 11              | B           |
pub type IonianMode = Mode<0, Diatonic>;

/// The Dorian diatonic mode.
///
/// The Dorian mode has the same notes as the natural major scale, starting from
/// the 2nd scale degree. Equivalently, it is the natural minor scale with a
/// raised 6th scale degree.
///
/// | Letter offsset | Semitone offset | Example (C) |
/// |----------------|-----------------|-------------|
/// | 0              | 0               | C           |
/// | 1              | 2               | D           |
/// | 2              | 3               | E♭          |
/// | 3              | 5               | F           |
/// | 4              | 7               | G           |
/// | 5              | 9               | A           |
/// | 6              | 10              | B♭          |
pub type DorianMode = Mode<1, Diatonic>;

/// The Phrygian diatonic mode.
///
/// The Phrygian mode has the same notes as the natural major scale, starting
/// from the 3rd scale degree. Equivalently, it is the natural minor scale with
/// a lowered 2nd scale degree.
///
/// | Letter offset | Semitone offset | Example (C) |
/// |---------------|-----------------|-------------|
/// | 0             | 0               | C           |
/// | 1             | 1               | D♭          |
/// | 2             | 3               | E♭          |
/// | 3             | 5               | F           |
/// | 4             | 7               | G           |
/// | 5             | 8               | A♭          |
/// | 6             | 10              | B♭          |
pub type PhrygianMode = Mode<2, Diatonic>;

/// The Lydian diatonic mode.
///
/// The Lydian mode has the same notes as the natural major scale, starting from
/// the 4th scale degree. Equivalently, it is the natural major scale with a
/// raised 4th scale degree.
///
/// | Letter offset | Semitone offset | Example (C) |
/// |---------------|-----------------|-------------|
/// | 0             | 0               | C           |
/// | 1             | 2               | D           |
/// | 2             | 4               | E           |
/// | 3             | 6               | F♯          |
/// | 4             | 7               | G           |
/// | 5             | 9               | A           |
/// | 6             | 11              | B           |
pub type LydianMode = Mode<3, Diatonic>;

/// The Mixolydian diatonic mode.
///
/// The Mixolydian mode has the same notes as the natural major scale, starting
/// from the 5th scale degree. Equivalently, it is the natural major scale with
/// a lowered 7th scale degree.
///
/// | Letter offset | Semitone offset | Example (C) |
/// |---------------|-----------------|-------------|
/// | 0             | 0               | C           |
/// | 1             | 2               | D           |
/// | 2             | 4               | E           |
/// | 3             | 5               | F           |
/// | 4             | 7               | G           |
/// | 5             | 9               | A           |
/// | 6             | 10              | B♭          |
pub type MixolydianMode = Mode<4, Diatonic>;

/// The Aeolian (natural minor) diatonic mode.
///
/// The Aeolian mode has the same notes as the natural major scale, starting
/// from the 6th scale degree. Equivalently, it is the natural minor scale, or
/// the natural major scale with lowered 3rd, 6th, and 7th scale degrees.
///
/// | Letter offset | Semitone offset | Example (C) |
/// |---------------|-----------------|-------------|
/// | 0             | 0               | C           |
/// | 1             | 2               | D           |
/// | 2             | 3               | E♭          |
/// | 3             | 5               | F           |
/// | 4             | 7               | G           |
/// | 5             | 8               | A♭          |
/// | 6             | 10              | B♭          |
pub type AeolianMode = Mode<5, Diatonic>;

/// The Locrian diatonic mode.
///
/// The Locrian mode has the same notes as the natural major scale, starting
/// from the 7th scale degree. Equivalently, it is the natural minor scale with
/// a lowered 2nd scale degree and a lowered 5th scale degree.
///
/// | Letter offset | Semitone offset | Example (C) |
/// |---------------|-----------------|-------------|
/// | 0             | 0               | C           |
/// | 1             | 1               | D♭          |
/// | 2             | 3               | E♭          |
/// | 3             | 5               | F           |
/// | 4             | 6               | G♭          |
/// | 5             | 8               | A♭          |
/// | 6             | 10              | B♭          |
pub type LocrianMode = Mode<6, Diatonic>;

/// The diatonic natural major mode.
///
/// As a [`ScaleKind`], this is equivalent to both [`IonianMode`] and [`Diatonic`].
pub type NaturalMajor = IonianMode;

/// The diatonic natural minor mode.
///
/// As a [`ScaleKind`], this is equivalent to [`AeolianMode`].
pub type NaturalMinor = AeolianMode;

#[cfg(test)]
mod test {
    use crate::mkpitch;

    use super::*;

    fn scale_eq<K: ScaleKind>(scale: Scale<K>, expected: impl IntoIterator<Item = PitchClass>) {
        let generated: Vec<_> = scale.scale_degrees().collect();
        let expected: Vec<_> = expected.into_iter().collect();

        assert_eq!(generated, expected);
    }

    #[test]
    fn ionian() {
        scale_eq(Scale::<IonianMode>::new(mkpitch!(C)), [
            mkpitch!(C),
            mkpitch!(D),
            mkpitch!(E),
            mkpitch!(F),
            mkpitch!(G),
            mkpitch!(A),
            mkpitch!(B),
        ]);

        scale_eq(Scale::<IonianMode>::new(mkpitch!(B)), [
            mkpitch!(B),
            mkpitch!(C s),
            mkpitch!(D s),
            mkpitch!(E),
            mkpitch!(F s),
            mkpitch!(G s),
            mkpitch!(A s),
        ]);
    }

    #[test]
    fn aeolian() {
        scale_eq(Scale::<AeolianMode>::new(mkpitch!(C)), [
            mkpitch!(C),
            mkpitch!(D),
            mkpitch!(E f),
            mkpitch!(F),
            mkpitch!(G),
            mkpitch!(A f),
            mkpitch!(B f),
        ]);

        scale_eq(Scale::<AeolianMode>::new(mkpitch!(G)), [
            mkpitch!(G),
            mkpitch!(A),
            mkpitch!(B f),
            mkpitch!(C),
            mkpitch!(D),
            mkpitch!(E f),
            mkpitch!(F),
        ]);
    }

    #[test]
    fn dorian() {
        scale_eq(Scale::<DorianMode>::new(mkpitch!(C)), [
            mkpitch!(C),
            mkpitch!(D),
            mkpitch!(E f),
            mkpitch!(F),
            mkpitch!(G),
            mkpitch!(A),
            mkpitch!(B f),
        ]);

        scale_eq(Scale::<DorianMode>::new(mkpitch!(E)), [
            mkpitch!(E),
            mkpitch!(F s),
            mkpitch!(G),
            mkpitch!(A),
            mkpitch!(B),
            mkpitch!(C s),
            mkpitch!(D),
        ]);
    }
}
