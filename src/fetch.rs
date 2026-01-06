use reqwest::{Error, blocking::Client};

pub const ADOVEISDUMB_URL: &str = "https://a.dove.isdumb.one/list.txt";

pub fn fetch_list() -> Result<String, Error> {
    Client::new()
        .get(ADOVEISDUMB_URL)
        .send()?
        .error_for_status()?
        .text()
}
