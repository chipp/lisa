use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::Result;
use hyper::{body::HttpBody, client::HttpConnector, Body, Client, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;
use md5::Context;
use serde::Deserialize;

pub async fn download_template(target_id: &str) -> Result<PathBuf> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let template_version = get_template_version(&client, target_id).await?;

    let request = Request::builder()
        .method(Method::POST)
        .uri("https://skyplatform.io/api/template/publishedVersionDownload")
        .body(Body::from(target_id.to_string()))
        .unwrap();

    let mut response = client.request(request).await?;

    if response.status() != StatusCode::OK {
        panic!("unable to get template");
    }

    let mut context = Context::new();

    let path = format!("./template-v{}.db", template_version);
    let path = Path::new(&path).to_path_buf();

    if path.exists() {
        return Ok(path);
    }

    const MAX_ALLOWED_RESPONSE_SIZE: u64 = 10 * 1024 * 1024;

    let response_content_length = match response.body().size_hint().upper() {
        Some(v) => v,
        None => MAX_ALLOWED_RESPONSE_SIZE + 1,
    };

    if response_content_length >= MAX_ALLOWED_RESPONSE_SIZE {
        panic!("too big file {}", response_content_length);
    }

    let mut template = std::fs::File::create(&path)?;

    let mut expected_hash = [0u8; 16];
    let mut hash_extracted = false;

    while let Some(chunk) = response.body_mut().data().await {
        let data = chunk?;
        let mut data = &data[..];

        if !hash_extracted {
            expected_hash.copy_from_slice(&data[..16]);
            hash_extracted = true;

            data = &data[16..];
        }

        context.consume(data);
        template.write_all(data)?;
    }

    let result_hash = context.compute();

    if &expected_hash[..] != &result_hash[..] {
        panic!("invalid hash");
    }

    Ok(path)
}

async fn get_template_version(
    client: &Client<HttpsConnector<HttpConnector>, Body>,
    target_id: &str,
) -> Result<u16> {
    #[derive(Deserialize)]
    struct ResponseBody {
        version: u16,
    }

    let request = Request::builder()
        .method(Method::POST)
        .uri("https://skyplatform.io/api/template/publishedVersion")
        .body(Body::from(target_id.to_string()))
        .unwrap();

    let response = client.request(request).await?;

    const MAX_ALLOWED_RESPONSE_SIZE: u64 = 4096;

    let response_content_length = match response.body().size_hint().upper() {
        Some(v) => v,
        None => MAX_ALLOWED_RESPONSE_SIZE + 1,
    };

    if response_content_length >= MAX_ALLOWED_RESPONSE_SIZE {
        panic!(
            "template file is bigger than expected {} >= {}",
            response_content_length, MAX_ALLOWED_RESPONSE_SIZE
        );
    }

    let body: ResponseBody =
        serde_json::from_slice(&hyper::body::to_bytes(response.into_body()).await?)?;

    Ok(body.version)
}
