use crate::{definitions::*, frame::Error, packet::*};
use bytes::{Buf, Bytes, BytesMut};
use num_traits::FromPrimitive;
use std::io::Cursor;

fn decode_flag(tag: u8, data: u8) -> Flags {
    match tag {
        3 => Flags(data & 1, (data & 6) >> 1, 0, (data & 8) >> 3),
        _ => Flags(data & 1, (data & 2) >> 1, (data & 4) >> 2, (data & 8) >> 3),
    }
}
pub fn decode_fix_header(src: &mut Cursor<&[u8]>) -> Option<FixHeader> {
    let data = src.get_u8();
    let packet_type = (data & 0b11110000) >> 4;
    let flags = decode_flag(packet_type, data);
    Some(FixHeader {
        control_packet_type: ControlPacketType::from_u8(packet_type).unwrap(),
        flags,
    })
}
pub fn decode_connect_packet(src: &mut Cursor<&[u8]>) -> Result<ConnectControlPacket, Error> {
    let mut connect_control_packet: ConnectControlPacket = Default::default();
    connect_control_packet.variable_header = decode_connect_variable_header(src).unwrap();
    connect_control_packet.payload = decode_connect_payload(src, connect_control_packet.variable_header.connect_flag.clone()).unwrap();
    Ok(connect_control_packet)
}
pub fn decode_connect_variable_header(src: &mut Cursor<&[u8]>) -> Result<ConnectVariableHeader, Error> {
    let mut connect_variable_header: ConnectVariableHeader = Default::default();
    connect_variable_header.protocol_name = decode_string(src).unwrap();
    connect_variable_header.protocol_version = src.get_u8();
    connect_variable_header.connect_flag = ConnectFlags::new(src.get_u8());
    connect_variable_header.keep_alive = src.get_u16();
    connect_variable_header.properties = decode_properties(src).unwrap();
    Ok(connect_variable_header)
}
pub fn decode_connect_payload(src: &mut Cursor<&[u8]>, connect_flag: ConnectFlags) -> Result<ConnectPayload, Error> {
    let mut connect_payload: ConnectPayload = Default::default();
    connect_payload.client_identifier = decode_string(src).unwrap();
    if connect_flag.will_flag {
        connect_payload.will_properties = decode_properties(src).unwrap();
        connect_payload.will_topic = Some(decode_string(src).unwrap());
        connect_payload.will_payload = Some(decode_binary_data(src).unwrap());
    }
    if connect_flag.user_name_flag {
        connect_payload.user_name = Some(decode_string(src).unwrap());
    }
    if connect_flag.password_flag {
        connect_payload.password = Some(decode_binary_data(src).unwrap());
    }
    Ok(connect_payload)
}

pub fn decode_publish_packet(src: &mut Cursor<&[u8]>, qos: u8) -> Result<PublishControlPacket, Error> {
    let mut publish_control_packet: PublishControlPacket = Default::default();
    publish_control_packet.variable_header = decode_publish_variable_header(src, qos).unwrap();
    publish_control_packet.payload = decode_publish_payload(src).unwrap();
    Ok(publish_control_packet)
}
pub fn decode_publish_variable_header(src: &mut Cursor<&[u8]>, qos: u8) -> Result<PublishVariableHeader, Error> {
    let mut publish_variable_header: PublishVariableHeader = Default::default();
    publish_variable_header.topic_name = decode_string(src).unwrap();
    if qos > 0 {
        publish_variable_header.packet_identifier = Some(src.get_u16());
    } else {
        publish_variable_header.packet_identifier = None;
    }
    publish_variable_header.set_properties(decode_properties(src).unwrap());
    Ok(publish_variable_header)
}
pub fn decode_publish_payload(src: &mut Cursor<&[u8]>) -> Result<PublishPayload, Error> {
    let mut public_payload: PublishPayload = Default::default();
    let position = src.position() as usize;
    public_payload.data = BytesMut::from(&src.get_ref()[position..]).freeze();
    src.advance(src.remaining());
    Ok(public_payload)
}

pub fn decode_pub_rel_packet(src: &mut Cursor<&[u8]>) -> Result<PubRelControlPacket, Error> {
    let mut pub_rel_control_packet: PubRelControlPacket = Default::default();
    pub_rel_control_packet.variable_header = decode_pub_rel_variable_header(src).unwrap();
    Ok(pub_rel_control_packet)
}
pub fn decode_pub_rel_variable_header(src: &mut Cursor<&[u8]>) -> Result<PubRelVariableHeader, Error> {
    let mut pub_rel_variable_header: PubRelVariableHeader = Default::default();
    pub_rel_variable_header.packet_identifier = src.get_u16();
    if src.has_remaining() {
        pub_rel_variable_header.reason_code = PubRelReasonCode::from_u8(src.get_u8()).unwrap();
        if src.has_remaining() {
            pub_rel_variable_header.set_properties(decode_properties(src).unwrap());
        }
    } else {
        pub_rel_variable_header.reason_code = PubRelReasonCode::Success;
    }
    Ok(pub_rel_variable_header)
}

pub fn decode_subscribe_packet(src: &mut Cursor<&[u8]>) -> Result<SubscribeControlPacket, Error> {
    let mut pub_rel_control_packet: SubscribeControlPacket = Default::default();
    pub_rel_control_packet.variable_header = decode_subscribe_variable_header(src).unwrap();
    Ok(pub_rel_control_packet)
}
pub fn decode_subscribe_variable_header(src: &mut Cursor<&[u8]>) -> Result<SubscribeVariableHeader, Error> {
    let mut subscribe_variable_header: SubscribeVariableHeader = Default::default();
    subscribe_variable_header.packet_identifier = src.get_u16();
    subscribe_variable_header.set_properties(decode_properties(src).unwrap());
    subscribe_variable_header.subscribe_payload = decode_subscribe_payload(src).unwrap();
    Ok(subscribe_variable_header)
}
pub fn decode_subscribe_payload(src: &mut Cursor<&[u8]>) -> Result<Vec<SubscribePayload>, Error> {
    let mut subscribe_payload: Vec<SubscribePayload> = Vec::new();
    while src.has_remaining() {
        subscribe_payload.push(SubscribePayload {
            topic_filter: decode_string(src).unwrap(),
            subscription_options: decode_subscription_options(src).unwrap(),
        })
    }
    Ok(subscribe_payload)
}

pub fn decode_subscription_options(src: &mut Cursor<&[u8]>) -> Result<SubscriptionOptions, Error> {
    Ok(SubscriptionOptions::new(src.get_u8()))
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

    let payload_bytes = BytesMut::from(&src.get_ref()[position..(position + data_size_bytes)]).freeze();
    let result = Ok(payload_bytes);
    src.advance(data_size_bytes);

    result
}
pub fn decode_properties(src: &mut Cursor<&[u8]>) -> Result<Vec<Option<Property>>, Error> {
    let variable_byte_integer = VariableByteInteger::from(src);
    let lenght = variable_byte_integer.data as u64;
    let mut properties: Vec<Option<Property>> = Vec::new();
    let current_pos = src.position();
    while src.position() - current_pos < lenght {
        let identifier = src.get_u8();
        properties.push(Some(match identifier {
            1 => Property::PayloadFormatIndicator(src.get_u8()),
            2 => Property::MessageExpiryInterval(src.get_u32()),
            3 => Property::ContentType(decode_string(src).unwrap()),
            8 => Property::ResponseTopic(decode_string(src).unwrap()),
            9 => Property::CorrelationData(decode_binary_data(src).unwrap()),
            11 => Property::SubscriptionIdentifier(VariableByteInteger::from(src)),
            17 => Property::SessionExpiryInterval(src.get_u32()),
            18 => Property::AssignedClientIdentifier(decode_string(src).unwrap()),
            19 => Property::ServerKeepAlive(src.get_u16()),
            21 => Property::AuthenticationMethod(decode_string(src).unwrap()),
            22 => Property::AuthenticationData(decode_binary_data(src).unwrap()),
            23 => Property::RequestProblemInformation(src.get_u8()),
            24 => Property::WillDelayInterval(src.get_u32()),
            25 => Property::RequestResponseInformation(src.get_u8()),
            26 => Property::ResponseInformation(decode_string(src).unwrap()),
            28 => Property::ServerReference(decode_string(src).unwrap()),
            31 => Property::ReasonString(decode_string(src).unwrap()),
            33 => Property::ReceiveMaximum(src.get_u16()),
            34 => Property::TopicAliasMaximum(src.get_u16()),
            35 => Property::TopicAlias(src.get_u16()),
            36 => Property::MaximumQoS(Qos::from_u8(src.get_u8()).unwrap()),
            37 => Property::RetainAvailable(src.get_u8()),
            38 => Property::UserProperty(decode_string(src).unwrap()),
            39 => Property::MaximumPacketSize(src.get_u32()),
            40 => Property::WildcardSubscriptionAvailable(src.get_u8()),
            41 => Property::SubscriptionIdentifierAvailable(src.get_u8()),
            42 => Property::SharedSubscriptionAvailable(src.get_u8()),
            _ => return Err(Error::Other(format!("Unknow Identifier {}", identifier))),
        }))
    }
    Ok(properties)
}
