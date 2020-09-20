use crate::definitions::*;
use crate::packet::*;
use bytes::{Buf, Bytes, BytesMut};
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
pub enum Frame {
    Connect(ControlPacketType,ConnectControlPacket),
}

impl Frame {
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        println!("start parsing: message size: {}", src.get_ref().len());
        if src.remaining() < 5 {
            return Err(Error::Incomplete(src.remaining()));
        }
        let pos = src.position();
        let control_packet_type = ControlPacketType::new(src.get_u8()).unwrap();
        println!("control_packet_type: {:?}", control_packet_type);
        let remianing_lenght = VariableByteInteger::new(src);
        println!("remianing_lenght: {}", remianing_lenght.data);
        if src.remaining() < usize::try_from(remianing_lenght.data).unwrap() {
            src.set_position(pos);
            return Err(Error::Incomplete(
                usize::try_from(remianing_lenght.data).unwrap(),
            ));
        }
        match control_packet_type {
            ControlPacketType::CONNECT(_, _, _, _) => Ok(Frame::Connect(control_packet_type,Frame::parse_connect_packet(src).unwrap())),
            _ => Err(Error::Other(format!("Not Implemented yet"))),
        }
    }
    pub fn parse_connect_packet(src: &mut Cursor<&[u8]>) -> Result<ConnectControlPacket, Error> {
        let mut connect_control_packet: ConnectControlPacket = Default::default();
        connect_control_packet.variable_header = Frame::parse_connect_variable_header(src).unwrap();
        connect_control_packet.payload = Frame::parse_connect_payload(
            src,
            connect_control_packet.variable_header.connect_flag.clone(),
        )
        .unwrap();
        Ok(connect_control_packet)
    }
    pub fn parse_connect_variable_header(
        src: &mut Cursor<&[u8]>,
    ) -> Result<ConnectVariableHeader, Error> {
        let mut connect_variable_header: ConnectVariableHeader = Default::default();
        connect_variable_header.protocol_name = Frame::parse_string(src).unwrap();
        connect_variable_header.protocol_version = src.get_u8();
        connect_variable_header.connect_flag = ConnectFlags::new(src.get_u8());
        connect_variable_header.keep_alive = src.get_u16();
        connect_variable_header.properties = Frame::parse_properties(src).unwrap();
        Ok(connect_variable_header)
    }
    pub fn parse_connect_payload(
        src: &mut Cursor<&[u8]>,
        connect_flag: ConnectFlags,
    ) -> Result<ConnectPayload, Error> {
        let mut connect_payload: ConnectPayload = Default::default();
        connect_payload.client_identifier = Frame::parse_string(src).unwrap();
        if connect_flag.will_flag {
            connect_payload.will_properties = Some(Frame::parse_properties(src).unwrap());
            connect_payload.will_topic = Some(Frame::parse_string(src).unwrap());
            connect_payload.will_payload = Some(Frame::parse_binary_data(src).unwrap());
        }
        if connect_flag.user_name_flag {
            connect_payload.user_name = Some(Frame::parse_string(src).unwrap());
        }
        if connect_flag.password_flag {
            connect_payload.password = Some(Frame::parse_binary_data(src).unwrap());
        }
        Ok(connect_payload)
    }
    pub fn parse_string(src: &mut Cursor<&[u8]>) -> Result<String, Error> {
        let str_size_bytes = src.get_u16() as usize;

        let position = src.position() as usize;

        // TODO - Use Cow<str> and from_utf8_lossy later for less copying
        match String::from_utf8(src.get_ref()[position..(position + str_size_bytes)].into()) {
            Ok(string) => {
                src.advance(str_size_bytes);
                Ok(string)
            }
            Err(_) => Err(Error::Other(format!("Parse string err"))),
        }
    }
    pub fn parse_binary_data(src: &mut Cursor<&[u8]>) -> Result<Bytes, Error> {
        let data_size_bytes = src.get_u16() as usize;

        let position = src.position() as usize;

        let payload_bytes =
            BytesMut::from(&src.get_ref()[position..(position + data_size_bytes)]).freeze();
        let result = Ok(payload_bytes);
        src.advance(data_size_bytes);

        result
    }
    pub fn parse_properties(src: &mut Cursor<&[u8]>) -> Result<Properties, Error> {
        let variable_byte_integer = VariableByteInteger::new(src);
        let lenght = variable_byte_integer.data as u64;
        let mut properties: Vec<Property> = Vec::new();
        let current_pos = src.position();
        while src.position() - current_pos < lenght {
            let identifier = src.get_u8();
            properties.push(match identifier {
                1 => Property::PayloadFormatIndicator(src.get_u8()),
                2 => Property::MessageExpiryInterval(src.get_u32()),
                3 => Property::ContentType(Frame::parse_string(src).unwrap()),
                8 => Property::ResponseTopic(Frame::parse_string(src).unwrap()),
                9 => Property::CorrelationData(Frame::parse_binary_data(src).unwrap()),
                11 => Property::SubscriptionIdentifier(VariableByteInteger::new(src)),
                17 => Property::SessionExpiryInterval(src.get_u32()),
                18 => Property::AssignedClientIdentifier(Frame::parse_string(src).unwrap()),
                19 => Property::ServerKeepAlive(src.get_u16()),
                21 => Property::AuthenticationMethod(Frame::parse_string(src).unwrap()),
                22 => Property::AuthenticationData(Frame::parse_binary_data(src).unwrap()),
                23 => Property::RequestProblemInformation(src.get_u8()),
                24 => Property::WillDelayInterval(src.get_u32()),
                25 => Property::RequestResponseInformation(src.get_u8()),
                26 => Property::ResponseInformation(Frame::parse_string(src).unwrap()),
                28 => Property::ServerReference(Frame::parse_string(src).unwrap()),
                31 => Property::ReasonString(Frame::parse_string(src).unwrap()),
                33 => Property::ReceiveMaximum(src.get_u16()),
                34 => Property::TopicAliasMaximum(src.get_u16()),
                35 => Property::TopicAlias(src.get_u16()),
                36 => Property::MaximumQoS(src.get_u8()),
                37 => Property::RetainAvailable(src.get_u8()),
                38 => Property::UserProperty(Frame::parse_string(src).unwrap()),
                39 => Property::MaximumPacketSize(src.get_u32()),
                40 => Property::WildcardSubscriptionAvailable(src.get_u8()),
                41 => Property::SubscriptionIdentifierAvailable(src.get_u8()),
                42 => Property::SharedSubscriptionAvailable(src.get_u8()),
                _ => return Err(Error::Other(format!("Unknow Identifier {}", identifier))),
            })
        }
        Ok(Properties {
            lenght: variable_byte_integer,
            properties: properties,
        })
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
