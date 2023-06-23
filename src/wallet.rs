use std::io::{Read, Write};

use crate::{error::CustomError, node_state::open_new_file, parser::BufferParser, utxo::UTXO};

#[derive(Clone, Debug)]
pub struct Movement {
    pub tx_hash: Vec<u8>,
    pub value: u64,
    pub block_hash: Option<Vec<u8>>,
}

impl Movement {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push(self.tx_hash.len() as u8);
        buffer.extend(self.tx_hash.clone());
        buffer.extend(self.value.to_le_bytes());
        match self.block_hash.clone() {
            Some(block_hash) => {
                buffer.push(1);
                buffer.push(block_hash.len() as u8);
                buffer.extend(block_hash);
            }
            None => {
                buffer.push(0);
            }
        }
        buffer
    }

    pub fn parse(parser: &mut BufferParser) -> Result<Self, CustomError> {
        let tx_hash_len = parser.extract_u8()? as usize;
        let tx_hash = parser.extract_buffer(tx_hash_len)?.to_vec();
        let value = parser.extract_u64()?;
        let block_hash_present = parser.extract_u8()?;
        let block_hash = match block_hash_present {
            0 => None,
            1 => {
                let block_hash_len = parser.extract_u8()? as usize;
                Some(parser.extract_buffer(block_hash_len)?.to_vec())
            }
            _ => {
                return Err(CustomError::Validation(String::from(
                    "Block hash presence incorrectly formatted",
                )))
            }
        };
        Ok(Self {
            tx_hash,
            value,
            block_hash,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Wallet {
    pub name: String,
    pub pubkey: String,
    pub privkey: String,
    pub history: Vec<Movement>,
}

impl Wallet {
    pub fn new(
        name: String,
        pubkey: String,
        privkey: String,
        utxo_set: &UTXO,
    ) -> Result<Self, CustomError> {
        let mut wallet = Self {
            name,
            pubkey,
            privkey,
            history: vec![],
        };
        for (outpoint, value) in &utxo_set.tx_set {
            if value.tx_out.is_sent_to_key(&wallet.get_pubkey_hash()?) {
                wallet.history.push(Movement {
                    tx_hash: outpoint.hash.clone(),
                    value: value.tx_out.value,
                    block_hash: Some(value.block_hash.clone()),
                });
            }
        }
        Ok(wallet)
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push(self.name.len() as u8);
        buffer.extend(self.name.as_bytes());
        buffer.push(self.pubkey.len() as u8);
        buffer.extend(self.pubkey.as_bytes());
        buffer.push(self.privkey.len() as u8);
        buffer.extend(self.privkey.as_bytes());
        buffer.extend((self.history.len() as u32).to_le_bytes());
        for movement in self.history.clone() {
            buffer.extend(movement.serialize());
        }
        buffer
    }

    pub fn parse_wallets(buffer: Vec<u8>) -> Result<Vec<Self>, CustomError> {
        let mut parser = BufferParser::new(buffer);
        let mut wallets = Vec::new();
        while !parser.is_empty() {
            let name_len = parser.extract_u8()? as usize;
            let name = parser.extract_string(name_len)?;

            let pubkey_len = parser.extract_u8()? as usize;
            let pubkey = parser.extract_string(pubkey_len)?;

            println!("pubkey: {}", pubkey);
            let privkey_len = parser.extract_u8()? as usize;
            let privkey = parser.extract_string(privkey_len)?;

            let history_len = parser.extract_u32()? as usize;
            let mut history = Vec::new();
            for _ in 0..history_len {
                history.push(Movement::parse(&mut parser)?);
            }
            wallets.push(Self {
                name,
                pubkey,
                privkey,
                history,
            });
        }
        Ok(wallets)
    }

    pub fn get_pubkey_hash(&self) -> Result<Vec<u8>, CustomError> {
        get_pubkey_hash(self.pubkey.clone())
    }

    pub fn get_privkey_hash(&self) -> Result<Vec<u8>, CustomError> {
        get_privkey_hash(self.privkey.clone())
    }

    pub fn get_script_pubkey(&self) -> Result<Vec<u8>, CustomError> {
        get_script_pubkey(self.pubkey.clone())
    }

    pub fn update_history(&mut self, movement: Movement) {
        self.history.push(movement);
    }

    pub fn get_history(&self) -> Vec<Movement> {
        self.history.clone()
    }

    pub fn save_wallets(wallets: &mut [Self]) -> Result<(), CustomError> {
        let mut wallets_file = open_new_file(String::from("store/wallets.bin"), false)?;
        let mut wallets_buffer = vec![];
        for wallet in wallets.iter() {
            wallets_buffer.append(&mut wallet.serialize());
        }
        wallets_file.write_all(&wallets_buffer)?;
        Ok(())
    }

    pub fn restore_wallets() -> Result<Vec<Self>, CustomError> {
        let mut wallets_file = open_new_file(String::from("store/wallets.bin"), false)?;
        let mut saved_wallets_buffer = vec![];
        wallets_file.read_to_end(&mut saved_wallets_buffer)?;
        let wallets = match Self::parse_wallets(saved_wallets_buffer) {
            Ok(wallets) => wallets,
            Err(_) => vec![],
        };
        Ok(wallets)
    }
}

pub fn get_pubkey_hash(pubkey: String) -> Result<Vec<u8>, CustomError> {
    let decoded_pubkey = bs58::decode(pubkey)
        .into_vec()
        .map_err(|_| CustomError::Validation(String::from("User PubKey incorrectly formatted")))?;

    match decoded_pubkey.get(1..21) {
        Some(pubkey_hash) => Ok(pubkey_hash.to_vec()),
        None => Err(CustomError::Validation(String::from(
            "User PubKey incorrectly formatted",
        ))),
    }
}

pub fn get_privkey_hash(privkey: String) -> Result<Vec<u8>, CustomError> {
    let decoded_privkey = bs58::decode(privkey)
        .into_vec()
        .map_err(|_| CustomError::Validation(String::from("User PrivKey incorrectly formatted")))?;

    println!("{:?}", decoded_privkey);

    match decoded_privkey.get(1..33) {
        Some(pubkey_hash) => Ok(pubkey_hash.to_vec()),
        None => Err(CustomError::Validation(String::from(
            "User PubKey incorrectly formatted",
        ))),
    }
}

pub fn get_script_pubkey(pubkey: String) -> Result<Vec<u8>, CustomError> {
    let mut script_pubkey = Vec::new();
    script_pubkey.push(0x76);
    script_pubkey.push(0xa9);
    script_pubkey.push(0x14);
    script_pubkey.extend(get_pubkey_hash(pubkey)?);
    script_pubkey.push(0x88);
    script_pubkey.push(0xac);
    Ok(script_pubkey)
}
