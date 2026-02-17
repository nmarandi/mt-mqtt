mod decoder;
mod encoder;

use crate::protocol::definitions::*;
pub use crate::protocol::packet::*;
use bytes::{Buf, BytesMut};
use decoder::*;
use encoder::*;
use std::{fmt, io::Cursor};

#[derive(Debug)]
pub enum Error {
    /// Not enough data is available to parse a message
    Incomplete(usize),

    /// Invalid message encoding
    Other(String),
}

#[derive(Debug)]
pub enum ControlPacket {
    Connect(ConnectControlPacket),
    ConnAck(ConnAckControlPacket),
    Publish(PublishControlPacket),
    PubAck(PubAckControlPacket),
    PubRec(PubRecControlPacket),
    PubRel(PubRelControlPacket),
    PubComp(PubCompControlPacket),
    Subscribe(SubscribeControlPacket),
    SubAck(SubAckControlPacket),
    Unsubscribe(()),
    UnsubAck(()),
    PingReq,
    PingResp,
    Disconnect(()),
    Auth(()),
}

#[derive(Debug)]
pub struct Frame {
    pub control_packet: ControlPacket,
    pub fix_header: FixHeader,
}

impl Frame {
    pub fn new(control_packet_type: ControlPacketType) -> Frame {
        match control_packet_type {
            ControlPacketType::CONNECT => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::Connect(Default::default()),
            },
            ControlPacketType::CONNACK => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::ConnAck(Default::default()),
            },
            ControlPacketType::PUBLISH => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::Publish(Default::default()),
            },
            ControlPacketType::PUBACK => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::PubAck(Default::default()),
            },
            ControlPacketType::PUBREC => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::PubRec(Default::default()),
            },
            ControlPacketType::PUBREL => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::PubRel(Default::default()),
            },
            ControlPacketType::PUBCOMP => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::PubComp(Default::default()),
            },
            ControlPacketType::SUBSCRIBE => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::Subscribe(Default::default()),
            },
            ControlPacketType::SUBACK => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::SubAck(Default::default()),
            },
            ControlPacketType::UNSUBSCRIBE => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::Unsubscribe(Default::default()),
            },
            ControlPacketType::UNSUBACK => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::UnsubAck(Default::default()),
            },
            ControlPacketType::PINGREQ => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::PingReq,
            },
            ControlPacketType::PINGRESP => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::PingResp,
            },
            ControlPacketType::DISCONNECT => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::Disconnect(Default::default()),
            },
            ControlPacketType::AUTH => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::Auth(Default::default()),
            },
        }
    }

    pub fn deserialize(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        // Check minimum bytes for fixed header (1 byte) + remaining length (at least 1 byte)
        let buffer_len = src.get_ref().len();
        if buffer_len < 2 {
            return Err(Error::Incomplete(2 - buffer_len));
        }

        // Save position to restore if we don't have enough data
        let start_position = src.position();

        let fix_header = match decode_fix_header(src) {
            Some(header) => header,
            None => return Err(Error::Incomplete(1)),
        };

        // Check if we have enough bytes to read the remaining length
        if !src.has_remaining() {
            src.set_position(start_position);
            return Err(Error::Incomplete(1));
        }

        // Read and decode the remaining length (variable byte integer)
        // This tells us how many bytes follow in the variable header + payload
        let remaining_length = VariableByteInteger::from(src);
        let packet_data_len = remaining_length.data as usize;

        // Get current position and ensure we have enough data
        let current_pos = src.position() as usize;

        if buffer_len < current_pos + packet_data_len {
            // Not enough data - reset position and return Incomplete
            let needed = packet_data_len - (buffer_len - current_pos);
            src.set_position(start_position);
            return Err(Error::Incomplete(needed));
        }

        // Create a bounded sub-cursor containing only this packet's data
        // This prevents decoders from reading into the next packet
        let packet_slice = &src.get_ref()[current_pos..current_pos + packet_data_len];
        let mut packet_cursor = Cursor::new(packet_slice);

        let control_packet_type = fix_header.control_packet_type;
        let qos = fix_header.flags.1; // Extract QoS from flags
        let control_packet = match control_packet_type {
            ControlPacketType::CONNECT => ControlPacket::Connect(decode_connect_packet(&mut packet_cursor)?),
            ControlPacketType::PUBLISH => ControlPacket::Publish(decode_publish_packet(&mut packet_cursor, qos)?),
            ControlPacketType::PUBACK => ControlPacket::PubAck(decode_pub_ack_packet(&mut packet_cursor)?),
            ControlPacketType::PUBREC => ControlPacket::PubRec(decode_pub_rec_packet(&mut packet_cursor)?),
            ControlPacketType::PUBREL => ControlPacket::PubRel(decode_pub_rel_packet(&mut packet_cursor)?),
            ControlPacketType::PUBCOMP => ControlPacket::PubComp(decode_pub_comp_packet(&mut packet_cursor)?),
            ControlPacketType::SUBSCRIBE => ControlPacket::Subscribe(decode_subscribe_packet(&mut packet_cursor)?),
            ControlPacketType::UNSUBSCRIBE => {
                // UNSUBSCRIBE not implemented yet - return unit
                ControlPacket::Unsubscribe(())
            }
            ControlPacketType::PINGREQ => ControlPacket::PingReq,
            ControlPacketType::DISCONNECT => {
                let _ = decode_disconnect_packet(&mut packet_cursor)?;
                ControlPacket::Disconnect(())
            }
            _ => return Err(Error::Other(format!("Unsupported packet type: {:?}", control_packet_type))),
        };

        // Advance the main cursor past this packet's data
        src.set_position((current_pos + packet_data_len) as u64);

        Ok(Frame { fix_header, control_packet })
    }

    pub fn serialize(frame: Frame) -> Result<BytesMut, Error> {
        let mut bytes = BytesMut::new();
        encode_fix_header(frame.fix_header, &mut bytes);
        let mut payload = BytesMut::new();
        match frame.control_packet {
            ControlPacket::ConnAck(packet) => encode_conn_ack_packet(packet, &mut payload),
            ControlPacket::Publish(packet) => encode_publish_packet(packet, &mut payload),
            ControlPacket::PubAck(packet) => encode_pub_ack_packet(packet, &mut payload),
            ControlPacket::PubRec(packet) => encode_pub_rec_packet(packet, &mut payload),
            ControlPacket::PubRel(packet) => encode_pub_rel_packet(packet, &mut payload),
            ControlPacket::PubComp(packet) => encode_pub_comp_packet(packet, &mut payload),
            ControlPacket::SubAck(packet) => encode_sub_ack_packet(packet, &mut payload),
            ControlPacket::PingResp => {}
            _ => return Err(Error::Other("Unsupported packet type for serialization".to_string())),
        }
        bytes.extend(VariableByteInteger::encode_u32(payload.len() as u32));
        bytes.extend(payload);
        Ok(bytes)
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(src: std::io::Error) -> Error {
        Error::Other(src.to_string())
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Incomplete(size) => write!(fmt, "Incomplete frame: need {} more bytes", size),
            Error::Other(err) => write!(fmt, "Frame error: {}", err),
        }
    }
}

#[cfg(test)]
mod tests;
