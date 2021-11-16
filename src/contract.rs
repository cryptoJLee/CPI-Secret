use cosmwasm_std::{Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, StdError, StdResult, Storage, to_binary};
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};

use crate::msg::{CountResponse, HandleMsg, InitMsg, QueryMsg, VaultHandleMsg, VaultInitMsg, VaultQueryMsg, VaultResponse};
use crate::state::{config, config_read, State};

const VAULT_CONTRACT_HASH: &str = "494d93a5c4b6024e582032388f1fd9e41c6dd1d1cb826c42331407dfd5a7bb0d";
const VAULT_ADDRESS: &str = "secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        count: msg.count,
        owner: deps.api.canonical_address(&env.message.sender)?,
    };

    config(&mut deps.storage).save(&state)?;

    // initialize the secret vault contract
    let vault_init_msg = VaultInitMsg::Init{
        seed_phrase: "skdfjkdsf".to_string()
   };
   
   let cosmos_msg = vault_init_msg.to_cosmos_msg(
       "my counter".to_string(),
       1,
       VAULT_CONTRACT_HASH.to_string(),
       None,
   )?;
   
   Ok(InitResponse {
       messages: vec![cosmos_msg],
       log: vec![],
   })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Increment {} => try_increment(deps, env),
        HandleMsg::Reset { count } => try_reset(deps, env, count),
    }
}

pub fn try_increment<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
) -> StdResult<HandleResponse> {
    config(&mut deps.storage).update(|mut state| {
        state.count += 1;
        // debug_print!("count = {}", state.count);
        Ok(state)
    })?;

    // debug_print("count incremented successfully");

    let newkey_msg = VaultHandleMsg::NewKey {
        key_seed: "SERVER".to_string()
    };
    
    let cosmos_msg = newkey_msg.to_cosmos_msg(
        VAULT_CONTRACT_HASH.to_string(),
        HumanAddr(VAULT_ADDRESS.to_string()),
        None,
    )?;
    
    Ok(HandleResponse {
        messages: vec![cosmos_msg],
        log: vec![],
        data: None,
    })
}

pub fn try_reset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    count: i32,
) -> StdResult<HandleResponse> {
    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
    config(&mut deps.storage).update(|mut state| {
        if sender_address_raw != state.owner {
            return Err(StdError::Unauthorized { backtrace: None });
        }
        state.count = count;
        Ok(state)
    })?;
    // debug_print("count reset successfully");
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<CountResponse> {
    let state = config_read(&deps.storage).load()?;

    let get_sign = VaultQueryMsg::Sign {
        passphrase: "".to_string(),
        api_key: "".to_string(),
        key_id: "SERVER_KEY_ID".to_string(), // hex string
        data: "server msg".to_string(),   // num string
    };
    let vault_response: VaultResponse = get_sign.query(
        &deps.querier,
        VAULT_CONTRACT_HASH.to_string(),
        HumanAddr(VAULT_ADDRESS.to_string())
    )?;

    Ok(CountResponse { count: state.count, messages: vault_response.messages })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // anyone can increment
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Increment {};
        let _res = handle(&mut deps, env, msg).unwrap();

        // should increase counter by 1
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies(20, &coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();

        // not anyone can reset
        let unauth_env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let res = handle(&mut deps, unauth_env, msg);
        match res {
            Err(StdError::Unauthorized { .. }) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_env = mock_env("creator", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let _res = handle(&mut deps, auth_env, msg).unwrap();

        // should now be 5
        let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
