use crate::definitions::*;
pub use crate::packet::*;
use bytes::{Buf, BufMut, BytesMut};
use std::{convert::TryFrom, fmt, io::Cursor};
mod decoder;
mod encoder;
use decoder::*;
use encoder::*;
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
    Unsubscribe(UnsubAckControlPacket),
    UnsubAck(UnsubAckControlPacket),
    PingReq,
    PingResp,
    Disconnect(DisconnectControlPacket),
    Auth(AuthControlPacket),
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
            ControlPacketType::CONNACK => {
                let mut conn_ack_control_packet: ConnAckControlPacket = Default::default();
                conn_ack_control_packet
                    .variable_header
                    .properties
                    .push(Some(Property::AssignedClientIdentifier(String::from("Assigned"))));
                conn_ack_control_packet.variable_header.conn_ack_flag.session_present_flag = false;
                Frame {
                    fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                    control_packet: ControlPacket::ConnAck(conn_ack_control_packet),
                }
            }
            ControlPacketType::PUBACK => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::PubAck(Default::default()),
            },
            ControlPacketType::PUBREC => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::PubRec(Default::default()),
            },
            ControlPacketType::PINGRESP => Frame {
                fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                control_packet: ControlPacket::PingResp
            },
            _ => panic!("not implemented yet"),
            /*ControlPacketType::PUBREL = 6,
            ControlPacketType::PUBCOMP = 7,
            ControlPacketType::SUBSCRIBE = 8,
            ControlPacketType::SUBACK = 9,
            ControlPacketType::UNSUBSCRIBE = 10,
            ControlPacketType::UNSUBACK = 11,
            ControlPacketType::PINGREQ = 12,
            ControlPacketType::DISCONNECT = 14,
            ControlPacketType::AUTH = 15,*/
        }
    }

    pub fn deserialize(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        if src.remaining() < 2 {
            return Err(Error::Incomplete(src.remaining()));
        }
        println!("start deserialize");
        let pos = src.position();
        let fix_header = decode_fix_header(src).unwrap();
        println!("fix_header: {:?}", fix_header);
        let remianing_lenght = VariableByteInteger::from(src);
        if src.remaining() < usize::try_from(remianing_lenght.data).unwrap() {
            src.set_position(pos);
            return Err(Error::Incomplete(usize::try_from(remianing_lenght.data).unwrap()));
        }
        match fix_header.control_packet_type {
            ControlPacketType::CONNECT => Ok(Frame {
                control_packet: ControlPacket::Connect(decode_connect_packet(src).unwrap()),
                fix_header,
            }),
            ControlPacketType::PUBLISH => Ok(Frame {
                control_packet: ControlPacket::Publish(decode_publish_packet(src, fix_header.flags.1).unwrap()),
                fix_header,
            }),
            ControlPacketType::PUBREL => Ok(Frame {
                control_packet: ControlPacket::PubRel(decode_pub_rel_packet(src).unwrap()),
                fix_header,
            }),
            ControlPacketType::SUBSCRIBE => Ok(Frame {
                control_packet: ControlPacket::Subscribe(decode_subscribe_packet(src).unwrap()),
                fix_header,
            }),
            ControlPacketType::DISCONNECT => Ok(Frame {
                control_packet: ControlPacket::Disconnect(decode_disconnect_packet(src).unwrap()),
                fix_header,
            }),
            ControlPacketType::PINGREQ => Ok(Frame {
                control_packet: ControlPacket::PingReq,
                fix_header,
            }),
            _ => Err(Error::Other(format!("Not Implemented yet"))),
        }
    }

    pub fn serialize(frame: Frame) -> Result<BytesMut, Error> {
        println!("start serialize: \n{:?}", frame);
        let mut data: BytesMut = BytesMut::new();
        encode_fix_header(frame.fix_header, &mut data);
        let mut src: BytesMut = BytesMut::new();
        match frame.control_packet {
            ControlPacket::ConnAck(control_packet) => {
                encode_conn_ack_packet(control_packet, &mut src);
            }
            ControlPacket::PubAck(control_packet) => {
                encode_pub_ack_packet(control_packet, &mut src);
            }
            ControlPacket::PubRec(control_packet) => {
                encode_pub_rec_packet(control_packet, &mut src);
            }
            ControlPacket::PubComp(control_packet) => {
                encode_pub_comp_packet(control_packet, &mut src);
            }
            ControlPacket::SubAck(control_packet) => {
                encode_sub_ack_packet(control_packet, &mut src);
            }
            ControlPacket::PingResp => (),
            _ => {
                return Err(Error::Other(format!("Not Implemented yet")));
            }
        };
        data.put_slice(&VariableByteInteger::encode_u32(src.len() as u32));
        data.extend(src);
        Ok(data)
    }
}

impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl std::error::Error for Error {
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Incomplete(no) => format!("stream ended early {}", no).fmt(fmt),
            Error::Other(err) => err.fmt(fmt),
        }
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Other(err.to_string())
    }
}
