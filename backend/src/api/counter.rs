use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::protobuf::Protobuf;
use std::sync::Arc;
use tracing::error;

use crate::db::DBHandle;
use crate::generated::v1::{ApiData, ApiResponse, CounterData, ResponseCode, api_data};
use super::auth_extractor::AuthenticatedUser;
use super::utils::internal_error;

pub async fn get_counter(auth: AuthenticatedUser) -> impl IntoResponse {
    let response = ApiResponse {
        code: ResponseCode::Success.into(),
        data: Some(ApiData {
            data: Some(api_data::Data::CounterData(CounterData {
                value: auth.user.counter,
            })),
        }),
    };
    (StatusCode::OK, Protobuf(response))
}

pub async fn set_counter(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
    Protobuf(payload): Protobuf<crate::generated::v1::SetCounterRequest>,
) -> impl IntoResponse {
    match db
        .user_login_table
        .update_counter(auth.user.user_id, payload.value)
        .await
    {
        Ok(_) => {
            let response = ApiResponse {
                code: ResponseCode::Success.into(),
                data: Some(ApiData {
                    data: Some(api_data::Data::CounterData(CounterData {
                        value: payload.value,
                    })),
                }),
            };
            (StatusCode::OK, Protobuf(response))
        }
        Err(e) => {
            error!("Failed to update counter: {:?}", e);
            return internal_error();
        }
    }
}
