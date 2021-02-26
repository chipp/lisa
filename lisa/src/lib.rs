// use alice::{Request, Response, ResponseButton};

// pub fn handler(request: Request) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
//     match request.payload.original_utterance.as_str() {
//         "ping" => Ok(Response::from_string("pong")),
//         "" => Ok(rooms_response()),
//         _ => Ok(Response::from_string(
//             "Привет! Температура в зале 23.5 градуса",
//         )),
//     }
// }

// fn rooms_response() -> Response {
//     let mut response = Response::from_string("Температура в какой комнате интересует?");

//     response.add_button(ResponseButton::with_title("в зале"));
//     response.add_button(ResponseButton::with_title("в спальне"));
//     response.add_button(ResponseButton::with_title("в детской"));

//     response
// }
