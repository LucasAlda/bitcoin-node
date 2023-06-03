use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    error::CustomError,
    messages::inv::{Inventory, InventoryType},
    node_state::NodeState,
    peer::PeerAction,
};

pub fn pending_blocks_loop(
    node_state_ref: Arc<Mutex<NodeState>>,
    peer_action_sender: mpsc::Sender<PeerAction>,
    logger_sender: mpsc::Sender<String>,
) {
    thread::spawn(move || -> Result<(), CustomError> {
        loop {
            let mut node_state = node_state_ref.lock()?;

            if node_state.is_blocks_sync() {
                drop(node_state);
                return Ok(());
            }

            let blocks_to_refetch = node_state.get_stale_block_downloads()?;

            if !blocks_to_refetch.is_empty() {
                logger_sender.send(format!(
                    "Refetching {} pending blocks...",
                    blocks_to_refetch.len()
                ))?;

                let mut inventories = vec![];

                for block_hash in blocks_to_refetch.iter() {
                    node_state.append_pending_block(block_hash.clone())?;
                    inventories.push(Inventory::new(InventoryType::GetBlock, block_hash.clone()));
                }
                drop(node_state);

                let chunks: Vec<&[Inventory]> = inventories.chunks(5).collect();

                for chunk in chunks {
                    peer_action_sender.send(PeerAction::GetData(chunk.to_vec()))?;
                }
            } else {
                drop(node_state);
            }

            thread::sleep(Duration::from_secs(1));
        }
    });
}