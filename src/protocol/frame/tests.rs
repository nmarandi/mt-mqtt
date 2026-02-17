#[cfg(test)]
mod tests {
    use super::super::*;
    use bytes::BytesMut;
    use std::io::Cursor;

    #[test]
    fn test_deserialize_mqtt311_connect() {
        // MQTT 3.1.1 CONNECT packet
        let packet = vec![
            0x10, // CONNECT packet type
            0x10, // Remaining length = 16
            0x00, 0x04, b'M', b'Q', b'T', b'T', // Protocol name
            0x04, // Protocol level (3.1.1)
            0x02, // Connect flags (clean session)
            0x00, 0x3c, // Keep alive (60s)
            0x00, 0x04, b't', b'e', b's', b't', // Client ID "test"
        ];

        let mut cursor = Cursor::new(&packet[..]);
        let result = Frame::deserialize(&mut cursor);
        
        assert!(result.is_ok(), "Failed to deserialize CONNECT packet");
        
        let frame = result.unwrap();
        assert_eq!(frame.fix_header.control_packet_type, ControlPacketType::CONNECT);
        
        if let ControlPacket::Connect(connect) = frame.control_packet {
            assert_eq!(connect.payload.client_identifier, "test");
            assert_eq!(connect.variable_header.protocol_version, 4);
            assert_eq!(connect.variable_header.keep_alive, 60);
        } else {
            panic!("Expected Connect packet");
        }
    }

    #[test]
    fn test_deserialize_incomplete_packet() {
        // Incomplete packet (only header, no payload)
        let packet = vec![0x10, 0x10, 0x00, 0x04];

        let mut cursor = Cursor::new(&packet[..]);
        let result = Frame::deserialize(&mut cursor);
        
        assert!(result.is_err(), "Should fail on incomplete packet");
        
        if let Err(Error::Incomplete(_)) = result {
            // Expected
        } else {
            panic!("Expected Incomplete error");
        }
    }

    #[test]
    fn test_serialize_connack() {
        let mut frame = Frame::new(ControlPacketType::CONNACK);
        
        if let ControlPacket::ConnAck(ref mut connack) = frame.control_packet {
            connack.variable_header.reason_code = ConnAckReasonCode::Success;
        }

        let result = Frame::serialize(frame);
        assert!(result.is_ok(), "Failed to serialize CONNACK");
        
        let bytes = result.unwrap();
        assert!(bytes.len() > 0, "Serialized packet should not be empty");
        
        // First byte should be CONNACK packet type
        assert_eq!(bytes[0] & 0xF0, 0x20);
    }

    #[test]
    fn test_deserialize_publish_qos0() {
        // PUBLISH packet QoS 0 with MQTT 3.1.1 format (no properties field)
        // Note: The decoder handles MQTT 3.1.1 without properties field
        let packet = vec![
            0x30, // PUBLISH packet type, QoS 0
            0x0E, // Remaining length = 14 (2 + 10 + 2)
            0x00, 0x0A, // Topic length = 10
            b't', b'e', b's', b't', b'/', b't', b'o', b'p', b'i', b'c', // Topic "test/topic"
            b'h', b'i', // Payload "hi"
        ];

        let mut cursor = Cursor::new(&packet[..]);
        let result = Frame::deserialize(&mut cursor);
        
        assert!(result.is_ok(), "Failed to deserialize PUBLISH packet: {:?}", result);
        
        let frame = result.unwrap();
        assert_eq!(frame.fix_header.control_packet_type, ControlPacketType::PUBLISH);
        
        if let ControlPacket::Publish(publish) = frame.control_packet {
            assert_eq!(publish.variable_header.topic_name, "test/topic");
            assert_eq!(&publish.payload.data[..], b"hi");
        } else {
            panic!("Expected Publish packet");
        }
    }

    #[test]
    fn test_deserialize_subscribe() {
        // SUBSCRIBE packet for MQTT 3.1.1 (no properties field)
        let packet = vec![
            0x82, // SUBSCRIBE packet type
            0x0F, // Remaining length = 15 (2 + 2 + 10 + 1)
            0x00, 0x01, // Packet ID = 1
            0x00, 0x0A, b't', b'e', b's', b't', b'/', b't', b'o', b'p', b'i', b'c', // Topic "test/topic"
            0x00, // QoS 0
        ];

        let mut cursor = Cursor::new(&packet[..]);
        let result = Frame::deserialize(&mut cursor);
        
        assert!(result.is_ok(), "Failed to deserialize SUBSCRIBE packet: {:?}", result.err());
        
        let frame = result.unwrap();
        assert_eq!(frame.fix_header.control_packet_type, ControlPacketType::SUBSCRIBE);
    }

    #[test]
    fn test_variable_byte_integer_encoding() {
        // Test encoding of variable byte integers
        assert_eq!(VariableByteInteger::encode_u32(0), vec![0x00]);
        assert_eq!(VariableByteInteger::encode_u32(127), vec![0x7F]);
        assert_eq!(VariableByteInteger::encode_u32(128), vec![0x80, 0x01]);
        assert_eq!(VariableByteInteger::encode_u32(16_383), vec![0xFF, 0x7F]);
        assert_eq!(VariableByteInteger::encode_u32(16_384), vec![0x80, 0x80, 0x01]);
    }

    #[test]
    fn test_variable_byte_integer_decoding() {
        let test_cases = vec![
            (vec![0x00], 0),
            (vec![0x7F], 127),
            (vec![0x80, 0x01], 128),
            (vec![0xFF, 0x7F], 16_383),
            (vec![0x80, 0x80, 0x01], 16_384),
        ];

        for (bytes, expected) in test_cases {
            let mut cursor = Cursor::new(&bytes[..]);
            let result = VariableByteInteger::decode(&mut cursor);
            assert_eq!(result, expected, "Failed to decode {:?}", bytes);
        }
    }

    #[test]
    fn test_string_encoding_decoding() {
        let test_str = "test/topic";
        
        // Encode
        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(&(test_str.len() as u16).to_be_bytes());
        bytes.extend_from_slice(test_str.as_bytes());
        
        // Decode
        let mut cursor = Cursor::new(&bytes[..]);
        let result = super::super::decoder::decode_string(&mut cursor);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_str);
    }

    #[test]
    fn test_decode_string_incomplete() {
        // String length says 10 bytes, but only 5 bytes available
        let bytes = vec![0x00, 0x0A, b't', b'e', b's', b't', b'!'];
        
        let mut cursor = Cursor::new(&bytes[..]);
        let result = super::super::decoder::decode_string(&mut cursor);
        
        assert!(result.is_err());
        if let Err(Error::Incomplete(_)) = result {
            // Expected
        } else {
            panic!("Expected Incomplete error");
        }
    }
}
