// Helper methods for quick error handling.

use std::fmt::Display;

use actix_web::{
    http::{header, StatusCode},
    HttpResponseBuilder, ResponseError,
};

use crate::api::GeneralResponse;

pub type AResult<T> = Result<T, AError>;

#[derive(Debug)]
#[repr(transparent)]
pub struct AError(actix_web::Error);

impl From<actix_web::Error> for AError {
    fn from(err: actix_web::Error) -> Self {
        Self(err)
    }
}

impl From<Error> for AError {
    fn from(err: Error) -> Self {
        Self(err.into())
    }
}

impl Display for AError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Into<actix_web::Error> for AError {
    fn into(self) -> actix_web::Error {
        self.0
    }
}

impl<T: Into<Error>> From<T> for AError {
    default fn from(err: T) -> Self {
        Self::from(T::into(err))
    }
}

#[derive(Debug)]
pub struct Error {
    pub status: StatusCode,
    pub message: String,
}

impl From<sea_orm::DbErr> for Error {
    fn from(err: sea_orm::DbErr) -> Self {
        match err {
            sea_orm::DbErr::RecordNotFound(_) => Self {
                status: StatusCode::NOT_FOUND,
                message: err.to_string(),
            },
            _ => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: err.to_string(),
            },
        }
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(err: argon2::password_hash::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: err.to_string(),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: err.to_string(),
        }
    }
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
error!(conflict, CONFLICT);
error!(iam_a_teapot, IM_A_TEAPOT);
error!(unauthorized, UNAUTHORIZED);
error!(forbidden, FORBIDDEN);
error!(not_found, NOT_FOUND);
error!(internal_server_error, INTERNAL_SERVER_ERROR);
error!(service_unavailable, SERVICE_UNAVAILABLE);
