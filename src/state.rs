use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const TOKEN: Item<Addr> = Item::new("token");
pub const TOKEN_COUNT: Item<u64> = Item::new("token_count");
pub const AVAILABLE_IDS: Item<Vec<u32>> = Item::new("available_ids");
