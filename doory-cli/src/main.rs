extern crate doory_core as core;

extern crate bincode;
extern crate otpauth_uri;
extern crate time;

use core::EntryAttempt;

use bincode::serialize;
use std::env;
use std::io;
use std::io::Read;
use std::net::UdpSocket;
use std::time::SystemTime;

use otpauth_uri::parser::parse_otpauth_uri;
use otpauth_uri::types::OTPGenerator;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();

    let uri = parse_otpauth_uri(buffer.trim_right());
    match uri {
        Ok(uri) => {
            let account = uri.label.accountname.clone();
            let key: OTPGenerator = uri.into();
            if let OTPGenerator::TOTPGenerator(key) = key {
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let code = key.generate(timestamp);

                let attempt = EntryAttempt::Card { account, code };

                match serialize(&attempt) {
                    Ok(encoded) => {
                        println!("sending {:?}", attempt);
                        socket
                            .send_to(
                                &encoded[..],
                                env::var("STRIKEPLATE_ADDR")
                                    .expect("STRIKEPLATE_ADDR must be defined"),
                            )
                            .expect("couldn't send data");
                    }
                    Err(err) => {
                        println!("error while serializing entry attempt: {}", err.to_string());
                    }
                };
            } else {
                return;
            }
        }
        Err(_) => println!("error must provide a valid otpauth uri"),
    }
}
