use poll_promise::Promise;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Error;
use reqwest::Method;

pub struct BatchRequest {
    pub headers: Vec<String>,
}

pub struct Request {
    pub idx: usize,
    pub url: String,
    pub method: Method,
    pub body: Vec<u8>,
    pub headers: HeaderMap,
}

impl Request {
    pub fn from(r: &Request) -> Self {
        let mut headers = HeaderMap::new();
        for (k, v) in r.headers.iter() {
            headers.append(k, HeaderValue::from_str(v.to_str().unwrap()).unwrap());
        }

        Self {
            idx: r.idx,
            url: r.url.to_string(),
            method: Method::from(&r.method),
            body: r.body.to_vec(),
            headers,
        }
    }

    pub fn from_strings(v: Vec<String>, ssl: bool, target: String) -> Vec<Self> {
        let mut out = vec![];
        for (idx, request) in v.iter().enumerate() {
            let method = request.split(' ').take(1).collect::<String>();
            let uri = request.split(' ').skip(1).take(1).collect::<String>();
            let url = format!("{}://{}{}", if ssl { "https" } else { "http" }, target, uri);
            let body = request
                .split("\r\n\r\n")
                .skip(1)
                .take(1)
                .collect::<String>()
                .as_bytes()
                .to_vec();
            let mut headers = HeaderMap::new();
            for header in request
                .split("\r\n")
                .skip(1)
                .map_while(|x| if !x.is_empty() { Some(x) } else { None })
                .collect::<Vec<&str>>()
            {
                let name = reqwest::header::HeaderName::from_bytes(
                    header.split(": ").take(1).collect::<String>().as_bytes(),
                )
                .unwrap();
                headers.insert(
                    name,
                    header
                        .split(": ")
                        .skip(1)
                        .collect::<String>()
                        .parse()
                        .unwrap(),
                );
            }
            let method = reqwest::Method::from_bytes(method.as_bytes()).unwrap();
            out.push(Self {
                idx,
                url,
                method,
                body,
                headers,
            });
        }
        out
    }
}

pub type SuccessTuple = (usize, String, String, String, String);
pub type ErrTuple = (usize, Error);
pub type RunResultUnitary = Result<SuccessTuple, ErrTuple>;
pub type VecPromiseType = Vec<Promise<Vec<RunResultUnitary>>>;
impl BatchRequest {
    pub fn run(payloads: &Vec<Request>, promises: &mut VecPromiseType) {
        let mut idx_worker = 0;
        let batch_size = if payloads.len() < 1000 {
            250
        } else {
            payloads.len() / 1000 + usize::from(payloads.len() % 1000 != 0)
        };
        for batch in Self::split(payloads, batch_size) {
            let promise =
                Promise::spawn_thread(&format!("rq{}", idx_worker), move || Self::by_batch(batch));
            promises.push(promise);
            idx_worker += 1
        }
    }

    fn split(payloads: &Vec<Request>, how_many: usize) -> Vec<Vec<Request>> {
        (0..payloads.len())
            .step_by(if payloads.len() > how_many {
                how_many
            } else {
                payloads.len()
            })
            .map(|n| {
                let mut v = vec![];
                let end = n + if payloads[n..].len() > how_many {
                    how_many
                } else {
                    payloads[n..].len()
                };
                for p in &payloads[n..end] {
                    v.push(Request::from(p));
                }
                v
            })
            .collect::<Vec<Vec<Request>>>()
    }

    fn by_batch(reqs: Vec<Request>) -> Vec<RunResultUnitary> {
        let mut out = vec![];
        for req in reqs {
            let cli = reqwest::blocking::Client::builder()
                .danger_accept_invalid_certs(true)
                .default_headers(req.headers)
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap();

            out.push({
                let r = cli.request(req.method, req.url).body(req.body).send();
                let idx = req.idx;
                match r {
                    Err(e) => Err((idx, e)),
                    Ok(r) => {
                        let headers: String = r
                            .headers()
                            .iter()
                            .map(|(key, value)| format!("{}: {}\r\n", key, value.to_str().unwrap()))
                            .collect();
                        let version = format!("{:?}", r.version());
                        let status = format!(
                            "{} {}",
                            r.status().as_str(),
                            r.status().canonical_reason().unwrap()
                        );
                        Ok((idx, version, status, headers, r.text().unwrap()))
                    }
                }
            });
        }

        out
    }
}
