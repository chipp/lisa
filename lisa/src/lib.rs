use alice::{Request, Response};

pub fn handler(request: Request) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
    if &request.payload.original_utterance == "ping" {
        Ok(Response::from_string("pong"))
    } else {
        Ok(Response::from_string(
            "Привет! Температура в зале 23.5 градуса",
        ))
    }
}
