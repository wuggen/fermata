//! Definitions and types for diatonic scales and their modes.

use super::*;

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

pub type IonianMode = Mode<0, Diatonic>;
pub type DorianMode = Mode<1, Diatonic>;
pub type PhrygianMode = Mode<2, Diatonic>;
pub type LydianMode = Mode<3, Diatonic>;
pub type MixolydianMode = Mode<4, Diatonic>;
pub type AeolianMode = Mode<5, Diatonic>;
pub type LocrianMode = Mode<6, Diatonic>;

pub type NaturalMajor = IonianMode;
pub type NaturalMinor = AeolianMode;
