use bitcoin_hashes::{sha256d, Hash};

use crate::{error::CustomError, parser::BufferParser};

#[derive(Debug, Clone)]
///Esta estructura representa el header de un bloque, el cual contiene la siguiente información:
/// - Version: Versión del bloque
/// - Prev_block_hash: Hash del bloque anterior
/// - Merkle_root: Hash de la raíz del árbol de merkle con las transacciones del bloque
/// - Timestamp: Marca de tiempo en la que se creó el bloque
/// - Bits: Bits de dificultad del bloque
/// - Nonce: Número aleatorio que se utiliza para generar el hash del bloque
pub struct BlockHeader {
    pub version: i32,
    pub prev_block_hash: Vec<u8>,
    pub merkle_root: Vec<u8>,
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
    pub hash: Vec<u8>,
    pub broadcasted: bool,
    pub block_downloaded: bool,
}

impl BlockHeader {
    ///Esta funcion se encarga de dado un BlockHeader, serializarlo en un vector de bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.extend(&self.version.to_le_bytes());
        buffer.extend(&self.prev_block_hash);
        buffer.extend(&self.merkle_root);
        buffer.extend(&self.timestamp.to_le_bytes());
        buffer.extend(&self.bits.to_le_bytes());
        buffer.extend(&self.nonce.to_le_bytes());

        buffer
    }

    pub fn serialize_for_backup(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.extend(&self.version.to_le_bytes());
        buffer.extend(&self.prev_block_hash);
        buffer.extend(&self.merkle_root);
        buffer.extend(&self.timestamp.to_le_bytes());
        buffer.extend(&self.bits.to_le_bytes());
        buffer.extend(&self.nonce.to_le_bytes());
        buffer.extend(&self.hash);

        buffer
    }

    ///Esta funcion se encarga de dado un vector de bytes, parsearlo a un BlockHeader con todos sus campos correspondientes
    /// Tambien se encarga de validar que el header sea valido, es decir, que cumpla con la proof of work, esto solo lo hace si el parametro validate es true.
    pub fn parse(buffer: Vec<u8>) -> Result<Self, CustomError> {
        let hash = sha256d::Hash::hash(&buffer).to_byte_array().to_vec();

        let mut parser = BufferParser::new(buffer);
        if parser.len() < 80 {
            return Err(CustomError::SerializedBufferIsInvalid);
        }

        let block_header = BlockHeader {
            version: parser.extract_i32()?,
            prev_block_hash: parser.extract_buffer(32)?.to_vec(),
            merkle_root: parser.extract_buffer(32)?.to_vec(),
            timestamp: parser.extract_u32()?,
            bits: parser.extract_u32()?,
            nonce: parser.extract_u32()?,
            hash,
            block_downloaded: false,
            broadcasted: false,
        };

        if !(block_header.validate()) {
            return Err(CustomError::HeaderInvalidPoW);
        }

        Ok(block_header)
    }

    pub fn parse_from_backup(buffer: Vec<u8>) -> Result<Self, CustomError> {
        let mut parser = BufferParser::new(buffer);
        if parser.len() < 112 {
            return Err(CustomError::SerializedBufferIsInvalid);
        }

        let block_header = BlockHeader {
            version: parser.extract_i32()?,
            prev_block_hash: parser.extract_buffer(32)?.to_vec(),
            merkle_root: parser.extract_buffer(32)?.to_vec(),
            timestamp: parser.extract_u32()?,
            bits: parser.extract_u32()?,
            nonce: parser.extract_u32()?,
            hash: parser.extract_buffer(32)?.to_vec(),
            block_downloaded: true,
            broadcasted: true,
        };

        if !(block_header.validate()) {
            return Err(CustomError::HeaderInvalidPoW);
        }

        Ok(block_header)
    }

    ///Esta funcion se encarga de validar la proof of work de un bloque.
    fn validate(&self) -> bool {
        let hash = self.hash();
        let bits_vec = self.bits.to_be_bytes().to_vec();

        let leading_zeros_start = bits_vec[0] as usize;
        let leading_zeros = hash[leading_zeros_start..32].to_vec();

        if leading_zeros.iter().any(|zero| *zero != 0_u8) {
            return false;
        }

        let mut significants = hash[(leading_zeros_start - 3)..leading_zeros_start].to_vec();
        significants.reverse();

        let mut bits_vec_pos = 1;
        for hash_byte in significants {
            if hash_byte != bits_vec[bits_vec_pos] {
                return hash_byte < bits_vec[bits_vec_pos];
            }
            bits_vec_pos += 1;
        }
        false
    }

    /// Esta funcion se encarga de calcular el hash del header de un bloque
    pub fn hash(&self) -> &Vec<u8> {
        &self.hash
    }

    /// Esta funcion se encarga de calcular el hash del header de un bloque y devolverlo como un string
    pub fn hash_as_string(&self) -> String {
        hash_as_string(self.hash().clone())
    }
}

/// Esta funcion se encarga de convertir un vector de bytes en hexa que forma un hash a un string
pub fn hash_as_string(hash: Vec<u8>) -> String {
    let mut filename = String::with_capacity(2 * hash.len());
    for byte in hash {
        filename.push_str(format!("{:02X}", byte).as_str());
    }
    filename
}

#[cfg(test)]
mod tests {
    use crate::structs::block_header::BlockHeader;

    #[test]
    fn blockheader_serialize_and_parse() {
        let buffer = vec![
            1, 0, 0, 0, 5, 159, 141, 74, 195, 4, 19, 253, 127, 1, 148, 149, 222, 143, 237, 24, 27,
            124, 186, 34, 123, 241, 216, 166, 203, 239, 86, 108, 0, 0, 0, 0, 233, 233, 109, 115,
            249, 241, 6, 200, 176, 73, 10, 24, 28, 209, 102, 159, 255, 179, 239, 72, 185, 225, 10,
            14, 219, 74, 174, 208, 207, 59, 18, 12, 170, 7, 195, 79, 255, 255, 0, 29, 14, 171, 58,
            61,
        ];

        let buffer_clone = buffer.clone();

        let block_header = BlockHeader::parse(buffer).unwrap();
        let serialized_block_header = block_header.serialize();

        assert_eq!(buffer_clone, serialized_block_header);
    }

    #[test]
    fn blockheader_too_short_buffer() {
        let buffer = vec![1, 0];

        let block_header = BlockHeader::parse(buffer);

        assert!(block_header.is_err());
    }

    #[test]
    fn blockheader_invalid_buffer() {
        let buffer = vec![
            1, 0, 0, 0, 5, 159, 141, 74, 195, 4, 19, 253, 127, 1, 148, 149, 222, 143, 237, 24, 27,
            124, 186, 34, 123, 241, 216, 166, 203, 239, 86, 108, 0, 0, 0, 0, 233, 233, 109, 115,
            249, 241, 6, 200, 176, 73, 10, 24, 28, 209, 102, 159, 255, 179, 239, 72, 185, 225, 10,
            14, 219,
        ];

        let block_header = BlockHeader::parse(buffer);

        assert!(block_header.is_err());
    }

    #[test]
    fn valid_pow_header() {
        let valid_header = BlockHeader {
            version: 2,
            prev_block_hash: vec![
                61, 8, 52, 163, 234, 98, 255, 92, 186, 170, 164, 90, 56, 131, 46, 171, 52, 239,
                104, 223, 166, 65, 183, 217, 36, 6, 53, 63, 0, 0, 0, 0,
            ],
            merkle_root: vec![
                45, 107, 6, 225, 181, 124, 4, 88, 86, 174, 58, 59, 113, 215, 174, 42, 209, 149,
                142, 110, 166, 53, 244, 88, 6, 76, 228, 77, 7, 10, 189, 126,
            ],
            timestamp: 1347149007,
            bits: 476726600,
            nonce: 240236131,
            hash: vec![
                10, 110, 89, 244, 38, 172, 240, 48, 75, 251, 139, 33, 16, 164, 179, 154, 22, 123,
                120, 81, 209, 213, 111, 183, 7, 9, 162, 49, 0, 0, 0, 0,
            ],
            block_downloaded: false,
            broadcasted: false,
        };

        valid_header.serialize();
        assert!(valid_header.validate());
    }

    #[test]
    fn invalid_pow_header() {
        let valid_header = BlockHeader {
            version: 2,
            prev_block_hash: vec![
                61, 8, 52, 163, 234, 98, 255, 92, 186, 170, 164, 90, 56, 131, 46, 171, 52, 239,
                104, 223, 166, 65, 183, 217, 36, 6, 53, 63, 0, 0, 0, 0,
            ],
            merkle_root: vec![
                45, 107, 6, 225, 181, 124, 4, 88, 86, 174, 58, 59, 113, 215, 174, 42, 209, 149,
                142, 110, 166, 53, 244, 88, 6, 76, 228, 77, 7, 10, 189, 126,
            ],
            timestamp: 1347149007,
            bits: 476726600,
            nonce: 123123,
            hash: vec![
                116, 18, 66, 212, 76, 145, 158, 131, 46, 212, 244, 136, 96, 84, 11, 220, 121, 121,
                78, 50, 3, 197, 235, 49, 172, 32, 11, 104, 118, 114, 161, 104,
            ],
            block_downloaded: false,
            broadcasted: false,
        };

        assert!(!valid_header.validate());
    }
}
