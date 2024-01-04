use bytes::{Buf, Bytes};
use num_derive::{FromPrimitive, ToPrimitive};
use std::io::Cursor;
use strum_macros::Display;

#[repr(u8)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum ControlPacketType {
    CONNECT = 1,
    CONNACK = 2,
    PUBLISH = 3,
    PUBACK = 4,
    PUBREC = 5,
    PUBREL = 6,
    PUBCOMP = 7,
    SUBSCRIBE = 8,
    SUBACK = 9,
    UNSUBSCRIBE = 10,
    UNSUBACK = 11,
    PINGREQ = 12,
    PINGRESP = 13,
    DISCONNECT = 14,
    AUTH = 15,
}
#[repr(u8)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
#[allow(dead_code)]
pub enum ConnAckReasonCode {
    Success = 0,
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
    BadAuthenticationMethod = 140,
    TopicNameInvalid = 144,
    PacketTooLarge = 149,
    QuotaExceeded = 151,
    PayloadFormatInvalid = 153,
    RetainNotSupported = 154,
    QoSNotSupported = 155,
    UseAnotherServer = 156,
    ServerMoved = 157,
    ConnectionRateExceeded = 159,
}
impl Default for ConnAckReasonCode {
    fn default() -> Self {
        Self::Success
    }
}
#[repr(u8)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
#[allow(dead_code)]
pub enum PubAckReasonCode {
    Success = 0,
    NoMatchingSubscribers = 16,
    UnspecifiedError = 128,
    ImplementationSpecificError = 131,
    NotAuthorized = 135,
    TopicNameInvalid = 144,
    PacketIdentifierInUse = 145,
    QuotaExceeded = 151,
    PayloadFormatInvalid = 153,
}
impl Default for PubAckReasonCode {
    fn default() -> Self {
        Self::Success
    }
}
#[repr(u8)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
#[allow(dead_code)]
pub enum PubRecReasonCode {
    Success = 0,
    NoMatchingSubscribers = 16,
    UnspecifiedError = 128,
    ImplementationSpecificError = 131,
    NotAuthorized = 135,
    TopicNameInvalid = 144,
    PacketIdentifierInUse = 145,
    QuotaExceeded = 151,
    PayloadFormatInvalid = 153,
}
impl Default for PubRecReasonCode {
    fn default() -> Self {
        Self::Success
    }
}
#[repr(u8)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
#[allow(dead_code)]
pub enum PubRelReasonCode {
    Success = 0,
    PacketIdentifierNotFound = 146,
}
impl Default for PubRelReasonCode {
    fn default() -> Self {
        Self::Success
    }
}
#[repr(u8)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
#[allow(dead_code)]
pub enum PubCompReasonCode {
    Success = 0,
    PacketIdentifierNotFound = 146,
}
impl Default for PubCompReasonCode {
    fn default() -> Self {
        Self::Success
    }
}
#[repr(u8)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
#[allow(dead_code)]
pub enum SubAckReasonCode {
    GrantedQoS0 = 0,
    GrantedQoS1 = 1,
    GrantedQoS2 = 2,
    UnspecifiedError = 128,
    ImplementationSpecificError = 131,
    NotAuthorized = 135,
    TopicFilterInvalid = 143,
    PacketIdentifierInUse = 145,
    QuotaExceeded = 151,
    SharedSubscriptionsNotSupported = 158,
    SubscriptionIdentifiersNotSupported = 161,
    WildcardSubscriptionsNotSupported = 162,
}
impl Default for SubAckReasonCode {
    fn default() -> Self {
        Self::UnspecifiedError
    }
}
#[repr(u8)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
#[allow(dead_code)]
pub enum UnSubAckReasonCode {
    Success = 0,
    UnspecifiedError = 128,
    ImplementationSpecificError = 131,
    NotAuthorized = 135,
    TopicFilterInvalid = 143,
    PacketIdentifierInUse = 145,
}
impl Default for UnSubAckReasonCode {
    fn default() -> Self {
        Self::Success
    }
}
#[repr(u8)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
#[allow(dead_code)]
pub enum DisconnectReasonCode {
    NormalDisconnection = 0,
    DisconnectWithWillMessage = 4,
    UnspecifiedError = 128,
    MalformedPacket = 129,
    ProtocolError = 130,
    ImplementationSpecificError = 131,
    NotAuthorized = 135,
    ServerBusy = 137,
    ServerShuttingDown = 139,
    KeepAliveTimeout = 141,
    SessionTakenOver = 142,
    TopicFilterInvalid = 143,
    TopicNameInvalid = 144,
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
impl Default for DisconnectReasonCode {
    fn default() -> Self {
        Self::NormalDisconnection
    }
}
#[repr(u8)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
#[allow(dead_code)]
pub enum AuthReasonCode {
    Success = 0,
    ContinueAuthentication = 24,
    ReAuthenticate = 25,
}
impl Default for AuthReasonCode {
    fn default() -> Self {
        Self::Success
    }
}
#[derive(Debug, Default, Copy, Clone)]
pub struct VariableByteInteger {
    pub data: u32,
}
impl VariableByteInteger {
    pub fn new() -> VariableByteInteger {
        VariableByteInteger { data: 0 }
    }

    pub fn from(encoded_byte: &mut Cursor<&[u8]>) -> VariableByteInteger {
        VariableByteInteger {
            data: VariableByteInteger::decode(encoded_byte),
        }
    }

    pub fn encode(self) -> Vec<u8> {
        VariableByteInteger::encode_u32(self.data)
    }

    pub fn encode_u32(data: u32) -> Vec<u8> {
        let mut encoded_bytes: Vec<u8> = Vec::new();
        let mut x = data;
        loop {
            let mut encoded_byte = (((x % 128) + 128) % 128) as u8;
            x /= 128;
            if x == 0 {
                encoded_bytes.push(encoded_byte);
                break;
            } else {
                encoded_byte |= 128;
            }
            encoded_bytes.push(encoded_byte);
        }
        encoded_bytes
    }

    pub fn decode(encoded_byte: &mut Cursor<&[u8]>) -> u32 {
        let mut multiplier: u32 = 1;
        let mut data = 0;
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
        }
        data
    }
}
#[repr(u8)]
#[derive(Display, Debug, Clone)]
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
    MaximumQoS(Qos) = 36,
    RetainAvailable(u8) = 37,
    UserProperty(String) = 38,
    MaximumPacketSize(u32) = 39,
    WildcardSubscriptionAvailable(u8) = 40,
    SubscriptionIdentifierAvailable(u8) = 41,
    SharedSubscriptionAvailable(u8) = 42,
}
#[allow(dead_code)]
pub fn have_packet_identifier(fix_header: FixHeader) -> bool {
    match fix_header.control_packet_type {
        ControlPacketType::CONNECT
        | ControlPacketType::CONNACK
        | ControlPacketType::PINGREQ
        | ControlPacketType::PINGRESP
        | ControlPacketType::DISCONNECT
        | ControlPacketType::AUTH => false,
        ControlPacketType::PUBACK
        | ControlPacketType::PUBREC
        | ControlPacketType::PUBREL
        | ControlPacketType::PUBCOMP
        | ControlPacketType::SUBSCRIBE
        | ControlPacketType::SUBACK
        | ControlPacketType::UNSUBSCRIBE
        | ControlPacketType::UNSUBACK => true,
        ControlPacketType::PUBLISH => {
            fix_header.flags.1 > 0 
        }
    }
}
#[derive(Debug)]
pub struct Flags(pub u8, pub u8, pub u8, pub u8);
#[derive(Debug)]
pub struct FixHeader {
    pub control_packet_type: ControlPacketType,
    pub flags: Flags,
}
impl FixHeader {
    pub fn new(control_packet_type: ControlPacketType, flags: Flags) -> FixHeader {
        FixHeader { control_packet_type, flags }
    }
}
pub enum PayloadCondition {
    Required,
    Optional,
    None,
}
#[repr(u8)]
#[derive(Display, Debug, Clone, FromPrimitive, ToPrimitive)]
pub enum Qos {
    AtMostOnce = 0,
    AtleastOnce = 1,
    ExactlyOnce = 2,
}
impl Default for Qos {
    fn default() -> Self {
        Self::AtMostOnce
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn variable_byte_integer_encode() {
        let var = VariableByteInteger { data: 128 };
        assert_eq!(var.encode().as_slice(), [0x80, 0x1]);
    }
    #[test]
    fn variable_byte_integer_decode() {
        let test_vec = vec![0x80, 0x1, 0, 0];
        let mut buff = Cursor::new(test_vec.as_slice());
        assert_eq!(VariableByteInteger::decode(&mut buff), 128);
    }
}
