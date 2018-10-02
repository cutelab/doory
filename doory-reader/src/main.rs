extern crate doory_core as core;

extern crate bincode;
#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate nfc_oath;
extern crate otpauth_uri;

use core::EntryAttempt;

use bincode::serialize;
use chan_signal::Signal;
use nfc_oath::{OathController, OathCredential};
use otpauth_uri::parser::parse_otpauth_label;
use otpauth_uri::types::OTPLabel;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::UdpSocket;
use std::path::Path;
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use nfc_oath::{OathAlgo, OathCredential, OathType};
    use otpauth_uri::types::OTPLabel;

    #[test]
    fn select_credential_works() {
        let list = vec![
            OathCredential::new(
                "Domain:your@email.com",
                OathType::Totp,
                false,
                OathAlgo::Sha1,
            ),
            OathCredential::new("Doory:ev", OathType::Totp, false, OathAlgo::Sha1),
        ];
        let answer = (
            OTPLabel {
                issuer: Some("Doory".to_string()),
                accountname: "ev".to_string(),
            },
            OathCredential::new("Doory:ev", OathType::Totp, false, OathAlgo::Sha1),
        );

        let key = ::select_credential(list).unwrap();
        assert_eq!(key.0.accountname, answer.0.accountname);
        assert_eq!(key.1, answer.1);
    }
}

fn select_credential(keys: Vec<OathCredential>) -> Option<(OTPLabel, OathCredential)> {
    keys.into_iter()
        .filter_map(|key| Some((parse_otpauth_label(&key.name).ok()?, key)))
        .find(|(label, _key)| label.issuer == Some("Doory".to_string()))
}

fn read_lines(s: chan::Sender<String>) {
    let fname = "/dev/ttyACM0";
    let path = Path::new(fname);
    let port = File::open(&path).unwrap();

    let buf_port = BufReader::new(port);

    for line in buf_port.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_err) => continue,
        };

        s.send(line);
    }
}

fn send_attempt(socket: &UdpSocket, attempt: &EntryAttempt) {
    let encoded: Vec<u8> = match serialize(attempt) {
        Ok(encoded) => encoded,
        Err(err) => {
            println!("error while serializing entry attempt: {}", err.to_string());
            return;
        }
    };
    debug!("sending {:?}", attempt);
    socket
        .send_to(&encoded[..], "192.168.1.1:8000")
        .expect("couldn't send data");
}

fn main() {
    env_logger::init().unwrap();

    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    let (keypad_tx, keypad_rx) = chan::sync(0);
    thread::spawn(move || read_lines(keypad_tx));

    let controller = OathController::new().unwrap();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    println!("waiting for keypad input or nfc token...");
    loop {
        chan_select! {
            default => {
                if !controller.poll(Some(Duration::from_millis(100))) {
                    continue
                }

                let creds = match controller.list() {
                    Ok(creds) => creds,
                    Err(err) => {
                        println!("error while reading nfc credentials: {}", err.to_string());
                        thread::sleep(Duration::from_millis(500));
                        continue
                    }
                };

                if let Some((label, mut cred)) = select_credential(creds) {
                    cred = controller.calculate(cred);
                    let oathcode = match cred.code {
                        Ok(oathcode) => oathcode,
                        Err(err) => {
                            println!("error while calculating nfc credential: {}", err.to_string());
                            thread::sleep(Duration::from_millis(500));
                            continue
                        }
                    };
                    let attempt = EntryAttempt::Card{account: label.accountname, code: String::from(format!("{}", oathcode))};
                    send_attempt(&socket, &attempt);
                    thread::sleep(Duration::from_secs(1));
                }
            },
            keypad_rx.recv() -> line => {
                let line: Option<String> = line;
                let line = match line {
                    Some(line) => line,
                    None => continue
                };

                let raw_code_parts: Vec<&str> = line.split("*").collect();

                let attempt = match raw_code_parts.len() {
                    1 => EntryAttempt::Static(String::from(raw_code_parts[0])),
                    2 => EntryAttempt::OTP{pin: String::from(raw_code_parts[0]), code: String::from(raw_code_parts[1])},
                    _ => continue,
                };

                send_attempt(&socket, &attempt);
            },
            signal.recv() -> _signal => {
                break; // Exit cleanly on INT or TERM
            },
        }
    }
    // NOTE: If we don't cleanly close it here, the controller becomes unusable until reboot :D
    controller.close();
}
