use std::sync::mpsc;

pub fn exec_req_raw(cli: reqwest::Client, req: reqwest::Request) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    
    rt.block_on(async move {
        let r = cli.execute(req).await.unwrap();

        let headers: String = r
            .headers()
            .iter()
            .map(|(key, value)| {
                format!("{}: {}\r\n", key, value.to_str().unwrap())
            })
            .collect();

        let rets = format!(
            "{} HTTP/1.1 {}\r\n{}\r\n{}",
            r.status().as_str(),
            r.status().canonical_reason().unwrap(),
            headers,
            r.text().await.unwrap()
        );

        tx.send(rets).unwrap();
    });

    rx
}


pub fn exec_req(cli: reqwest::Client, req: reqwest::Request) -> mpsc::Receiver<(String,String,String,String)> {
    let (tx, rx) = mpsc::channel();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    /*version status header body*/
    rt.block_on(async move {
        let r = cli.execute(req).await.unwrap();

        let headers: String = r
            .headers()
            .iter()
            .map(|(key, value)| {
                format!("{}: {}\r\n", key, value.to_str().unwrap())
            })
            .collect();

        let version = "HTTP/1.1".to_string();
        let status = format!(
            "{} {}",
            r.status().as_str(),
            r.status().canonical_reason().unwrap()
        );
        let body = r.text().await.unwrap();

        tx.send((version, status, headers, body)).unwrap();
    });

    rx
}

pub fn exec_req_body(cli: reqwest::Client, req: reqwest::Request) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    
    rt.block_on(async move {
        let r = cli.execute(req).await.unwrap();

        let body = r.text().await.unwrap();

        tx.send(body).unwrap();
    });

    rx
}