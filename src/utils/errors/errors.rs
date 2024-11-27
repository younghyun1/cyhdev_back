use axum::response::IntoResponse;
use chrono::Utc;
use serde_derive::{Deserialize, Serialize};

use crate::utils::gadgets::stopwatch::Stopwatch;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ErrResp {
    success: bool,
    data: ErrRespDatFin,
    meta: SvrErrorRespMeta,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ErrRespDat {
    code: u8,
    message: &'static str,
    status_code: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ErrRespDatFin {
    code: u8,
    message: String,
    status_code: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SvrErrorRespMeta {
    time_taken: String,
    timestamp: String,
}

impl IntoResponse for ErrResp {
    fn into_response(self) -> axum::response::Response {
        let serialized_body =
            bincode::serialize(&self).expect("Failed to serialize ErrResp with bincode");
        let response = match axum::response::Response::builder()
            .status(self.data.status_code)
            .header("Content-Type", "application/octet-stream")
            .body(axum::body::Body::from(serialized_body))
        {
            Ok(resp) => resp,
            Err(e) => panic!("Failed to build Axum response: {:?}", e),
        };
        response
    }
}

impl ErrResp {
    pub fn from(dat: ErrRespDat, start: &Stopwatch, error: anyhow::Error) -> Self {
        ErrResp {
            success: false,
            data: ErrRespDatFin {
                code: dat.code,
                message: {
                    let mut nstr = dat.message.to_owned();
                    nstr.push_str(&error.to_string());

                    nstr
                },
                status_code: dat.status_code,
            },
            meta: SvrErrorRespMeta {
                time_taken: format!("{:?}", start.get_original_start().elapsed()),
                timestamp: Utc::now().to_rfc3339(),
            },
        }
    }
}

impl ErrRespDat {
    pub const COULD_NOT_GET_CONN_FROM_POOL: ErrRespDat = ErrRespDat {
        code: 1,
        message: "Could not get connection from pool; ",
        status_code: 500, // INTERNAL SERVER ERROR
    };
    pub const COULD_NOT_BUILD_TRANSACTION_FROM_CONN: ErrRespDat = ErrRespDat {
        code: 2,
        message: "Could not build transaction from connection; ",
        status_code: 500, // INTERNAL SERVER ERROR
    };
    pub const WRONG_EMAIL_FORMAT: ErrRespDat = ErrRespDat {
        code: 3,
        message: "The provided email format is incorrect.",
        status_code: 400, // BAD IN
    };
    pub const WRONG_PW_FORMAT: ErrRespDat = ErrRespDat {
        code: 4,
        message: "Password format is incorrect. Must be at least 8 characters and include uppercase, lowercase, number, and special characters among: [@, $, !, %, *, ?, &, #]",
        status_code: 400, // BAD REQUEST
    };
    pub const COULD_NOT_INSERT_USER: ErrRespDat = ErrRespDat {
        code: 5,
        message: "Could not insert user into database; ",
        status_code: 500, // INTERNAL SERVER ERROR
    };
    pub const COULD_NOT_INSERT_USER_TOKEN: ErrRespDat = ErrRespDat {
        code: 6,
        message: "Could not insert user token into database; ",
        status_code: 500, // INTERNAL SERVER ERROR
    };
    pub const COULD_NOT_CONSTRUCT_EMAIL: ErrRespDat = ErrRespDat {
        code: 7,
        message: "Could not construct email content; ",
        status_code: 500, // INTERNAL SERVER ERROR
    };
    pub const COULD_NOT_SEND_MAIL: ErrRespDat = ErrRespDat {
        code: 8,
        message: "Could not send email; ",
        status_code: 500, // INTERNAL SERVER ERROR
    };
    pub const COULD_NOT_SERIALIZE_BINCODE: ErrRespDat = ErrRespDat {
        code: 9,
        message: "Failed to serialize data with bincode; ",
        status_code: 500, // INTERNAL SERVER ERROR
    };
    pub const COULD_NOT_COMMIT_TRANSACTION: ErrRespDat = ErrRespDat {
        code: 10,
        message: "Could not commit transaction; ",
        status_code: 500, // INTERNAL SERVER ERROR
    };
}
