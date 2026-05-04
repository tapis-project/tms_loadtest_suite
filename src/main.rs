#![forbid(unsafe_code)]

use std::env;
use std::collections::HashMap;

use lazy_static::lazy_static;
use serde_json::{json, Map, Value};

use goose::prelude::*;
use goose::goose::GooseMethod;
use reqwest::{header::HeaderMap, Body};

// ***************************************************************************
//                             Static Variables
// ***************************************************************************
// ---------------------------------------------------------------------------
// RuntimeCtx:
// ---------------------------------------------------------------------------
#[derive(Debug)]
#[allow(dead_code)]
pub struct RuntimeCtx {
    pub env_vars: HashMap<&'static str, String>,
}

// Lazily initialize our runtime context with a 'static lifetime.
lazy_static! {
    static ref RUNTIME_CTX: RuntimeCtx = init_runtime_context();
}

// ---------------------------------------------------------------------------
// main:
// ---------------------------------------------------------------------------
#[tokio::main]
async fn main() -> Result<(), GooseError> {
    println!("Starting tms_loadtest");

    GooseAttack::initialize()?
        .register_scenario(scenario!("getclient")
            .register_transaction(transaction!(get_tms_client))
        )
        .register_scenario(scenario!("createkey")
            .register_transaction(transaction!(create_tms_key))
        )
        .register_scenario(scenario!("getversion")
            .register_transaction(transaction!(get_tms_version))
        )
        .register_scenario(scenario!("getkey")
            .register_transaction(transaction!(get_tms_key))
        )
        .execute()
        .await?;

    Ok(())
}

// ******************************************************************************
//                               Constants
// ******************************************************************************
// Environment variable names.
const X_TMS_TENANT: &str = "X_TMS_TENANT";
const X_TMS_CLIENT_ID: &str = "X_TMS_CLIENT_ID";
const X_TMS_CLIENT_SECRET: &str = "X_TMS_CLIENT_SECRET";
const X_TMS_ADMIN_ID: &str = "X_TMS_ADMIN_ID";
const X_TMS_ADMIN_SECRET: &str = "X_TMS_ADMIN_SECRET";
const TMS_VERBOSE: &str = "TMS_VERBOSE";                  // default is false
const TMS_PARSE_RESPONSE: &str = "TMS_PARSE_RESPONSE";    // default is false 
const TMS_PUBKEY_FINGERPRINT: &str = "TMS_PUBKEY_FINGERPRINT";
const TMS_PUBKEY_KEYTYPE: &str = "TMS_PUBKEY_KEYTYPE"; // default is ssh-ed25519
const TMS_PUBKEY_USER: &str = "TMS_PUBKEY_USER";
const TMS_PUBKEY_USERID: &str = "TMS_PUBKEY_USERID";
const TMS_PUBKEY_HOST: &str = "TMS_PUBKEY_HOST";

// ******************************************************************************
//                           Transaction Functions
// ******************************************************************************
// ------------------------------------------------------------------------------
// get_tms_client:
// ------------------------------------------------------------------------------
/// Get the default test client information.
async fn get_tms_client(user: &mut GooseUser) -> TransactionResult {

    // Get custom settings from the environment.
    let env_vars = &RUNTIME_CTX.env_vars;
    let verbose = env_vars.get(TMS_VERBOSE).unwrap();
    let parse_response = env_vars.get(TMS_PARSE_RESPONSE).unwrap();

    // TMS inputs.
    let tenant = env_vars.get(X_TMS_TENANT)
        .unwrap_or_else(|| panic!("* FATAL ERROR: Required environment variable '{}' is not set.", X_TMS_TENANT));
    let client_id = env_vars.get(X_TMS_CLIENT_ID)
        .unwrap_or_else(|| panic!("* FATAL ERROR: Required environment variable '{}' is not set.", X_TMS_CLIENT_ID));
    let client_secret = env_vars.get(X_TMS_CLIENT_SECRET)
        .unwrap_or_else(|| panic!("* FATAL ERROR: Required environment variable '{}' is not set.", X_TMS_CLIENT_SECRET));

    // Set the headers needed to issue the get_client call.
    let mut headers = HeaderMap::new();
    headers.insert("X-TMS-TENANT", tenant.parse().unwrap());
    headers.insert("X-TMS-CLIENT-ID", client_id.parse().unwrap());
    headers.insert("X-TMS-CLIENT-SECRET", client_secret.parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());

    // Use the user parameter to generate a reqwest RequestBuilder tailored to the
    // method and targeting our server.
    let reqbuilder = user.get_request_builder(&GooseMethod::Get, 
                                                        "v1/tms/client/testclient1")?;
    
    // Incorporate the lower level reqwest builder into a GooseRequest.
    let goose_request = GooseRequest::builder()
        // Acquire the headers.
        .set_request_builder(reqbuilder.headers(headers))
        // Build the GooseRequest object.
        .build();

    // Use the user parameter to send the GooseRequest and capture response.
    match user.request(goose_request).await?.response {
        Ok(r) => {
            if parse_response != "false" {
                match r.text().await {
                    Ok(content) => {
                        if verbose != "false" {println!("*** Client: {}", content);}
                    },
                    Err(e) => {
                        return TransactionResult::Err(Box::new(TransactionError::Reqwest(e)));
                    }
                };
            }
        },
        Err(e) => {
            return TransactionResult::Err(Box::new(TransactionError::Reqwest(e)));
        }
    };
    //println!("{:#?}", goose_resp);

    Ok(())
}

// ------------------------------------------------------------------------------
// create_tms_key:
// ------------------------------------------------------------------------------
/// Transaction to create an ssh keypair for the user
async fn create_tms_key(user: &mut GooseUser) -> TransactionResult {
    // Get custom settings from the environment.
    let env_vars = &RUNTIME_CTX.env_vars;
    let verbose = env_vars.get(TMS_VERBOSE).unwrap();
    let parse_response = env_vars.get(TMS_PARSE_RESPONSE).unwrap();

    // TMS inputs.
    let tenant = env_vars.get(X_TMS_TENANT)
        .unwrap_or_else(|| panic!("* FATAL ERROR: Required environment variable '{}' is not set.", X_TMS_TENANT));
    let client_id = env_vars.get(X_TMS_CLIENT_ID)
        .unwrap_or_else(|| panic!("* FATAL ERROR: Required environment variable '{}' is not set.", X_TMS_CLIENT_ID));
    let client_secret = env_vars.get(X_TMS_CLIENT_SECRET)
        .unwrap_or_else(|| panic!("* FATAL ERROR: Required environment variable '{}' is not set.", X_TMS_CLIENT_SECRET));

    // Set the headers needed to issue the get_client call.
    let mut headers = HeaderMap::new();
    headers.insert("X-TMS-TENANT", tenant.parse().unwrap());
    headers.insert("X-TMS-CLIENT-ID", client_id.parse().unwrap());
    headers.insert("X-TMS-CLIENT-SECRET", client_secret.parse().unwrap());
    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("Accept", "application/json".parse().unwrap());
    // headers.insert("Connection", "keep-alive".parse().unwrap());

    // Assemble the body of the post request.
    // TODO vary the client, host and host_account values based on user number
    let json = json!({
        "client_user_id": "testuser1", 
        "host": "testhost1", 
        "host_account": "testhostaccount1",
        "num_uses": -1, 
        "ttl_minutes": -1, 
        "key_type": ""
    });
    let body = Body::from(json.to_string());

    // Use the user parameter to generate a reqwest RequestBuilder tailored to the
    // method and targeting our server.
    let reqbuilder = user.get_request_builder(&GooseMethod::Post, 
                                                              "v1/tms/pubkeys/creds")?;

    // Incorporate the lower level reqwest builder into a GooseRequest.
    let goose_request = GooseRequest::builder()
        // Acquire the headers.
        .set_request_builder(reqbuilder.headers(headers).body(body))
        // Build the GooseRequest object.
        .build();

    // Use the user parameter to send the GooseRequest and capture response.
    match user.request(goose_request).await?.response {
        Ok(r) => {
            if parse_response != "false" {
                match r.text().await {
                    Ok(content) => {
                        if verbose != "false" {println!("*** Pubkey: {}", content);}
                    },
                    Err(e) => {
                        return TransactionResult::Err(Box::new(TransactionError::Reqwest(e)));
                    }
                };
            }
        },
        Err(e) => {
            return TransactionResult::Err(Box::new(TransactionError::Reqwest(e)));
        }
    };
    //println!("{:#?}", goose_resp);

    Ok(())
}

// ------------------------------------------------------------------------------
// get_tms_version:
// ------------------------------------------------------------------------------
/// A very simple transaction that simply retrieves version information.
async fn get_tms_version(user: &mut GooseUser) -> TransactionResult {
    // Get custom settings from the environment.
    let env_vars = &RUNTIME_CTX.env_vars;
    let verbose = env_vars.get(TMS_VERBOSE).unwrap();
    let parse_response = env_vars.get(TMS_PARSE_RESPONSE).unwrap();

    // Issue the command.
    let goose_resp = user.get("v1/tms/version").await?;
    match goose_resp.response {
        Ok(r) => {
            if parse_response != "false" {
                match r.text().await 
                {
                    Ok(content) => {
                        if verbose != "false" {println!("*** Version: {}", content);}
                    },
                    Err(e) => {
                        return TransactionResult::Err(Box::new(TransactionError::Reqwest(e)));
                    }
                };
           }
        },
        Err(e) => {
            println!("*** Error: {}", e);
        }
    }

    Ok(())
}

// ------------------------------------------------------------------------------
// get_tms_key:
// ------------------------------------------------------------------------------
// Get ssh keypair for a user
// Public key fingerprint, user, userid and keytype must be provided in env variables
async fn get_tms_key(user: &mut GooseUser) -> TransactionResult {
    // Get custom settings from the environment.
    let env_vars = &RUNTIME_CTX.env_vars;
    let verbose = env_vars.get(TMS_VERBOSE).unwrap();
    let parse_response = env_vars.get(TMS_PARSE_RESPONSE).unwrap();

    // TMS inputs.
    let pubkey_fingerprint = env_vars.get(TMS_PUBKEY_FINGERPRINT)
        .unwrap_or_else(|| panic!("* FATAL ERROR: Required environment variable '{}' is not set.", TMS_PUBKEY_FINGERPRINT));
    let pubkey_host = env_vars.get(TMS_PUBKEY_HOST)
    .unwrap_or_else(|| panic!("* FATAL ERROR: Required environment variable '{}' is not set.", TMS_PUBKEY_HOST));
    let pubkey_user = env_vars.get(TMS_PUBKEY_USER)
    .unwrap_or_else(|| panic!("* FATAL ERROR: Required environment variable '{}' is not set.", TMS_PUBKEY_USER));
    let pubkey_userid = env_vars.get(TMS_PUBKEY_USERID)
    .unwrap_or_else(|| panic!("* FATAL ERROR: Required environment variable '{}' is not set.", TMS_PUBKEY_USERID));
    let pubkey_keytype = env_vars.get(TMS_PUBKEY_KEYTYPE)
    .unwrap_or_else(|| panic!("* FATAL ERROR: Required environment variable '{}' is not set.", TMS_PUBKEY_KEYTYPE));

    // Set the headers needed to issue the get_client call.
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
//    headers.insert("Accept", "application/json".parse().unwrap());

    // Assemble the body of the post request.
    let mut map = Map::new();
    map.insert("user".to_string(), Value::String(pubkey_user.to_string()));
    map.insert("user_uid".to_string(), Value::String(pubkey_userid.to_string()));
    map.insert("keytype".to_string(), Value::String(pubkey_keytype.to_string()));
    map.insert("host".to_string(), Value::String(pubkey_host.to_string()));
    map.insert("public_key_fingerprint".to_string(), Value::String(pubkey_fingerprint.to_string()));

    let json_obj = Value::Object(map);
    if verbose != "false" {println!("*** Json request body: {}", json_obj);}
    let body = Body::from(json_obj.to_string());

    // Use the user parameter to generate a reqwest RequestBuilder tailored to the
    // method and targeting our server.
    let reqbuilder = user.get_request_builder(&GooseMethod::Post, 
                                                              "v1/tms/pubkeys/creds/retrieve")?;

    // Incorporate the lower level reqwest builder into a GooseRequest.
    let goose_request = GooseRequest::builder()
        // Acquire the headers.
        .set_request_builder(reqbuilder.headers(headers).body(body))
        // Build the GooseRequest object.
        .build();

    // Use the user parameter to send the GooseRequest and capture response.
    match user.request(goose_request).await?.response {
        Ok(r) => {
            if parse_response != "false" {
                match r.text().await {
                    Ok(content) => {
                        if verbose != "false" {println!("*** Pubkey: {}", content);}
                    },
                    Err(e) => {
                        return TransactionResult::Err(Box::new(TransactionError::Reqwest(e)));
                    }
                };
            }
        },
        Err(e) => {
            return TransactionResult::Err(Box::new(TransactionError::Reqwest(e)));
        }
    };
    //println!("{:#?}", goose_resp);

    Ok(())
}

// ******************************************************************************
//                             Private Utilities
// ******************************************************************************
// ------------------------------------------------------------------------------
// init_runtime_context:
// ------------------------------------------------------------------------------
pub fn init_runtime_context() -> RuntimeCtx {
    RuntimeCtx {env_vars: get_env_vars()}
}

// ------------------------------------------------------------------------------
// get_env_vars:
// ------------------------------------------------------------------------------
fn get_env_vars() -> HashMap<&'static str, String> {
    // Create the environment variable hashmap.
    let mut env_map = HashMap::new();

    // ----- X_TMS_TENANT
    let val = env::var(X_TMS_TENANT).unwrap_or_else(
                                |_| {"".to_string()});
    if !val.is_empty() {env_map.insert(X_TMS_TENANT, val);}

    // ----- X_TMS_CLIENT_ID
    let val = env::var(X_TMS_CLIENT_ID).unwrap_or_else(
                                |_| {"".to_string()});
    if !val.is_empty() {env_map.insert(X_TMS_CLIENT_ID, val);}

    // ----- X_TMS_CLIENT_SECRET
    let val = env::var(X_TMS_CLIENT_SECRET).unwrap_or_else(
                                |_| {"".to_string()});
    if !val.is_empty() {env_map.insert(X_TMS_CLIENT_SECRET, val);}

    // ----- X_TMS_ADMIN_ID
    let val = env::var(X_TMS_ADMIN_ID).unwrap_or_else(
                                |_| {"".to_string()});
    if !val.is_empty() {env_map.insert(X_TMS_ADMIN_ID, val);}

    // ----- X_TMS_ADMIN_SECRET
    let val = env::var(X_TMS_ADMIN_SECRET).unwrap_or_else(
                                |_| {"".to_string()});
    if !val.is_empty() {env_map.insert(X_TMS_ADMIN_SECRET, val);}

    // ----- TMS_VERBOSE
    // Set to the default "false" if not found; anything other than  
    // "false" will trigger printing.  This only takes effect if 
    // TMS_PARSE_RESPONSE = true.
    let val = env::var(TMS_VERBOSE).unwrap_or_else(
                                |_| {"false".to_string()});
    env_map.insert(TMS_VERBOSE, val);

    // ----- TMS_PARSE_RESPONSE
    // Set to the default "false" if not found; anything other than  
    // "false" will trigger the response to be parsed on receipt.
    let val = env::var(TMS_PARSE_RESPONSE).unwrap_or_else(
                                |_| {"false".to_string()});
    env_map.insert(TMS_PARSE_RESPONSE, val);

    // ----- TMS getkey settings
    let val = env::var(TMS_PUBKEY_USER).unwrap_or_else(
        |_| {"".to_string()});
    if !val.is_empty() {env_map.insert(TMS_PUBKEY_USER, val);}
    let val = env::var(TMS_PUBKEY_USERID).unwrap_or_else(
        |_| {"".to_string()});
    if !val.is_empty() {env_map.insert(TMS_PUBKEY_USERID, val);}
    let val = env::var(TMS_PUBKEY_HOST).unwrap_or_else(
        |_| {"".to_string()});
    if !val.is_empty() {env_map.insert(TMS_PUBKEY_HOST, val);}
    let val = env::var(TMS_PUBKEY_KEYTYPE).unwrap_or_else(
        |_| {"ssh-ed25519".to_string()});
    if !val.is_empty() {env_map.insert(TMS_PUBKEY_KEYTYPE, val);}
    let val = env::var(TMS_PUBKEY_FINGERPRINT).unwrap_or_else(
        |_| {"".to_string()});
    if !val.is_empty() {env_map.insert(TMS_PUBKEY_FINGERPRINT, val);}


    // Always output the environment settings.
    // NOTE: Secrets are printed out!
    println!("\n-------------------------------------------");
    println!("TMS Environment Map: {:#?}", env_map);
    println!("-------------------------------------------\n");
    
    env_map
}
