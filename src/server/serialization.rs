use axum::{response::IntoResponse, Json};
use serde::Serialize;
use serde_json::{value::Serializer, Value};

use crate::error::AppError;

#[derive(Serialize)]
pub struct PaginatedData<T> {
    pub nodes: Vec<T>,
    pub page: u64,
    pub total: u64,
}

pub trait SerializedResponse {
    fn remove_null_fields(&self) -> Result<Value, AppError>;
    fn into_response(self) -> Result<Json<Value>, AppError>;
}

impl<T> SerializedResponse for T
where
    T: Serialize,
{
    fn remove_null_fields(&self) -> Result<Value, AppError> {
        let mut value = serde_json::to_value(self)?;
        recursively_remove_null_fields(&mut value);
        Ok(value)
    }

    fn into_response(self) -> Result<Json<Value>, AppError> {
        self.remove_null_fields().map(|value| Json(value))
    }
}

fn recursively_remove_null_fields(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.keys()
                .cloned()
                .collect::<Vec<String>>()
                .iter()
                .for_each(|key| {
                    let Some(inner_value) = map.get_mut(key) else {
                        return;
                    };

                    if inner_value.is_null() {
                        map.remove(key);
                    } else {
                        recursively_remove_null_fields(inner_value);
                    }
                });
        }
        Value::Array(arr) => {
            for item in arr {
                recursively_remove_null_fields(item);
            }
        }
        _ => {}
    }
}
