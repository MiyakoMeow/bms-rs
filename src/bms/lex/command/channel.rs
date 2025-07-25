//! Definitions of channel command argument data.
//!
//! For more details, please see [`Channel`] enum and its related types.
//! For documents of modes, please see [BMS command memo#KEYMAP Table](https://hitkey.bms.ms/cmds.htm#KEYMAP-TABLE)
//!
//! - Pre-defined channel parsers:
//!   - `read_channel_beat` for Beat 5K/7K/10K/14K
//!   - `read_channel_pms_bme_type` for PMS BME-type
//!   - `read_channel_pms` for PMS
//!   - `read_channel_beat_nanasi` for Beat nanasi/angolmois
//!   - `read_channel_dsc_oct_fp` for DSC & OCT/FP

use super::{Key, NoteKind, PlayerSide};

/// The channel, or lane, where the note will be on.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Channel {
    /// The BGA channel.
    BgaBase,
    /// The BGA channel but overlay to [`Channel::BgaBase`] channel.
    BgaLayer,
    /// The POOR BGA channel.
    BgaPoor,
    /// For the note which will be auto-played.
    Bgm,
    /// For the bpm change by an [`u8`] integer.
    BpmChangeU8,
    /// For the bpm change object.
    BpmChange,
    /// For the change option object.
    ChangeOption,
    /// For the note which the user can interact.
    Note {
        /// The kind of the note.
        kind: NoteKind,
        /// The note for the player side.
        side: PlayerSide,
        /// The key which corresponds to the note.
        key: Key,
    },
    /// For the section length change object.
    SectionLen,
    /// For the stop object.
    Stop,
    /// For the scroll speed change object.
    Scroll,
    /// For the note spacing change object.
    Speed,
}

/// Reads a channel from a string.
///
/// For general part, please call this function when using other functions.
fn read_channel_general(channel: &str) -> Option<Channel> {
    use Channel::*;
    Some(match channel.to_uppercase().as_str() {
        "01" => Bgm,
        "02" => SectionLen,
        "03" => BpmChangeU8,
        "08" => BpmChange,
        "04" => BgaBase,
        "06" => BgaPoor,
        "07" => BgaLayer,
        "09" => Stop,
        "SC" => Scroll,
        "SP" => Speed,
        _ => return None,
    })
}

/// Reads a note kind from a character. (For general part)
/// Can be directly use in BMS/BME/PMS types, and be converted to other types.
fn get_note_kind_general(kind_char: char) -> Option<(NoteKind, PlayerSide)> {
    Some(match kind_char {
        '1' => (NoteKind::Visible, PlayerSide::Player1),
        '2' => (NoteKind::Visible, PlayerSide::Player2),
        '3' => (NoteKind::Invisible, PlayerSide::Player1),
        '4' => (NoteKind::Invisible, PlayerSide::Player2),
        '5' => (NoteKind::Long, PlayerSide::Player1),
        '6' => (NoteKind::Long, PlayerSide::Player2),
        'D' => (NoteKind::Landmine, PlayerSide::Player1),
        'E' => (NoteKind::Landmine, PlayerSide::Player2),
        _ => return None,
    })
}

/// Reads a key from a character. (For Beat 5K/7K/10K/14K)
fn get_key_beat(key: char) -> Option<Key> {
    use Key::*;
    Some(match key {
        '1' => Key1,
        '2' => Key2,
        '3' => Key3,
        '4' => Key4,
        '5' => Key5,
        '6' => Scratch,
        '7' => FreeZone,
        '8' => Key6,
        '9' => Key7,
        _ => return None,
    })
}

/// Reads a channel from a string. (For Beat 5K/7K/10K/14K)
pub fn read_channel_beat(channel: &str) -> Option<Channel> {
    if let Some(channel) = read_channel_general(channel) {
        return Some(channel);
    }
    let mut channel_chars = channel.chars();
    let (kind, side) = get_note_kind_general(channel_chars.next()?)?;
    let key = get_key_beat(channel_chars.next()?)?;
    Some(Channel::Note { kind, side, key })
}

/// Reads a key from a character. (For PMS)
fn get_key_pms_bme_type(key: char) -> Option<Key> {
    use Key::*;
    Some(match key {
        '1' => Key1,
        '2' => Key2,
        '3' => Key3,
        '4' => Key4,
        '5' => Key5,
        '6' => Key8,
        '7' => Key9,
        '8' => Key6,
        '9' => Key7,
        _ => return None,
    })
}

/// Reads a channel from a string. (For PMS BME-type)
/// This preset supports 9K with 2-players.
pub fn read_channel_pms_bme_type(channel: &str) -> Option<Channel> {
    if let Some(channel) = read_channel_general(channel) {
        return Some(channel);
    }
    let mut channel_chars = channel.chars();
    let (kind, side) = get_note_kind_general(channel_chars.next()?)?;
    let key = get_key_pms_bme_type(channel_chars.next()?)?;
    Some(Channel::Note { kind, side, key })
}

/// Reads a channel from a string. (For PMS)
pub fn read_channel_pms(channel: &str) -> Option<Channel> {
    if let Some(channel) = read_channel_general(channel) {
        return Some(channel);
    }
    let mut channel_chars = channel.chars();
    let (kind, side) = get_note_kind_general(channel_chars.next()?)?;
    let bme_key = get_key_pms_bme_type(channel_chars.next()?)?;
    // Translate BME type to PMS type.
    use Key::*;
    use PlayerSide::*;
    let key = match (side, bme_key) {
        (Player1, Key1 | Key2 | Key3 | Key4 | Key5) => bme_key,
        (Player2, Key2) => Key6,
        (Player2, Key3) => Key7,
        (Player2, Key4) => Key8,
        (Player2, Key5) => Key9,
        _ => return None,
    };
    Some(Channel::Note {
        kind,
        side: PlayerSide::Player1,
        key,
    })
}

/// Reads a channel from a string. (For Beat nanasi/angolmois)
pub fn read_channel_beat_nanasi(channel: &str) -> Option<Channel> {
    if let Some(channel) = read_channel_general(channel) {
        return Some(channel);
    }
    let mut channel_chars = channel.chars();
    let (kind, side) = get_note_kind_general(channel_chars.next()?)?;
    let bme_key = get_key_beat(channel_chars.next()?)?;
    // Translate BME type to Beat nanasi/angolmois type.
    use Key::*;
    let key = match bme_key {
        Key1 | Key2 | Key3 | Key4 | Key5 | Scratch => bme_key,
        FreeZone => FootPedal,
        _ => return None,
    };
    Some(Channel::Note { kind, side, key })
}

/// Reads a channel from a string. (For DSC & OCT/FP)
pub fn read_channel_dsc_oct_fp(channel: &str) -> Option<Channel> {
    if let Some(channel) = read_channel_general(channel) {
        return Some(channel);
    }
    let mut channel_chars = channel.chars();
    let (kind, side) = get_note_kind_general(channel_chars.next()?)?;
    let bme_key = get_key_pms_bme_type(channel_chars.next()?)?;
    // Translate BME type to PMS type.
    use Key::*;
    use PlayerSide::*;
    let key = match (side, bme_key) {
        (Player1, Key1 | Key2 | Key3 | Key4 | Key5 | Key6 | Key7 | Scratch) => bme_key,
        (Player2, Key1) => FootPedal,
        (Player2, Key2) => Key8,
        (Player2, Key3) => Key9,
        (Player2, Key4) => Key10,
        (Player2, Key5) => Key11,
        (Player2, Key6) => Key12,
        (Player2, Key7) => Key13,
        (Player2, Scratch) => ScratchExtra,
        _ => return None,
    };
    Some(Channel::Note {
        kind,
        side: PlayerSide::Player1,
        key,
    })
}
