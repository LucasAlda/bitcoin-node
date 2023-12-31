use crate::{error::CustomError, parser::BufferParser};

#[derive(Debug, Clone, PartialEq)]
/// Este enum contiene los tipos de inventarios que se pueden enviar:
/// - Tx = 1
/// - Block = 2
/// - FilteredBlock = 3
/// - CompactBlock = 4
/// - WitnessTx = 5
/// - WitnessBlock = 6
/// - FilteredWitnessBlock = 7
pub enum InventoryType {
    Tx,
    Block,
    FilteredBlock,
    CompactBlock,
    WitnessTx,
    WitnessBlock,
    FilteredWitnessBlock,
}

#[derive(Debug, Clone, PartialEq)]
///Esta es la estructura de un inventario, la cual contiene un tipo de inventario y un hash del inventario en si.
pub struct Inventory {
    pub inventory_type: InventoryType,
    pub hash: Vec<u8>,
}

impl Inventory {
    /// Esta funcion se encarga de crear un nuevo inventario, dado un tipo de inventario y un hash recibidos como parametros.
    pub fn new(inventory_type: InventoryType, hash: Vec<u8>) -> Self {
        Self {
            inventory_type,
            hash,
        }
    }

    /// Esta funcion se encarga de serializar un inventario en un vector de bytes.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        let inventory_type = match self.inventory_type {
            InventoryType::Tx => 1_u32,
            InventoryType::Block => 2_u32,
            InventoryType::FilteredBlock => 3_u32,
            InventoryType::CompactBlock => 4_u32,
            InventoryType::WitnessTx => 0x40000001,
            InventoryType::WitnessBlock => 0x40000002,
            InventoryType::FilteredWitnessBlock => 0x40000003,
        };
        buffer.extend(inventory_type.to_le_bytes());
        buffer.extend(&self.hash);
        buffer
    }

    /// Esta funcion se encarga de parsear un vector de bytes a un inventario.
    /// Devuelve CustomError si:
    /// - El vector de bytes no tiene 36 bytes de longitud
    /// - El tipo de inventario no es valido
    /// - El hash no tiene 32 bytes de longitud
    pub fn parse(buffer: Vec<u8>) -> Result<Self, CustomError> {
        let mut parser = BufferParser::new(buffer);
        if parser.len() != 36 {
            return Err(CustomError::SerializedBufferIsInvalid);
        }
        let inventory_type = match parser.extract_u32()? {
            1_u32 => InventoryType::Tx,
            2_u32 => InventoryType::Block,
            3_u32 => InventoryType::FilteredBlock,
            4_u32 => InventoryType::CompactBlock,
            0x40000001 => InventoryType::WitnessTx,
            0x40000002 => InventoryType::WitnessBlock,
            0x40000003 => InventoryType::FilteredWitnessBlock,
            _ => {
                return Err(CustomError::SerializedBufferIsInvalid);
            }
        };
        Ok(Self {
            inventory_type,
            hash: parser.extract_buffer(32)?.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::structs::inventory::{Inventory, InventoryType};

    #[test]
    fn inventory_block_serialize_and_parse() {
        let inventory = Inventory::new(
            InventoryType::Block,
            [
                220, 9, 210, 68, 121, 44, 33, 165, 243, 235, 28, 125, 43, 136, 29, 116, 190, 43,
                124, 200, 30, 144, 40, 190, 229, 44, 93, 83, 110, 112, 225, 220,
            ]
            .to_vec(),
        );
        let buffer = inventory.serialize();
        let parsed_inventory = Inventory::parse(buffer).unwrap();
        assert_eq!(inventory, parsed_inventory);

        assert_ne!(parsed_inventory.inventory_type, InventoryType::Tx);
        assert_ne!(
            parsed_inventory.inventory_type,
            InventoryType::FilteredBlock
        );
        assert_ne!(parsed_inventory.inventory_type, InventoryType::CompactBlock);
        assert_ne!(parsed_inventory.inventory_type, InventoryType::WitnessTx);
        assert_ne!(parsed_inventory.inventory_type, InventoryType::WitnessBlock);
        assert_ne!(
            parsed_inventory.inventory_type,
            InventoryType::FilteredWitnessBlock
        );
    }

    #[test]
    fn inventory_tx_serialize_and_parse() {
        let inventory = Inventory::new(
            InventoryType::Tx,
            [
                220, 9, 210, 68, 121, 44, 33, 165, 243, 235, 28, 125, 43, 136, 29, 116, 190, 43,
                124, 200, 30, 144, 40, 190, 229, 44, 93, 83, 110, 112, 225, 220,
            ]
            .to_vec(),
        );
        let buffer = inventory.serialize();
        let parsed_inventory = Inventory::parse(buffer).unwrap();
        assert_eq!(inventory, parsed_inventory);
    }

    #[test]
    fn inventory_filtered_block_serialize_and_parse() {
        let inventory = Inventory::new(
            InventoryType::FilteredBlock,
            [
                220, 9, 210, 68, 121, 44, 33, 165, 243, 235, 28, 125, 43, 136, 29, 116, 190, 43,
                124, 200, 30, 144, 40, 190, 229, 44, 93, 83, 110, 112, 225, 220,
            ]
            .to_vec(),
        );
        let buffer = inventory.serialize();
        let parsed_inventory = Inventory::parse(buffer).unwrap();
        assert_eq!(inventory, parsed_inventory);
    }

    #[test]
    fn inventory_compact_block_serialize_and_parse() {
        let inventory = Inventory::new(
            InventoryType::CompactBlock,
            [
                220, 9, 210, 68, 121, 44, 33, 165, 243, 235, 28, 125, 43, 136, 29, 116, 190, 43,
                124, 200, 30, 144, 40, 190, 229, 44, 93, 83, 110, 112, 225, 220,
            ]
            .to_vec(),
        );
        let buffer = inventory.serialize();
        let parsed_inventory = Inventory::parse(buffer).unwrap();
        assert_eq!(inventory, parsed_inventory);
    }

    #[test]
    fn inventory_witness_tx_serialize_and_parse() {
        let inventory = Inventory::new(
            InventoryType::WitnessTx,
            [
                220, 9, 210, 68, 121, 44, 33, 165, 243, 235, 28, 125, 43, 136, 29, 116, 190, 43,
                124, 200, 30, 144, 40, 190, 229, 44, 93, 83, 110, 112, 225, 220,
            ]
            .to_vec(),
        );
        let buffer = inventory.serialize();
        let parsed_inventory = Inventory::parse(buffer).unwrap();
        assert_eq!(inventory, parsed_inventory);
    }

    #[test]
    fn inventory_witness_block_serialize_and_parse() {
        let inventory = Inventory::new(
            InventoryType::WitnessBlock,
            [
                220, 9, 210, 68, 121, 44, 33, 165, 243, 235, 28, 125, 43, 136, 29, 116, 190, 43,
                124, 200, 30, 144, 40, 190, 229, 44, 93, 83, 110, 112, 225, 220,
            ]
            .to_vec(),
        );
        let buffer = inventory.serialize();
        let parsed_inventory = Inventory::parse(buffer).unwrap();
        assert_eq!(inventory, parsed_inventory);
    }

    #[test]
    fn inventory_filtered_witness_block_serialize_and_parse() {
        let inventory = Inventory::new(
            InventoryType::FilteredWitnessBlock,
            [
                220, 9, 210, 68, 121, 44, 33, 165, 243, 235, 28, 125, 43, 136, 29, 116, 190, 43,
                124, 200, 30, 144, 40, 190, 229, 44, 93, 83, 110, 112, 225, 220,
            ]
            .to_vec(),
        );
        let buffer = inventory.serialize();
        let parsed_inventory = Inventory::parse(buffer).unwrap();
        assert_eq!(inventory, parsed_inventory);
    }

    #[test]
    fn inventory_invalid_buffer() {
        let inventory = Inventory {
            inventory_type: InventoryType::Block,
            hash: [
                220, 9, 210, 68, 121, 44, 33, 165, 243, 235, 28, 125, 43, 136, 29, 116, 190, 43,
                124, 200, 30, 144, 40, 190, 229, 44, 93, 83, 110, 112, 225, 220, 100, 200, 129,
                233, 45, 56, 82, 56, 124, 200, 30, 144, 40, 190, 229, 44, 93, 83, 110, 112, 46,
            ]
            .to_vec(),
        };
        let buffer = inventory.serialize();
        let parsed_inventory = Inventory::parse(buffer);
        assert!(parsed_inventory.is_err());
    }
}
