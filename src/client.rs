use bytes::{Buf,BytesMut};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use std::io::Cursor;
use crate::frame::*;

pub struct Client {
    stream: TcpStream,
    buffer: BytesMut,
}

impl Client {
    pub fn new(stream: TcpStream) -> Client {
        Client {
            stream,
            // Allocate the buffer with 4kb of capacity.
            buffer: BytesMut::with_capacity(4096),
        }
    }
    pub async fn read_frame(&mut self) -> Result<Frame, Error> {
        loop {
            // Attempt to parse a frame from the buffered data. If
            // enough data has been buffered, the frame is
            // returned.
            if let Some(frame) = self.parse_frame()? {
                return Ok(frame);
            }

            // There is not enough buffered data to read a frame.
            // Attempt to read more data from the socket.
            //
            // On success, the number of bytes is returned. `0`
            // indicates "end of stream".
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
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
        }
    }
    fn parse_frame(&mut self) -> Result<Option<Frame>,Error> {
        // Create the `T: Buf` type.
        let mut buf = Cursor::new(&self.buffer[..]);

        // Check whether a full frame is available
        match Frame::parse(&mut buf) {
            Ok(frame) => {
                // Get the byte length of the frame
                let len = buf.position() as usize;

                // Discard the frame from the buffer
                self.buffer.advance(len);

                // Return the frame to the caller.
                Ok(Some(frame))
            }
            // Not enough data has been buffered
            Err(Incomplete) => Ok(None),
            // An error was encountered
            Err(e) => Err(e.into()),
        }
    }
    pub async fn run(mut self) {
        while let Ok(msg) = self.read_frame().await {
            match msg {
                Frame::Connect(header,connection_packet) => {
                    
                },
            }
        }
    }
}
