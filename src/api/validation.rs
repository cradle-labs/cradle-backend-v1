use crate::api::error::ApiError;

pub fn validate_uuid(uuid_str: &str) -> Result<uuid::Uuid, ApiError> {
    uuid::Uuid::parse_str(uuid_str)
        .map_err(|_| ApiError::bad_request("Invalid UUID format"))
}

pub fn validate_not_empty(value: &str, field_name: &str) -> Result<(), ApiError> {
    if value.is_empty() {
        return Err(ApiError::bad_request(format!("{} cannot be empty", field_name)));
    }
    Ok(())
}
