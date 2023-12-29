use actix_web::HttpResponse;

#[derive(Debug, PartialEq, Eq)]
pub enum ServerError {
    InternalServerError,
    Unauthorized,
    NotAcceptable,
}

impl From<ServerError> for HttpResponse {
    fn from(error: ServerError) -> Self {
        match error {
            ServerError::InternalServerError => HttpResponse::InternalServerError().finish(),
            ServerError::Unauthorized => HttpResponse::Unauthorized().finish(),
            ServerError::NotAcceptable => HttpResponse::NotAcceptable().finish(),
        }
    }
}
