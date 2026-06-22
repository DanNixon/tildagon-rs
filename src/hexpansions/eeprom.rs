//! Utilities for serializing/deserializing the EEPROM header format.
//! See https://tildagon.badge.emfcamp.org/hexpansions/eeprom/#eeprom-format

use defmt::Format;
use heapless::{String, Vec};

#[derive(Debug, Format, PartialEq, Eq, Clone)]
pub struct HexpansionEepromHeader {
    pub version: HexpansionManifestVersion,

    pub filesystem_offset: u16,
    pub eeprom_page_size: u16,
    pub eeprom_total_size: u32,

    pub vid: u16,
    pub pid: u16,
    pub uid: u16,
    pub friendly_name: String<9>,
}

impl HexpansionEepromHeader {
    pub fn from_bytes(data: &[u8; 32]) -> Result<Self, HexpansionEepromHeaderError> {
        // Verify checksum
        let chk_expected = data[31];
        let chk_got = checksum(data);
        if chk_got != chk_expected {
            return Err(HexpansionEepromHeaderError::ChecksumMismatch {
                expected: chk_expected,
                got: chk_got,
            });
        }

        // Verify magic bytes
        if &data[0..=3] != b"THEX" {
            return Err(HexpansionEepromHeaderError::MagicBytesIncorrect);
        }

        let version = match &data[4..=7] {
            b"2024" => HexpansionManifestVersion::V2024,
            b"2026" => HexpansionManifestVersion::V2026,
            _ => {
                return Err(HexpansionEepromHeaderError::UnknownVersion);
            }
        };

        let filesystem_offset = u16::from_le_bytes([data[8], data[9]]);
        let eeprom_page_size = u16::from_le_bytes([data[10], data[11]]);
        let eeprom_total_size = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let vid = u16::from_le_bytes([data[16], data[17]]);
        let pid = u16::from_le_bytes([data[18], data[19]]);
        let uid = u16::from_le_bytes([data[20], data[21]]);

        let mut friendly_name = Vec::<u8, 9>::new();
        for c in &data[22..=30] {
            if *c != 0 {
                friendly_name.push(*c).unwrap();
            }
        }
        let friendly_name = String::from_utf8(friendly_name)
            .map_err(|_| HexpansionEepromHeaderError::StringError)?;

        Ok(HexpansionEepromHeader {
            version,
            filesystem_offset,
            eeprom_page_size,
            eeprom_total_size,
            vid,
            pid,
            uid,
            friendly_name,
        })
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let mut result = [0u8; 32];

        // Magic bytes
        result[0] = b'T';
        result[1] = b'H';
        result[2] = b'E';
        result[3] = b'X';

        match self.version {
            HexpansionManifestVersion::V2024 => {
                result[4] = b'2';
                result[5] = b'0';
                result[6] = b'2';
                result[7] = b'4';
            }
            HexpansionManifestVersion::V2026 => {
                result[4] = b'2';
                result[5] = b'0';
                result[6] = b'2';
                result[7] = b'6';
            }
        }

        let b = self.filesystem_offset.to_le_bytes();
        result[8] = b[0];
        result[9] = b[1];

        let b = self.eeprom_page_size.to_le_bytes();
        result[10] = b[0];
        result[11] = b[1];

        let b = self.eeprom_total_size.to_le_bytes();
        result[12] = b[0];
        result[13] = b[1];
        result[14] = b[2];
        result[15] = b[3];

        let b = self.vid.to_le_bytes();
        result[16] = b[0];
        result[17] = b[1];

        let b = self.pid.to_le_bytes();
        result[18] = b[0];
        result[19] = b[1];

        let b = self.uid.to_le_bytes();
        result[20] = b[0];
        result[21] = b[1];

        let b = self.friendly_name.as_bytes();
        for idx in 0..9 {
            result[22 + idx] = *b.get(idx).unwrap_or(&0);
        }

        result[31] = checksum(&result);

        result
    }
}

// Apparently "2024" is the only valid value, yet the official firmware accepts both "2024" and "2026" and treats both the same
#[derive(Debug, Format, Copy, Clone, PartialEq, Eq)]
pub enum HexpansionManifestVersion {
    V2024,
    V2026,
}

#[derive(Debug, Format, Copy, Clone, PartialEq, Eq)]
pub enum HexpansionEepromHeaderError {
    ChecksumMismatch { expected: u8, got: u8 },
    MagicBytesIncorrect,
    UnknownVersion,
    StringError,
}

fn checksum(header: &[u8]) -> u8 {
    let mut value = 0x55;

    for b in &header[1..=30] {
        value ^= b;
    }

    value
}
