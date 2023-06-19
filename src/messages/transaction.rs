use std::collections::HashMap;

use bitcoin_hashes::{sha256, Hash};

use crate::{
    error::CustomError,
    message::Message,
    parser::{BufferParser, VarIntSerialize},
    utxo::UTXO,
    wallet::{get_script_pubkey, Movement, Wallet},
};

#[derive(Debug, Clone)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub lock_time: u32,
}

impl Transaction {
    pub fn hash(&self) -> Vec<u8> {
        sha256::Hash::hash(sha256::Hash::hash(self.serialize().as_slice()).as_byte_array())
            .as_byte_array()
            .to_vec()
    }
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.extend(self.version.to_le_bytes());
        buffer.extend(self.inputs.len().to_varint_bytes());
        for input in &self.inputs {
            buffer.extend(input.serialize());
        }
        buffer.extend(self.outputs.len().to_varint_bytes());
        for output in &self.outputs {
            buffer.extend(output.serialize());
        }
        buffer.extend(self.lock_time.to_le_bytes());
        //buffer.extend(1_u32.to_le_bytes());
        buffer
    }

    pub fn parse(parser: &mut BufferParser) -> Result<Self, CustomError> {
        let version = parser.extract_u32()?;
        //chequear lo del flag
        let tx_in_count = parser.extract_varint()? as usize;
        let mut inputs = vec![];
        for _ in 0..tx_in_count {
            inputs.push(TransactionInput::parse(parser)?);
        }
        let tx_out_count = parser.extract_varint()? as usize;
        let mut outputs = vec![];
        for _ in 0..tx_out_count {
            outputs.push(TransactionOutput::parse(parser)?);
        }

        let lock_time = parser.extract_u32()?;
        Ok(Self {
            version,
            inputs,
            outputs,
            lock_time,
        })
    }

    pub fn get_movement(&self, public_key_hash: &Vec<u8>, utxo: &UTXO) -> Option<Movement> {
        let mut value = 0;

        for output in &self.outputs {
            if output.is_sent_to_key(public_key_hash) {
                value += output.value;
            }
        }
        for input in &self.inputs {
            if let Some(output) = utxo.tx_set.get(&input.previous_output) {
                if output.is_sent_to_key(public_key_hash) {
                    value -= output.value;
                }
            }
        }
        if value != 0 {
            Some(Movement {
                tx_hash: self.hash(),
                value,
                block_hash: None,
            })
        } else {
            None
        }
    }

    pub fn create(
        sender_wallet: &Wallet,
        inputs_outpoints: Vec<OutPoint>,
        outputs: HashMap<String, u64>,
    ) -> Result<Self, CustomError> {
        //println!("Wallet: {:?}", sender_wallet);
        println!("Inputs: {:?}", inputs_outpoints);
        println!("Outputs: {:?}", outputs);
        let mut transaction = Transaction {
            version: 1,
            inputs: vec![],
            outputs: vec![],
            lock_time: 0,
        };
        let script_pubkey = sender_wallet.get_script_pubkey()?;
        println!("script pubkey: {:?}", script_pubkey);
        for outpoint in inputs_outpoints {
            let input = TransactionInput {
                previous_output: outpoint,
                script_sig: script_pubkey.clone(),
                sequence: 0xffffffff,
            };
            transaction.inputs.push(input);
        }
        for (pubkey, value) in outputs {
            let script_pubkey = get_script_pubkey(pubkey)?;
            let output = TransactionOutput {
                value,
                script_pubkey,
            };
            transaction.outputs.push(output);
        }
        println!("Transaction: {:?}", transaction);

        Ok(transaction)
    }
}

impl Message for Transaction {
    fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.extend(self.version.to_le_bytes());
        buffer.extend(self.inputs.len().to_varint_bytes());
        for input in &self.inputs {
            buffer.extend(input.serialize());
        }
        buffer.extend(self.outputs.len().to_varint_bytes());
        for output in &self.outputs {
            buffer.extend(output.serialize());
        }
        buffer.extend(self.lock_time.to_le_bytes());
        buffer
    }

    fn get_command(&self) -> String {
        String::from("tx")
    }

    fn parse(buffer: Vec<u8>) -> Result<Self, crate::error::CustomError> {
        let mut parser = BufferParser::new(buffer);

        let version = parser.extract_u32()?;
        //chequear lo del flag
        let tx_in_count = parser.extract_varint()? as usize;
        let mut inputs = vec![];
        for _ in 0..tx_in_count {
            inputs.push(TransactionInput::parse(&mut parser)?);
        }
        let tx_out_count = parser.extract_varint()? as usize;
        let mut outputs = vec![];
        for _ in 0..tx_out_count {
            outputs.push(TransactionOutput::parse(&mut parser)?);
        }

        let lock_time = parser.extract_u32()?;
        Ok(Self {
            version,
            inputs,
            outputs,
            lock_time,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TransactionInput {
    pub previous_output: OutPoint,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}

impl TransactionInput {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.extend(self.previous_output.serialize());
        buffer.extend(self.script_sig.len().to_varint_bytes());
        buffer.extend(self.script_sig.clone());
        buffer.extend(self.sequence.to_le_bytes());
        buffer
    }
    pub fn parse(parser: &mut BufferParser) -> Result<Self, CustomError> {
        let previous_output = OutPoint::parse(parser.extract_buffer(36)?.to_vec())?;
        let script_sig_length = parser.extract_varint()? as usize;
        let script_sig = parser.extract_buffer(script_sig_length)?.to_vec();
        let sequence = parser.extract_u32()?;
        Ok(Self {
            previous_output,
            script_sig,
            sequence,
        })
    }
}

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

#[derive(Debug, Clone)]
pub struct TransactionOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

impl TransactionOutput {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.extend(self.value.to_le_bytes());
        buffer.extend(self.script_pubkey.len().to_varint_bytes());
        buffer.extend(self.script_pubkey.clone());
        buffer
    }

    pub fn parse(parser: &mut BufferParser) -> Result<Self, CustomError> {
        let value = parser.extract_u64()?;
        let script_pk_length = parser.extract_varint()? as usize;
        let script_pubkey = parser.extract_buffer(script_pk_length)?.to_vec();
        Ok(Self {
            value,
            script_pubkey,
        })
    }

    pub fn is_sent_to_key(&self, public_key_hash: &Vec<u8>) -> bool {
        let parser = &mut BufferParser::new(self.script_pubkey.clone());
        match parser.extract_u8() {
            Ok(0x76) => compare_p2pkh(parser, public_key_hash),
            _ => false,
        }
    }
}

fn compare_p2pkh(parser: &mut BufferParser, public_key_hash: &Vec<u8>) -> bool {
    match parser.extract_u8() {
        Ok(0xa9) => (),
        _ => return false,
    }
    match parser.extract_u8() {
        Ok(0x14) => (),
        _ => return false,
    }
    let hash = parser.extract_buffer(20).unwrap().to_vec();

    hash == *public_key_hash
}
