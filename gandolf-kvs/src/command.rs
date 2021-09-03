use crate::{Frame, Parse, Db, Connection};

use bytes::Bytes;

use tracing::debug;

#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set)
}


#[derive(Debug)]
pub struct Get {
    key: String,
}

#[derive(Debug)]
pub struct Set {
    key: String,
    value: Bytes,
}

impl Command {
    pub fn from_frame(frame: Frame) -> crate::Result<Command> {
        let mut parse = Parse::new(frame)?;

        let cmd_name = parse.next_string()?.to_lowercase();

        debug!("cmd name is {:?}", cmd_name);

        let cmd = match &cmd_name[..] {
            "get" => Command::Get(Get::from_parse(&mut parse)?),
            "set" => Command::Set(Set::from_parse(&mut parse)?),
            _ => {
                return Err("Could not parse the command".into())
            }
        };

        parse.finish()?;
        return Ok(cmd);
    }


    pub async fn apply(self, db: &Db, con: &mut Connection) -> crate::Result<()>{
        match self {
            Command::Get(cmd) => cmd.apply(db, con).await,
            Command::Set(cmd) => cmd.apply(db, con).await
        }
    }
}


impl Get {
    pub fn new(key: String) -> Get {
        Get {
            key: key.to_string()
        }
    }

    pub fn from_parse(parse: &mut Parse) -> crate::Result<Get> {
        let key = parse.next_string()?;
        Ok(Get {
            key: key
        })
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub async fn apply(self, db: &Db, con: &mut Connection) -> crate::Result<()> {
        let response = if let Some(value) = db.get(&self.key) {
            Frame::Bulk(value)
        } else {
            Frame::Null
        };

        debug!(?response);
        con.write_frame(&response).await?;

        Ok(())
    }

    pub fn into_frame(self) -> Frame {
        let name = Frame::Bulk(Bytes::from("get".as_bytes()));
        let key = Frame::Bulk(Bytes::from(self.key.into_bytes()));
        Frame::Array(vec![name, key])
    }

}


impl Set {
    pub fn new(key: impl ToString, value: Bytes) -> Set {
        Set {
            key: key.to_string(),
            value: value
        }
    }

    pub fn from_parse(parse: &mut Parse) -> crate::Result<Set> {
        let key = parse.next_string()?;
        let value = parse.next_bytes()?;
        Ok(Set {
            key: key,
            value: value
        })
    }

    pub async fn apply(self, db: &Db, con: &mut Connection) -> crate::Result<()>{
        db.set(self.key, self.value);
        let response = Frame::Simple("OK".into());

        debug!(?response);
        con.write_frame(&response).await?;

        Ok(())
    }

    pub fn into_frame(self) -> Frame {
        let name = Frame::Bulk(Bytes::from("set".as_bytes()));
        let key = Frame::Bulk(Bytes::from(self.key.into_bytes()));
        let value = Frame::Bulk(Bytes::from(self.value));
        Frame::Array(vec![name, key, value])
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &Bytes {
        &self.value
    }
}
