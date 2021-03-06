use bytes::{Buf, BufMut, BytesMut, Bytes};
use std::io::Cursor;

use gandalf_kvs::frame::{self, Frame};
use gandalf_kvs::Command;

use crate::parser::{Parser, Kind};

use std::io::Write;

#[derive(Clone, Debug)]
pub struct KvsParser;

impl crate::ClientData for Frame {}

impl KvsParser {
    fn write_value(&self, buf: &mut BytesMut, frame: Frame) -> crate::Result<()> {
        match frame {
            Frame::Simple(val) => {
                buf.put_u8(b'+');
                buf.put(val.as_bytes());
                buf.put(&b"\r\n"[..]);
            }
            Frame::Error(val) => {
                buf.put_u8(b'-');
                buf.put(val.as_bytes());
                buf.put(&b"\r\n"[..]);
            }
            Frame::Integer(val) => {
                buf.put_u8(b':');
                let mut buff = [0u8; 20];
                let mut buff = Cursor::new(&mut buff[..]);

                write!(&mut buff, "{}", val)?;
                let pos = buff.position() as usize;

                buf.put(&buff.get_ref()[..pos]);
            }
            Frame::Null => {
                buf.put(&b"$-1\r\n"[..]);
            }
            Frame::Bulk(val) => {
                let len = val.len();

                buf.put_u8(b'$');

                self.write_decimal(buf, len as u64)?;

                buf.put(val);
                buf.put(&b"\r\n"[..]);
            }
            Frame::Array(val) => {
                buf.put_u8(b'*');
                self.write_decimal(buf, val.len() as u64)?;
                for entity in val {
                    self.write_value(buf, entity)?;
                }
            },
        }
        Ok(())
    }

    fn write_decimal(&self, buf: &mut BytesMut, value: u64) -> crate::Result<()> {
        let mut buff = [0u8; 20];
        let mut buff = Cursor::new(&mut buff[..]);

        write!(&mut buff, "{}", value)?;
        let pos = buff.position() as usize;

        buf.put(&buff.get_ref()[..pos]);
        buf.put(&b"\r\n"[..]);

        Ok(())
    }

}

impl Parser<Frame> for KvsParser {
    fn parse(&self, buffer: &mut BytesMut) -> crate::Result<Option<Kind<Frame>>> {
        let mut cursor = Cursor::new(&buffer[..]);
        match Frame::check(&mut cursor) {
            Ok(_) => {
                let len = cursor.position() as usize;
                cursor.set_position(0);

                let frame = Frame::parse(&mut cursor)?;
                
                buffer.advance(len);

                match Command::from_frame(frame.clone())? {
                    Command::Get(_) => return Ok(Some(Kind::Read(frame))),
                    Command::Snap(_) => return Ok(Some(Kind::Read(frame))),
                    Command::Set(_) => return Ok(Some(Kind::Write(frame))),
                    Command::Load(_) => return Ok(Some(Kind::Write(frame))),
                }

            }

            Err(frame::Error::Incomplete) => Ok(None),

            Err(e) => Err(e.into()),
        }
    }

    fn unparse(&self, data: Frame) -> crate::Result<Bytes> { 

        let mut buf = BytesMut::with_capacity(1024);
        match data {
            Frame::Array(val) => {
                buf.put_u8(b'*');
                self.write_decimal(&mut buf, val.len() as u64)?;
                for entity in val {
                    self.write_value(&mut buf, entity)?;
                }
            }
            _ => self.write_value(&mut buf, data)?
        }

        return Ok(buf.freeze());
    }

    fn into_error(&self, data: &str) -> crate::Result<Bytes> {
        self.unparse(Frame::Error(data.to_string()))
    }
}
