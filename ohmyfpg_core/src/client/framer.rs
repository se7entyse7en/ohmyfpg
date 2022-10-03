use crate::messages::{RawBackendMessage, SerializeMessage};
use std::fmt;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::task;

const IO_READ_BUFFER_SIZE: usize = 8 * 1024;
const FRAMER_BUFFER_SIZE: usize = 100;
const FRAME_HEADER_SIZE: usize = 5;

pub struct Framer {
    reader: ReadFramer,
    writer: WriteFramer,
}

impl Framer {
    pub fn new(stream: TcpStream) -> Self {
        let (read_half, write_half) = stream.into_split();

        Framer {
            reader: ReadFramer::new(read_half),
            writer: WriteFramer::new(write_half),
        }
    }

    pub async fn write_frame<T>(&mut self, msg: T) -> io::Result<()>
    where
        T: SerializeMessage + fmt::Debug,
    {
        self.writer.write_frame(msg.serialize()).await
    }

    pub async fn read_frame(&mut self) -> Option<RawBackendMessage> {
        self.reader
            .read_frame()
            .await
            .map(|raw_frame| RawBackendMessage::new(raw_frame.0, raw_frame.1))
    }
}

struct WriteFramer {
    write_half_stream: OwnedWriteHalf,
}

impl WriteFramer {
    pub fn new(write_half_stream: OwnedWriteHalf) -> Self {
        WriteFramer { write_half_stream }
    }

    pub async fn write_frame(&mut self, frame: Vec<u8>) -> io::Result<()> {
        self.write_half_stream.write_all(&frame).await
    }
}

type RawReadFrame = ([u8; 1], Vec<u8>);

struct ReadFramer {
    framer_rx: mpsc::Receiver<RawReadFrame>,
    // TODO: these might be needed for proper shutdown
    _io_handle: task::JoinHandle<()>,
    _framer_handle: task::JoinHandle<()>,
}

impl ReadFramer {
    pub fn new(mut read_half_stream: OwnedReadHalf) -> Self {
        let (io_tx, mut io_rx) = mpsc::channel(IO_READ_BUFFER_SIZE);
        let (framer_tx, framer_rx) = mpsc::channel(FRAMER_BUFFER_SIZE);

        let io_handle = task::spawn({
            async move {
                loop {
                    let mut buffer = [0; IO_READ_BUFFER_SIZE];
                    match read_half_stream.read(&mut buffer[..]).await {
                        Ok(n_bytes_read) => {
                            io_tx.send(buffer[0..n_bytes_read].to_vec()).await.unwrap()
                        }
                        _ => {
                            break;
                        }
                    }
                }
            }
        });

        let framer_handle = task::spawn({
            async move {
                let mut buf: Vec<u8> = Vec::with_capacity(IO_READ_BUFFER_SIZE);
                loop {
                    if buf.len() >= FRAME_HEADER_SIZE {
                        let type_: [u8; 1] = buf[0..1].try_into().unwrap();
                        let count: [u8; 4] = buf[1..5].try_into().unwrap();
                        let body_size: usize = (u32::from_be_bytes(count) - 4).try_into().unwrap();
                        let required_size = FRAME_HEADER_SIZE + body_size;
                        if buf.len() >= required_size {
                            let body: Vec<u8> = buf[5..required_size].try_into().unwrap();
                            buf.drain(..required_size);
                            framer_tx.send((type_, body)).await.unwrap();
                            continue;
                        }
                    }

                    if let Some(msg) = io_rx.recv().await {
                        buf.extend(msg)
                    }
                }
            }
        });

        ReadFramer {
            _io_handle: io_handle,
            _framer_handle: framer_handle,
            framer_rx,
        }
    }

    pub async fn read_frame(&mut self) -> Option<RawReadFrame> {
        self.framer_rx.recv().await
    }
}
