use std::{borrow::Cow, fmt, str::FromStr};
use std::{net::SocketAddr, sync::Arc};

use chrono::Duration;
use log::{error, info, trace};

use bytes::Buf;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Method, Request, Response, Server, StatusCode};
use url::Url;

use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_urlencoded::de;

use tokio::{
    io::{AsyncBufReadExt, BufReader, BufWriter, ReadHalf},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task,
};

use alice::{Device, DeviceCapability, DeviceProperty, DeviceType, Mode, ModeFunction};
use alice::{StateRequest, StateResponse};
use alice::{UpdateStateRequest, UpdateStateResponse};

use lisa::{state_for_device, update_devices_state, Commander, DeviceId, Room};

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init_timed();

    let commander = Arc::from(Mutex::from(Commander::new()));

    let (server, tcp) = tokio::try_join!(
        task::spawn(listen_api(commander.clone())),
        task::spawn(listen_tcp(commander.clone()))
    )?;

    server?;
    tcp?;

    Ok(())
}

async fn listen_api(cmd: Arc<Mutex<Commander>>) -> Result<()> {
    let cmd = cmd.clone();

    let make_svc = make_service_fn(move |_| {
        let cmd = cmd.clone();
        async move {
            Ok::<_, ErasedError>(service_fn(move |req| {
                let cmd = cmd.clone();
                async move { service(req, cmd).await }
            }))
        }
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);
    info!("Listening http://{}", addr);

    server.await?;

    Ok(())
}

async fn listen_tcp(cmd: Arc<Mutex<Commander>>) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    let tcp_listener = TcpListener::bind(addr).await?;

    info!("Listening socket {}", addr);

    let mut handles = vec![];

    loop {
        match tcp_listener.accept().await {
            Ok((stream, addr)) => {
                let (read, write) = tokio::io::split(stream);
                let mut cmd = cmd.clone().lock_owned().await;
                cmd.set_stream(BufWriter::new(write));

                handles.push(task::spawn(read_from_socket(read, addr)))
            }
            Err(error) => eprintln!("{}", error),
        }
    }
}

async fn read_from_socket(stream: ReadHalf<TcpStream>, addr: SocketAddr) -> Result<()> {
    info!("Got a new client {}", addr);

    let mut reader = BufReader::new(stream);

    loop {
        trace!("waiting for message...");

        let mut buffer = vec![];
        let bytes_count = reader.read_until(b'\n', &mut buffer).await?;

        trace!("received some bytes {}", bytes_count);

        if bytes_count == 0 {
            break;
        }

        unsafe {
            println!("{}", std::str::from_utf8_unchecked(&buffer));
        }

        match serde_json::from_slice::<elisheva::SensorData>(&buffer) {
            Ok(value) => println!("{:?}", value),
            Err(error) => eprintln!("{}", error),
        }
    }

    info!("Client did disconnect {}", addr);

    Ok(())
}

async fn service(request: Request<Body>, cmd: Arc<Mutex<Commander>>) -> Result<Response<Body>> {
    match (request.uri().path(), request.method()) {
        ("/auth", &Method::GET) => {
            let response;

            match params_for_auth_page(&request).and_then(auth_html) {
                Some(html) => {
                    info!("starting authentication process");

                    response = Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::from(html))?;
                }
                None => {
                    response = Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from("invalid request"))?;
                }
            }

            Ok(response)
        }
        ("/auth", &Method::POST) => {
            let body = hyper::body::aggregate(request).await?;

            let credentials = de::from_bytes(body.chunk()).unwrap();

            if verify_credentials(credentials) {
                let auth_params = de::from_bytes(body.chunk()).unwrap();
                let redirect_url = get_redirect_url_from_params(auth_params).unwrap();

                info!("received credentials, generating an authorization code");

                Ok(Response::builder()
                    .status(StatusCode::FOUND)
                    .header(header::LOCATION, redirect_url.as_str())
                    .body(Body::empty())?)
            } else {
                let response = Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from("invalid request"))?;

                Ok(response)
            }
        }
        ("/token", &Method::POST) => {
            let body = hyper::body::aggregate(request).await?;
            let client_creds: ClientCreds = de::from_bytes(body.chunk()).unwrap();

            if !validate_client_creds(&client_creds) {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::from("invalid client creds"))?);
            }

            match client_creds.grant_type {
                GrantType::AuthorizationCode => {
                    let auth_code: AuthorizationCode = de::from_bytes(body.chunk()).unwrap();

                    if is_valid_token(auth_code.value, TokenType::Code) {
                        // TODO: save token version

                        info!("received a valid authorization code, generating access and refresh tokens");

                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body(Body::from(serde_json::to_vec(&TokenResponse::new())?))?)
                    } else {
                        info!("received an invalid authorization code");

                        Ok(Response::builder()
                            .status(StatusCode::FORBIDDEN)
                            .body(Body::from("invalid auth code"))?)
                    }
                }
                GrantType::RefreshToken => {
                    let refresh_token: RefreshToken = de::from_bytes(body.chunk()).unwrap();

                    if is_valid_token(refresh_token.value, TokenType::Refresh) {
                        // TODO: increment token version

                        info!("received a valid refresh token, generating new access and refresh tokens");

                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body(Body::from(serde_json::to_vec(&TokenResponse::new())?))?)
                    } else {
                        info!("received an invalid refresh token");

                        Ok(Response::builder()
                            .status(StatusCode::FORBIDDEN)
                            .body(Body::from("invalid refresh token"))?)
                    }
                }
            }
        }
        ("/v1.0", &Method::HEAD) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())?),
        ("/v1.0/user/devices", &Method::GET) => {
            validate_autorization(request, "devices", |request| async move {
                let request_id =
                    std::str::from_utf8(request.headers().get("X-Request-Id").unwrap().as_bytes())
                        .unwrap();

                let json = json!({
                    "request_id": request_id,
                    "payload": {
                        "user_id": "chipp",
                        "devices": [
                            sensor_device(Room::Bedroom),
                            sensor_device(Room::LivingRoom),
                            sensor_device(Room::Nursery),
                            vacuum_cleaner_device(Room::Hallway),
                            vacuum_cleaner_device(Room::Corridor),
                            vacuum_cleaner_device(Room::Bathroom),
                            vacuum_cleaner_device(Room::Nursery),
                            vacuum_cleaner_device(Room::Bedroom),
                            vacuum_cleaner_device(Room::Kitchen),
                            vacuum_cleaner_device(Room::LivingRoom),
                        ]
                    }
                });

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(serde_json::to_vec(&json)?))?)
            })
            .await
        }
        ("/v1.0/user/devices/query", &Method::POST) => {
            validate_autorization(request, "devices_query", |request| async move {
                let request_id = String::from(std::str::from_utf8(
                    request.headers().get("X-Request-Id").unwrap().as_bytes(),
                )?);

                let body = hyper::body::aggregate(request).await?;
                unsafe {
                    trace!("[query]: {}", std::str::from_utf8_unchecked(body.chunk()));
                }

                let query: StateRequest = serde_json::from_slice(body.chunk())?;
                let devices = query
                    .devices
                    .iter()
                    .filter_map(|device| DeviceId::from_str(device.id).ok())
                    .filter_map(|id| state_for_device(id))
                    .collect();

                let response = StateResponse::new(request_id, devices);

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(serde_json::to_vec(&response)?))?)
            })
            .await
        }
        ("/v1.0/user/devices/action", &Method::POST) => {
            validate_autorization(request, "devices_action", |request| async move {
                let request_id = String::from(std::str::from_utf8(
                    request.headers().get("X-Request-Id").unwrap().as_bytes(),
                )?);

                let body = hyper::body::aggregate(request).await?;
                unsafe {
                    println!("[action]: {}", std::str::from_utf8_unchecked(body.chunk()));
                }

                let action: UpdateStateRequest = serde_json::from_slice(body.chunk())?;
                let devices = update_devices_state(action.payload.devices, cmd).await;

                let response = UpdateStateResponse::new(request_id, devices);

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(serde_json::to_vec(&response)?))?)
            })
            .await
        }
        _ => {
            error!("Unsupported request: {:?}", request);

            let body = hyper::body::aggregate(request).await?;

            match std::str::from_utf8(body.chunk()) {
                Ok(body) if !body.is_empty() => error!("Body {}", body),
                _ => (),
            }

            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("invalid request"))?;

            Ok(response)
        }
    }
}

// Request { method: POST, uri: /v1.0/user/unlink, version: HTTP/1.1, headers: {"host": "lisa.burdukov.by", "connection": "close", "x-real-ip": "37.9.87.110", "x-forwarded-for": "37.9.87.110", "x-forwarded-proto": "https", "x-forwarded-ssl": "on", "x-forwarded-port": "443", "content-length": "0", "authorization": "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJzdWIiOiJ5YW5kZXgiLCJleHAiOjE2MTMyNTAyMDB9.zNVe2gc7xuA6oVjPYAh4kAOiM-6ZyK7MNSRS6NqhMei1OUjvgWKfKD4uiKLmz4iY_VK28c7r55TH-MXDHIvgPw", "x-request-id": "6e76e639-b535-4001-8069-8cd5413638e1", "user-agent": "Yandex LLC", "accept-encoding": "gzip"}, body: Body(Empty) }

fn sensor_device(room: Room) -> Device {
    let room_name = room.name().to_string();

    Device {
        id: DeviceId::temperature_sensor_at_room(room).to_string(),
        name: "Датчик температуры".to_string(),
        description: format!("в {}", room_name),
        room: room_name,
        device_type: DeviceType::Sensor,
        properties: vec![
            DeviceProperty::humidity().retrievable().reportable(),
            DeviceProperty::temperature().retrievable().reportable(),
        ],
        capabilities: vec![],
    }
}

fn vacuum_cleaner_device(room: Room) -> Device {
    let room_name = room.name().to_string();

    Device {
        id: DeviceId::vacuum_cleaner_at_room(room).to_string(),
        name: "Джордан".to_string(),
        description: format!("в {}", room_name),
        room: room_name,
        device_type: DeviceType::VacuumCleaner,
        properties: vec![],
        capabilities: vec![
            DeviceCapability::on_off(false).retrievable(),
            DeviceCapability::mode(
                ModeFunction::WorkSpeed,
                vec![Mode::Quiet, Mode::Normal, Mode::Medium, Mode::Turbo],
            )
            .retrievable(),
        ],
    }
}

#[derive(Debug, Deserialize)]
struct Credentials<'a> {
    user: Cow<'a, str>,
    password: Cow<'a, str>,
}

#[derive(Debug, Deserialize)]
struct AuthParams<'a> {
    state: Cow<'a, str>,
    redirect_uri: Cow<'a, str>,
    response_type: Cow<'a, str>,
    client_id: Cow<'a, str>,
}

static AUTH_HTML: &str = include_str!("./auth.html");

fn params_for_auth_page<'a>(request: &'a Request<Body>) -> Option<AuthParams> {
    let query = request.uri().query()?;
    serde_urlencoded::de::from_str(query).ok()
}

fn auth_html(auth: AuthParams) -> Option<String> {
    let mut html = String::from(AUTH_HTML);

    html = html.replace("#CLIENT_ID#", auth.client_id.as_ref());
    html = html.replace("#RESPONSE_TYPE#", auth.response_type.as_ref());
    html = html.replace("#REDIRECT_URI#", auth.redirect_uri.as_ref());
    html = html.replace("#STATE#", auth.state.as_ref());

    Some(html)
}

fn verify_credentials(credentials: Credentials) -> bool {
    match (credentials.user.as_ref(), credentials.password.as_ref()) {
        ("kek", "lol") => true,
        _ => false,
    }
}

fn get_redirect_url_from_params(auth: AuthParams) -> Option<Url> {
    let mut url = Url::parse(auth.redirect_uri.as_ref()).ok()?;

    let code = create_token_with_expiration_in(Duration::seconds(30), TokenType::Code);
    url.query_pairs_mut()
        .append_pair("state", &auth.state)
        .append_pair("code", &code);

    Some(url)
}

#[derive(Debug, Deserialize)]
struct ClientCreds<'a> {
    grant_type: GrantType,
    client_id: Cow<'a, str>,
    client_secret: Cow<'a, str>,
    redirect_uri: Option<Cow<'a, str>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GrantType {
    AuthorizationCode,
    RefreshToken,
}

#[derive(Debug, Deserialize)]
struct AuthorizationCode<'a> {
    #[serde(rename = "code")]
    value: Cow<'a, str>,
}

#[derive(Debug, Deserialize)]
struct RefreshToken<'a> {
    #[serde(rename = "refresh_token")]
    value: Cow<'a, str>,
}

fn validate_client_creds(client_creds: &ClientCreds) -> bool {
    let redirect_uri_valid = client_creds
        .redirect_uri
        .as_ref()
        .map(|uri| uri == "https://social.yandex.net/broker/redirect")
        .unwrap_or(true);

    client_creds.client_id == "tbd" && client_creds.client_secret == "tbd" && redirect_uri_valid
}

use std::future::Future;

async fn validate_autorization<F, T>(
    request: Request<Body>,
    request_name: &'static str,
    success: F,
) -> Result<Response<Body>>
where
    F: FnOnce(Request<Body>) -> T,
    T: Future<Output = Result<Response<Body>>>,
{
    match extract_token_from_headers(&request.headers()) {
        Some(token) if is_valid_token(token, TokenType::Access) => {
            info!(target: request_name, "received a valid access token");
            success(request).await
        }
        Some(_) => {
            error!(
                target: request_name,
                "an expired access token has been provided"
            );

            let response = Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header(
                    "WWW-Authenticate",
                    r#"Bearer 
                        error="invalid_token" 
                        error_description="The access token has expired"
                    "#,
                )
                .body(Body::from("invalid token"))?;

            Ok(response)
        }
        None => {
            let response = Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .header("WWW-Authenticate", r#"Bearer error="invalid_token" error_description="No access token has been provided""#)
                        .body(Body::from("invalid token"))?;

            Ok(response)
        }
    }
}

const BEARER: &str = "Bearer ";
fn extract_token_from_headers<'a>(headers: &'a header::HeaderMap) -> Option<&'a str> {
    let authorization = headers.get("Authorization")?;
    let authorization = std::str::from_utf8(&authorization.as_bytes()).ok()?;

    if authorization.starts_with(BEARER) {
        Some(&authorization[BEARER.len()..])
    } else {
        None
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum TokenType {
    Code,
    Access,
    Refresh,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}

fn is_valid_token<T: AsRef<str>>(token: T, token_type: TokenType) -> bool {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    let mut validation = Validation::new(Algorithm::HS512);
    validation.sub = Some("yandex".to_owned());
    validation.set_audience(&[token_type.to_string()]);

    let decoded = decode::<Claims>(
        token.as_ref(),
        &DecodingKey::from_secret(b"123456"),
        &validation,
    );

    decoded.is_ok()
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    aud: Vec<String>,
}

fn create_token_with_expiration_in(expiration: Duration, token_type: TokenType) -> String {
    use chrono::Utc;
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

    let expiration = Utc::now()
        .checked_add_signed(expiration)
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: "yandex".to_owned(),
        exp: expiration as usize,
        aud: vec![token_type.to_string()],
    };

    let header = Header::new(Algorithm::HS512);
    encode(&header, &claims, &EncodingKey::from_secret(b"123456")).unwrap()
}

#[derive(Serialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,

    #[serde(serialize_with = "duration_ser::serialize")]
    expires_in: Duration,
}

impl TokenResponse {
    fn access_token_exp() -> Duration {
        Duration::hours(1)
    }

    fn refresh_token_exp() -> Duration {
        Duration::weeks(1)
    }

    fn new() -> TokenResponse {
        TokenResponse {
            access_token: create_token_with_expiration_in(
                Self::access_token_exp(),
                TokenType::Access,
            ),
            refresh_token: create_token_with_expiration_in(
                Self::refresh_token_exp(),
                TokenType::Refresh,
            ),
            token_type: "Bearer".to_string(),
            expires_in: Self::access_token_exp(),
        }
    }
}

mod duration_ser {
    use chrono::Duration;
    use serde::ser;

    pub fn serialize<S>(dur: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_i64(dur.num_seconds())
    }
}
