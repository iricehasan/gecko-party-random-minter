#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, wasm_instantiate, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply,
    Response, StdResult, SubMsg, Uint256, WasmMsg,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, Metadata, MigrateMsg, QueryMsg};
use crate::state::{AVAILABLE_IDS, TOKEN, TOKEN_COUNT};

use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use cw721_base::InstantiateMsg as Cw721InstantaiteMsg;
use sha2::{Digest, Sha256};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:gecko-random-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const INSTANTIATE_REPLY: u64 = 1;
pub const MINT_REPLY: u64 = 2;

pub type Extension = Option<Metadata>;
pub type Cw721BaseExecuteMsg = cw721_base::ExecuteMsg<Extension, Empty>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let available_ids: Vec<u32> = (1..=10000).collect();
    AVAILABLE_IDS.save(deps.storage, &available_ids)?;

    let cw721_init_msg = Cw721InstantaiteMsg {
        name: msg.name,
        symbol: msg.symbol,
        minter: env.contract.address.to_string(),
    };

    let submsg = SubMsg::reply_on_success(
        wasm_instantiate(
            msg.cw721_code_id,
            &cw721_init_msg,
            vec![],
            "Nft Contract".to_owned(),
        )
        .unwrap(),
        INSTANTIATE_REPLY,
    );

    TOKEN.save(deps.storage, &Addr::unchecked(""))?;
    TOKEN_COUNT.save(deps.storage, &10000u64)?; // start from 10000 as the number of NFTs

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_submessage(submsg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MintToken {} => {
            let mut available_ids = AVAILABLE_IDS.load(deps.storage)?;

            let token_addr = TOKEN.load(deps.storage)?;
            let mut token_count = TOKEN_COUNT.load(deps.storage)?;

            let random_index = _random_chance(&env, &info, token_count)
                .to_string()
                .parse::<u64>()
                .unwrap();

            let token_id = available_ids[random_index as usize];

            let token_uri = format!(
                "https://bafybeigc7xxqksy5vpzevkwxvrlgcri3q4izc4s5wtcqzpienee3ljopi4.ipfs.w3s.link/{}.json",
                token_id
            );

            let image_uri = format!(
                "https://bafybeiezubaoizfmpwsgv3wnlxzhczjod6ehybytvpu7ftj3ji3cinudjq.ipfs.w3s.link/{}.png",
                token_id
            );

            let submsg_mint = SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: token_addr.clone().to_string(),
                    msg: to_json_binary(&Cw721BaseExecuteMsg::Mint {
                        token_id: token_id.to_string(),
                        owner: info.sender.to_string(),
                        token_uri: Some(token_uri),
                        extension: Some(Metadata {
                            image: Some(image_uri),
                            ..Metadata::default()
                        }),
                    })?,
                    funds: vec![],
                },
                MINT_REPLY,
            );

            available_ids.swap_remove(random_index as usize);
            AVAILABLE_IDS.save(deps.storage, &available_ids)?;
            token_count -= 1; // decrease with each minted NFT
            TOKEN_COUNT.save(deps.storage, &token_count)?;

            Ok(Response::new()
                .add_attribute("action", "transfer")
                .add_attribute("token_id", token_id.to_string().clone())
                .add_submessage(submsg_mint))
        }
    }
}

fn _random_chance(env: &Env, info: &MessageInfo, token_count: u64) -> Uint256 {
    // Combine block height and timestamp
    let seed = format!("{}{}{}", env.block.height, env.block.time, info.sender).into_bytes();

    // Hash the combined seed
    let hash = Sha256::digest(&seed);

    // Convert the hash to Uint256
    let result = Uint256::from_be_bytes(hash.as_slice().try_into().unwrap());

    return result % Uint256::from(token_count) as Uint256;
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        INSTANTIATE_REPLY => {
            let res = parse_reply_instantiate_data(reply).unwrap();
            let contract_address = deps.api.addr_validate(&res.contract_address).unwrap();
            TOKEN.save(deps.storage, &contract_address)?;

            Ok(Response::default())
        }
        MINT_REPLY => Ok(Response::new().add_attribute("Operation", "mint")),
        _ => Err(ContractError::UnrecognizedReply {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Token {} => Ok(to_json_binary(&TOKEN.load(deps.storage)?)?),
        QueryMsg::AvailableIds {} => Ok(to_json_binary(&AVAILABLE_IDS.load(deps.storage)?)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}
