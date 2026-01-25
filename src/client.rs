use crate::{
    broker::{BrokerMessage, PublishMessage},
    message_state::MessageStateTracker,
    packet_id::PacketIdManager,
    protocol::{
        definitions::*,
        frame::{ControlPacket, Error, Frame},
        packet::*,
    },
};
use bytes::{Buf, BytesMut};
use std::io::Cursor;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};

#[derive(Debug)]
pub struct Client {
    read: ReadHalf<TcpStream>,
    write: WriteHalf<TcpStream>,
    buffer: BytesMut,
    id: String,
    broker_sender: Sender<BrokerMessage>,
    #[allow(dead_code)]
    message_receiver: Receiver<PublishMessage>,
    // QoS management
    packet_id_manager: PacketIdManager,
    message_state_tracker: MessageStateTracker,
}

impl Client {
    pub fn new(stream: TcpStream, broker_sender: Sender<BrokerMessage>) -> Client {
        let (rd, wr) = tokio::io::split(stream);
        let (_msg_sender, msg_receiver) = mpsc::channel(100);
        
        Client {
            read: rd,
            write: wr,
            // Allocate the buffer with 4kb of capacity.
            buffer: BytesMut::with_capacity(4096),
            id: String::from(""),
            broker_sender,
            message_receiver: msg_receiver,
            packet_id_manager: PacketIdManager::new(),
            message_state_tracker: MessageStateTracker::new(),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Frame, Error> {
        loop {
            // There is not enough buffered data to read a frame.
            // Attempt to read more data from the socket.
            //
            // On success, the number of bytes is returned. `0`
            // indicates "end of stream".
            if 0 == self.read.read_buf(&mut self.buffer).await? {
                // The remote closed the connection. For this to be
                // a clean shutdown, there should be no data in the
                // read buffer. If there is, this means that the
                // peer closed the socket while sending a frame.
                if self.buffer.is_empty() {
                    return Err(Error::Other("connection ended by peer".into()));
                } else {
                    return Err(Error::Other("connection reset by peer".into()));
                }
            }
            // Attempt to parse a frame from the buffered data. If
            // enough data has been buffered, the frame is
            // returned.
            if let Some(frame) = self.deserialize_frame()? {
                return Ok(frame);
            }
        }
    }

    fn deserialize_frame(&mut self) -> Result<Option<Frame>, Error> {
        // Create the `T: Buf` type.
        let mut buf = Cursor::new(&self.buffer[..]);

        // Check whether a full frame is available
        match Frame::deserialize(&mut buf) {
            Ok(frame) => {
                // Get the byte length of the frame
                let len = buf.position() as usize;

                // Discard the frame from the buffer
                self.buffer.advance(len);

                // Return the frame to the caller.
                Ok(Some(frame))
            }
            // Not enough data has been buffered
            Err(Error::Incomplete(_)) => Ok(None),
            // An error was encountered
            Err(e) => Err(e),
        }
    }

    pub async fn write_value(&mut self, src: &mut BytesMut) -> std::io::Result<()> {
        println!("write_value: {:?}", src);
        self.write.write_buf(src).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub async fn run(mut self) {
        let (msg_sender, mut msg_receiver) = mpsc::channel(100);
        
        loop {
            tokio::select! {
                // Handle incoming frames from client
                frame_result = self.read_frame() => {
                    match frame_result {
                        Ok(msg) => {
                            if !self.handle_frame(msg, &msg_sender).await {
                                break;
                            }
                        }
                        Err(Error::Incomplete(_)) => {
                            println!("Not enough data has been buffered");
                        }
                        Err(Error::Other(err)) => {
                            println!("Error: {}", err);
                            break;
                        }
                    }
                }
                // Handle outgoing publish messages from broker
                Some(publish_msg) = msg_receiver.recv() => {
                    if let Err(e) = self.send_publish_to_client(publish_msg).await {
                        println!("Failed to send publish to client: {}", e);
                        break;
                    }
                }
            }
        }
        
        // Notify broker of disconnect
        let _ = self.broker_sender.send(BrokerMessage::Disconnect {
            client_id: self.id.clone(),
        }).await;
        
        println!("Client {} disconnected", self.id);
    }
    
    async fn handle_frame(&mut self, msg: Frame, msg_sender: &Sender<PublishMessage>) -> bool {
        println!("connection_packet: {:?}", msg.control_packet);
        
        match msg.control_packet {
            ControlPacket::Connect(control_packet) => {
                self.id = control_packet.payload.client_identifier.clone();
                
                // Generate client ID if empty (MQTT 3.1.1 allows empty client_id for clean session)
                if self.id.is_empty() {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_micros();
                    self.id = format!("auto-{}", timestamp);
                }
                
                println!("Client connected with ID: {}", self.id);
                
                // Register with broker
                let _ = self.broker_sender.send(BrokerMessage::Connect {
                    client_id: self.id.clone(),
                    sender: msg_sender.clone(),
                }).await;
                
                // Send CONNACK
                self.write_value(&mut Frame::serialize(Frame::new(ControlPacketType::CONNACK)).unwrap())
                    .await
                    .unwrap();
                true
            }
            
            ControlPacket::Publish(control_packet) => {
                let qos = msg.fix_header.flags.1;
                let topic = control_packet.variable_header.topic_name.clone();
                let payload = control_packet.payload.data.to_vec();
                let retain = msg.fix_header.flags.0 == 1;
                
                println!("Received PUBLISH: topic='{}', qos={}, retain={}, payload_len={}", 
                        topic, qos, retain, payload.len());
                
                // Send to broker for distribution
                let _ = self.broker_sender.send(BrokerMessage::Publish(PublishMessage {
                    topic: topic.clone(),
                    payload,
                    qos,
                    retain,
                })).await;
                
                // Send acknowledgment based on QoS
                match qos {
                    1 => {
                        // QoS 1: Send PUBACK
                        if let Some(packet_id) = control_packet.variable_header.packet_identifier {
                            let pub_ack = Frame {
                                fix_header: FixHeader::new(ControlPacketType::PUBACK, Flags(0, 0, 0, 0)),
                                control_packet: ControlPacket::PubAck(PubAckControlPacket {
                                    variable_header: PubAckVariableHeader::from(
                                        packet_id,
                                        PubAckReasonCode::Success,
                                        Vec::new(),
                                    ),
                                }),
                            };
                            self.write_value(&mut Frame::serialize(pub_ack).unwrap()).await.unwrap();
                        }
                    }
                    2 => {
                        // QoS 2: Send PUBREC, track state
                        if let Some(packet_id) = control_packet.variable_header.packet_identifier {
                            self.message_state_tracker.mark_received_publish_qos2(packet_id, topic);
                            
                            let pub_rec = Frame {
                                fix_header: FixHeader::new(ControlPacketType::PUBREC, Flags(0, 0, 0, 0)),
                                control_packet: ControlPacket::PubRec(PubRecControlPacket {
                                    variable_header: PubRecVariableHeader::from(
                                        packet_id,
                                        PubRecReasonCode::Success,
                                        Vec::new(),
                                    ),
                                }),
                            };
                            self.write_value(&mut Frame::serialize(pub_rec).unwrap()).await.unwrap();
                        }
                    }
                    _ => {} // QoS 0 - no acknowledgment needed
                }
                true
            }
            
            ControlPacket::PubAck(control_packet) => {
                // Received PUBACK for our outgoing QoS 1 message
                let packet_id = control_packet.variable_header.packet_identifier;
                println!("Received PUBACK for packet_id={}", packet_id);
                
                if self.message_state_tracker.handle_puback(packet_id) {
                    self.packet_id_manager.release(packet_id);
                    println!("QoS 1 message {} acknowledged", packet_id);
                } else {
                    println!("Warning: Received PUBACK for unknown packet_id={}", packet_id);
                }
                true
            }
            
            ControlPacket::PubRec(control_packet) => {
                // Received PUBREC for our outgoing QoS 2 message - send PUBREL
                let packet_id = control_packet.variable_header.packet_identifier;
                println!("Received PUBREC for packet_id={}", packet_id);
                
                if self.message_state_tracker.handle_pubrec(packet_id) {
                    // Send PUBREL
                    let pub_rel = Frame {
                        fix_header: FixHeader::new(ControlPacketType::PUBREL, Flags(0, 1, 0, 0)), // QoS 1 for PUBREL
                        control_packet: ControlPacket::PubRel(PubRelControlPacket {
                            variable_header: PubRelVariableHeader::from(
                                packet_id,
                                PubRelReasonCode::Success,
                                Vec::new(),
                            ),
                        }),
                    };
                    self.write_value(&mut Frame::serialize(pub_rel).unwrap()).await.unwrap();
                    println!("Sent PUBREL for packet_id={}", packet_id);
                } else {
                    println!("Warning: Received PUBREC for unknown packet_id={}", packet_id);
                }
                true
            }
            
            ControlPacket::PubRel(control_packet) => {
                // Received PUBREL for incoming QoS 2 message - send PUBCOMP
                let packet_id = control_packet.variable_header.packet_identifier;
                println!("Received PUBREL for packet_id={}", packet_id);
                
                self.message_state_tracker.handle_pubrel(packet_id);
                
                let pub_comp = Frame {
                    fix_header: FixHeader::new(ControlPacketType::PUBCOMP, Flags(0, 0, 0, 0)),
                    control_packet: ControlPacket::PubComp(PubCompControlPacket {
                        variable_header: PubCompVariableHeader::from(
                            packet_id,
                            PubCompReasonCode::Success,
                            Vec::new(),
                        ),
                    }),
                };
                self.write_value(&mut Frame::serialize(pub_comp).unwrap()).await.unwrap();
                println!("Sent PUBCOMP for packet_id={}", packet_id);
                true
            }
            
            ControlPacket::PubComp(control_packet) => {
                // Received PUBCOMP - QoS 2 flow complete
                let packet_id = control_packet.variable_header.packet_identifier;
                println!("Received PUBCOMP for packet_id={}", packet_id);
                
                if self.message_state_tracker.handle_pubcomp(packet_id) {
                    self.packet_id_manager.release(packet_id);
                    println!("QoS 2 message {} completed", packet_id);
                } else {
                    println!("Warning: Received PUBCOMP for unknown packet_id={}", packet_id);
                }
                true
            }
            
            ControlPacket::Subscribe(control_packet) => {
                let topics: Vec<String> = control_packet.variable_header.subscribe_payload
                    .iter()
                    .map(|sub| sub.topic_filter.clone())
                    .collect();
                    
                println!("Client {} subscribing to topics: {:?}", self.id, topics);
                
                // Notify broker
                let _ = self.broker_sender.send(BrokerMessage::Subscribe {
                    client_id: self.id.clone(),
                    topics,
                }).await;
                
                // Send SUBACK with granted QoS levels matching the requested ones
                let mut sub_ack_payload = SubAckPayload::default();
                for sub in &control_packet.variable_header.subscribe_payload {
                    // Grant the requested QoS level
                    let granted_qos = match sub.subscription_options.maximum_qos {
                        Qos::AtMostOnce => SubAckReasonCode::GrantedQoS0,
                        Qos::AtleastOnce => SubAckReasonCode::GrantedQoS1,
                        Qos::ExactlyOnce => SubAckReasonCode::GrantedQoS2,
                    };
                    sub_ack_payload.sub_ack_reason_codes.push(granted_qos);
                }
                
                let sub_ack = Frame {
                    fix_header: FixHeader::new(ControlPacketType::SUBACK, Flags(0, 0, 0, 0)),
                    control_packet: ControlPacket::SubAck(SubAckControlPacket {
                        variable_header: SubAckVariableHeader::from(
                            control_packet.variable_header.packet_identifier,
                            sub_ack_payload,
                            Vec::new(),
                        ),
                    }),
                };
                self.write_value(&mut Frame::serialize(sub_ack).unwrap()).await.unwrap();
                true
            }
            
            ControlPacket::PingReq => {
                self.write_value(&mut Frame::serialize(Frame::new(ControlPacketType::PINGRESP)).unwrap())
                    .await
                    .unwrap();
                true
            }
            
            ControlPacket::Disconnect(_) => {
                println!("Client {} requested disconnect", self.id);
                false
            }
            
            _ => {
                println!("Unhandled packet type");
                true
            }
        }
    }
    
    async fn send_publish_to_client(&mut self, msg: PublishMessage) -> Result<(), Box<dyn std::error::Error>> {
        println!("Sending PUBLISH to client {}: topic={}, qos={}", self.id, msg.topic, msg.qos);
        
        let mut publish_packet = PublishControlPacket {
            variable_header: PublishVariableHeader::new(),
            payload: PublishPayload {
                data: bytes::Bytes::from(msg.payload.clone()),
            },
        };
        
        // Update topic name
        publish_packet.variable_header.topic_name = msg.topic.clone();
        
        // Allocate packet ID for QoS 1 and 2
        let packet_id = if msg.qos > 0 {
            match self.packet_id_manager.allocate() {
                Some(id) => {
                    // Track the message state
                    match msg.qos {
                        1 => {
                            self.message_state_tracker.track_qos1(
                                id,
                                msg.topic.clone(),
                                msg.payload.clone(),
                                msg.retain,
                            );
                        }
                        2 => {
                            self.message_state_tracker.track_qos2(
                                id,
                                msg.topic.clone(),
                                msg.payload.clone(),
                                msg.retain,
                            );
                        }
                        _ => {}
                    }
                    Some(id)
                }
                None => {
                    println!("Warning: No packet IDs available, cannot send QoS {} message", msg.qos);
                    return Ok(());
                }
            }
        } else {
            None
        };
        
        publish_packet.variable_header.packet_identifier = packet_id;
        
        let frame = Frame {
            fix_header: FixHeader::new(ControlPacketType::PUBLISH, Flags(
                if msg.retain { 1 } else { 0 },
                msg.qos,
                0,
                0,
            )),
            control_packet: ControlPacket::Publish(publish_packet),
        };
        
        self.write_value(&mut Frame::serialize(frame)?).await?;
        
        if let Some(id) = packet_id {
            println!("Sent QoS {} PUBLISH with packet_id={}", msg.qos, id);
        }
        
        Ok(())
    }
}
