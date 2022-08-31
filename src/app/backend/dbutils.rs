use rusqlite::{Connection, Error};
use std::path::Path;

pub fn try_open_conn(projectpath: &str) -> Result<Connection, Error> {
    let fpath = format!("{}/hist.db", projectpath);
    Connection::open(fpath)
}

pub fn is_valid_project_path(fpath: &String) -> bool {
    if Path::new(&fpath).exists() {
        return try_open_conn(&fpath).is_ok();
    }

    false
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug)]
pub struct HistLine {
    pub id: usize,
    pub uri: String,
    pub method: String,
    pub params: bool,
    pub status: usize,
    pub size: usize,
    pub raw: String,
    pub ssl: bool,
    pub response: String,
    pub response_time: String,
    pub host: String,
}

pub fn get_new_from_last_id(last_id: usize, path: &String) -> Option<Vec<HistLine>> {
    if let Ok(conn) = try_open_conn(path) {
        let mut out = vec![];

        let mut stmt = conn
            .prepare("SELECT * FROM history WHERE id > ? ORDER BY id Asc")
            .unwrap();
        let rows = stmt
            .query_map([last_id], |row| {
                Ok(HistLine {
                    id: row.get(0).unwrap(),
                    uri: row.get(1).unwrap(),
                    method: row.get(2).unwrap(),
                    size: row.get(5).unwrap(),
                    params: matches!(row.get(3).unwrap(), 1),
                    status: row.get(4).unwrap(),
                    raw: row.get(6).unwrap(),
                    ssl: row.get(7).unwrap(),
                    response: row.get(8).unwrap(),
                    response_time: row.get(9).unwrap(),
                    host: row
                        .get::<usize, String>(6)
                        .unwrap()
                        .split("ost: ")
                        .skip(1)
                        .take(1)
                        .collect::<String>()
                        .split("\r\n")
                        .take(1)
                        .collect::<String>(),
                })
            })
            .unwrap();

        for row in rows {
            out.push(row.unwrap());
        }

        return Some(out);
    }

    None
}
