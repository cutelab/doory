extern crate doory_core as core;

#[macro_use] extern crate chan;
#[macro_use] extern crate log;
extern crate bincode;
extern crate chan_signal;
extern crate env_logger;
extern crate nfc_oath;

use core::EntryAttempt;

use bincode::{serialize, Infinite};
use chan_signal::Signal;
use nfc_oath::{OathController, OathCredential, OathType, OathAlgo};
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::net::UdpSocket;

fn read_lines(s: chan::Sender<String>) {
    let fname = "/dev/ttyACM0";
    let path = Path::new(fname);
    let port = File::open(&path).unwrap();

    let buf_port = BufReader::new(port);

    for line in buf_port.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_err) => continue
        };

        s.send(line);
    }
}

fn main() {
    env_logger::init().unwrap();

    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    let (s, r) = chan::sync(0);
    thread::spawn(move || read_lines(s));

    let controller = OathController::new().unwrap();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    loop {
        println!("waiting for keypad input...");
        chan_select! {
            signal.recv() -> _signal => {
                break; // Exit cleanly on INT or TERM
            },
            r.recv() -> line => {
                let line: Option<String> = line;
                let line = match line {
                    Some(line) => line,
                    None => continue
                };

                let mut attempt = EntryAttempt{ pin:0, totp:0 };
                let pin: String;
                let totp: String;

                let raw_code_parts: Vec<&str> = line.split("*").collect();

                if raw_code_parts.len() == 1 {
                    println!("polling for nfc token...");
                    if !controller.poll(Some(Duration::from_secs(5))) {
                        continue
                    }

                    //let mut cred = OathCredential::new("Doory:doory@cutelab.house", OathType::TOTP, false, OathAlgo::SHA256);
                    let mut cred = OathCredential::new("FidesmoOTPTutorial:tutorial@fidesmo.com", OathType::Totp, false, OathAlgo::Sha256);
                    cred = controller.calculate(cred);
                    let oathcode = match cred.code {
                        Ok(oathcode) => oathcode,
                        Err(err) => {
                            println!("error while reading nfc credential: {}", err.to_string());
                            continue
                        }
                    };

                    pin = String::from(raw_code_parts[0]);
                    totp = String::from(format!("{}", oathcode));
                } else if raw_code_parts.len() == 2 {
                    pin = String::from(raw_code_parts[0]);
                    totp = String::from(raw_code_parts[1]);
                } else {
                    continue
                }
                attempt.pin = match pin.trim_matches(0 as char).parse::<u32>() {
                    Ok(pin) => pin,
                    Err(err) => {
                        println!("error while parsing pin: {}", err.to_string());
                        continue
                    }
                };
                attempt.totp = match totp.trim_matches(0 as char).parse::<u32>() {
                    Ok(totp) => totp,
                    Err(err) => {
                        println!("error while parsing totp: {}", err.to_string());
                        continue
                    }
                };

                let encoded: Vec<u8> = match serialize(&attempt, Infinite) {
                    Ok(encoded) => encoded,
                    Err(err) => {
                        println!("error while serializing entry attempt: {}", err.to_string());
                        continue
                    }
                };
                debug!("sending pin {} totp {}", pin, totp);
                socket.send_to(&encoded[..], "192.168.1.1:8000").expect("couldn't send data");
            }
        }
    }
    // NOTE: If we don't cleanly close it here, the controller becomes unusable until reboot :D
    controller.close();
}
