use crate::app::backend::dbutils;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub fn get_new_from_last_id(
    last_id: usize,
    url: &str,
    secret: &str,
) -> Result<String, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_bytes(b"rp_auth").unwrap(),
        HeaderValue::from_bytes(format!("{}", secret).as_bytes()).unwrap(),
    );
    let cli = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .default_headers(headers)
        .build()?;
    let r = cli
        .get(format!("https://{}/api/requests/{}", url, last_id))
        .send();

    match r {
        Ok(r) => r.text(),
        Err(e) => {
            println!("{}", e);
            Err(e)
        }
    }
}

pub fn parse_result(s: String) -> Vec<dbutils::HistLine> {
    let res: Vec<dbutils::HistLine> = serde_json::from_str(&s).unwrap();
    res
}
