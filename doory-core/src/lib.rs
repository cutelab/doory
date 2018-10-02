extern crate serde;
#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum EntryAttempt {
    Static(String),
    OTP { pin: String, code: String },
    Card { account: String, code: String },
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
