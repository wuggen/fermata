//! Type representations for MIDI messages and values.
//!
//! This module includes definitions for the basic MIDI specification. The
//! General MIDI sound maps are defined in the
//! [`general_midi`](crate::general_midi) module.

use std::fmt::{self, Debug, Formatter};
use std::ops::Deref;

/// A MIDI message.
///
/// All data byte fields in this enum accept the full `u8` range. Per the MIDI
/// specification, all encoded data bytes must have their high bit cleared;
/// hence, during encoding, all data bytes will be masked to their low seven
/// bits. During decoding, an expected data byte with its high bit set will
/// result in an error.
///
/// Similarly, for channel messages, this enum allows a full `u8` for the
/// channel number. This will be masked to the low four bits during encoding,
/// and decoding will never result in a channel number greater than 15 (0x0f).
///
/// # Handling of System Exclusive messages
///
/// System exclusive messages can contain an arbitrary amount of data in their
/// payload. In order to keep this type small and `Copy`able, it does not
/// contain the data payload of system exclusive messages; instead, it contains
/// only the manufacturer ID that precedes the payload. It is the responsibility
/// of user code to encode and decode the contents of system exclusive messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Message {
    NoteOff {
        channel: u8,
        note: u8,
        velocity: u8,
    },

    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },

    KeyPressure {
        channel: u8,
        note: u8,
        pressure: u8,
    },

    ControlChange {
        channel: u8,
        controller: Controller,
        value: u8,
    },

    ProgramChange {
        channel: u8,
        program: u8,
    },

    ChannelPressure {
        channel: u8,
        pressure: u8,
    },

    PitchBend {
        channel: u8,
        pitch_bend: u16,
    },

    ChannelMode {
        channel: u8,
        mode: ChannelMode,
        value: u8,
    },

    SystemExclusive {
        id: ManufacturerId,
    },

    QuarterFrame {
        value: u8,
    },

    SongPositionPointer {
        pointer: u16,
    },

    SongSelect {
        song: u8,
    },

    TuneRequest,

    EndOfExclusive,

    TimingClock,

    Start,

    Continue,

    Stop,

    ActiveSensing,

    SystemReset,
}

/// An encoded MIDI message.
///
/// This is a thin wrapper around a small constant-sized byte array. Since MIDI
/// messages (except for system exclusive messages) are a maximum of four bytes
/// long, they can be encoded into such an array without need for allocation.
///
/// For system exclusive messages, this struct can be used to contain the
/// status byte and manufacturer ID only; the data payload for system exclusive
/// messages must be handled separately. See the note on system exclusive
/// messages in the documentation of [`Message`] for further details.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EncodedMessage {
    buf: [u8; 4],
    len: u8,
}

impl Deref for EncodedMessage {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buf[..self.len as usize]
    }
}

impl AsRef<[u8]> for EncodedMessage {
    fn as_ref(&self) -> &[u8] {
        &**self
    }
}

impl Debug for EncodedMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

/// Usage:
///
/// ```ignore
/// let b = chan!(c); // Clear the high four bits of `c`
/// let b = chan!(status, c); // Clears the high four bits of `c` and OR's the result with `status`
/// ```
macro_rules! chan {
    ($ch:expr) => {
        $ch & status::CHANNEL_MASK
    };

    ($st:expr, $ch:expr) => {
        $st | chan!($ch)
    };
}

/// Usage:
///
/// ```ignore
/// let b = data!(val); // Clears the high bit of `val`
/// ```
macro_rules! data {
    ($d:expr) => {
        $d & status::DATA_MASK
    };
}

/// Usage:
///
/// ```ignore
/// ck_data!(b1, b2, b3); // `return Err(InvalidDataByte)` if any arguments have their high bit set
/// ck_data!([] iterable); // `return Err(InvalidDataByte)` if any item of the given iterable has its high bit set
/// ```
macro_rules! ck_data {
    ([] $bytes:expr) => {
        for b in $bytes {
            ck_data!(b);
        }
    };

    ($($d:expr),* $(,)?) => {
        $(
            if $d & !status::DATA_MASK != 0 {
                return Err(DecodeError::InvalidDataByte);
            }
        )*
    };
}

/// Usage:
///
/// ```ignore
/// // `return Err(IncorrectLen)` if `array.len()` does not satisfy the given comparison
/// ck_len!(array, < 2);
/// ck_len!(array, == 1);
/// ck_len!(array, <= 5);
/// ```
macro_rules! ck_len {
    ($bytes:expr, $op:tt $len:expr) => {
        if !($bytes.len() $op $len) {
            return Err(DecodeError::IncorrectLen);
        }
    };
}

impl Message {
    /// Get the status byte of this message.
    pub fn status(&self) -> u8 {
        use status::*;

        match self {
            Message::NoteOff { channel, .. } => chan!(NOTE_OFF, channel),
            Message::NoteOn { channel, .. } => chan!(NOTE_ON, channel),
            Message::KeyPressure { channel, .. } => chan!(KEY_PRESSURE, channel),
            Message::ControlChange { channel, .. } => chan!(CONTROL_CHANGE, channel),
            Message::ProgramChange { channel, .. } => chan!(PROGRAM_CHANGE, channel),
            Message::ChannelPressure { channel, .. } => chan!(CHANNEL_PRESSURE, channel),
            Message::PitchBend { channel, .. } => chan!(PITCH_BEND, channel),
            Message::ChannelMode { channel, .. } => chan!(CHANNEL_MODE, channel),
            Message::SystemExclusive { .. } => SYSTEM_EXCLUSIVE,
            Message::QuarterFrame { .. } => SC_MTC_QUARTER_FRAME,
            Message::SongPositionPointer { .. } => SC_SONG_POSITION_POINTER,
            Message::SongSelect { .. } => SC_SONG_SELECT,
            Message::TuneRequest => SC_TUNE_REQUEST,
            Message::EndOfExclusive => SC_EOX,
            Message::TimingClock => RT_TIMING_CLOCK,
            Message::Start => RT_START,
            Message::Continue => RT_CONTINUE,
            Message::Stop => RT_STOP,
            Message::ActiveSensing => RT_ACTIVE_SENSING,
            Message::SystemReset => RT_SYSTEM_RESET,
        }
    }

    /// Get the encoded byte length of this message.
    ///
    /// This assumes no running status, i.e. the status byte is included in
    /// the encoding.
    ///
    /// For system exclusive messages, this function computes only the byte
    /// length of the status and manufacturer ID components of the message,
    /// disregarding any data payload.
    pub const fn encoded_len(&self) -> usize {
        match self {
            Message::NoteOff { .. }
            | Message::NoteOn { .. }
            | Message::KeyPressure { .. }
            | Message::ControlChange { .. }
            | Message::PitchBend { .. }
            | Message::ChannelMode { .. }
            | Message::SongPositionPointer { .. } => 3,
            Message::ProgramChange { .. }
            | Message::ChannelPressure { .. }
            | Message::QuarterFrame { .. }
            | Message::SongSelect { .. } => 2,
            Message::TuneRequest
            | Message::EndOfExclusive
            | Message::TimingClock
            | Message::Start
            | Message::Continue
            | Message::Stop
            | Message::ActiveSensing
            | Message::SystemReset => 1,
            Message::SystemExclusive { id } => match id {
                ManufacturerId::Basic(_) => 2,
                ManufacturerId::Extended(_) => 4,
            },
        }
    }

    /// Encode this message its over-the-wire byte form.
    ///
    /// For system exclusive messages, the data payload must be encoded
    /// separately; see [`Message`] for further details.
    ///
    /// If `encode_status` is false, the message status byte is not encoded,
    /// unless this would result in an empty message (i.e. `self` represents
    /// a message that has no data payload). This functionality is to support
    /// encoding of running status.
    pub fn encode(&self, encode_status: bool) -> EncodedMessage {
        let mut enc = EncodedMessage::new();

        if encode_status {
            enc.push(self.status());
        }

        match self {
            Message::NoteOff { note, velocity, .. } | Message::NoteOn { note, velocity, .. } => {
                enc.push_all(&[data!(note), data!(velocity)])
            }
            Message::KeyPressure { note, pressure, .. } => {
                enc.push_all(&[data!(note), data!(pressure)])
            }
            Message::ControlChange {
                controller, value, ..
            } => enc.push_all(&[controller.controller_number(), data!(value)]),
            Message::ProgramChange { program, .. } => enc.push(data!(program)),
            Message::ChannelPressure { pressure, .. } => enc.push(data!(pressure)),
            Message::PitchBend { pitch_bend, .. } => {
                let lsb = *pitch_bend as u8 & status::DATA_MASK;
                let msb = (pitch_bend >> 7) as u8 & status::DATA_MASK;
                enc.push_all(&[lsb, msb]);
            }
            Message::ChannelMode { mode, value, .. } => {
                enc.push_all(&[mode.mode_number(), data!(value)])
            }
            Message::SystemExclusive { id } => id.encode(&mut enc),
            Message::QuarterFrame { value } => enc.push(data!(value)),
            Message::SongPositionPointer { pointer } => {
                let lsb = *pointer as u8 & status::DATA_MASK;
                let msb = (pointer >> 7) as u8 & status::DATA_MASK;
                enc.push_all(&[lsb, msb]);
            }
            Message::SongSelect { song } => enc.push(data!(song)),

            // For dataless messages, always encode the status byte (there would
            // be no message otherwise)
            _ => {
                if !encode_status {
                    enc.push(self.status());
                }
            }
        }

        enc
    }

    /// Decode a message from a byte slice.
    ///
    /// The given slice must contain exactly one MIDI message, beginning with
    /// a valid status byte unless a running status is provided. If a running
    /// status is provided, and the given slice does not begin with a status
    /// byte, the given running status is assumed.
    ///
    /// For system exclusive messages, this function will decode the
    /// manufacturer ID only; the contents of the slice following the ID is
    /// assumed to be the data payload, and is not examined.
    pub fn decode(mut bytes: &[u8], running_status: Option<u8>) -> Result<Self, DecodeError> {
        let first = *bytes.get(0).ok_or(DecodeError::EmptyBuffer)?;

        let status = match (running_status, first) {
            (Some(running), stat) if stat & 0x80 == 0 && running & 0x80 != 0 => running,
            (_, stat) if stat & 0x80 != 0 => {
                bytes = bytes.split_off(1..).unwrap();
                stat
            }
            _ => return Err(DecodeError::MissingStatus),
        };

        debug_assert!(status & 0x80 != 0);

        match status {
            0x80..=0xef => {
                let channel = status & 0x0f;
                let status = status & 0xf0;

                match status {
                    status::NOTE_OFF | status::NOTE_ON => {
                        ck_len!(bytes, ==2);
                        ck_data!([] bytes);
                        let note = bytes[0];
                        let velocity = bytes[1];
                        if status == status::NOTE_OFF {
                            Ok(Message::NoteOff {
                                channel,
                                note,
                                velocity,
                            })
                        } else {
                            Ok(Message::NoteOn {
                                channel,
                                note,
                                velocity,
                            })
                        }
                    }
                    status::KEY_PRESSURE => {
                        ck_len!(bytes, ==2);
                        ck_data!([] bytes);
                        let note = bytes[0];
                        let pressure = bytes[1];
                        Ok(Message::KeyPressure {
                            channel,
                            note,
                            pressure,
                        })
                    }
                    // Also the status byte for ChannelMode:
                    status::CONTROL_CHANGE => {
                        ck_len!(bytes, ==2);
                        ck_data!([] bytes);
                        let value = bytes[1];

                        if let Some(controller) = Controller::from_controller_number(bytes[0]) {
                            Ok(Message::ControlChange {
                                channel,
                                controller,
                                value,
                            })
                        } else {
                            let mode = ChannelMode::from_mode_number(bytes[0]).unwrap();
                            Ok(Message::ChannelMode {
                                channel,
                                mode,
                                value,
                            })
                        }
                    }
                    status::PROGRAM_CHANGE => {
                        ck_len!(bytes, ==1);
                        ck_data!(bytes[0]);
                        let program = bytes[0];
                        Ok(Message::ProgramChange { channel, program })
                    }
                    status::CHANNEL_PRESSURE => {
                        ck_len!(bytes, ==1);
                        ck_data!(bytes[0]);
                        let pressure = bytes[0];
                        Ok(Message::ChannelPressure { channel, pressure })
                    }
                    status::PITCH_BEND => {
                        ck_len!(bytes, ==2);
                        ck_data!([] bytes);
                        let lsb = bytes[0];
                        let msb = bytes[1];
                        let pitch_bend = (lsb as u16) | ((msb as u16) << 7);
                        Ok(Message::PitchBend {
                            channel,
                            pitch_bend,
                        })
                    }
                    _ => unreachable!(),
                }
            }

            status::SYSTEM_EXCLUSIVE => {
                ck_len!(bytes, >=1);
                let id = if bytes[0] == 0 {
                    ck_len!(bytes, >= 3);
                    ManufacturerId::Extended((bytes[2] as u16) | ((bytes[1] as u16) << 8))
                } else {
                    ck_data!(bytes[0]);
                    ManufacturerId::Basic(bytes[0])
                };

                Ok(Message::SystemExclusive { id })
            }

            status::SC_MTC_QUARTER_FRAME => {
                ck_len!(bytes, ==1);
                ck_data!(bytes[0]);
                let value = bytes[0];
                Ok(Message::QuarterFrame { value })
            }

            status::SC_SONG_POSITION_POINTER => {
                ck_len!(bytes, ==2);
                ck_data!([] bytes);
                let lsb = bytes[0] as u16;
                let msb = (bytes[1] as u16) << 7;
                let pointer = msb | lsb;
                Ok(Message::SongPositionPointer { pointer })
            }

            status::SC_SONG_SELECT => {
                ck_len!(bytes, ==1);
                ck_data!(bytes[0]);
                let song = bytes[0];
                Ok(Message::SongSelect { song })
            }

            status::SC_TUNE_REQUEST => {
                ck_len!(bytes, ==0);
                Ok(Message::TuneRequest)
            }

            status::SC_EOX => {
                ck_len!(bytes, ==0);
                Ok(Message::EndOfExclusive)
            }

            status::RT_TIMING_CLOCK => {
                ck_len!(bytes, ==0);
                Ok(Message::TimingClock)
            }

            status::RT_START => {
                ck_len!(bytes, ==0);
                Ok(Message::Start)
            }

            status::RT_CONTINUE => {
                ck_len!(bytes, ==0);
                Ok(Message::Continue)
            }

            status::RT_STOP => {
                ck_len!(bytes, ==0);
                Ok(Message::Stop)
            }

            status::RT_ACTIVE_SENSING => {
                ck_len!(bytes, ==0);
                Ok(Message::ActiveSensing)
            }

            status::RT_SYSTEM_RESET => {
                ck_len!(bytes, ==0);
                Ok(Message::SystemReset)
            }

            _ => Err(DecodeError::UnrecognizedStatus),
        }
    }
}

impl EncodedMessage {
    /// Create a new zeroed encoded message buffer.
    const fn new() -> Self {
        Self {
            buf: [0; 4],
            len: 0,
        }
    }

    /// Push a single byte to this buffer.
    ///
    /// Panics if the buffer already contains 4 bytes.
    fn push(&mut self, byte: u8) {
        *self
            .buf
            .get_mut(self.len as usize)
            .expect("overfull message buffer") = byte;
        self.len += 1;
    }

    /// Push all of the given bytes in sequence.
    ///
    /// Panics if this would result in more than four bytes in the buffer.
    fn push_all(&mut self, bytes: &[u8]) {
        self.buf
            .get_mut(self.len as usize..self.len as usize + bytes.len())
            .expect("overfull message buffer")
            .copy_from_slice(bytes);
        self.len += bytes.len() as u8;
    }
}

/// MIDI message decoding errors.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    /// An empty message buffer was provided.
    #[error("empty message")]
    EmptyBuffer,

    /// The message buffer does not start with a status byte, and no running
    /// status was provided.
    #[error("missing status byte")]
    MissingStatus,

    /// An expected data byte had its high bit set.
    #[error("invalid data byte in message")]
    InvalidDataByte,

    /// The message buffer is of an incorrect length for the message type.
    #[error("incorrect message length")]
    IncorrectLen,

    /// The status byte for the message is unrecognized or undefined.
    ///
    /// The MIDI specification recommends that such messages be silently
    /// ignored.
    #[error("unrecognized status byte")]
    UnrecognizedStatus,
}

macro_rules! impl_controller {
    (
        $($Name:ident = $num:expr),* $(,)?
    ) => {
        /// A MIDI controller.
        ///
        /// This enum includes all defined MIDI controller numbers, plus the variant `Undefined` for all
        /// other values.
        ///
        /// The `Undefined` variant is unrestricted in the byte value it can contain,
        /// and when encoding it will be written faithfully (except that the high bit
        /// will be cleared, as with all data bytes). When decoding, however, a more
        /// specific variant will always be returned if it exists.
        ///
        /// Note that bytes 120 through 127 (`0x78` through `0x7f`) are not
        /// valid controller numbers, even for an undefined controller; rather,
        /// they are reserved to designate [`ChannelMode`]s, since the status
        /// bytes for Control Change and Channel Mode messages are equal. Here
        /// too, such a controller number will be encoded faithfully if given in
        /// an `Undefined`; however, decoding such a message will instead yield
        /// a [`Message::ChannelMode`].
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Controller {
            $(
                $Name,
            )*
            Undefined(u8),
        }

        impl Controller {
            /// Get the data byte for this controller.
            ///
            /// If `self` is `Controller::Undefined`, the high bit of the
            /// contained byte will be cleared.
            pub const fn controller_number(self) -> u8 {
                match self {
                    $(
                        Self::$Name => $num,
                    )*
                    Self::Undefined(b) => data!(b),
                }
            }

            /// Create a `Controller` from its encoded data byte.
            ///
            /// This will return None if the given byte is either (a) not a data byte
            /// (high bit is set) or (b) encodes a [`ChannelMode`] instead (in the range
            /// 120-127).
            pub const fn from_controller_number(number: u8) -> Option<Self> {
                match number {
                    $(
                        $num => Some(Self::$Name),
                    )*

                    120..=255 => None,
                    b => Some(Self::Undefined(b)),
                }
            }
        }
    };
}

impl_controller! {
    BankSelectMsb = 0x00,
    BankSelectLsb = 0x20,
    ModulationMsb = 0x01,
    ModulationLsb = 0x21,
    BreathControlMsb = 0x02,
    BreathControlLsb = 0x22,
    FootControllerMsb = 0x04,
    FootControllerLsb = 0x24,
    PortamentoTimeMsb = 0x05,
    PortamentoTimeLsb = 0x25,
    DataEntryMsb = 0x06,
    DataEntryLsb = 0x26,
    ChannelVolumeMsb = 0x07,
    ChannelVolumeLsb = 0x27,
    BalanceMsb = 0x08,
    BalanceLsb = 0x28,
    PanMsb = 0x0a,
    PanLsb = 0x2a,
    ExpressionControllerMsb = 0x0b,
    ExpressionControllerLsb = 0x2b,
    EffectControl1Msb = 0x0c,
    EffectControl1Lsb = 0x2c,
    EffectControl2Msb = 0x0d,
    EffectControl2Lsb = 0x2d,
    GeneralPurposeController1Msb = 0x10,
    GeneralPurposeController1Lsb = 0x30,
    GeneralPurposeController2Msb = 0x11,
    GeneralPurposeController2Lsb = 0x31,
    GeneralPurposeController3Msb = 0x12,
    GeneralPurposeController3Lsb = 0x32,
    GeneralPurposeController4Msb = 0x13,
    GeneralPurposeController4Lsb = 0x33,

    Sustain = 0x40,
    PortamentoOnOff = 0x41,
    Sostenuto = 0x42,
    SoftPedal = 0x43,
    LegatoFootswitch = 0x44,
    Hold2 = 0x45,
    SoundController1Variation = 0x46,
    SoundController2Timbre = 0x47,
    SoundController3ReleaseTime = 0x48,
    SoundController4AttackTime = 0x49,
    SoundController5Brightness = 0x4a,
    SoundController6 = 0x4b,
    SoundController7 = 0x4c,
    SoundController8 = 0x4d,
    SoundController9 = 0x4e,
    SoundController10 = 0x4f,

    GeneralPurposeController5 = 0x50,
    GeneralPurposeController6 = 0x51,
    GeneralPurposeController7 = 0x52,
    GeneralPurposeController8 = 0x53,
    PortamentoControl = 0x54,
    Effects1Depth = 0x5b,
    Effects2Depth = 0x5c,
    Effects3Depth = 0x5d,
    Effects4Depth = 0x5e,
    Effects5Depth = 0x5f,

    DataIncrement = 0x60,
    DataDecrement = 0x61,
    NonRegisteredParamLsb = 0x62,
    NonRegisteredParamMsb = 0x63,
    RegisteredParamLsb = 0x64,
    RegisteredParamMsb = 0x65,
}

macro_rules! impl_channel_mode {
    (
        $(
            $Name:ident = $num:expr
        ),* $(,)?
    ) => {
        /// A MIDI channel mode.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum ChannelMode {
            $(
                $Name,
            )*
        }

        impl ChannelMode {
            /// Get the encoded data byte for this channel mode.
            pub const fn mode_number(self) -> u8 {
                match self {
                    $(
                        Self::$Name => $num,
                    )*
                }
            }

            /// Create a `ChannelMode` from its encoded data byte.
            ///
            /// Returns None if the given byte does not encode a channel mode.
            pub const fn from_mode_number(number: u8) -> Option<Self> {
                match number {
                    $(
                        $num => Some(Self::$Name),
                    )*
                    _ => None,
                }
            }
        }
    };
}

impl_channel_mode! {
    AllSoundOff = 120,
    ResetAllControllers = 121,
    LocalControl = 122,
    AllNotesOff = 123,
    OmniModeOff = 124,
    OmniModeOn = 125,
    MonoModeOn = 126,
    PolyModeOn = 127,
}

/// A system exclusive manufacturer ID.
///
/// MIDI System Exclusive messages are identified by one or three bytes
/// following the status byte. If the first byte following the status is
/// non-zero, then it is the ID; if it is zero, then the following two bytes
/// constitute the ID. This enum represents these two modes of ID with the
/// variants `Basic` (for single-byte IDs) and `Extended` (for two-byte IDs).
///
/// Extended IDs are represented here by a `u16`; the high byte corresponds
/// to the first non-zero byte of the ID, and the low byte corresponds to the
/// second non-zero byte.
///
/// Manufacturer ID bytes are the only non-status bytes that are permitted
/// to have their high bits set; there are a handful of registered extended
/// manufacturer IDs whose third byte has a set high bit. This library permits
/// (for both encoding and decoding) any byte in either basic or extended IDs to
/// have a set high bit.
///
/// This type has associated contants for those ID numbers defined or reserved
/// by the MIDI specification for universal system exclusive messages, or for use in
/// non-commercial purposes. For all other registered manufacturer IDs, please
/// refer to the MIDI specification.
///
/// Note that, although the data payload format for universal system exclusive
/// messages is defined by the MIDI specification, this library does not
/// facilitate the encoding or decoding of such messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManufacturerId {
    Basic(u8),
    Extended(u16),
}

impl ManufacturerId {
    pub const NON_COMMERCIAL: Self = Self::Basic(0x7d);
    pub const UNIVERSAL_NON_REAL_TIME: Self = Self::Basic(0x7e);
    pub const UNIVERSAL_REAL_TIME: Self = Self::Basic(0x7f);

    fn encode(&self, enc: &mut EncodedMessage) {
        match self {
            ManufacturerId::Basic(id) => {
                assert!(*id != 0, "manufacturer ID cannot be 0");
                enc.push(*id);
            }
            ManufacturerId::Extended(id) => {
                let lsb = *id as u8;
                let msb = (id >> 8) as u8;
                enc.push_all(&[0, msb, lsb]);
            }
        }
    }
}

pub mod status {
    //! Constants for MIDI message status bytes.

    /// Bit mask for data bytes in a MIDI message.
    ///
    /// Data bytes must have their high byte cleared.
    pub const DATA_MASK: u8 = 0x7f;

    /// Status byte for a Note Off message on channel 0.
    pub const NOTE_OFF: u8 = 0x80;

    /// Status byte for a Note On message on channel 0.
    pub const NOTE_ON: u8 = 0x90;

    /// Status byte for a Polyphonic Key Pressure message on channel 0.
    pub const KEY_PRESSURE: u8 = 0xa0;

    /// Status byte for a Control Change message on channel 0.
    ///
    /// This is equal to [`CHANNEL_MODE`], the status byte for Select Channel Mode
    /// messages. The two message kinds are distinguished by the values of their
    /// first data byte; Control Change messages must have a first data byte
    /// less than 120 (0x78).
    pub const CONTROL_CHANGE: u8 = 0xb0;

    /// Status byte for a Program Change message on channel 0.
    pub const PROGRAM_CHANGE: u8 = 0xc0;

    /// Status byte for a Channel Pressure message on channel 0.
    pub const CHANNEL_PRESSURE: u8 = 0xd0;

    /// Status byte for a Pitch Bend message on channel 0.
    pub const PITCH_BEND: u8 = 0xe0;

    /// Status byte for a Select Channel Mode message on channel 0.
    ///
    /// This is equal to [`CONTROL_CHANGE`], the status byte for Control Change
    /// messages. The two message kinds are distinguished by the values of their
    /// first data byte; Channel Mode messages must have a first data byte
    /// greater than or equal to 120 (0x78).
    pub const CHANNEL_MODE: u8 = 0xb0;

    /// Bit mask for channel values.
    ///
    /// For channel messages, the targeted channel is encoded in the low four
    /// bits of the status byte. This mask can be used to isolate the low four
    /// bits of a byte, the result of which can then be OR'd with the status
    /// byte constant.
    pub const CHANNEL_MASK: u8 = 0x0f;

    /// Status byte for a System Exclusive message.
    pub const SYSTEM_EXCLUSIVE: u8 = 0xf0;

    /// Status byte for an MTC Quarter Frame system common message.
    pub const SC_MTC_QUARTER_FRAME: u8 = 0xf1;

    /// Status byte for a Song Position Pointer system common message.
    pub const SC_SONG_POSITION_POINTER: u8 = 0xf2;

    /// Status byte for a Song Select system common message.
    pub const SC_SONG_SELECT: u8 = 0xf3;

    /// Status byte for a Tune Request system common message.
    pub const SC_TUNE_REQUEST: u8 = 0xf6;

    /// Status byte for an EOX (End Of Exclusive) system common message.
    pub const SC_EOX: u8 = 0xf7;

    /// Status byte for a Timing Clock system real time message.
    pub const RT_TIMING_CLOCK: u8 = 0xf8;

    /// Status byte for a Start system real time message.
    pub const RT_START: u8 = 0xfa;

    /// Status byte for a Continue system real time message.
    pub const RT_CONTINUE: u8 = 0xfb;

    /// Status byte for a Stop system real time message.
    pub const RT_STOP: u8 = 0xfc;

    /// Status byte for an Active Sensing system real time message.
    pub const RT_ACTIVE_SENSING: u8 = 0xfe;

    /// Status byte for a System Reset system real time message.
    pub const RT_SYSTEM_RESET: u8 = 0xff;
}

pub mod param {
    //! Constants for MIDI Registered Parameter Numbers.
    //!
    //! These are the performance configuration parameters numbers defined by
    //! the MIDI specification, for use with controller numbers 100 and 101,
    //! Registered Parameter Number LSB and Registered Parameter Number MSB.

    pub const PITCH_BEND_SENSITIVITY_LSB: u8 = 0x00;
    pub const PITCH_BEND_SENSITIVITY_MSB: u8 = 0x00;

    pub const FINE_TUNING_LSB: u8 = 0x01;
    pub const FINE_TUNING_MSB: u8 = 0x00;

    pub const COARSE_TUNING_LSB: u8 = 0x02;
    pub const COARSE_TUNING_MSB: u8 = 0x00;

    pub const TUNING_PROGRAM_SELECT_LSB: u8 = 0x03;
    pub const TUNING_PROGRAM_SELECT_MSB: u8 = 0x00;

    pub const TUNING_BANK_SELECT_LSB: u8 = 0x04;
    pub const TUNING_BANK_SELECT_MSB: u8 = 0x00;
}

#[cfg(test)]
mod test {
    use super::*;

    /// Invalid controller numbers result in `None` from `Controller::from_controller_number`
    #[test]
    fn from_controller_number_none() {
        assert!(Controller::from_controller_number(120).is_none());
        assert!(Controller::from_controller_number(128).is_none());
        assert!(Controller::from_controller_number(255).is_none());
    }

    /// Agreement between `Message::encode()` and `Message::encoded_len()`
    #[test]
    fn encoded_len() {
        let messages = [
            Message::NoteOff {
                channel: 0,
                note: 64,
                velocity: 64,
            },
            Message::NoteOn {
                channel: 0,
                note: 64,
                velocity: 64,
            },
            Message::KeyPressure {
                channel: 0,
                note: 64,
                pressure: 64,
            },
            Message::ControlChange {
                channel: 0,
                controller: Controller::Sustain,
                value: 64,
            },
            Message::ProgramChange {
                channel: 0,
                program: 0,
            },
            Message::ChannelPressure {
                channel: 0,
                pressure: 64,
            },
            Message::PitchBend {
                channel: 0,
                pitch_bend: 128,
            },
            Message::ChannelMode {
                channel: 0,
                mode: ChannelMode::AllSoundOff,
                value: 64,
            },
            Message::SystemExclusive {
                id: ManufacturerId::Basic(12),
            },
            Message::SystemExclusive {
                id: ManufacturerId::Extended(130),
            },
            Message::QuarterFrame { value: 64 },
            Message::SongPositionPointer { pointer: 130 },
            Message::SongSelect { song: 12 },
            Message::TuneRequest,
            Message::EndOfExclusive,
            Message::TimingClock,
            Message::Start,
            Message::Continue,
            Message::Stop,
            Message::ActiveSensing,
            Message::SystemReset,
        ];

        for msg in messages {
            let encoded = msg.encode(true);
            let encoded_len = msg.encoded_len();

            assert_eq!(
                encoded.len(),
                encoded_len,
                "mismatch between expected and actual encoded len: {msg:?} => {encoded:?} (expected len {encoded_len})"
            );
        }
    }
}
