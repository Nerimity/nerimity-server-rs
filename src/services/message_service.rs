use crate::{AppState, api_error::ApiError, db::channel_repo::find_by_id, proto::pb::EmitRequest};

pub async fn create_message(state: &AppState, channel_id: String) -> Result<(), ApiError> {
    let mut socket = state.socket.clone();

    let _channel = find_by_id(&state.db, channel_id)
        .await?
        .ok_or_else(|| ApiError::not_found("Channel not found"))?;

    socket
        .emit(EmitRequest {
            data: "123".to_string(),
            to: "456".to_string(),
        })
        .await
        .unwrap();
    Ok(())
}
