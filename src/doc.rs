use crate::UsernamePasswordPair;
use std::{
    fmt::Write as _,
    io::{BufRead as _, Write as _},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("Failed to read line from input: {0}")]
    ReadFailed(#[from] std::io::Error),
    #[error("Input line missing `=` delimeter: {0:?}")]
    DelimeterMissing(String),
    #[error("Input line has an invalid key: {0:?}")]
    KeyInvalid(String),
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("Failed to write line to buffer: {0}")]
    FormatFailed(#[from] std::fmt::Error),
    #[error("Failed to write output: {0}")]
    WriteFailed(#[source] std::io::Error),
    #[error("Failed to flush output: {0}")]
    FlushFailed(#[source] std::io::Error),
}

// https://git-scm.com/docs/git-credential#IOFMT
#[derive(Debug, Default)]
pub struct Doc {
    protocol: Option<String>,
    host: Option<String>,
    path: Option<String>,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
    quit: Option<bool>,
}

impl Doc {
    fn parse_line(&mut self, line: &str) -> Result<(), ReadError> {
        let (key, value) = line
            .trim()
            .split_once('=')
            .ok_or_else(|| ReadError::DelimeterMissing(line.to_owned()))?;
        let field = match key {
            "protocol" => &mut self.protocol,
            "host" => &mut self.host,
            "path" => &mut self.path,
            "username" => &mut self.username,
            "password" => &mut self.password,
            "url" => &mut self.url,
            _ => return Err(ReadError::KeyInvalid(key.to_owned())),
        };
        *field = Some(value.to_owned());
        Ok(())
    }

    pub fn read() -> Result<Self, ReadError> {
        let mut this = Self::default();
        let stdin = std::io::stdin();
        let lock = stdin.lock();
        for line in lock.lines() {
            this.parse_line(&line?)?;
        }
        log::info!("parsed input: `{:#?}`", this);
        Ok(this)
    }

    fn protocol_host_pair(&self) -> Option<(&str, &str)> {
        if let Some(url) = &self.url {
            url.split_once("://")
        } else {
            self.protocol.as_deref().zip(self.host.as_deref())
        }
    }

    pub fn matches_url(&self, protocol: &str, host: &str) -> bool {
        self.protocol_host_pair() == Some((protocol, host))
    }

    fn username_password_pair(&self) -> Option<(&str, &str)> {
        self.username.as_deref().zip(self.password.as_deref())
    }

    pub fn creds(&self) -> Option<UsernamePasswordPair> {
        self.username_password_pair()
            .map(|(username, password)| UsernamePasswordPair {
                username: username.to_owned(),
                password: password.to_owned(),
            })
    }

    pub fn matches_creds(&self, creds: &UsernamePasswordPair) -> bool {
        self.username_password_pair() == Some((&creds.username, &creds.password))
    }

    pub fn with_creds(self, creds: UsernamePasswordPair) -> Self {
        Self {
            username: Some(creds.username),
            password: Some(creds.password),
            ..self
        }
    }

    pub fn with_quit(self, quit: bool) -> Self {
        Self {
            quit: Some(quit),
            ..self
        }
    }

    fn to_string(&self) -> Result<String, WriteError> {
        let mut buf = String::new();
        macro_rules! write_field {
            ($field:ident, $buf:ident) => {
                if let Some(value) = &self.$field {
                    write!(&mut $buf, "{}={}\n", stringify!($field), value)?;
                }
            };
        }
        write_field!(protocol, buf);
        write_field!(host, buf);
        write_field!(path, buf);
        write_field!(username, buf);
        write_field!(password, buf);
        write_field!(url, buf);
        write_field!(quit, buf);
        Ok(buf)
    }

    pub fn write(self) -> Result<(), WriteError> {
        let buf = self.to_string()?;
        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        lock.write_all(buf.as_bytes())
            .map_err(WriteError::WriteFailed)?;
        lock.flush().map_err(WriteError::FlushFailed)?;
        Ok(())
    }
}
