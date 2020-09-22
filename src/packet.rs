use crate::definitions::{Property, ReasonCode};
use bytes::Bytes;

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
    pub properties: Vec<Property>,
}
#[derive(Debug, Default)]
pub struct ConnectPayload {
    pub client_identifier: String,
    pub will_properties: Option<Vec<Property>>,
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
    pub reason_code: ReasonCode,
    pub properties: Vec<Property>,
}
#[derive(Debug, Default)]
pub struct ConnAckControlPacket {
    pub variable_header: ConnAckVariableHeader,
}
