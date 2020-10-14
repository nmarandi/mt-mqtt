use crate::{definitions::*, packet::*};
use bytes::{BufMut, Bytes, BytesMut};
use num_traits::ToPrimitive;

pub fn encode_fix_header(src: FixHeader, bytes: &mut BytesMut) {
    bytes.put_u8((src.control_packet_type.to_u8().unwrap() << 4) | src.flags.0 | (src.flags.1 << 1) | (src.flags.2 << 2) | (src.flags.3 << 3));
}
pub fn encode_conn_ack_packet(src: ConnAckControlPacket, bytes: &mut BytesMut) {
    bytes.put_u8(src.variable_header.conn_ack_flag.session_present_flag as u8);
    bytes.put_u8(src.variable_header.reason_code.to_u8().unwrap());
    encode_properties(src.variable_header.properties, bytes);
}
pub fn encode_pub_ack_packet(src: PubAckControlPacket, bytes: &mut BytesMut) {
    bytes.put_u16(src.variable_header.packet_identifier);
    bytes.put_u8(src.variable_header.reason_code.to_u8().unwrap());
    encode_properties(src.variable_header.get_properties(), bytes);
}
pub fn encode_pub_rec_packet(src: PubRecControlPacket, bytes: &mut BytesMut) {
    bytes.put_u16(src.variable_header.packet_identifier);
    bytes.put_u8(src.variable_header.reason_code.to_u8().unwrap());
    encode_properties(src.variable_header.get_properties(), bytes);
}
pub fn encode_pub_comp_packet(src: PubCompControlPacket, bytes: &mut BytesMut) {
    bytes.put_u16(src.variable_header.packet_identifier);
    bytes.put_u8(src.variable_header.reason_code.to_u8().unwrap());
    encode_properties(src.variable_header.get_properties(), bytes);
}
pub fn encode_sub_ack_packet(src: SubAckControlPacket, bytes: &mut BytesMut) {
    bytes.put_u16(src.variable_header.packet_identifier);
    encode_properties(src.variable_header.get_properties(), bytes);
    encode_sub_ack_payload(src.variable_header.sub_ack_payload, bytes);
}
pub fn encode_sub_ack_payload(src: SubAckPayload, bytes: &mut BytesMut) {
    for iter in src.sub_ack_reason_codes {
        bytes.put_u8(iter.to_u8().unwrap());
    }
}
pub fn encode_properties(src: Vec<Option<Property>>, bytes: &mut BytesMut) {
    let mut data: BytesMut = BytesMut::new();
    for elem in src.iter() {
        match elem {
            Some(property) => encode_property(property, &mut data),
            None => (),
        }
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
            encode_string(p_data, bytes);
        }
        Property::ResponseTopic(p_data) => {
            bytes.put_u8(8);
            encode_string(p_data, bytes);
        }
        Property::CorrelationData(p_data) => {
            bytes.put_u8(9);
            encode_binary_data(p_data, bytes);
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
            encode_string(p_data, bytes);
        }
        Property::ServerKeepAlive(p_data) => {
            bytes.put_u8(19);
            bytes.put_u16(*p_data);
        }
        Property::AuthenticationMethod(p_data) => {
            bytes.put_u8(21);
            encode_string(p_data, bytes);
        }
        Property::AuthenticationData(p_data) => {
            bytes.put_u8(22);
            encode_binary_data(p_data, bytes);
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
            encode_string(p_data, bytes);
        }
        Property::ServerReference(p_data) => {
            bytes.put_u8(28);
            encode_string(p_data, bytes);
        }
        Property::ReasonString(p_data) => {
            bytes.put_u8(31);
            encode_string(p_data, bytes);
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
            bytes.put_u8(p_data.to_u8().unwrap());
        }
        Property::RetainAvailable(p_data) => {
            bytes.put_u8(37);
            bytes.put_u8(*p_data);
        }
        Property::UserProperty(p_data) => {
            bytes.put_u8(38);
            encode_string(p_data, bytes);
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
pub fn encode_string(src: &str, bytes: &mut BytesMut) {
    bytes.put_u16(src.len() as u16);
    bytes.put_slice(src.as_bytes());
}
pub fn encode_binary_data(src: &Bytes, bytes: &mut BytesMut) {
    bytes.put_u16(src.len() as u16);
    bytes.put_slice(src.as_ref());
}
