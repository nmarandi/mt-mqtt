use crate::{definitions::*, frame::*};
use bytes::{Buf, BytesMut};
use std::io::Cursor;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

pub struct Client {
    read: ReadHalf<TcpStream>,
    write: WriteHalf<TcpStream>,
    buffer: BytesMut,
    id: String,
}

impl Client {
    pub fn new(stream: TcpStream) -> Client {
        let (rd, wr) = tokio::io::split(stream);
        Client {
            read: rd,
            write: wr,
            // Allocate the buffer with 4kb of capacity.
            buffer: BytesMut::with_capacity(4096),
            id: String::from(""),
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

    pub async fn run(mut self) {
        loop {
            match self.read_frame().await {
                Ok(msg) => {
                    println!("connection_packet: {:?}", msg.control_packet);
                    match msg.control_packet {
                        ControlPacket::Connect(control_packet) => {
                            self.id = control_packet.payload.client_identifier;
                            self.write_value(&mut Frame::serialize(Frame::new(ControlPacketType::CONNACK)).unwrap())
                                .await
                                .unwrap();
                        }
                        ControlPacket::Publish(control_packet) => match msg.fix_header.flags.1 {
                            1 => {
                                let pub_ack_control_packet = PubAckControlPacket {
                                    variable_header: PubAckVariableHeader::from(
                                        control_packet.variable_header.packet_identifier.unwrap(),
                                        PubAckReasonCode::Success,
                                        Vec::new(),
                                    ),
                                };
                                let pub_ack = Frame {
                                    fix_header: FixHeader::new(ControlPacketType::PUBACK, Flags(0, 0, 0, 0)),
                                    control_packet: ControlPacket::PubAck(pub_ack_control_packet),
                                };
                                self.write_value(&mut Frame::serialize(pub_ack).unwrap()).await.unwrap()
                            }
                            2 => {
                                let pub_rec_control_packet = PubRecControlPacket {
                                    variable_header: PubRecVariableHeader::from(
                                        control_packet.variable_header.packet_identifier.unwrap(),
                                        PubRecReasonCode::Success,
                                        Vec::new(),
                                    ),
                                };
                                let pub_ack = Frame {
                                    fix_header: FixHeader::new(ControlPacketType::PUBREC, Flags(0, 0, 0, 0)),
                                    control_packet: ControlPacket::PubRec(pub_rec_control_packet),
                                };
                                self.write_value(&mut Frame::serialize(pub_ack).unwrap()).await.unwrap()
                            }
                            _ => (),
                        },
                        ControlPacket::PubRel(control_packet) => {
                            let pub_comp_control_packet = PubCompControlPacket {
                                variable_header: PubCompVariableHeader::from(
                                    control_packet.variable_header.packet_identifier,
                                    PubCompReasonCode::Success,
                                    Vec::new(),
                                ),
                            };
                            let pub_ack = Frame {
                                fix_header: FixHeader::new(ControlPacketType::PUBCOMP, Flags(0, 0, 0, 0)),
                                control_packet: ControlPacket::PubComp(pub_comp_control_packet),
                            };
                            self.write_value(&mut Frame::serialize(pub_ack).unwrap()).await.unwrap()
                        }
                        ControlPacket::Subscribe(control_packet) => {
                            let mut sub_ack_payload = SubAckPayload::default();
                            for iter in control_packet.variable_header.subscribe_payload {
                                sub_ack_payload.sub_ack_reason_codes.push(SubAckReasonCode::GrantedQoS0);
                            }
                            let sub_ack_control_packet = SubAckControlPacket {
                                variable_header: SubAckVariableHeader::from(
                                    control_packet.variable_header.packet_identifier,
                                    sub_ack_payload,
                                    Vec::new(),
                                ),
                            };
                            let pub_ack = Frame {
                                fix_header: FixHeader::new(ControlPacketType::PUBCOMP, Flags(0, 0, 0, 0)),
                                control_packet: ControlPacket::SubAck(sub_ack_control_packet),
                            };
                            self.write_value(&mut Frame::serialize(pub_ack).unwrap()).await.unwrap()
                        }
                        ControlPacket::PingReq => self
                            .write_value(&mut Frame::serialize(Frame::new(ControlPacketType::PINGRESP)).unwrap())
                            .await
                            .unwrap(),
                        _ => break,
                    }
                }
                // Not enough data has been buffered
                Err(Error::Incomplete(_)) => println!("Not enough data has been buffered"),
                // An error was encountered
                Err(Error::Other(err)) => {
                    println!("{}", err);
                    break;
                }
            }
        }
    }
}
