use crate::{error::CustomError, parser::BufferParser};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct OutPoint {
    pub hash: Vec<u8>,
    pub index: u32,
}

impl OutPoint {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.extend(self.hash.clone());
        buffer.extend(self.index.to_le_bytes());
        buffer
    }
    pub fn parse(buffer: Vec<u8>) -> Result<Self, CustomError> {
        let mut parser = BufferParser::new(buffer);
        let hash = parser.extract_buffer(32)?.to_vec();
        let index = parser.extract_u32()?;
        Ok(Self { hash, index })
    }
}

#[cfg(test)]
mod tests {
    use crate::structs::outpoint::OutPoint;

    #[test]
    fn serialize_and_parse() {
        let outpoint = OutPoint {
            hash: vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1, 2, 3, 4, 5, 6, 7,
                8, 9, 10, 1, 2,
            ],
            index: 0,
        };
        let serialized = outpoint.serialize();
        let parsed_outpoint = OutPoint::parse(serialized).unwrap();
        println!("{:?}", parsed_outpoint);
        assert_eq!(outpoint, parsed_outpoint);
    }
}
