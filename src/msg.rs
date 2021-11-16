use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_toolkit::utils::{InitCallback, HandleCallback, Query};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Increment {},
    Reset { count: i32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetCount {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
    pub messages: String,
}

/// InitMsg is a placeholder where we don't take any input
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum VaultInitMsg {
    Init { seed_phrase: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum VaultHandleMsg {
    NewKey {
        key_seed: String,
    },
}

/// QueryMsg is a placeholder where we don't take any input
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum VaultQueryMsg {

    Encrypt {
        key_id: String, // hex string
        user_public_key: String, // public key of user
        data: String,   // num string
    },
    Sign {
        passphrase: String,
        api_key: String,
        key_id: String, // hex string
        data: String,   // num string
    },
    Verify {
        passphrase: String,
        api_key: String,
        key_id: String, // hex string
        data: String,   // num string
    },
    PublicKey {
        key_id: String,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultResponse {
    pub messages: String,
}


impl InitCallback for VaultInitMsg {
    const BLOCK_SIZE: usize = 256;
}

impl HandleCallback for VaultHandleMsg {
    const BLOCK_SIZE: usize = 256;
}

impl Query for VaultQueryMsg {
    const BLOCK_SIZE: usize = 256;
}