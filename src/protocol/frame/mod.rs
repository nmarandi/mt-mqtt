mod decoder;
mod encoder;

use crate::protocol::definitions::*;
pub use crate::protocol::packet::*;
use bytes::BytesMut;
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
        let fix_header = decode_fix_header(src).unwrap();
        
        // Read and decode the remaining length (variable byte integer)
        // This tells us how many bytes follow in the variable header + payload
        let _remaining_length = VariableByteInteger::from(src);
        
        let control_packet_type = fix_header.control_packet_type;
        let qos = fix_header.flags.1; // Extract QoS from flags
        let control_packet = match control_packet_type {
            ControlPacketType::CONNECT => ControlPacket::Connect(decode_connect_packet(src)?),
            ControlPacketType::PUBLISH => ControlPacket::Publish(decode_publish_packet(src, qos)?),
            ControlPacketType::PUBACK => ControlPacket::PubAck(decode_pub_ack_packet(src)?),
            ControlPacketType::PUBREC => ControlPacket::PubRec(decode_pub_rec_packet(src)?),
            ControlPacketType::PUBREL => ControlPacket::PubRel(decode_pub_rel_packet(src)?),
            ControlPacketType::PUBCOMP => ControlPacket::PubComp(decode_pub_comp_packet(src)?),
            ControlPacketType::SUBSCRIBE => ControlPacket::Subscribe(decode_subscribe_packet(src)?),
            ControlPacketType::UNSUBSCRIBE => {
                // UNSUBSCRIBE not implemented yet - return unit
                ControlPacket::Unsubscribe(())
            }
            ControlPacketType::PINGREQ => ControlPacket::PingReq,
            ControlPacketType::DISCONNECT => {
                let _ = decode_disconnect_packet(src)?;
                ControlPacket::Disconnect(())
            }
            _ => return Err(Error::Other(format!("Unsupported packet type: {:?}", control_packet_type))),
        };

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
