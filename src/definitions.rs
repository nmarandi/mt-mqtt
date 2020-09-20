use bytes::{Buf, Bytes};
use std::io::Cursor;

#[repr(u8)]
#[derive(Debug)]
pub enum ControlPacketType {
    CONNECT(u8, u8, u8, u8) = 1,
    CONNACK(u8, u8, u8, u8) = 2,
    PUBLISH(u8, u8, u8, u8) = 3,
    PUBACK(u8, u8, u8, u8) = 4,
    PUBREC(u8, u8, u8, u8) = 5,
    PUBREL(u8, u8, u8, u8) = 6,
    PUBCOMP(u8, u8, u8, u8) = 7,
    SUBSCRIBE(u8, u8, u8, u8) = 8,
    SUBACK(u8, u8, u8, u8) = 9,
    UNSUBSCRIBE(u8, u8, u8, u8) = 10,
    UNSUBACK(u8, u8, u8, u8) = 11,
    PINGREQ(u8, u8, u8, u8) = 12,
    PINGRESP(u8, u8, u8, u8) = 13,
    DISCONNECT(u8, u8, u8, u8) = 14,
    AUTH(u8, u8, u8, u8) = 15,
}

impl ControlPacketType {
    pub fn new(data: u8) -> Option<ControlPacketType> {
        let packet_type = (data & 0b11110000) >> 4;
        let flags = ControlPacketType::flag_parser(packet_type, data);
        match packet_type {
            1 => Some(ControlPacketType::CONNECT(
                flags.0, flags.1, flags.2, flags.3,
            )),
            2 => Some(ControlPacketType::CONNACK(
                flags.0, flags.1, flags.2, flags.3,
            )),
            3 => Some(ControlPacketType::PUBLISH(
                flags.0, flags.1, flags.2, flags.3,
            )),
            4 => Some(ControlPacketType::PUBACK(
                flags.0, flags.1, flags.2, flags.3,
            )),
            5 => Some(ControlPacketType::PUBREC(
                flags.0, flags.1, flags.2, flags.3,
            )),
            6 => Some(ControlPacketType::PUBREL(
                flags.0, flags.1, flags.2, flags.3,
            )),
            7 => Some(ControlPacketType::PUBCOMP(
                flags.0, flags.1, flags.2, flags.3,
            )),
            8 => Some(ControlPacketType::SUBSCRIBE(
                flags.0, flags.1, flags.2, flags.3,
            )),
            9 => Some(ControlPacketType::SUBACK(
                flags.0, flags.1, flags.2, flags.3,
            )),
            10 => Some(ControlPacketType::UNSUBSCRIBE(
                flags.0, flags.1, flags.2, flags.3,
            )),
            11 => Some(ControlPacketType::UNSUBACK(
                flags.0, flags.1, flags.2, flags.3,
            )),
            12 => Some(ControlPacketType::PINGREQ(
                flags.0, flags.1, flags.2, flags.3,
            )),
            13 => Some(ControlPacketType::PINGRESP(
                flags.0, flags.1, flags.2, flags.3,
            )),
            14 => Some(ControlPacketType::DISCONNECT(
                flags.0, flags.1, flags.2, flags.3,
            )),
            15 => Some(ControlPacketType::AUTH(flags.0, flags.1, flags.2, flags.3)),
            _ => None,
        }
    }
    fn flag_parser(tag: u8, data: u8) -> (u8, u8, u8, u8) {
        match tag {
            3 => (data & 1, data & 6, data & 8, 0),
            _ => (data & 1, data & 2, data & 4, data & 8),
        }
    }
}
#[repr(u8)]
pub enum ReasonCode {
    SuccessNormalDisconnectionGrantedQoS0 = 0,
    GrantedQoS1 = 1,
    GrantedQoS2 = 2,
    DisconnectWithWillMessage = 4,
    NoMatchingSubscribers = 16,
    Nosubscriptionexisted = 17,
    ContinueAuthentication = 24,
    ReAuthenticate = 25,
    UnspecifiedError = 128,
    MalformedPacket = 129,
    ProtocolError = 130,
    ImplementationSpecificError = 131,
    UnsupportedProtocolVersion = 132,
    ClientIdentifierNotValid = 133,
    BadUserNameOrPassword = 134,
    NotAuthorized = 135,
    ServerUnavailable = 136,
    ServerBusy = 137,
    Banned = 138,
    ServerShuttingDown = 139,
    BadAuthenticationMethod = 140,
    KeepAliveTimeout = 141,
    SessionTakenOver = 142,
    TopicFilterInvalid = 143,
    TopicNameInvalid = 144,
    PacketIdentifierInUse = 145,
    PacketIdentifierNotFound = 146,
    ReceiveMaximumExceeded = 147,
    TopicAliasInvalid = 148,
    PacketTooLarge = 149,
    MessageRateTooHigh = 150,
    QuotaExceeded = 151,
    AdministrativeAction = 152,
    PayloadFormatInvalid = 153,
    RetainNotSupported = 154,
    QoSNotSupported = 155,
    UseAnotherServer = 156,
    ServerMoved = 157,
    SharedSubscriptionsNotSupported = 158,
    ConnectionRateExceeded = 159,
    MaximumConnectTime = 160,
    SubscriptionIdentifiersNotSupported = 161,
    WildcardSubscriptionsNotSupported = 162,
}
#[derive(Debug, Default)]
pub struct VariableByteInteger {
    pub data: u32,
}
impl VariableByteInteger {
    pub fn new(encoded_byte: &mut Cursor<&[u8]>) -> VariableByteInteger {
        VariableByteInteger {
            data: VariableByteInteger::decode(encoded_byte),
        }
    }
    pub fn encode(self) -> [u8; 4] {
        let mut encoded_byte: [u8; 4] = [0; 4];
        let mut x = self.data;
        let mut i = 0;
        loop {
            encoded_byte[i] = (((x % 128) + 128) % 128) as u8;
            x = x / 128;
            if x == 0 {
                break;
            } else {
                encoded_byte[i] = encoded_byte[i] | 128
            }
            i += 1;
        }
        encoded_byte
    }
    pub fn decode(encoded_byte: &mut Cursor<&[u8]>) -> u32 {
        let mut multiplier: u32 = 1;
        let mut data = 0;
        let mut i = 0;
        loop {
            let read_byte = encoded_byte.get_u8();
            data += (read_byte & 127) as u32 * multiplier;
            if multiplier > 128 * 128 * 128 {
                panic!("Malformed Variable Byte Integer");
            }
            multiplier *= 128;
            if (read_byte & 128) == 0 {
                break;
            }
            i += 1;
        }
        data
    }
}
#[repr(u8)]
#[derive(Debug)]
pub enum Property {
    PayloadFormatIndicator(u8) = 1,
    MessageExpiryInterval(u32) = 2,
    ContentType(String) = 3,
    ResponseTopic(String) = 8,
    CorrelationData(Bytes) = 9,
    SubscriptionIdentifier(VariableByteInteger) = 11,
    SessionExpiryInterval(u32) = 17,
    AssignedClientIdentifier(String) = 18,
    ServerKeepAlive(u16) = 19,
    AuthenticationMethod(String) = 21,
    AuthenticationData(Bytes) = 22,
    RequestProblemInformation(u8) = 23,
    WillDelayInterval(u32) = 24,
    RequestResponseInformation(u8) = 25,
    ResponseInformation(String) = 26,
    ServerReference(String) = 28,
    ReasonString(String) = 31,
    ReceiveMaximum(u16) = 33,
    TopicAliasMaximum(u16) = 34,
    TopicAlias(u16) = 35,
    MaximumQoS(u8) = 36,
    RetainAvailable(u8) = 37,
    UserProperty(String) = 38,
    MaximumPacketSize(u32) = 39,
    WildcardSubscriptionAvailable(u8) = 40,
    SubscriptionIdentifierAvailable(u8) = 41,
    SharedSubscriptionAvailable(u8) = 42,
}
pub fn have_packet_identifier(control_packet_type: ControlPacketType) -> bool {
    match control_packet_type {
        ControlPacketType::CONNECT(_, _, _, _)
        | ControlPacketType::CONNACK(_, _, _, _)
        | ControlPacketType::PINGREQ(_, _, _, _)
        | ControlPacketType::PINGRESP(_, _, _, _)
        | ControlPacketType::DISCONNECT(_, _, _, _)
        | ControlPacketType::AUTH(_, _, _, _) => false,
        ControlPacketType::PUBACK(_, _, _, _)
        | ControlPacketType::PUBREC(_, _, _, _)
        | ControlPacketType::PUBREL(_, _, _, _)
        | ControlPacketType::PUBCOMP(_, _, _, _)
        | ControlPacketType::SUBSCRIBE(_, _, _, _)
        | ControlPacketType::SUBACK(_, _, _, _)
        | ControlPacketType::UNSUBSCRIBE(_, _, _, _)
        | ControlPacketType::UNSUBACK(_, _, _, _) => true,
        ControlPacketType::PUBLISH(_, qos, _, _) => {
            if qos > 0 {
                return true;
            } else {
                return false;
            }
        }
    }
}
#[derive(Debug)]
pub struct FixHeader {
    control_packet_type: ControlPacketType,
    remianing_lenght: VariableByteInteger,
}

pub struct PacketIdentifier {
    identifier: [u8; 2],
}
#[derive(Debug, Default)]
pub struct Properties {
    pub lenght: VariableByteInteger,
    pub properties: Vec<Property>,
}
pub struct VariableHeader {
    packet_identifier: PacketIdentifier,
}
pub enum PayloadCondition {
    Required,
    Optional,
    None,
}
pub struct Payload {}
pub struct ControlPacket {
    fix_header: FixHeader,
    variable_header: VariableHeader,
    payload: Payload,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn variable_byte_integer_encode() {
        let var = VariableByteInteger { data: 128 };
        assert_eq!(var.encode(), [0x80, 0x1, 0, 0]);
    }
    #[test]
    fn variable_byte_integer_decode() {
        let test_vec = vec![0x80, 0x1, 0, 0];
        let mut buff = Cursor::new(test_vec.as_slice());
        assert_eq!(VariableByteInteger::decode(&mut buff), 128);
    }
}
