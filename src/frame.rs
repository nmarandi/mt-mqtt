use crate::definitions::*;
use crate::packet::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use num_traits::{FromPrimitive, ToPrimitive};
use std::convert::TryFrom;
use std::fmt;
use std::io::Cursor;

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
                    .push(Property::AssignedClientIdentifier(String::from("")));
                conn_ack_control_packet
                    .variable_header
                    .conn_ack_flag
                    .session_present_flag = true;
                Frame {
                    fix_header: FixHeader::new(control_packet_type, Flags(0, 0, 0, 0)),
                    control_packet: ControlPacket::ConnAck(conn_ack_control_packet),
                }
            }
            _ => panic!("not implemented yet"),
            /*ControlPacketType::PUBLISH = 3,
            ControlPacketType::PUBACK = 4,
            ControlPacketType::PUBREC = 5,
            ControlPacketType::PUBREL = 6,
            ControlPacketType::PUBCOMP = 7,
            ControlPacketType::SUBSCRIBE = 8,
            ControlPacketType::SUBACK = 9,
            ControlPacketType::UNSUBSCRIBE = 10,
            ControlPacketType::UNSUBACK = 11,
            ControlPacketType::PINGREQ = 12,
            ControlPacketType::PINGRESP = 13,
            ControlPacketType::DISCONNECT = 14,
            ControlPacketType::AUTH = 15,*/
        }
    }
    pub fn deserialize(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        if src.remaining() < 5 {
            return Err(Error::Incomplete(src.remaining()));
        }
        println!("start deserialize");
        let pos = src.position();
        let fix_header = Frame::decode_fix_header(src).unwrap();
        println!("fix_header: {:?}", fix_header);
        let remianing_lenght = VariableByteInteger::new(src);
        if src.remaining() < usize::try_from(remianing_lenght.data).unwrap() {
            src.set_position(pos);
            return Err(Error::Incomplete(
                usize::try_from(remianing_lenght.data).unwrap(),
            ));
        }
        match fix_header.control_packet_type {
            ControlPacketType::CONNECT => Ok(Frame {
                control_packet: ControlPacket::Connect(Frame::decode_connect_packet(src).unwrap()),
                fix_header,
            }),
            _ => Err(Error::Other(format!("Not Implemented yet"))),
        }
    }
    pub fn decode_fix_header(src: &mut Cursor<&[u8]>) -> Option<FixHeader> {
        let data = src.get_u8();
        let packet_type = (data & 0b11110000) >> 4;
        let flags = Frame::flag_parser(packet_type, data);
        Some(FixHeader {
            control_packet_type: ControlPacketType::from_u8(packet_type).unwrap(),
            flags,
        })
    }
    fn flag_parser(tag: u8, data: u8) -> Flags {
        match tag {
            3 => Flags(data & 1, data & 6, data & 8, 0),
            _ => Flags(data & 1, data & 2, data & 4, data & 8),
        }
    }
    pub fn decode_connect_packet(src: &mut Cursor<&[u8]>) -> Result<ConnectControlPacket, Error> {
        let mut connect_control_packet: ConnectControlPacket = Default::default();
        connect_control_packet.variable_header =
            Frame::decode_connect_variable_header(src).unwrap();
        connect_control_packet.payload = Frame::decode_connect_payload(
            src,
            connect_control_packet.variable_header.connect_flag.clone(),
        )
        .unwrap();
        Ok(connect_control_packet)
    }
    pub fn decode_connect_variable_header(
        src: &mut Cursor<&[u8]>,
    ) -> Result<ConnectVariableHeader, Error> {
        let mut connect_variable_header: ConnectVariableHeader = Default::default();
        connect_variable_header.protocol_name = Frame::decode_string(src).unwrap();
        connect_variable_header.protocol_version = src.get_u8();
        connect_variable_header.connect_flag = ConnectFlags::new(src.get_u8());
        connect_variable_header.keep_alive = src.get_u16();
        connect_variable_header.properties = Frame::decode_properties(src).unwrap();
        Ok(connect_variable_header)
    }
    pub fn decode_connect_payload(
        src: &mut Cursor<&[u8]>,
        connect_flag: ConnectFlags,
    ) -> Result<ConnectPayload, Error> {
        let mut connect_payload: ConnectPayload = Default::default();
        connect_payload.client_identifier = Frame::decode_string(src).unwrap();
        if connect_flag.will_flag {
            connect_payload.will_properties = Some(Frame::decode_properties(src).unwrap());
            connect_payload.will_topic = Some(Frame::decode_string(src).unwrap());
            connect_payload.will_payload = Some(Frame::decode_binary_data(src).unwrap());
        }
        if connect_flag.user_name_flag {
            connect_payload.user_name = Some(Frame::decode_string(src).unwrap());
        }
        if connect_flag.password_flag {
            connect_payload.password = Some(Frame::decode_binary_data(src).unwrap());
        }
        Ok(connect_payload)
    }
    pub fn decode_string(src: &mut Cursor<&[u8]>) -> Result<String, Error> {
        let str_size_bytes = src.get_u16() as usize;

        let position = src.position() as usize;

        // TODO - Use Cow<str> and from_utf8_lossy later for less copying
        match String::from_utf8(src.get_ref()[position..(position + str_size_bytes)].into()) {
            Ok(string) => {
                src.advance(str_size_bytes);
                Ok(string)
            }
            Err(_) => Err(Error::Other(format!("decode string err"))),
        }
    }
    pub fn decode_binary_data(src: &mut Cursor<&[u8]>) -> Result<Bytes, Error> {
        let data_size_bytes = src.get_u16() as usize;

        let position = src.position() as usize;

        let payload_bytes =
            BytesMut::from(&src.get_ref()[position..(position + data_size_bytes)]).freeze();
        let result = Ok(payload_bytes);
        src.advance(data_size_bytes);

        result
    }
    pub fn decode_properties(src: &mut Cursor<&[u8]>) -> Result<Vec<Property>, Error> {
        let variable_byte_integer = VariableByteInteger::new(src);
        let lenght = variable_byte_integer.data as u64;
        let mut properties: Vec<Property> = Vec::new();
        let current_pos = src.position();
        while src.position() - current_pos < lenght {
            let identifier = src.get_u8();
            properties.push(match identifier {
                1 => Property::PayloadFormatIndicator(src.get_u8()),
                2 => Property::MessageExpiryInterval(src.get_u32()),
                3 => Property::ContentType(Frame::decode_string(src).unwrap()),
                8 => Property::ResponseTopic(Frame::decode_string(src).unwrap()),
                9 => Property::CorrelationData(Frame::decode_binary_data(src).unwrap()),
                11 => Property::SubscriptionIdentifier(VariableByteInteger::new(src)),
                17 => Property::SessionExpiryInterval(src.get_u32()),
                18 => Property::AssignedClientIdentifier(Frame::decode_string(src).unwrap()),
                19 => Property::ServerKeepAlive(src.get_u16()),
                21 => Property::AuthenticationMethod(Frame::decode_string(src).unwrap()),
                22 => Property::AuthenticationData(Frame::decode_binary_data(src).unwrap()),
                23 => Property::RequestProblemInformation(src.get_u8()),
                24 => Property::WillDelayInterval(src.get_u32()),
                25 => Property::RequestResponseInformation(src.get_u8()),
                26 => Property::ResponseInformation(Frame::decode_string(src).unwrap()),
                28 => Property::ServerReference(Frame::decode_string(src).unwrap()),
                31 => Property::ReasonString(Frame::decode_string(src).unwrap()),
                33 => Property::ReceiveMaximum(src.get_u16()),
                34 => Property::TopicAliasMaximum(src.get_u16()),
                35 => Property::TopicAlias(src.get_u16()),
                36 => Property::MaximumQoS(src.get_u8()),
                37 => Property::RetainAvailable(src.get_u8()),
                38 => Property::UserProperty(Frame::decode_string(src).unwrap()),
                39 => Property::MaximumPacketSize(src.get_u32()),
                40 => Property::WildcardSubscriptionAvailable(src.get_u8()),
                41 => Property::SubscriptionIdentifierAvailable(src.get_u8()),
                42 => Property::SharedSubscriptionAvailable(src.get_u8()),
                _ => return Err(Error::Other(format!("Unknow Identifier {}", identifier))),
            })
        }
        Ok(properties)
    }
    pub fn serialize(frame: Frame) -> Result<BytesMut, Error> {
        println!("start serialize: \n{:?}", frame);
        let mut data: BytesMut = BytesMut::new();
        Frame::encode_fix_header(frame.fix_header, &mut data);
        let mut src: BytesMut = BytesMut::new();
        match frame.control_packet {
            ControlPacket::ConnAck(conn_ack_packet) => {
                Frame::encode_conn_ack_packet(conn_ack_packet, &mut src);
            }
            _ => {
                return Err(Error::Other(format!("Not Implemented yet")));
            }
        };
        data.put_slice(&VariableByteInteger::encode_u32(src.len() as u32));
        data.extend(src);
        Ok(data)
    }
    pub fn encode_conn_ack_packet(src: ConnAckControlPacket, bytes: &mut BytesMut) {
        bytes.put_u8(src.variable_header.conn_ack_flag.session_present_flag as u8);
        bytes.put_u8(src.variable_header.reason_code.to_u8().unwrap());
        Frame::encode_properties(src.variable_header.properties, bytes);
    }
    pub fn encode_fix_header(src: FixHeader, bytes: &mut BytesMut) {
        bytes.put_u8(
            (src.control_packet_type.to_u8().unwrap() << 4)
                | src.flags.0
                | src.flags.1
                | src.flags.2
                | src.flags.3,
        );
    }
    pub fn encode_properties(src: Vec<Property>, bytes: &mut BytesMut) {
        let mut data: BytesMut = BytesMut::new();
        for elem in src.iter() {
            Frame::encode_property(elem, &mut data);
        }
        bytes.extend(VariableByteInteger::encode_u32(data.len() as u32));
        bytes.extend(data);
    }
    pub fn encode_property(src: &Property, bytes: &mut BytesMut) {
        match src {
            Property::PayloadFormatIndicator(p_data) => {
                bytes.put_u8(1);
                bytes.put_u8(*p_data);
            }
            Property::MessageExpiryInterval(p_data) => {
                bytes.put_u8(2);
                bytes.put_slice(&p_data.to_be_bytes());
            }
            Property::ContentType(p_data) => {
                bytes.put_u8(3);
                Frame::encode_string(p_data, bytes);
            }
            Property::ResponseTopic(p_data) => {
                bytes.put_u8(8);
                Frame::encode_string(p_data, bytes);
            }
            Property::CorrelationData(p_data) => {
                bytes.put_u8(9);
                Frame::encode_binary_data(p_data, bytes);
            }
            Property::SubscriptionIdentifier(p_data) => {
                bytes.put_u8(11);
                bytes.put_slice(&p_data.encode());
            }
            Property::SessionExpiryInterval(p_data) => {
                bytes.put_u8(17);
                bytes.put_slice(&p_data.to_be_bytes());
            }
            Property::AssignedClientIdentifier(p_data) => {
                bytes.put_u8(18);
                Frame::encode_string(p_data, bytes);
            }
            Property::ServerKeepAlive(p_data) => {
                bytes.put_u8(19);
                bytes.put_u16(*p_data);
            }
            Property::AuthenticationMethod(p_data) => {
                bytes.put_u8(21);
                Frame::encode_string(p_data, bytes);
            }
            Property::AuthenticationData(p_data) => {
                bytes.put_u8(22);
                Frame::encode_binary_data(p_data, bytes);
            }
            Property::RequestProblemInformation(p_data) => {
                bytes.put_u8(23);
                bytes.put_u8(*p_data);
            }
            Property::WillDelayInterval(p_data) => {
                bytes.put_u8(24);
                bytes.put_u32(*p_data);
            }
            Property::RequestResponseInformation(p_data) => {
                bytes.put_u8(25);
                bytes.put_u8(*p_data);
            }
            Property::ResponseInformation(p_data) => {
                bytes.put_u8(26);
                Frame::encode_string(p_data, bytes);
            }
            Property::ServerReference(p_data) => {
                bytes.put_u8(28);
                Frame::encode_string(p_data, bytes);
            }
            Property::ReasonString(p_data) => {
                bytes.put_u8(31);
                Frame::encode_string(p_data, bytes);
            }
            Property::ReceiveMaximum(p_data) => {
                bytes.put_u8(33);
                bytes.put_u16(*p_data);
            }
            Property::TopicAliasMaximum(p_data) => {
                bytes.put_u8(34);
                bytes.put_u16(*p_data);
            }
            Property::TopicAlias(p_data) => {
                bytes.put_u8(35);
                bytes.put_u16(*p_data);
            }
            Property::MaximumQoS(p_data) => {
                bytes.put_u8(36);
                bytes.put_u8(*p_data);
            }
            Property::RetainAvailable(p_data) => {
                bytes.put_u8(37);
                bytes.put_u8(*p_data);
            }
            Property::UserProperty(p_data) => {
                bytes.put_u8(38);
                Frame::encode_string(p_data, bytes);
            }
            Property::MaximumPacketSize(p_data) => {
                bytes.put_u8(39);
                bytes.put_u32(*p_data);
            }
            Property::WildcardSubscriptionAvailable(p_data) => {
                bytes.put_u8(40);
                bytes.put_u8(*p_data);
            }
            Property::SubscriptionIdentifierAvailable(p_data) => {
                bytes.put_u8(41);
                bytes.put_u8(*p_data);
            }
            Property::SharedSubscriptionAvailable(p_data) => {
                bytes.put_u8(42);
                bytes.put_u8(*p_data);
            }
        };
    }
    pub fn encode_string(src: &String, bytes: &mut BytesMut) {
        bytes.put_u16(src.len() as u16);
        bytes.put_slice(src.as_bytes());
    }
    pub fn encode_binary_data(src: &Bytes, bytes: &mut BytesMut) {
        bytes.put_u16(src.len() as u16);
        bytes.put_slice(src.as_ref());
    }
}

impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl std::error::Error for Error {}

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
