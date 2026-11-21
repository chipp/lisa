use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::Result;
use crypto::md5;
use log::info;
use serde::Deserialize;

pub async fn download_template(target_id: &str) -> Result<PathBuf> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let template_version = get_template_version(&client, target_id).await?;

    let path = format!("./template-v{template_version}.db");
    let path = Path::new(&path).to_path_buf();

    if path.exists() {
        info!("template-v{template_version}.db is already downloaded");
        return Ok(path);
    }

    info!("downloading template-v{template_version}.db...");

    let response = download(&client, "https://skyplatform.io/api", target_id).await?;

    let mut expected_hash = [0u8; 16];
    expected_hash.copy_from_slice(&response[..16]);

    let result_hash = md5::digest(&response[16..]);

    if expected_hash[..] != result_hash[..] {
        panic!("invalid hash");
    }

    let mut template = std::fs::File::create(&path)?;
    template.write_all(&response[16..])?;

    info!("downloaded template-v{template_version}.db");

    Ok(path)
}

async fn download(client: &reqwest::Client, base_url: &str, target_id: &str) -> Result<Vec<u8>> {
    let response = client
        .post(format!("{base_url}/template/publishedVersionDownload"))
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(target_id.to_owned())
        .send()
        .await?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(crate::error::Error::HttpStatus(response.status()));
    }
    Ok(response.bytes().await?.to_vec())
}

async fn get_template_version(client: &reqwest::Client, target_id: &str) -> Result<u16> {
    get_version(client, "https://skyplatform.io/api", target_id).await
}

async fn get_version(client: &reqwest::Client, base_url: &str, target_id: &str) -> Result<u16> {
    #[derive(Deserialize)]
    struct ResponseBody {
        version: u16,
    }

    let response = client
        .post(format!("{base_url}/template/publishedVersion"))
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(target_id.to_owned())
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(crate::error::Error::HttpStatus(response.status()));
    }
    let body: ResponseBody = response.json().await?;
    Ok(body.version)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn server(status: &str, body: &str) -> (String, std::thread::JoinHandle<String>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let response = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nLocation: /redirected\r\nConnection: close\r\n\r\n{body}", body.len());
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut buf = [0; 1024];
            loop {
                let n = stream.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                bytes.extend_from_slice(&buf[..n]);
                if let Some(pos) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&bytes[..pos]);
                    let len = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    if bytes.len() >= pos + 4 + len {
                        break;
                    }
                }
            }
            stream.write_all(response.as_bytes()).unwrap();
            String::from_utf8(bytes).unwrap()
        });
        (format!("http://{address}"), handle)
    }

    fn client() -> reqwest::Client {
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn requests_and_statuses() {
        let (base, handle) = server("201 Created", "{\"version\":42}");
        assert_eq!(get_version(&client(), &base, "target").await.unwrap(), 42);
        let request = handle.join().unwrap().to_lowercase();
        assert!(request.starts_with("post /template/publishedversion http/1.1\r\n"));
        assert!(request.contains("content-type: application/x-www-form-urlencoded"));
        assert!(request.ends_with("\r\n\r\ntarget"));
        let (base, handle) = server("200 OK", "content");
        assert_eq!(
            download(&client(), &base, "target").await.unwrap(),
            b"content"
        );
        assert!(handle
            .join()
            .unwrap()
            .starts_with("POST /template/publishedVersionDownload "));
        for status in ["201 Created", "302 Found", "500 Internal Server Error"] {
            let (base, handle) = server(status, "error");
            assert!(download(&client(), &base, "target").await.is_err());
            handle.join().unwrap();
        }
        let (base, handle) = server("302 Found", "{}");
        assert!(get_version(&client(), &base, "target").await.is_err());
        handle.join().unwrap();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        assert!(get_version(&client(), &base, "target").await.is_err());
    }
}
