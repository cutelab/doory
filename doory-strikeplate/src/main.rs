#![feature(alloc_system)] 
#![feature(const_fn)]

extern crate doory_core as core;

extern crate alloc_system;
#[macro_use] extern crate hyper;
#[macro_use] extern crate serde_derive;
extern crate bincode;
extern crate serde;
extern crate serde_json;

use core::EntryAttempt;

use bincode::deserialize;
use hyper::Client;
use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::io;
use std::mem;
use std::net::UdpSocket;
use std::str;
use std::{thread, time};

#[derive(Serialize, Debug)]
struct VaultValidationReq { code: String }

#[derive(Deserialize, Debug)]
struct VaultErrorResp { errors: Vec<String> }

#[derive(Deserialize, Debug)]
struct VaultKVResp { data: VaultKVRespData }

#[derive(Deserialize, Debug)]
struct VaultKVRespData { key: String }

#[derive(Deserialize, Debug)]
struct VaultValidationResp { data: VaultValidationRespData }

#[derive(Deserialize, Debug)]
struct VaultValidationRespData { valid: Option<bool> }

#[derive(Debug)]
enum MyError {
    Err(String)
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

fn validate(attempt: &EntryAttempt) -> Result<(), MyError> {
    let prefix = format!("{}", attempt.pin);
    let code = format!("{}", attempt.totp);

    let client = Client::new();
    let mut url = format!("http://127.0.0.1:8200/v1/secret/prefix/{}", prefix);
    let mut res = client.get(&url)
        .header(XVaultToken(env::var("VAULT_TOKEN").expect("VAULT_TOKEN must be defined"))).send()?;
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
    let key = resp.data.key;

    url = format!("http://127.0.0.1:8200/v1/totp/code/{}", key);

    body = serde_json::to_string(&VaultValidationReq { code: code.to_owned() })?;
    res = client.post(&url).body(&body)
        .header(XVaultToken(env::var("VAULT_TOKEN").expect("VAULT_TOKEN must be defined"))).send()?;
    body = String::new();
    res.read_to_string(&mut body)?;
    match res.status {
        hyper::Ok => {
            let resp: VaultValidationResp = serde_json::from_str(&body)?;
            resp.data.valid
        }
        _ => {
            let resp: VaultErrorResp = serde_json::from_str(&body)?;
            return Err(MyError::Err(resp.errors.join(", ").to_owned()));
        },
    };
    Ok(())
}

fn main() {
    loop {
        let socket = UdpSocket::bind("0.0.0.0:8000").unwrap();

        // read from the socket
        let mut buf = [0; mem::size_of::<EntryAttempt>()];
        let (_amt, _) = socket.recv_from(&mut buf).unwrap();

        let decoded: EntryAttempt = deserialize(&buf).unwrap();

        let code_is_valid = validate(&decoded);
        println!("{:?}", code_is_valid);

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
