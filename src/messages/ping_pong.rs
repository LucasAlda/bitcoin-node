use crate::{error::CustomError, message::Message, parser::BufferParser};

const NONCE_BYTES: usize = 8;

/// Esta es la estructura de un mensaje ping, la cual contiene un nonce
pub struct Ping {
    pub nonce: u64,
}

/// Implementa el trait Message para el mensaje ping.
/// Permite serializar, parsear y obtener el comando
impl Message for Ping {
    fn get_command(&self) -> String {
        String::from("ping")
    }
    fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.extend(&self.nonce.to_le_bytes());
        buffer
    }
    fn parse(buffer: Vec<u8>) -> Result<Self, CustomError> {
        if buffer.len() != NONCE_BYTES {
            return Err(CustomError::SerializedBufferIsInvalid);
        }
        let mut parser = BufferParser::new(buffer);
        let nonce = parser.extract_u64()?;
        Ok(Ping { nonce })
    }
}

/// Esta es la estructura de un mensaje pong, la cual contiene un nonce que debe ser igual al nonce del mensaje ping
pub struct Pong {
    pub nonce: u64,
}

/// Implementa el trait Message para el mensaje pong.
/// Permite serializar, parsear y obtener el comando
impl Message for Pong {
    fn get_command(&self) -> String {
        String::from("pong")
    }
    fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.extend(&self.nonce.to_le_bytes());
        buffer
    }
    fn parse(buffer: Vec<u8>) -> Result<Self, CustomError> {
        if buffer.len() != NONCE_BYTES {
            return Err(CustomError::SerializedBufferIsInvalid);
        }
        let mut parser = BufferParser::new(buffer);
        let nonce = parser.extract_u64()?;
        Ok(Pong { nonce })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_ping() {
        let ping = Ping { nonce: 1024 };
        let serialized_ping = ping.serialize();
        assert_eq!(serialized_ping, vec![0, 4, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn serialize_pong() {
        let ping = Pong { nonce: 1024 };
        let serialized_ping = ping.serialize();
        assert_eq!(serialized_ping, vec![0, 4, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn parse_ping() {
        let ping = Ping { nonce: 1024 };
        let serialized_ping = ping.serialize();
        let parsed_ping = Ping::parse(serialized_ping).unwrap();
        assert_eq!(parsed_ping.nonce, ping.nonce);
    }

    #[test]
    fn parse_pong() {
        let ping = Ping { nonce: 1024 };
        let serialized_ping = ping.serialize();
        let parsed_pong = Pong::parse(serialized_ping).unwrap();
        assert_eq!(parsed_pong.nonce, ping.nonce);
    }

    #[test]
    fn parse_invalid_pong() {
        let buffer_too_long = vec![0x00];
        let parsed_pong = Pong::parse(buffer_too_long);
        assert_eq!(parsed_pong.is_err(), true);
    }
}
