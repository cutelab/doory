#![feature(alloc_system)]
#![feature(const_fn)]

extern crate doory_core as core;

extern crate alloc_system;
extern crate bincode;
extern crate env_logger;
#[macro_use]
extern crate hyper;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use core::EntryAttempt;

use bincode::deserialize;
use hyper::Client;
use std::env;
use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};
use std::net::UdpSocket;
use std::str;
use std::{thread, time};

#[derive(Serialize, Debug)]
struct VaultValidationReq {
    code: String,
}

#[derive(Deserialize, Debug)]
struct VaultErrorResp {
    errors: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct VaultKVResp {
    data: VaultKVRespData,
}

#[derive(Deserialize, Debug)]
struct VaultKVRespData {
    key: String,
}

#[derive(Deserialize, Debug)]
struct VaultValidationResp {
    data: VaultValidationRespData,
}

#[derive(Deserialize, Debug)]
struct VaultValidationRespData {
    valid: bool,
}

#[derive(Debug)]
enum MyError {
    Err(String),
}

impl From<hyper::Error> for MyError {
    fn from(e: hyper::Error) -> MyError {
        MyError::Err(e.to_string())
    }
}
impl From<io::Error> for MyError {
    fn from(e: io::Error) -> MyError {
        MyError::Err(e.to_string())
    }
}
impl From<serde_json::Error> for MyError {
    fn from(e: serde_json::Error) -> MyError {
        MyError::Err(e.to_string())
    }
}

header! { (XVaultToken, "X-Vault-Token") => [String] }

fn validate_code(account: &str, code: &str) -> Result<(), MyError> {
    let client = Client::new();
    let url = format!("http://127.0.0.1:8200/v1/totp/code/{}", account);

    let mut body = serde_json::to_string(&VaultValidationReq {
        code: code.to_string(),
    })?;
    let mut res = client
        .post(&url)
        .body(&body)
        .header(XVaultToken(
            env::var("VAULT_TOKEN").expect("VAULT_TOKEN must be defined"),
        ))
        .send()?;
    body = String::new();
    res.read_to_string(&mut body)?;
    match res.status {
        hyper::Ok => {
            let resp: VaultValidationResp = serde_json::from_str(&body)?;
            if !resp.data.valid {
                return Err(MyError::Err("not valid".to_owned()));
            }
        }
        _ => {
            let resp: VaultErrorResp = serde_json::from_str(&body)?;
            return Err(MyError::Err(resp.errors.join(", ").to_owned()));
        }
    };
    Ok(())
}

fn validate(attempt: &EntryAttempt) -> Result<(), MyError> {
    match attempt {
        &EntryAttempt::Static(ref code) => {
            let client = Client::new();
            let url = format!("http://127.0.0.1:8200/v1/secret/static/{}", code);
            let mut res = client
                .get(&url)
                .header(XVaultToken(
                    env::var("VAULT_TOKEN").expect("VAULT_TOKEN must be defined"),
                ))
                .send()?;
            let mut body = String::new();
            res.read_to_string(&mut body)?;
            match res.status {
                hyper::Ok => (),
                _ => {
                    let resp: VaultErrorResp = serde_json::from_str(&body)?;
                    return Err(MyError::Err(resp.errors.join(", ").to_owned()));
                }
            }
            Ok(())
        }
        &EntryAttempt::OTP { ref pin, ref code } => {
            let client = Client::new();
            let mut url = format!("http://127.0.0.1:8200/v1/secret/prefix/{}", pin);
            let mut res = client
                .get(&url)
                .header(XVaultToken(
                    env::var("VAULT_TOKEN").expect("VAULT_TOKEN must be defined"),
                ))
                .send()?;
            let mut body = String::new();
            res.read_to_string(&mut body)?;
            match res.status {
                hyper::Ok => (),
                _ => {
                    let resp: VaultErrorResp = serde_json::from_str(&body)?;
                    return Err(MyError::Err(resp.errors.join(", ").to_owned()));
                }
            }
            let resp: VaultKVResp = serde_json::from_str(&body)?;

            validate_code(&resp.data.key, code)
        }
        &EntryAttempt::Card {
            ref account,
            ref code,
        } => validate_code(account, code),
    }
}

fn main() {
    env_logger::init().unwrap();

    loop {
        let socket = UdpSocket::bind("0.0.0.0:8000").unwrap();

        // read from the socket
        let mut buf = [0; 1024];
        let (_amt, _) = socket.recv_from(&mut buf).unwrap();

        let decoded: EntryAttempt = deserialize(&buf).unwrap();

        let code_is_valid = validate(&decoded);
        debug!("{:?}: {:?}", decoded, code_is_valid);

        if let Ok(()) = code_is_valid {
            let mut file = OpenOptions::new()
                .write(true)
                .open("/sys/class/leds/rt2800soc-phy0::radio/brightness")
                .unwrap();
            file.write(b"1").unwrap();
            file.flush().unwrap();

            let ten_secs = time::Duration::from_secs(10);

            thread::sleep(ten_secs);
            file.write(b"0").unwrap();
            file.flush().unwrap();
        }
    } // the socket is closed here
}
