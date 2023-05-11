// Helper methods for quick error handling.

use std::fmt::Display;

use actix_web::{
    http::{header, StatusCode},
    HttpResponseBuilder, ResponseError,
};

use crate::api::GeneralResponse;

#[derive(Debug)]
pub struct Error {
    pub status: StatusCode,
    pub message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = format!("{}: {}", self.status, self.message);
        f.write_str(&error)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        self.status
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut builder = HttpResponseBuilder::new(self.status_code());
        if self.status_code() == StatusCode::UNAUTHORIZED {
            builder.append_header((header::WWW_AUTHENTICATE, "Basic realm=Restricted"));
        }
        builder.json(GeneralResponse {
            message: self.message.clone(),
        })
    }
}

macro_rules! error {
    ($fn_name:ident, $status:ident) => {
        pub fn $fn_name<T: std::fmt::Display>(message: T) -> actix_web::Error {
            Error {
                status: StatusCode::$status,
                message: message.to_string(),
            }
            .into()
        }
    };
}

error!(bad_request, BAD_REQUEST);
error!(unauthorized, UNAUTHORIZED);
error!(forbidden, FORBIDDEN);
error!(not_found, NOT_FOUND);
error!(internal_server_error, INTERNAL_SERVER_ERROR);
error!(bad_gateway, BAD_GATEWAY);
error!(service_unavailable, SERVICE_UNAVAILABLE);
error!(gateway_timeout, GATEWAY_TIMEOUT);
