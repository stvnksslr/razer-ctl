use anyhow::{ensure, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

/// USB HID feature report packet for Razer device communication.
///
/// 90-byte structure following the openrazer protocol. Commands are sent as feature
/// reports and responses are read back in the same format.
///
/// # Protocol
/// - Bytes 0-1: Status and transaction ID
/// - Bytes 2-7: Protocol metadata (remaining packets, type, size, command)
/// - Bytes 8-87: Command arguments (80 bytes)
/// - Byte 88: CRC (XOR of bytes 2-87)
/// - Byte 89: Reserved
///
/// See `data/README.md` for reverse engineering details.
#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Packet {
    status: u8,
    id: u8,
    remaining_packets: u16,
    protocol_type: u8,
    data_size: u8,
    command_class: u8,
    command_id: u8,
    #[serde(with = "BigArray")]
    args: [u8; 80],
    crc: u8,
    reserved: u8,
}

/// Status codes for USB HID command packets (per openrazer protocol).
enum CommandStatus {
    /// Initial status for outgoing packets (not yet processed)
    New = 0x00,
    /// Device is busy processing the command
    Busy = 0x01,
    /// Command completed successfully
    Successful = 0x02,
    /// Command failed (generic failure)
    Failure = 0x03,
    /// Command timed out (no response from device)
    Timeout = 0x04,
    /// Command not supported by this device
    NotSupported = 0x05,
}

impl Packet {
    /// Creates a new packet with the given command and arguments.
    ///
    /// The command is a 16-bit value where the high byte is the command class
    /// and the low byte is the command ID (e.g., 0x0d02 for SET_PERF_MODE).
    pub fn new(command: u16, args: &[u8]) -> Packet {
        let mut args_buffer = [0x00; 80];
        args_buffer[..args.len()].copy_from_slice(args);

        let mut packet = Packet {
            status: CommandStatus::New as u8,
            id: rand::thread_rng().gen(),
            remaining_packets: 0x0000,
            protocol_type: 0x00,
            data_size: args.len() as u8,
            command_class: (command >> 8) as u8,
            command_id: (command & 0xff) as u8,
            args: args_buffer,
            crc: 0x00,
            reserved: 0x00,
        };
        packet.crc = packet.calculate_crc();
        packet
    }

    /// Calculate CRC by XORing bytes 2-87 of the packet (per openrazer protocol).
    fn calculate_crc(&self) -> u8 {
        let mut crc: u8 = 0;
        // XOR remaining_packets (2 bytes, little-endian)
        crc ^= (self.remaining_packets & 0xff) as u8;
        crc ^= (self.remaining_packets >> 8) as u8;
        // XOR protocol_type, data_size, command_class, command_id
        crc ^= self.protocol_type;
        crc ^= self.data_size;
        crc ^= self.command_class;
        crc ^= self.command_id;
        // XOR all 80 args bytes
        for byte in &self.args {
            crc ^= byte;
        }
        crc
    }

    /// Returns the valid argument bytes (up to data_size).
    pub fn get_args(&self) -> &[u8] {
        &self.args[..self.data_size as usize]
    }

    /// Validates that this response packet matches the original report.
    ///
    /// Checks command class, command ID, transaction ID, and status code.
    pub fn ensure_matches_report(self, report: &Packet) -> Result<Self> {
        ensure!(
            (report.command_class, report.command_id, report.id)
                == (self.command_class, self.command_id, self.id),
            "Response does not match the report"
        );

        ensure!(
            self.remaining_packets == report.remaining_packets
            || (self.command_class, self.command_id) == (0x07, 0x92) /* 0x0792 (bho) has special handling */
            || (self.command_class, self.command_id) == (0x07, 0x8f), /* 0x078f max fan speed mode has special handling */
            "Response command does not match the report"
        );

        match self.status {
            s if s == CommandStatus::Successful as u8 => {}
            s if s == CommandStatus::NotSupported as u8 => {
                anyhow::bail!("Command not supported by device")
            }
            s if s == CommandStatus::Busy as u8 => {
                anyhow::bail!("Device busy, try again")
            }
            s if s == CommandStatus::Failure as u8 => {
                anyhow::bail!("Command failed")
            }
            s if s == CommandStatus::Timeout as u8 => {
                anyhow::bail!("Command timed out")
            }
            s => {
                anyhow::bail!("Command failed with unknown status: 0x{:02X}", s)
            }
        }

        Ok(self)
    }
}

impl From<&Packet> for Vec<u8> {
    fn from(packet: &Packet) -> Vec<u8> {
        bincode::serialize(packet).expect("Packet serialization failed - this is a bug")
    }
}

impl TryFrom<&[u8]> for Packet {
    type Error = anyhow::Error;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        ensure!(
            data.len() == std::mem::size_of::<Packet>(),
            "Invalid raw data size"
        );

        Ok(bincode::deserialize::<Packet>(data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_size() {
        assert_eq!(std::mem::size_of::<Packet>(), 90);
    }

    #[test]
    fn test_packet_new_sets_command() {
        let packet = Packet::new(0x0d02, &[0x01, 0x02]);
        assert_eq!(packet.command_class, 0x0d);
        assert_eq!(packet.command_id, 0x02);
        assert_eq!(packet.data_size, 2);
    }

    #[test]
    fn test_packet_get_args_returns_valid_slice() {
        let packet = Packet::new(0x0d02, &[0x01, 0x02, 0x03]);
        let args = packet.get_args();
        assert_eq!(args.len(), 3);
        assert_eq!(args, &[0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_packet_serialization_roundtrip() {
        let original = Packet::new(0x0d02, &[0x01, 0x02, 0x03, 0x04]);
        let bytes: Vec<u8> = (&original).into();
        assert_eq!(bytes.len(), 90);

        let restored = Packet::try_from(bytes.as_slice()).unwrap();
        assert_eq!(restored.command_class, original.command_class);
        assert_eq!(restored.command_id, original.command_id);
        assert_eq!(restored.data_size, original.data_size);
        assert_eq!(restored.get_args(), original.get_args());
    }

    #[test]
    fn test_packet_crc_calculation() {
        let packet = Packet::new(0x0d02, &[0x01, 0x02]);
        // CRC should be non-zero for non-trivial packets
        assert_ne!(packet.crc, 0);
    }

    #[test]
    fn test_packet_try_from_invalid_size() {
        let short_data = vec![0u8; 50];
        assert!(Packet::try_from(short_data.as_slice()).is_err());
    }
}
