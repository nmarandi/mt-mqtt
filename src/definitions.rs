#[repr(u8)]
pub enum ControlPacketType {
    CONNECT(u8, u8, u8, u8) = 1,
    CONNACK(u8, u8, u8, u8) = 2,
    PUBLISH(u8, u8, u8) = 3,
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
pub struct VariableByteInteger {
    data: u32,
}
impl VariableByteInteger {
    fn encode(self) -> [u8; 4] {
        let mut encodedByte: [u8; 4] = [0; 4];
        let mut x = self.data;
        let mut i = 0;
        loop {
            encodedByte[i] = (((x % 128) + 128) % 128) as u8;
            x = x / 128;
            if x == 0 {
                break;
            } else {
                encodedByte[i] = encodedByte[i] | 128
            }
            i += 1;
        }
        encodedByte
    }
    fn decode(mut self, encodedByte: [u8; 4]) -> u32 {
        let mut multiplier: u32 = 1;
        self.data = 0;
        let mut i = 0;
        loop {
            self.data += (encodedByte[i] & 127) as u32 * multiplier;
            if multiplier > 128 * 128 * 128 {
                panic!("Malformed Variable Byte Integer");
            }
            multiplier *= 128;
            if (encodedByte[i] & 128) == 0 {
                break;
            }
            i += 1;
        }
        self.data
    }
}
#[repr(u8)]
pub enum Property {
    PayloadFormatIndicator(u8) = 1,
    MessageExpiryInterval(u32) = 2,
    ContentType(String) = 3,
    ResponseTopic(String) = 8,
    CorrelationData(Vec<u8>) = 9,
    SubscriptionIdentifier(VariableByteInteger) = 11,
    SessionExpiryInterval(u32) = 17,
    AssignedClientIdentifier(String) = 18,
    ServerKeepAlive(u16) = 19,
    AuthenticationMethod(String) = 21,
    AuthenticationData(Vec<u8>) = 22,
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
pub fn have_PacketIdentifier(control_packet_type: ControlPacketType) -> bool {
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
        ControlPacketType::PUBLISH(_, qos, _) => {
            if qos > 0 {
                return true;
            } else {
                return false;
            }
        }
    }
}
pub enum ControlPacketTypeFlags {}
pub struct FixHeader {
    control_packet: ControlPacketType,
    control_packet_type_flags: ControlPacketTypeFlags,
    remianing_lenght: i32,
}
pub struct PacketIdentifier {
    identifier: [u8; 2],
}
pub struct Properties {
    lenght: u8,
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
    fn VariableByteInteger_encode() {
        let var = VariableByteInteger { data: 128 };
        assert_eq!(var.encode(), [0x80, 0x1, 0, 0]);
    }
    #[test]
    fn VariableByteInteger_decode() {
        let var = VariableByteInteger { data: 0 };
        assert_eq!(var.decode([0x80, 0x1, 0, 0]), 128);
    }
}
