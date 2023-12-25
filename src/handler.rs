use super::{request, response};

pub type Handler = fn(request: &request::HttpRequest, response: &mut response::HttpResponse);