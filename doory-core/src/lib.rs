#[macro_use]
extern crate serde_derive;
extern crate serde;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct EntryAttempt {
    pub pin: u32,
    pub totp: u32,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
