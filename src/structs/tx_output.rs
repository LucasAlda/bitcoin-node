use crate::{
    error::CustomError,
    parser::{BufferParser, VarIntSerialize},
};

#[derive(Debug, Clone, PartialEq, Eq)]

/// Esta estructura representa un output de una transaccion, la cual contiene:
/// - value: Valor del output
/// - script_pubkey: public key como bitcoin script
pub struct TransactionOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

impl TransactionOutput {
    /// Esta funcion se encarga de serializar un output en un vector de bytes.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.extend(self.value.to_le_bytes());
        buffer.extend(self.script_pubkey.len().to_varint_bytes());
        buffer.extend(self.script_pubkey.clone());
        buffer
    }

    /// Esta funcion se encarga de parsear un output a partir de un BufferParser.
    pub fn parse(parser: &mut BufferParser) -> Result<Self, CustomError> {
        let value = parser.extract_u64()?;
        let script_pk_length = parser.extract_varint()? as usize;
        let script_pubkey = parser.extract_buffer(script_pk_length)?.to_vec();
        Ok(Self {
            value,
            script_pubkey,
        })
    }

    /// Esta funcion se encarga de verificar si un output esta enviado a una clave publica del tipo P2PKH.
    pub fn is_sent_to_key(&self, public_key_hash: &Vec<u8>) -> Result<bool, CustomError> {
        let parser = &mut BufferParser::new(self.script_pubkey.clone());
        match parser.extract_u8() {
            Ok(0x76) => compare_p2pkh(parser, public_key_hash),
            _ => Ok(false),
        }
    }
}

/// Esta funcion se encarga de comparar un script pubkey con una clave publica del tipo P2PKH.
fn compare_p2pkh(
    parser: &mut BufferParser,
    public_key_hash: &Vec<u8>,
) -> Result<bool, CustomError> {
    match parser.extract_u8() {
        Ok(0xa9) => (),
        _ => return Ok(false),
    }
    match parser.extract_u8() {
        Ok(0x14) => (),
        _ => return Ok(false),
    }
    let hash = parser.extract_buffer(20)?.to_vec();

    Ok(hash == *public_key_hash)
}

#[cfg(test)]
mod tests {
    use crate::{
        messages::transaction::Transaction, parser::BufferParser, states::utxo_state::UTXO,
        structs::tx_output::TransactionOutput, wallet::Wallet,
    };

    #[test]
    fn serialize_and_parse() {
        let output = TransactionOutput {
            value: 100,
            script_pubkey: vec![4, 5, 6],
        };
        let serialized = output.serialize();
        let mut parser = BufferParser::new(serialized);
        let parsed_output = TransactionOutput::parse(&mut parser).unwrap();
        assert_eq!(output, parsed_output);
    }

    #[test]
    fn is_sent_to_key() {
        let mut found = false;
        let wallet = Wallet::new(
            String::from("test"),
            String::from("mscatccDgq7azndWHFTzvEuZuywCsUvTRu"),
            String::from("test"),
            &UTXO::new(String::from("tests"), String::from("test_utxo.bin")).unwrap(),
        )
        .unwrap();
        let buffer = vec![
            0x01, 0x00, 0x00, 0x00, 0x01, 0x6D, 0xBD, 0xDB, 0x08, 0x5B, 0x1D, 0x8A, 0xF7, 0x51,
            0x84, 0xF0, 0xBC, 0x01, 0xFA, 0xD5, 0x8D, 0x12, 0x66, 0xE9, 0xB6, 0x3B, 0x50, 0x88,
            0x19, 0x90, 0xE4, 0xB4, 0x0D, 0x6A, 0xEE, 0x36, 0x29, 0x00, 0x00, 0x00, 0x00, 0x8B,
            0x48, 0x30, 0x45, 0x02, 0x21, 0x00, 0xF3, 0x58, 0x1E, 0x19, 0x72, 0xAE, 0x8A, 0xC7,
            0xC7, 0x36, 0x7A, 0x7A, 0x25, 0x3B, 0xC1, 0x13, 0x52, 0x23, 0xAD, 0xB9, 0xA4, 0x68,
            0xBB, 0x3A, 0x59, 0x23, 0x3F, 0x45, 0xBC, 0x57, 0x83, 0x80, 0x02, 0x20, 0x59, 0xAF,
            0x01, 0xCA, 0x17, 0xD0, 0x0E, 0x41, 0x83, 0x7A, 0x1D, 0x58, 0xE9, 0x7A, 0xA3, 0x1B,
            0xAE, 0x58, 0x4E, 0xDE, 0xC2, 0x8D, 0x35, 0xBD, 0x96, 0x92, 0x36, 0x90, 0x91, 0x3B,
            0xAE, 0x9A, 0x01, 0x41, 0x04, 0x9C, 0x02, 0xBF, 0xC9, 0x7E, 0xF2, 0x36, 0xCE, 0x6D,
            0x8F, 0xE5, 0xD9, 0x40, 0x13, 0xC7, 0x21, 0xE9, 0x15, 0x98, 0x2A, 0xCD, 0x2B, 0x12,
            0xB6, 0x5D, 0x9B, 0x7D, 0x59, 0xE2, 0x0A, 0x84, 0x20, 0x05, 0xF8, 0xFC, 0x4E, 0x02,
            0x53, 0x2E, 0x87, 0x3D, 0x37, 0xB9, 0x6F, 0x09, 0xD6, 0xD4, 0x51, 0x1A, 0xDA, 0x8F,
            0x14, 0x04, 0x2F, 0x46, 0x61, 0x4A, 0x4C, 0x70, 0xC0, 0xF1, 0x4B, 0xEF, 0xF5, 0xFF,
            0xFF, 0xFF, 0xFF, 0x02, 0x40, 0x4B, 0x4C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x76,
            0xA9, 0x14, 0x1A, 0xA0, 0xCD, 0x1C, 0xBE, 0xA6, 0xE7, 0x45, 0x8A, 0x7A, 0xBA, 0xD5,
            0x12, 0xA9, 0xD9, 0xEA, 0x1A, 0xFB, 0x22, 0x5E, 0x88, 0xAC, 0x80, 0xFA, 0xE9, 0xC7,
            0x00, 0x00, 0x00, 0x00, 0x19, 0x76, 0xA9, 0x14, 0x0E, 0xAB, 0x5B, 0xEA, 0x43, 0x6A,
            0x04, 0x84, 0xCF, 0xAB, 0x12, 0x48, 0x5E, 0xFD, 0xA0, 0xB7, 0x8B, 0x4E, 0xCC, 0x52,
            0x88, 0xAC, 0x00, 0x00, 0x00, 0x00,
        ];
        let public_key_hash = wallet.get_pubkey_hash().unwrap();

        let mut parser = BufferParser::new(buffer);
        let tx = Transaction::parse_from_parser(&mut parser).unwrap();
        let tx_outputs = tx.outputs.clone();
        for output in tx_outputs {
            found = output.is_sent_to_key(&public_key_hash).unwrap();
        }
        assert_eq!(found, false);
    }
}
