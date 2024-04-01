use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::Result;
use chipp_http::{HttpClient, HttpMethod, NoInterceptor};
use log::info;
use md5::Context;
use serde::Deserialize;

pub async fn download_template(target_id: &str) -> Result<PathBuf> {
    let client = HttpClient::new("https://skyplatform.io/api").unwrap();

    let template_version = get_template_version(&client, target_id).await?;

    let path = format!("./template-v{template_version}.db");
    let path = Path::new(&path).to_path_buf();

    if path.exists() {
        info!("template-v{template_version}.db is already downloaded");
        return Ok(path);
    }

    info!("downloading template-v{template_version}.db...");

    let mut request = client.new_request(["template", "publishedVersionDownload"]);
    request.method = HttpMethod::Post;
    request.body = Some(target_id.as_bytes().to_vec());

    let response = client
        .perform_request(request, |req, res| {
            if res.status_code == 200 {
                Ok(res.body)
            } else {
                Err((req, res).into())
            }
        })
        .await?;

    let mut context = Context::new();

    let mut expected_hash = [0u8; 16];
    expected_hash.copy_from_slice(&response[..16]);

    context.consume(&response[16..]);

    let result_hash = context.compute();

    if expected_hash[..] != result_hash[..] {
        panic!("invalid hash");
    }

    let mut template = std::fs::File::create(&path)?;
    template.write_all(&response[16..])?;

    info!("downloaded template-v{template_version}.db");

    Ok(path)
}

async fn get_template_version(client: &HttpClient<NoInterceptor>, target_id: &str) -> Result<u16> {
    #[derive(Deserialize)]
    struct ResponseBody {
        version: u16,
    }

    let mut request = client.new_request(["template", "publishedVersion"]);
    request.method = HttpMethod::Post;
    request.body = Some(target_id.as_bytes().to_vec());

    let body: ResponseBody = client
        .perform_request(request, chipp_http::json::parse_json)
        .await?;

    Ok(body.version)
}
