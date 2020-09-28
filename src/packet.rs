use crate::definitions::*;
use bytes::Bytes;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::collections::HashMap;
use std::string::ToString;
use strum_macros::Display;

#[derive(Debug, Default)]
pub struct Properties {
    pub properties: HashMap<String, Option<Property>>,
}
impl Properties {
    pub fn set_properties(&mut self, property: Option<Property>) {
        if let Some(property) = property {
            *(self.properties.get_mut(&property.to_string()).unwrap()) = Some(property.clone());
        }
    }
    pub fn set_properties_vec(&mut self, properties: Vec<Option<Property>>) {
        for property in properties {
            self.set_properties(property);
        }
    }
}
#[derive(Debug, Clone, Default)]
pub struct ConnectFlags {
    pub clean_start: bool,
    pub will_flag: bool,
    pub will_qos: u8,
    pub will_retain: bool,
    pub password_flag: bool,
    pub user_name_flag: bool,
}
impl ConnectFlags {
    pub fn new(byte: u8) -> ConnectFlags {
        ConnectFlags {
            clean_start: (byte & 0b0000_0010) != 0,
            will_flag: (byte & 0b0000_0100) != 0,
            will_qos: (byte & 0b0001_1000) >> 3,
            will_retain: (byte & 0b0010_0000) != 0,
            password_flag: (byte & 0b0100_0000) != 0,
            user_name_flag: (byte & 0b1000_0000) != 0,
        }
    }
}
#[derive(Debug, Default)]
pub struct ConnectVariableHeader {
    pub protocol_name: String,
    pub protocol_version: u8,
    pub connect_flag: ConnectFlags,
    pub keep_alive: u16,
    pub properties: Vec<Option<Property>>,
}
#[derive(Debug, Default)]
pub struct ConnectPayload {
    pub client_identifier: String,
    pub will_properties: Vec<Option<Property>>,
    pub will_topic: Option<String>,
    pub will_payload: Option<Bytes>,
    pub user_name: Option<String>,
    pub password: Option<Bytes>,
}
#[derive(Debug, Default)]
pub struct ConnectControlPacket {
    pub variable_header: ConnectVariableHeader,
    pub payload: ConnectPayload,
}
#[derive(Debug, Clone, Default)]
pub struct ConnAckFlags {
    pub session_present_flag: bool,
}
impl ConnAckFlags {
    pub fn new(byte: u8) -> ConnAckFlags {
        ConnAckFlags {
            session_present_flag: (byte & 0b0000_0001) != 0,
        }
    }
}
#[derive(Debug, Default)]
pub struct ConnAckVariableHeader {
    pub conn_ack_flag: ConnAckFlags,
    pub reason_code: ConnAckReasonCode,
    pub properties: Vec<Option<Property>>,
}
#[derive(Debug, Default)]
pub struct ConnAckControlPacket {
    pub variable_header: ConnAckVariableHeader,
}
#[derive(Debug, Default)]
pub struct PublishControlPacket {
    pub variable_header: PublishVariableHeader,
    pub payload: PublishPayload,
}
#[derive(Debug, Default)]
pub struct PublishPayload {
    pub data: Bytes,
}
#[derive(Debug)]
pub struct PublishVariableHeader {
    pub topic_name: String,
    pub packet_identifier: u16,
    properties: Properties,
}
impl PublishVariableHeader {
    pub fn new() -> Self {
        let mut properties_map = HashMap::new();
        properties_map.insert(Property::PayloadFormatIndicator(0).to_string(), None);
        properties_map.insert(Property::MessageExpiryInterval(0).to_string(), None);
        properties_map.insert(Property::TopicAlias(0).to_string(), None);
        properties_map.insert(Property::ResponseTopic(String::from("")).to_string(), None);
        properties_map.insert(Property::CorrelationData(Bytes::new()).to_string(), None);
        properties_map.insert(Property::UserProperty(String::from("")).to_string(), None);
        properties_map.insert(
            Property::SubscriptionIdentifier(VariableByteInteger::new()).to_string(),
            None,
        );
        properties_map.insert(Property::ContentType(String::from("")).to_string(), None);
        let properties = Properties {
            properties: properties_map,
        };
        Self {
            topic_name: String::from(""),
            packet_identifier: 0,
            properties,
        }
    }
    pub fn from(
        topic_name: String,
        packet_identifier: u16,
        _properties: Vec<Option<Property>>,
    ) -> Self {
        let mut publish_variable_header = Self::new();
        publish_variable_header.set_properties(_properties);
        publish_variable_header.topic_name = topic_name;
        publish_variable_header.packet_identifier = packet_identifier;
        publish_variable_header
    }
    pub fn set_properties(&mut self, _properties: Vec<Option<Property>>) {
        self.properties.set_properties_vec(_properties);
    }
    pub fn get_properties(&self) -> Vec<Option<Property>> {
        self.properties.properties.values().cloned().collect()
    }
}
impl Default for PublishVariableHeader {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Default)]
pub struct PubAckControlPacket {
    pub variable_header: PubAckVariableHeader,
}
#[derive(Debug)]
pub struct PubAckVariableHeader {
    pub packet_identifier: u16,
    pub reason_code: PubAckReasonCode,
    properties: Properties,
}
impl PubAckVariableHeader {
    pub fn new() -> Self {
        let mut properties_map = HashMap::new();
        properties_map.insert(Property::ReasonString(String::from("")).to_string(), None);
        properties_map.insert(Property::UserProperty(String::from("")).to_string(), None);
        let properties = Properties {
            properties: properties_map,
        };
        Self {
            packet_identifier: 0,
            reason_code: PubAckReasonCode::default(),
            properties,
        }
    }
    pub fn from(
        packet_identifier: u16,
        reason_code: PubAckReasonCode,
        _properties: Vec<Option<Property>>,
    ) -> Self {
        let mut pub_ack_variable_header = Self::new();
        pub_ack_variable_header.set_properties(_properties);
        pub_ack_variable_header.packet_identifier = packet_identifier;
        pub_ack_variable_header.reason_code = reason_code;
        pub_ack_variable_header
    }
    pub fn set_properties(&mut self, _properties: Vec<Option<Property>>) {
        self.properties.set_properties_vec(_properties);
    }
    pub fn get_properties(&self) -> Vec<Option<Property>> {
        self.properties.properties.values().cloned().collect()
    }
}
impl Default for PubAckVariableHeader {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Default)]
pub struct PubRecControlPacket {
    pub variable_header: PubRecVariableHeader,
}
#[derive(Debug)]
pub struct PubRecVariableHeader {
    pub packet_identifier: u16,
    pub reason_code: PubRecReasonCode,
    properties: Properties,
}
impl PubRecVariableHeader {
    pub fn new() -> Self {
        let mut properties_map = HashMap::new();
        properties_map.insert(Property::ReasonString(String::from("")).to_string(), None);
        properties_map.insert(Property::UserProperty(String::from("")).to_string(), None);
        let properties = Properties {
            properties: properties_map,
        };
        Self {
            packet_identifier: 0,
            reason_code: PubRecReasonCode::default(),
            properties,
        }
    }
    pub fn from(
        packet_identifier: u16,
        reason_code: PubRecReasonCode,
        _properties: Vec<Option<Property>>,
    ) -> Self {
        let mut pub_ack_variable_header = Self::new();
        pub_ack_variable_header.set_properties(_properties);
        pub_ack_variable_header.packet_identifier = packet_identifier;
        pub_ack_variable_header.reason_code = reason_code;
        pub_ack_variable_header
    }
    pub fn set_properties(&mut self, _properties: Vec<Option<Property>>) {
        self.properties.set_properties_vec(_properties);
    }
    pub fn get_properties(&self) -> Vec<Option<Property>> {
        self.properties.properties.values().cloned().collect()
    }
}
impl Default for PubRecVariableHeader {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Default)]
pub struct PubRelControlPacket {
    pub variable_header: PubRelVariableHeader,
}
#[derive(Debug)]
pub struct PubRelVariableHeader {
    pub packet_identifier: u16,
    pub reason_code: PubRelReasonCode,
    properties: Properties,
}
impl PubRelVariableHeader {
    pub fn new() -> Self {
        let mut properties_map = HashMap::new();
        properties_map.insert(Property::ReasonString(String::from("")).to_string(), None);
        properties_map.insert(Property::UserProperty(String::from("")).to_string(), None);
        let properties = Properties {
            properties: properties_map,
        };
        Self {
            packet_identifier: 0,
            reason_code: PubRelReasonCode::default(),
            properties,
        }
    }
    pub fn from(
        packet_identifier: u16,
        reason_code: PubRelReasonCode,
        _properties: Vec<Option<Property>>,
    ) -> Self {
        let mut pub_ack_variable_header = Self::new();
        pub_ack_variable_header.set_properties(_properties);
        pub_ack_variable_header.packet_identifier = packet_identifier;
        pub_ack_variable_header.reason_code = reason_code;
        pub_ack_variable_header
    }
    pub fn set_properties(&mut self, _properties: Vec<Option<Property>>) {
        self.properties.set_properties_vec(_properties);
    }
    pub fn get_properties(&self) -> Vec<Option<Property>> {
        self.properties.properties.values().cloned().collect()
    }
}
impl Default for PubRelVariableHeader {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Default)]
pub struct PubCompControlPacket {
    pub variable_header: PubCompVariableHeader,
}
#[derive(Debug)]
pub struct PubCompVariableHeader {
    pub packet_identifier: u16,
    pub reason_code: PubCompReasonCode,
    properties: Properties,
}
impl PubCompVariableHeader {
    pub fn new() -> Self {
        let mut properties_map = HashMap::new();
        properties_map.insert(Property::ReasonString(String::from("")).to_string(), None);
        properties_map.insert(Property::UserProperty(String::from("")).to_string(), None);
        let properties = Properties {
            properties: properties_map,
        };
        Self {
            packet_identifier: 0,
            reason_code: PubCompReasonCode::default(),
            properties,
        }
    }
    pub fn from(
        packet_identifier: u16,
        reason_code: PubCompReasonCode,
        _properties: Vec<Option<Property>>,
    ) -> Self {
        let mut pub_ack_variable_header = Self::new();
        pub_ack_variable_header.set_properties(_properties);
        pub_ack_variable_header.packet_identifier = packet_identifier;
        pub_ack_variable_header.reason_code = reason_code;
        pub_ack_variable_header
    }
    pub fn set_properties(&mut self, _properties: Vec<Option<Property>>) {
        self.properties.set_properties_vec(_properties);
    }
    pub fn get_properties(&self) -> Vec<Option<Property>> {
        self.properties.properties.values().cloned().collect()
    }
}
impl Default for PubCompVariableHeader {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Default)]
pub struct SubscribeControlPacket {
    pub variable_header: SubscribeVariableHeader,
}
#[repr(u8)]
#[derive(Display, Debug, Clone, FromPrimitive)]
pub enum RetainHandlingOption {
    SendRetainedMessageSubTime = 0,
    SendRetainedMessageSubNotExist = 1,
    NotSendRetainedMessage = 2,
}
impl Default for RetainHandlingOption {
    fn default() -> Self {
        Self::SendRetainedMessageSubTime
    }
}
#[derive(Debug, Default)]
pub struct SubscriptionOptions {
    pub maximum_qos: Qos,
    pub no_local_option: bool,
    pub retain_as_published: bool,
    pub retain_handling: RetainHandlingOption,
    pub reserved: u8,
}
impl SubscriptionOptions {
    pub fn new(byte: u8) -> SubscriptionOptions {
        SubscriptionOptions {
            maximum_qos: Qos::from_u8(byte & 0b0000_0011).unwrap(),
            no_local_option: (byte & 0b0000_0100) != 0,
            retain_as_published: (byte & 0b0000_1000) != 0,
            retain_handling: RetainHandlingOption::from_u8((byte & 0b0011_0000) >> 4).unwrap(),
            reserved: (byte & 0b1100_0000) >> 6,
        }
    }
}
#[derive(Debug, Default)]
pub struct SubscribePayload {
    pub subscription_options: SubscriptionOptions,
    pub topic_filter: String,
}
#[derive(Debug)]
pub struct SubscribeVariableHeader {
    pub packet_identifier: u16,
    pub subscribe_payload: Vec<SubscribePayload>,
    properties: Properties,
}
impl SubscribeVariableHeader {
    pub fn new() -> Self {
        let mut properties_map = HashMap::new();
        properties_map.insert(
            Property::SubscriptionIdentifier(VariableByteInteger::new()).to_string(),
            None,
        );
        properties_map.insert(Property::UserProperty(String::from("")).to_string(), None);
        let properties = Properties {
            properties: properties_map,
        };
        Self {
            packet_identifier: 0,
            subscribe_payload: Vec::new(),
            properties,
        }
    }
    pub fn from(
        packet_identifier: u16,
        subscribe_payload: Vec<SubscribePayload>,
        _properties: Vec<Option<Property>>,
    ) -> Self {
        let mut pub_ack_variable_header = Self::new();
        pub_ack_variable_header.set_properties(_properties);
        pub_ack_variable_header.packet_identifier = packet_identifier;
        pub_ack_variable_header.subscribe_payload = subscribe_payload;
        pub_ack_variable_header
    }
    pub fn set_properties(&mut self, _properties: Vec<Option<Property>>) {
        self.properties.set_properties_vec(_properties);
    }
    pub fn get_properties(&self) -> Vec<Option<Property>> {
        self.properties.properties.values().cloned().collect()
    }
}
impl Default for SubscribeVariableHeader {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Default)]
pub struct SubAckControlPacket {
    pub variable_header: SubAckVariableHeader,
}
#[derive(Debug, Default)]
pub struct SubAckPayload {
    pub sub_ack_reason_codes : Vec<SubAckReasonCode>,
}
#[derive(Debug)]
pub struct SubAckVariableHeader {
    pub packet_identifier: u16,
    pub sub_ack_payload: SubAckPayload,
    properties: Properties,
}
impl SubAckVariableHeader {
    pub fn new() -> Self {
        let mut properties_map = HashMap::new();
        properties_map.insert(
            Property::SubscriptionIdentifier(VariableByteInteger::new()).to_string(),
            None,
        );
        properties_map.insert(Property::UserProperty(String::from("")).to_string(), None);
        let properties = Properties {
            properties: properties_map,
        };
        Self {
            packet_identifier: 0,
            sub_ack_payload: Default::default(),
            properties,
        }
    }
    pub fn from(
        packet_identifier: u16,
        sub_ack_payload: SubAckPayload,
        _properties: Vec<Option<Property>>,
    ) -> Self {
        let mut pub_ack_variable_header = Self::new();
        pub_ack_variable_header.set_properties(_properties);
        pub_ack_variable_header.packet_identifier = packet_identifier;
        pub_ack_variable_header.sub_ack_payload = sub_ack_payload;
        pub_ack_variable_header
    }
    pub fn set_properties(&mut self, _properties: Vec<Option<Property>>) {
        self.properties.set_properties_vec(_properties);
    }
    pub fn get_properties(&self) -> Vec<Option<Property>> {
        self.properties.properties.values().cloned().collect()
    }
}
impl Default for SubAckVariableHeader {
    fn default() -> Self {
        Self::new()
    }
}
