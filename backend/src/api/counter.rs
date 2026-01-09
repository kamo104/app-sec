use axum::{
    extract::State,
    response::IntoResponse,
};
use axum_extra::protobuf::Protobuf;
use std::sync::Arc;
use tracing::error;

use crate::db::DBHandle;
use proto_types::v1::{ApiData, SuccessCode, CounterData, api_data};
use super::auth_extractor::AuthenticatedUser;
use super::utils::{internal_error, success_response};

pub async fn get_counter(auth: AuthenticatedUser) -> impl IntoResponse {
    success_response(
        SuccessCode::SuccessOk,
        Some(ApiData {
            data: Some(api_data::Data::CounterData(CounterData {
                value: auth.user.counter,
            })),
        }),
    )
}

pub async fn set_counter(
    State(db): State<Arc<DBHandle>>,
    auth: AuthenticatedUser,
    Protobuf(payload): Protobuf<proto_types::v1::SetCounterRequest>,
) -> impl IntoResponse {
    match db
        .user_login_table
        .update_counter(auth.user.user_id, payload.value)
        .await
    {
        Ok(_) => {
            success_response(
                SuccessCode::SuccessCounterUpdated,
                Some(ApiData {
                    data: Some(api_data::Data::CounterData(CounterData {
                        value: payload.value,
                    })),
                }),
            )
        }
        Err(e) => {
            error!("Failed to update counter: {:?}", e);
            return internal_error();
        }
    }
}
