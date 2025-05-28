//! Type representations for MIDI messages and values.

use std::fmt::{self, Debug, Formatter};
use std::io;
use std::ops::Deref;

#[macro_export]
macro_rules! note_off {
    ($n:expr, $v:expr $(,)?) => {
        $crate::note_off!(0, $n, $v)
    };

    ($c:expr, $n:expr, $v:expr $(,)?) => {
        $crate::midi::Message::Channel {
            channel: $c,
            message: $crate::midi::ChannelMessage::NoteOff {
                note: $n,
                velocity: $v,
            },
        }
    };
}

#[macro_export]
macro_rules! note_on {
    ($n:expr, $v:expr $(,)?) => {
        $crate::note_on!(0, $n, $v)
    };

    ($c:expr, $n:expr, $v:expr $(,)?) => {
        $crate::midi::Message::Channel {
            channel: $c,
            message: $crate::midi::ChannelMessage::NoteOn {
                note: $n,
                velocity: $v,
            },
        }
    };
}

/// An encoded MIDI message.
///
/// This is a thin wrapper around a small constant-sized byte array. Since MIDI
/// messages (except for system exclusive messages) are a maximum of three bytes
/// long, they can be encoded into such an array without need for allocation.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EncodedMessage {
    buf: [u8; 3],
    len: usize,
}

impl Debug for EncodedMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl Deref for EncodedMessage {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buf[..self.len]
    }
}

impl AsRef<[u8]> for EncodedMessage {
    fn as_ref(&self) -> &[u8] {
        &*self
    }
}

impl EncodedMessage {
    fn new() -> Self {
        Self {
            buf: [0; 3],
            len: 0,
        }
    }

    fn push(&mut self, b: u8) {
        debug_assert!(self.len < 3);
        self.buf[self.len] = b;
        self.len += 1;
    }

    fn push_all(&mut self, bs: &[u8]) {
        debug_assert!(self.len + bs.len() <= 3);
        let subbuf = &mut self.buf[self.len..self.len + bs.len()];
        subbuf.copy_from_slice(bs);
        self.len += bs.len();
    }
}

/// A MIDI message.
///
/// All data byte fields of all message types in this enum accept the full `u8`
/// range. The MIDI specification requires that encoded data bytes have their
/// high bit cleared; hence, during encoding, all data byte values will be
/// truncated to their low seven bits.
///
/// Similarly, for channel messages, this enum allows a full `u8` for the
/// channel number. When encoding a message to a byte stream, this number will
/// be truncated to the low _four_ bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Message {
    Channel {
        channel: u8,
        message: ChannelMessage,
    },
    System(SystemMessage),
}

impl Message {
    /// Get the status byte for this message.
    pub const fn status(&self) -> u8 {
        match self {
            Message::Channel { channel, message } => message.status() | (*channel & 0x0f),
            Message::System(message) => message.status(),
        }
    }

    /// Decode a MIDI message from a byte array.
    ///
    /// The given byte array must contain exactly one encoded MIDI message.
    ///
    /// If the given bytes do not have an initial status byte, and
    /// `running_status` is Some, the value of `running_status` is regarded as
    /// the status byte of the current MIDI message.
    pub fn decode(mut bytes: &[u8], running_status: Option<u8>) -> Result<Self, DecodeError> {
        if bytes.is_empty() {
            return Err(DecodeError::EmptyMessage);
        }

        let status = if let Some(running) = running_status {
            if bytes[0] & 0x80 == 0 {
                running
            } else {
                let (stat, rest) = bytes.split_first().unwrap();
                bytes = rest;
                *stat
            }
        } else {
            if bytes[0] & 0x80 == 0 {
                return Err(DecodeError::MissingStatus);
            } else {
                let (stat, rest) = bytes.split_first().unwrap();
                bytes = rest;
                *stat
            }
        };

        let high_nibble = status & 0xf0;

        if high_nibble == 0xf0 {
            Ok(Self::System(SystemMessage::decode(status, bytes)?))
        } else {
            ChannelMessage::decode(status, bytes)
        }
    }

    /// Encode this message into a byte array.
    ///
    /// If `encode_status` is false, the message's status byte will not be
    /// encoded (unless the message consists of _only_ a status byte). This is
    /// to facilitate the transmission of running status messages.
    pub fn encode(&self, encode_status: bool) -> EncodedMessage {
        let mut enc = EncodedMessage::new();

        if encode_status {
            enc.push(self.status());
        }

        match self {
            Message::Channel { message, .. } => message.encode(&mut enc),
            Message::System(message) => message.encode(&mut enc),
        }

        if enc.is_empty() {
            enc.push(self.status());
        }

        enc
    }

    /// Encode a MIDI message into a byte stream.
    ///
    /// If `encode_status` is false, the message's status byte will not be
    /// written to the stream (unless the message consists of _only_ a status
    /// byte). This is to facilitate the transmission of running status
    /// messages.
    pub fn encode_into<W: io::Write>(
        &self,
        output: &mut W,
        encode_status: bool,
    ) -> io::Result<usize> {
        let enc = self.encode(encode_status);
        output.write_all(&enc)?;
        Ok(enc.len())
    }
}

/// A MIDI channel message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelMessage {
    NoteOff { note: u8, velocity: u8 },

    NoteOn { note: u8, velocity: u8 },

    KeyPressure { note: u8, value: u8 },

    ControlChange { controller: Controller, value: u8 },

    ProgramChange { value: u8 },

    ChannelPressure { value: u8 },

    PitchBend { value: u16 },

    ChannelMode { mode: ChannelMode, value: u8 },
}

macro_rules! decode_helper {
    (lenck, $val:expr, $bytes:ident) => {
        if $bytes.len() != $val {
            return Err(DecodeError::IncorrectLen);
        }
    };

    (datack, $($b:expr),*) => {
        $(
            if $b & 0x80 != 0 {
                return Err(DecodeError::InvalidValue);
            }
        )*
    };
}

impl ChannelMessage {
    /// Get the status byte for this message type.
    ///
    /// Channel messages encode their target channel in the low four bits of
    /// the status byte. Since this enum does not encode the channel number, the
    /// status byte returned by this method will be targeted to channel 0.
    pub const fn status(&self) -> u8 {
        match self {
            ChannelMessage::NoteOff { .. } => status::NOTE_OFF,
            ChannelMessage::NoteOn { .. } => status::NOTE_ON,
            ChannelMessage::KeyPressure { .. } => status::KEY_PRESSURE,
            ChannelMessage::ControlChange { .. } => status::CONTROL_CHANGE,
            ChannelMessage::ProgramChange { .. } => status::PROGRAM_CHANGE,
            ChannelMessage::ChannelPressure { .. } => status::CHANNEL_PRESSURE,
            ChannelMessage::PitchBend { .. } => status::PITCH_BEND,
            ChannelMessage::ChannelMode { .. } => status::CHANNEL_MODE,
        }
    }

    fn decode(status: u8, data: &[u8]) -> Result<Message, DecodeError> {
        let channel = status & 0x0f;
        let message = match status & 0xf0 {
            status::NOTE_OFF | status::NOTE_ON => {
                decode_helper!(lenck, 2, data);

                let note = data[0];
                let velocity = data[1];

                decode_helper!(datack, note, velocity);

                if status & 0xf0 == status::NOTE_OFF {
                    Ok(ChannelMessage::NoteOff { note, velocity })
                } else {
                    Ok(ChannelMessage::NoteOn { note, velocity })
                }
            }

            status::KEY_PRESSURE => {
                decode_helper!(lenck, 2, data);

                let note = data[0];
                let value = data[1];
                decode_helper!(datack, note, value);

                Ok(ChannelMessage::KeyPressure { note, value })
            }

            status::CONTROL_CHANGE => {
                decode_helper!(lenck, 2, data);
                decode_helper!(datack, data[0], data[1]);

                if data[0] < 120 {
                    let controller = Controller::from_controller_number(data[0]).unwrap();
                    let value = data[1];
                    Ok(ChannelMessage::ControlChange { controller, value })
                } else {
                    let mode = ChannelMode::from_mode_number(data[0]).unwrap();
                    let value = data[1];
                    Ok(ChannelMessage::ChannelMode { mode, value })
                }
            }

            status::PROGRAM_CHANGE => {
                decode_helper!(lenck, 1, data);

                let value = data[0];
                decode_helper!(datack, value);

                Ok(ChannelMessage::ProgramChange { value })
            }

            status::CHANNEL_PRESSURE => {
                decode_helper!(lenck, 1, data);
                let value = data[0];
                decode_helper!(datack, value);

                Ok(ChannelMessage::ChannelPressure { value })
            }

            status::PITCH_BEND => {
                decode_helper!(lenck, 2, data);

                let lsb = data[0];
                let msb = data[1];
                decode_helper!(datack, lsb, msb);

                let value = (lsb as u16) | ((msb as u16) << 7);
                Ok(ChannelMessage::PitchBend { value })
            }

            _ => unreachable!(),
        }?;

        Ok(Message::Channel { channel, message })
    }

    fn encode(&self, enc: &mut EncodedMessage) {
        match self {
            ChannelMessage::NoteOff { note, velocity }
            | ChannelMessage::NoteOn { note, velocity } => {
                enc.push_all(&[note & 0x7f, velocity & 0x7f]);
            }
            ChannelMessage::KeyPressure { note, value } => {
                enc.push_all(&[note & 0x7f, value & 0x7f]);
            }
            ChannelMessage::ControlChange { controller, value } => {
                enc.push_all(&[controller.controller_number(), value & 0x7f]);
            }
            ChannelMessage::ProgramChange { value } => {
                enc.push(value & 0x7f);
            }
            ChannelMessage::ChannelPressure { value } => {
                enc.push(value & 0x7f);
            }
            ChannelMessage::PitchBend { value } => {
                let lsb = *value as u8 & 0x7f;
                let msb = (value >> 7) as u8 & 0x7f;
                enc.push_all(&[lsb, msb]);
            }
            ChannelMessage::ChannelMode { mode, value } => {
                enc.push_all(&[mode.mode_number(), value & 0x7f]);
            }
        }
    }
}

/// A MIDI system message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemMessage {
    SystemExclusive,

    SystemCommon(SystemCommonMessage),

    SystemRealTime(SystemRealTimeMessage),
}

impl SystemMessage {
    /// Get the status byte for this system message.
    pub const fn status(&self) -> u8 {
        match self {
            SystemMessage::SystemExclusive => status::SYSTEM_EXCLUSIVE,
            SystemMessage::SystemCommon(msg) => msg.status(),
            SystemMessage::SystemRealTime(msg) => msg.status(),
        }
    }

    fn encode(&self, encoded: &mut EncodedMessage) {
        match self {
            SystemMessage::SystemCommon(message) => message.encode(encoded),
            _ => {}
        }
    }

    fn decode(status: u8, data: &[u8]) -> Result<Self, DecodeError> {
        if status == status::SYSTEM_EXCLUSIVE {
            Ok(Self::SystemExclusive)
        } else if status & 0x08 == 0 {
            Ok(Self::SystemCommon(SystemCommonMessage::decode(
                status, data,
            )?))
        } else {
            decode_helper!(lenck, 0, data);
            Ok(Self::SystemRealTime(
                SystemRealTimeMessage::from_status(status).unwrap(),
            ))
        }
    }
}

/// A MIDI System Common message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemCommonMessage {
    MtcQuarterFrame { kind: u8, value: u8 },

    SongPositionPointer { value: u8 },

    SongSelect { value: u8 },

    TuneRequest,

    EndOfExclusive,
}

impl SystemCommonMessage {
    /// Get the status byte for this message.
    pub const fn status(&self) -> u8 {
        match self {
            SystemCommonMessage::MtcQuarterFrame { .. } => status::SC_MTC_QUARTER_FRAME,
            SystemCommonMessage::SongPositionPointer { .. } => status::SC_SONG_POSITION_POINTER,
            SystemCommonMessage::SongSelect { .. } => status::SC_SONG_SELECT,
            SystemCommonMessage::TuneRequest => status::SC_TUNE_REQUEST,
            SystemCommonMessage::EndOfExclusive => status::SC_EOX,
        }
    }

    fn decode(status: u8, data: &[u8]) -> Result<Self, DecodeError> {
        match status {
            status::SC_MTC_QUARTER_FRAME => {
                decode_helper!(lenck, 2, data);

                let kind = data[0];
                let value = data[1];
                decode_helper!(datack, kind, value);

                Ok(Self::MtcQuarterFrame { kind, value })
            }

            status::SC_SONG_POSITION_POINTER => {
                decode_helper!(lenck, 1, data);
                let value = data[0];
                decode_helper!(datack, value);
                Ok(Self::SongPositionPointer { value })
            }

            status::SC_SONG_SELECT => {
                decode_helper!(lenck, 1, data);
                let value = data[0];
                decode_helper!(datack, value);
                Ok(Self::SongSelect { value })
            }

            status::SC_TUNE_REQUEST => {
                decode_helper!(lenck, 0, data);
                Ok(Self::TuneRequest)
            }

            status::SC_EOX => {
                decode_helper!(lenck, 0, data);
                Ok(Self::EndOfExclusive)
            }

            _ => unreachable!(),
        }
    }

    fn encode(&self, enc: &mut EncodedMessage) {
        match self {
            SystemCommonMessage::MtcQuarterFrame { kind, value } => {
                enc.push_all(&[kind & 0x7f, value & 0x7f])
            }
            SystemCommonMessage::SongPositionPointer { value }
            | SystemCommonMessage::SongSelect { value } => {
                enc.push(value & 0x7f);
            }
            _ => {}
        }
    }
}

/// A MIDI System Real Time message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemRealTimeMessage {
    TimingClock,
    Start,
    Continue,
    Stop,
    ActiveSensing,
    SystemReset,
}

impl SystemRealTimeMessage {
    /// Get the status byte for this message.
    pub const fn status(&self) -> u8 {
        match self {
            SystemRealTimeMessage::TimingClock => status::RT_TIMING_CLOCK,
            SystemRealTimeMessage::Start => status::RT_START,
            SystemRealTimeMessage::Continue => status::RT_CONTINUE,
            SystemRealTimeMessage::Stop => status::RT_STOP,
            SystemRealTimeMessage::ActiveSensing => status::RT_ACTIVE_SENSING,
            SystemRealTimeMessage::SystemReset => status::RT_SYSTEM_RESET,
        }
    }

    /// Create a System Real Time message from a status byte.
    ///
    /// Since all system real time messages consist only of a status byte with
    /// no data bytes, this method can be used to decode such a message.
    ///
    /// Returns None if the given status byte does not encode a system real time message.
    pub const fn from_status(status: u8) -> Option<Self> {
        Some(match status {
            status::RT_TIMING_CLOCK => Self::TimingClock,
            status::RT_START => Self::Start,
            status::RT_CONTINUE => Self::Continue,
            status::RT_STOP => Self::Stop,
            status::RT_ACTIVE_SENSING => Self::ActiveSensing,
            status::RT_SYSTEM_RESET => Self::SystemReset,
            _ => return None,
        })
    }
}

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
/// Note that bytes 120 through 127 (`0x78` through `0x7f`) are not valid
/// controller numbers, even for an undefined controller; rather, they are
/// reserved to designate [`ChannelMode`]s, since the status bytes for Control
/// Change and Channel Mode messages are equal. Here too, such a controller
/// number will be encoded faithfully if given; however, decoding such a message
/// will instead return a [`ChannelMessage::ChannelMode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Controller {
    BankSelectMsb,
    BankSelectLsb,
    ModulationMsb,
    ModulationLsb,
    BreathControlMsb,
    BreathControlLsb,
    FootControllerMsb,
    FootControllerLsb,
    PortamentoTimeMsb,
    PortamentoTimeLsb,
    DataEntryMsb,
    DataEntryLsb,
    ChannelVolumeMsb,
    ChannelVolumeLsb,
    BalanceMsb,
    BalanceLsb,
    PanMsb,
    PanLsb,
    ExpressionControllerMsb,
    ExpressionControllerLsb,
    EffectControl1Msb,
    EffectControl1Lsb,
    EffectControl2Msb,
    EffectControl2Lsb,
    GeneralPurposeController1Msb,
    GeneralPurposeController1Lsb,
    GeneralPurposeController2Msb,
    GeneralPurposeController2Lsb,
    GeneralPurposeController3Msb,
    GeneralPurposeController3Lsb,
    GeneralPurposeController4Msb,
    GeneralPurposeController4Lsb,
    Sustain,
    PortamentoOnOff,
    Sostenuto,
    SoftPedal,
    LegatoFootswitch,
    Hold2,
    SoundController1Variation,
    SoundController2Timbre,
    SoundController3ReleaseTime,
    SoundController4AttackTime,
    SoundController5Brightness,
    SoundController6,
    SoundController7,
    SoundController8,
    SoundController9,
    SoundController10,
    GeneralPurposeController5,
    GeneralPurposeController6,
    GeneralPurposeController7,
    GeneralPurposeController8,
    PortamentoControl,
    Effects1Depth,
    Effects2Depth,
    Effects3Depth,
    Effects4Depth,
    Effects5Depth,
    DataIncrement,
    DataDecrement,
    NonRegisteredParamLsb,
    NonRegisteredParamMsb,
    RegisteredParamMsb,
    RegisteredParamLsb,
    Undefined(u8),
}

impl Controller {
    /// Get the data byte for this controller.
    pub const fn controller_number(self) -> u8 {
        match self {
            Self::BankSelectMsb => 0x00,
            Self::BankSelectLsb => 0x20,
            Self::ModulationMsb => 0x01,
            Self::ModulationLsb => 0x21,
            Self::BreathControlMsb => 0x02,
            Self::BreathControlLsb => 0x22,
            Self::FootControllerMsb => 0x04,
            Self::FootControllerLsb => 0x24,
            Self::PortamentoTimeMsb => 0x05,
            Self::PortamentoTimeLsb => 0x25,
            Self::DataEntryMsb => 0x06,
            Self::DataEntryLsb => 0x26,
            Self::ChannelVolumeMsb => 0x07,
            Self::ChannelVolumeLsb => 0x27,
            Self::BalanceMsb => 0x08,
            Self::BalanceLsb => 0x28,
            Self::PanMsb => 0x0a,
            Self::PanLsb => 0x2a,
            Self::ExpressionControllerMsb => 0x0b,
            Self::ExpressionControllerLsb => 0x2b,
            Self::EffectControl1Msb => 0x0c,
            Self::EffectControl1Lsb => 0x2c,
            Self::EffectControl2Msb => 0x0d,
            Self::EffectControl2Lsb => 0x2d,
            Self::GeneralPurposeController1Msb => 0x10,
            Self::GeneralPurposeController1Lsb => 0x30,
            Self::GeneralPurposeController2Msb => 0x11,
            Self::GeneralPurposeController2Lsb => 0x31,
            Self::GeneralPurposeController3Msb => 0x12,
            Self::GeneralPurposeController3Lsb => 0x32,
            Self::GeneralPurposeController4Msb => 0x13,
            Self::GeneralPurposeController4Lsb => 0x33,

            Self::Sustain => 0x40,
            Self::PortamentoOnOff => 0x41,
            Self::Sostenuto => 0x42,
            Self::SoftPedal => 0x43,
            Self::LegatoFootswitch => 0x44,
            Self::Hold2 => 0x45,
            Self::SoundController1Variation => 0x46,
            Self::SoundController2Timbre => 0x47,
            Self::SoundController3ReleaseTime => 0x48,
            Self::SoundController4AttackTime => 0x49,
            Self::SoundController5Brightness => 0x4a,
            Self::SoundController6 => 0x4b,
            Self::SoundController7 => 0x4c,
            Self::SoundController8 => 0x4d,
            Self::SoundController9 => 0x4e,
            Self::SoundController10 => 0x4f,

            Self::GeneralPurposeController5 => 0x50,
            Self::GeneralPurposeController6 => 0x51,
            Self::GeneralPurposeController7 => 0x52,
            Self::GeneralPurposeController8 => 0x53,
            Self::PortamentoControl => 0x54,
            Self::Effects1Depth => 0x5b,
            Self::Effects2Depth => 0x5c,
            Self::Effects3Depth => 0x5d,
            Self::Effects4Depth => 0x5e,
            Self::Effects5Depth => 0x5f,

            Self::DataIncrement => 0x60,
            Self::DataDecrement => 0x61,
            Self::NonRegisteredParamLsb => 0x62,
            Self::NonRegisteredParamMsb => 0x63,
            Self::RegisteredParamLsb => 0x64,
            Self::RegisteredParamMsb => 0x65,

            Self::Undefined(b) => b,
        }
    }

    /// Create a [`Controller`] from its encoded data byte.
    ///
    /// This will return None if the given byte is either (a) not a data byte
    /// (high bit is set) or (b) encodes a [`ChannelMode`] instead (in the range
    /// 120-127).
    pub const fn from_controller_number(byte: u8) -> Option<Self> {
        Some(match byte {
            0x00 => Self::BankSelectMsb,
            0x20 => Self::BankSelectLsb,
            0x01 => Self::ModulationMsb,
            0x21 => Self::ModulationLsb,
            0x02 => Self::BreathControlMsb,
            0x22 => Self::BreathControlLsb,
            0x04 => Self::FootControllerMsb,
            0x24 => Self::FootControllerLsb,
            0x05 => Self::PortamentoTimeMsb,
            0x25 => Self::PortamentoTimeLsb,
            0x06 => Self::DataEntryMsb,
            0x26 => Self::DataEntryLsb,
            0x07 => Self::ChannelVolumeMsb,
            0x27 => Self::ChannelVolumeLsb,
            0x08 => Self::BalanceMsb,
            0x28 => Self::BalanceLsb,
            0x0a => Self::PanMsb,
            0x2a => Self::PanLsb,
            0x0b => Self::ExpressionControllerMsb,
            0x2b => Self::ExpressionControllerLsb,
            0x0c => Self::EffectControl1Msb,
            0x2c => Self::EffectControl1Lsb,
            0x0d => Self::EffectControl2Msb,
            0x2d => Self::EffectControl2Lsb,
            0x10 => Self::GeneralPurposeController1Msb,
            0x30 => Self::GeneralPurposeController1Lsb,
            0x11 => Self::GeneralPurposeController2Msb,
            0x31 => Self::GeneralPurposeController2Lsb,
            0x12 => Self::GeneralPurposeController3Msb,
            0x32 => Self::GeneralPurposeController3Lsb,
            0x13 => Self::GeneralPurposeController4Msb,
            0x33 => Self::GeneralPurposeController4Lsb,

            0x40 => Self::Sustain,
            0x41 => Self::PortamentoOnOff,
            0x42 => Self::Sostenuto,
            0x43 => Self::SoftPedal,
            0x44 => Self::LegatoFootswitch,
            0x45 => Self::Hold2,
            0x46 => Self::SoundController1Variation,
            0x47 => Self::SoundController2Timbre,
            0x48 => Self::SoundController3ReleaseTime,
            0x49 => Self::SoundController4AttackTime,
            0x4a => Self::SoundController5Brightness,
            0x4b => Self::SoundController6,
            0x4c => Self::SoundController7,
            0x4d => Self::SoundController8,
            0x4e => Self::SoundController9,
            0x4f => Self::SoundController10,

            0x50 => Self::GeneralPurposeController5,
            0x51 => Self::GeneralPurposeController6,
            0x52 => Self::GeneralPurposeController7,
            0x53 => Self::GeneralPurposeController8,
            0x54 => Self::PortamentoControl,
            0x5b => Self::Effects1Depth,
            0x5c => Self::Effects2Depth,
            0x5d => Self::Effects3Depth,
            0x5e => Self::Effects4Depth,
            0x5f => Self::Effects5Depth,

            0x60 => Self::DataIncrement,
            0x61 => Self::DataDecrement,
            0x62 => Self::NonRegisteredParamLsb,
            0x63 => Self::NonRegisteredParamMsb,
            0x64 => Self::RegisteredParamLsb,
            0x65 => Self::RegisteredParamMsb,

            120..=127 => return None,

            b => Self::Undefined(b),
        })
    }
}

/// A MIDI channel mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelMode {
    AllSoundOff,
    ResetAllControllers,
    LocalControl,
    AllNotesOff,
    OmniModeOff,
    OmniModeOn,
    MonoModeOn,
    PolyModeOn,
}

impl ChannelMode {
    /// Get the encoded data byte for this channel mode.
    pub const fn mode_number(self) -> u8 {
        match self {
            Self::AllSoundOff => 120,
            Self::ResetAllControllers => 121,
            Self::LocalControl => 122,
            Self::AllNotesOff => 123,
            Self::OmniModeOff => 124,
            Self::OmniModeOn => 125,
            Self::MonoModeOn => 126,
            Self::PolyModeOn => 127,
        }
    }

    /// Create a [`ChannelMode`] from its encoded data byte.
    ///
    /// Returns None if the given byte does not encode a channel mode.
    pub const fn from_mode_number(byte: u8) -> Option<Self> {
        Some(match byte {
            120 => Self::AllSoundOff,
            121 => Self::ResetAllControllers,
            122 => Self::LocalControl,
            123 => Self::AllNotesOff,
            124 => Self::OmniModeOff,
            125 => Self::OmniModeOn,
            126 => Self::MonoModeOn,
            127 => Self::PolyModeOn,
            _ => return None,
        })
    }
}

/// MIDI decode errors.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    /// The given message buffer was empty.
    #[error("empty message buffer")]
    EmptyMessage,

    /// The given message buffer has the incorrect length for its message type.
    #[error("incorrect number of data bytes")]
    IncorrectLen,

    /// The given message buffer contained an invalid value.
    ///
    /// This is usually due to a byte that was expected to be a data byte which
    /// has its high bit set.
    #[error("data byte with invalid value")]
    InvalidValue,

    /// The given message buffer did not contain an initial status byte, and no
    /// running status was given.
    #[error("missing status byte with no running status")]
    MissingStatus,
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
    /// first data byte; Control Change messages must have a first data byte
    /// less than 120 (0x78).
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

#[cfg(test)]
mod test {
    use super::*;

    fn encode_test(idx: usize, msg: &Message, encode_status: bool, expected: &[u8]) {
        let enc = msg.encode(encode_status);
        assert_eq!(
            expected, &*enc,
            "message {idx} encoded incorrectly:\n{msg:02x?}\nexpected: {expected:02x?}\ngot: {enc:02x?}",
        );
    }

    fn decode_test(idx: usize, encoded: &[u8], running: Option<u8>, expected: &Message) {
        let msg = Message::decode(encoded, running).unwrap_or_else(|e| {
            panic!("failed to decode message {idx} {encoded:02x?}: {e}");
        });

        assert_eq!(
            expected, &msg,
            "message {idx} decoded incorrectly:\n{encoded:02x?}\nexpected: {expected:02x?}\ngot: {msg:02x?}",
        );
    }

    #[test]
    fn encode_voice_messages() {
        let messages: &[(Message, &[u8])] = &[
            (
                Message::Channel {
                    channel: 0,
                    message: ChannelMessage::NoteOff {
                        note: 60,
                        velocity: 64,
                    },
                },
                &[status::NOTE_OFF, 60, 64],
            ),
            (
                Message::Channel {
                    channel: 12,
                    message: ChannelMessage::KeyPressure {
                        note: 12,
                        value: 70,
                    },
                },
                &[status::KEY_PRESSURE | 12, 12, 70],
            ),
            (
                Message::Channel {
                    channel: 4,
                    message: ChannelMessage::ProgramChange { value: 120 },
                },
                &[status::PROGRAM_CHANGE | 4, 120],
            ),
        ];

        for (i, (msg, expected)) in messages.iter().enumerate() {
            encode_test(i, msg, true, *expected);
        }
    }

    #[test]
    fn encode_voice_messages_running() {
        let messages: &[(Message, &[u8])] = &[
            (
                Message::Channel {
                    channel: 0,
                    message: ChannelMessage::NoteOn {
                        note: 86,
                        velocity: 0,
                    },
                },
                &[86, 0],
            ),
            (
                Message::Channel {
                    channel: 3,
                    message: ChannelMessage::ChannelPressure { value: 48 },
                },
                &[48],
            ),
        ];

        for (i, (msg, expected)) in messages.iter().enumerate() {
            encode_test(i, msg, false, expected);
        }
    }

    #[test]
    fn decode_voice_messages() {
        let messages: &[(&[u8], Message)] = &[
            (
                &[status::NOTE_OFF, 70, 64],
                Message::Channel {
                    channel: 0,
                    message: ChannelMessage::NoteOff {
                        note: 70,
                        velocity: 64,
                    },
                },
            ),
            (
                &[status::PITCH_BEND | 7, 0x7f, 0x40],
                Message::Channel {
                    channel: 7,
                    message: ChannelMessage::PitchBend { value: 0x207f },
                },
            ),
            (
                &[
                    status::CONTROL_CHANGE | 15,
                    Controller::Sustain.controller_number(),
                    127,
                ],
                Message::Channel {
                    channel: 15,
                    message: ChannelMessage::ControlChange {
                        controller: Controller::Sustain,
                        value: 127,
                    },
                },
            ),
        ];

        for (i, (encoded, expected)) in messages.iter().enumerate() {
            decode_test(i, encoded, None, expected);
        }
    }

    #[test]
    fn decode_voice_messages_running() {
        let messages: &[(u8, &[u8], Message)] = &[
            (
                status::NOTE_OFF,
                &[45, 64],
                Message::Channel {
                    channel: 0,
                    message: ChannelMessage::NoteOff {
                        note: 45,
                        velocity: 64,
                    },
                },
            ),
            (
                status::KEY_PRESSURE | 9,
                &[45, 13],
                Message::Channel {
                    channel: 9,
                    message: ChannelMessage::KeyPressure {
                        note: 45,
                        value: 13,
                    },
                },
            ),
            (
                status::CHANNEL_PRESSURE,
                &[status::CHANNEL_PRESSURE, 70],
                Message::Channel {
                    channel: 0,
                    message: ChannelMessage::ChannelPressure { value: 70 },
                },
            ),
            (
                status::NOTE_ON,
                &[status::NOTE_ON | 7, 16, 120],
                Message::Channel {
                    channel: 7,
                    message: ChannelMessage::NoteOn {
                        note: 16,
                        velocity: 120,
                    },
                },
            ),
            (
                status::NOTE_ON | 10,
                &[status::PROGRAM_CHANGE, 82],
                Message::Channel {
                    channel: 0,
                    message: ChannelMessage::ProgramChange { value: 82 },
                },
            ),
        ];

        for (i, (running, encoded, expected)) in messages.iter().enumerate() {
            decode_test(i, encoded, Some(*running), expected);
        }
    }
}
