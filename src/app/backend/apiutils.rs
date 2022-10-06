use crate::app::backend::dbutils;
use crate::app::backend::requestor;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub fn get_new_from_last_id(
    last_id: usize,
    url: &str,
    secret: &str,
) -> Result<String, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_bytes(b"Authentication").unwrap(),
        HeaderValue::from_bytes(format!("Bearer {}", secret).as_bytes()).unwrap(),
    );
    let cli = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .default_headers(headers)
        .build()?;

    let r = cli.request(reqwest::Method::from_bytes(b"GET").unwrap(), format!("https://{}/api/requests/{}", url, last_id))
        .build()?;
    
    let rx = requestor::exec_req_body(cli, r);

    Ok(rx.recv().unwrap())
}

pub fn parse_result(s: String) -> Vec<dbutils::HistLine> {
    let res: Vec<dbutils::HistLine> = serde_json::from_str(&s).unwrap();
    res
}
