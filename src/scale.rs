//! Scales in 12-tone tuning systems.

pub mod diatonic;

use crate::pitch::{Accidental, PitchClass};

/// Information defining a particular pitch class in a scale (scale degree).
///
/// This contains information about both the pitch and its preferred spelling:
///
/// - The `semitone` field contains the mod-12 semitone offset from the scale's
///   root pitch class.
/// - The `letter` field contains the mod-7 letter offset from the root pitch
///   class.
///
/// For instance, the third scale degree in a diatonic major scale would be
/// represented as follows:
///
/// ```
/// # use fermata::scale::ScaleDegree;
/// let diatonic_major_3 = ScaleDegree {
///     semitone: 4,
///     letter: 2,
/// };
/// ```
///
/// In C major, the third scale degree is E; E♮ is 4 semitones above C♮, and
/// E is 2 letters above C. In D major, the third scale degree is F♯; F♯ is 4
/// semitones above D♮, and F is 2 letters above D.
///
/// Generally speaking, to derive a specific pitch class from a `ScaleDegree`
/// and the pitch class of the scale root:
///
/// - Offset the letter of the base note, without accidentals, by `letter` offset.
/// - Append whatever accidental is necessary to achieve the required `semitone`
///   offset.
///
/// For instance, in C major, the third scale degree is computed as follows:
///
/// - Offset the letter of the root note (C) by 2 to arrive at E.
/// - E is already 4 semitones above the root note C, so no accidental is necessary.
///
/// In D major:
///
/// - Offset the letter of the root note (D) by 2 to arrive at F.
/// - F is only 3 semitones above the root note D, so we must append a sharp to achieve a
///   4-semitone offset, giving us F♯.
///
/// The same procedure applies when the root note itself has an accidental; e.g.
/// in G♭ major:
///
/// - Offset the letter of the root note (G) by 2 to arrive at B.
/// - B is 5 semitones above the root note G♭, so we must append a flat to
///   achieve a 4-semitone offset, giving us B♭.
///
/// > NB: Not all combinations of semitone and letter offsets are representable
/// > as pitch classes. Specifically, the semitone offset of the scale degree
/// > must be within ±2 of the semitone offset of the letter computed from the
/// > root letter and the letter offset; if this is not the case, converting
/// > to a pitch class via [`ScaleDegree::to_pitch_class`] will panic. For
/// > instance, `ScaleDegree { semitone: 5, letter: 1 }` is not representable
/// > in any scale. In C major for instance, letter offset 1 gives D, whose
/// > semitone offset from C is 2; a double sharp is not sufficient to bring
/// > this to the required semitone offset of 5, and so the conversion fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaleDegree {
    pub letter: u8,
    pub semitone: u8,
}

impl ScaleDegree {
    pub const fn new(letter: u8, semitone: u8) -> Self {
        Self { letter, semitone }
    }

    /// Compute the pitch class of this scale degree from a given root pitch.
    ///
    /// # Panics
    ///
    /// Panics if the computed letter offset from the base pitch is more than
    /// two semitones away from the semitone offset required by this scale
    /// degree. For instance:
    ///
    /// ```should_panic
    /// # use fermata::{scale::ScaleDegree, pitch::{PitchClass, Letter, Accidental}};
    /// let degree = ScaleDegree {
    ///     semitone: 5,
    ///     letter: 1,
    /// };
    /// let root = PitchClass {
    ///     letter: Letter::C,
    ///     accidental: Accidental::Natural,
    /// };
    /// degree.to_pitch_class(root); // Panic!
    /// ```
    ///
    /// Here, the letter offset of 1 maps C to D, whose semitone offset from C
    /// natural is 2. Then, the required semitone offset of 5 is not reachable
    /// by appending any available accidental; the best we can do is double
    /// sharp, which would get us to semitone offset 4.
    pub fn to_pitch_class(self, base_pitch: PitchClass) -> PitchClass {
        let letter = base_pitch.letter.offset_by(self.letter);
        let letter_offset_from_base = PitchClass::natural(letter).semitone_offset_from(base_pitch);
        let semitone_error = letter_offset_from_base as i8 - self.semitone as i8;
        let accidental =
            Accidental::from_offset(-semitone_error)
            .unwrap_or_else(|| {
                panic!("invalid scale degree: letter {letter} (offset {}, semitones {}) is more than two semitones from required offset {}",
                    self.letter,
                    letter_offset_from_base,
                    self.semitone);
            });
        // .expect("invalid scale degree; letter offset is more than two semitones away from target semitone");

        PitchClass { letter, accidental }
    }

    /// Compute the pitch class of this scale degree from a given root pitch, with error checking.
    ///
    /// Returns `None` if the computed letter offset from the base pitch is
    /// more than two semitones away from the semitone offset required by this
    /// scale degree. See [`ScaleDegree::to_pitch_class`] for an example of
    /// this error.
    pub fn try_to_pitch_class(self, base_pitch: PitchClass) -> Option<PitchClass> {
        let letter = base_pitch.letter.offset_by(self.letter);
        let semitone_error = letter.semitone_offset() as i8 - self.semitone as i8;
        let accidental = Accidental::from_offset(-semitone_error)?;
        Some(PitchClass { letter, accidental })
    }
}

/// A family of scales all sharing the same scale degree offsets.
///
/// For instance, the diatonic major scales have the following scale degrees:
///
/// | Letter offset | Semitone offset | Example (D major) |
/// |---------------|-----------------|-------------------|
/// | 0             | 0               | D                 |
/// | 1             | 2               | E                 |
/// | 2             | 4               | F♯                |
/// | 3             | 5               | G                 |
/// | 4             | 7               | A                 |
/// | 5             | 9               | B                 |
/// | 6             | 11              | C♯                |
pub trait ScaleKind {
    /// Get the number of scale degrees in this scale kind.
    ///
    /// This trait method has a default implementation that simply counts the
    /// items yielded by [`ScaleKind::scale_degrees`]. It is encouraged to
    /// provide a more efficient implementation if possible, with the caveat
    /// that its value should match that returned by the default implementation.
    fn num_degrees() -> usize {
        Self::scale_degrees().count()
    }

    /// Get the n'th (modulo `self.num_degrees()`) scale degree of this scale kind.
    ///
    /// This trait method has a default implementation that simply takes the
    /// n'th item yielded by [`ScaleKind::scale_degrees`], modulo the value of
    /// [`ScaleKind::num_degrees`]. It is encouraged to provide a more efficient
    /// implementation if possible, with the caveat that its value should match
    /// that returned by the default implementation.
    fn scale_degree(n: usize) -> ScaleDegree {
        Self::scale_degrees().nth(n % Self::num_degrees()).unwrap()
    }

    /// Get an iterator over the scale degrees of this scale kind.
    fn scale_degrees() -> impl Iterator<Item = ScaleDegree>;
}

/// A mode of a scale kind.
///
/// A scale mode is characterized by a base [`ScaleKind`] (`K`) and a scale
/// degree offset (`N`). For instance, the natural minor scale is the Aeolian
/// mode of the diatonic scale, with the 6th scale degree of the diatonic major
/// scale mapped to the 1st scale degree of the Aeolian mode, so its base kind
/// is [`Diatonic`](diatonic::Diatonic) and its (zero-based) offset is 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mode<const N: usize, K> {
    _marker: std::marker::PhantomData<([(); N], K)>,
}

impl<const N: usize, K: ScaleKind> ScaleKind for Mode<N, K> {
    fn num_degrees() -> usize {
        K::num_degrees()
    }

    fn scale_degree(n: usize) -> ScaleDegree {
        K::scale_degree(n + N)
    }

    fn scale_degrees() -> impl Iterator<Item = ScaleDegree> {
        let n = Self::num_degrees();
        let offset = N % n;
        let mut base = K::scale_degree(offset);
        base.letter %= 7;
        base.semitone %= 12;
        K::scale_degrees()
            .skip(offset)
            .chain(K::scale_degrees().take(offset))
            .map(move |degree| {
                let letter = ((degree.letter % 7) + 7 - base.letter) % 7;
                let semitone = ((degree.semitone % 12) + 12 - base.semitone) % 12;
                ScaleDegree { letter, semitone }
            })
    }
}

/// A rooted scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Scale<K> {
    root: PitchClass,
    _marker: std::marker::PhantomData<K>,
}

impl<K> Scale<K> {
    /// Create a scale of kind `K` rooted at the given pitch class.
    pub fn new(root: PitchClass) -> Self {
        Self {
            root,
            _marker: Default::default(),
        }
    }

    /// Get the root pitch class of this scale.
    pub fn root(&self) -> PitchClass {
        self.root
    }
}

impl<K: ScaleKind> Scale<K> {
    /// Get the pitch class of the n'th (modulo the number of scale degrees) scale degree.
    ///
    /// # Panics
    ///
    /// Panics if the n'th scale degree from this scale's root note is not
    /// representable as a pitch class. See [`ScaleDegree::to_pitch_class`] for
    /// further information.
    pub fn scale_degree(&self, n: usize) -> PitchClass {
        let degree = K::scale_degree(n);
        degree.to_pitch_class(self.root())
    }

    /// Get the pitch class of the n'th (modulo the number of scale degrees)
    /// scale degree, with error checking.
    ///
    /// Returns `None` if the n'th scale degree from this scale's root note is
    /// not representable as a pitch class. See [`ScaleDegree::to_pitch_class`]
    /// for further information.
    pub fn try_scale_degree(&self, n: usize) -> Option<PitchClass> {
        let degree = K::scale_degree(n);
        degree.try_to_pitch_class(self.root())
    }

    /// Get an iterator over the pitch classes of this scale.
    pub fn scale_degrees(&self) -> impl Iterator<Item = PitchClass> {
        K::scale_degrees().map(|d| d.to_pitch_class(self.root))
    }
}
