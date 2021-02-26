use std::borrow::Cow;

use bytes::buf::Buf;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Method, Request, Response, Server, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_urlencoded::de;
use url::Url;

use alice::{
    Device, DeviceCapability, DeviceMode, DeviceModeFunction, DeviceModeParameters,
    DeviceOnOffParameters, DeviceProperty, DevicePropertyParameters, DevicePropertyType,
    DeviceType, HumidityUnit, TemperatureUnit,
};

type ErasedError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, ErasedError>;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let make_svc =
        make_service_fn(|_| async { Ok::<_, ErasedError>(service_fn(move |req| service(req))) });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);
    println!("Listening http://{}", addr);

    server.await?;

    Ok(())
}

pub async fn service(request: Request<Body>) -> Result<Response<Body>> {
    match (request.uri().path(), request.method()) {
        ("/auth", &Method::GET) => {
            let response;

            match params_for_auth_page(&request).and_then(auth_html) {
                Some(html) => {
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

            let credentials = de::from_bytes(body.bytes()).unwrap();

            if verify_credentials(credentials) {
                let auth_params = de::from_bytes(body.bytes()).unwrap();
                let redirect_url = get_redirect_url_from_params(auth_params).unwrap();

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
            println!("{:?}", request.headers());

            let body = hyper::body::aggregate(request).await?;
            let client_creds: ClientCreds = de::from_bytes(body.bytes()).unwrap();

            if !validate_client_creds(&client_creds) {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::from("invalid client creds"))?);
            }

            match client_creds.grant_type {
                GrantType::AuthorizationCode => {
                    let auth_code: AuthorizationCode = de::from_bytes(body.bytes()).unwrap();

                    if validate_token(auth_code.value) {
                        let json = json!({
                            "access_token": create_token_with_expiration_in(chrono::Duration::minutes(10)),
                            "refresh_token": create_token_with_expiration_in(chrono::Duration::days(1)),
                            "token_type": "Bearer",
                            "expires_in": chrono::Duration::days(1).num_seconds()
                        });

                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body(Body::from(serde_json::to_vec(&json)?))?)
                    } else {
                        Ok(Response::builder()
                            .status(StatusCode::FORBIDDEN)
                            .body(Body::from("invalid auth code"))?)
                    }
                }
                GrantType::RefreshToken => {
                    let refresh_token: RefreshToken = de::from_bytes(body.bytes()).unwrap();

                    if validate_token(refresh_token.value) {
                        // TODO: increment token version

                        let json = json!({
                            "access_token": create_token_with_expiration_in(chrono::Duration::days(1)),
                            "refresh_token": create_token_with_expiration_in(chrono::Duration::days(10)),
                            "token_type": "Bearer",
                            "expires_in": chrono::Duration::days(1).num_seconds()
                        });

                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body(Body::from(serde_json::to_vec(&json)?))?)
                    } else {
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
        }
        _ => {
            println!("{:?}", request);

            let body = hyper::body::aggregate(request).await?;
            println!("{}", std::str::from_utf8(body.bytes()).unwrap());

            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("invalid request"))?;

            Ok(response)
        }
    }
}

// Request { method: POST, uri: /v1.0/user/unlink, version: HTTP/1.1, headers: {"host": "lisa.burdukov.by", "connection": "close", "x-real-ip": "37.9.87.110", "x-forwarded-for": "37.9.87.110", "x-forwarded-proto": "https", "x-forwarded-ssl": "on", "x-forwarded-port": "443", "content-length": "0", "authorization": "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJzdWIiOiJ5YW5kZXgiLCJleHAiOjE2MTMyNTAyMDB9.zNVe2gc7xuA6oVjPYAh4kAOiM-6ZyK7MNSRS6NqhMei1OUjvgWKfKD4uiKLmz4iY_VK28c7r55TH-MXDHIvgPw", "x-request-id": "6e76e639-b535-4001-8069-8cd5413638e1", "user-agent": "Yandex LLC", "accept-encoding": "gzip"}, body: Body(Empty) }

fn sensor_device(room: Room) -> Device {
    Device {
        id: format!("temp-sensor/{}", room.id()),
        name: "Датчик температуры".to_string(),
        description: format!("в {}", room.name()),
        room: room.name().to_string(),
        device_type: DeviceType::Sensor,
        properties: vec![
            DeviceProperty {
                property_type: DevicePropertyType::Float,
                retrievable: true,
                reportable: true,
                parameters: DevicePropertyParameters::Humidity {
                    unit: HumidityUnit::Percent,
                },
            },
            DeviceProperty {
                property_type: DevicePropertyType::Float,
                retrievable: true,
                reportable: true,
                parameters: DevicePropertyParameters::Temperature {
                    unit: TemperatureUnit::Celsius,
                },
            },
        ],
        capabilities: vec![],
    }
}

fn vacuum_cleaner_device(room: Room) -> Device {
    Device {
        id: format!("vacuum-cleaner/{}", room.id()),
        name: "Джордан".to_string(),
        description: format!("в {}", room.name()),
        room: room.name().to_string(),
        device_type: DeviceType::VacuumCleaner,
        properties: vec![],
        capabilities: vec![
            DeviceCapability::OnOff {
                reportable: true,
                retreivable: true,
                parameters: DeviceOnOffParameters { split: false },
            },
            DeviceCapability::Mode {
                reportable: false,
                retreivable: true,
                parameters: DeviceModeParameters {
                    instance: DeviceModeFunction::FanSpeed,
                    modes: vec![
                        DeviceMode::Quiet,
                        DeviceMode::Medium,
                        DeviceMode::High,
                        DeviceMode::Turbo,
                    ],
                },
            },
        ],
    }
}

enum Room {
    Hallway,
    Corridor,
    Bathroom,
    Nursery,
    Bedroom,
    Kitchen,
    LivingRoom,
}

impl Room {
    fn id(&self) -> &'static str {
        match self {
            Room::Hallway => "hall",
            Room::Corridor => "corridor",
            Room::Bathroom => "bathroom",
            Room::Nursery => "nursery",
            Room::Bedroom => "bedroom",
            Room::Kitchen => "kitchen",
            Room::LivingRoom => "living-room",
        }
    }

    fn name(&self) -> &str {
        match self {
            Room::Hallway => "Прихожая",
            Room::Corridor => "Коридор",
            Room::Bathroom => "Ванная",
            Room::Nursery => "Детская",
            Room::Bedroom => "Спальня",
            Room::Kitchen => "Кухня",
            Room::LivingRoom => "Зал",
        }
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

    let code = create_token_with_expiration_in(chrono::Duration::seconds(30))?;
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

fn validate_token(token: Cow<str>) -> bool {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    let mut validation = Validation::new(Algorithm::HS512);
    validation.sub = Some("yandex".to_owned());

    let decoded = decode::<Claims>(&token, &DecodingKey::from_secret(b"123456"), &validation);

    decoded.is_ok()
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

fn create_token_with_expiration_in(expiration: chrono::Duration) -> Option<String> {
    use chrono::Utc;
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

    let expiration = Utc::now()
        .checked_add_signed(expiration)
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: "yandex".to_owned(),
        exp: expiration as usize,
    };

    let header = Header::new(Algorithm::HS512);
    encode(&header, &claims, &EncodingKey::from_secret(b"123456")).ok()
}
