use serde::Serialize;


pub struct AppState {
    pub pool: sqlx::mysql::MySqlPool,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

impl ApiResponse<()> {
    pub fn ok() -> Self {
        Self::msg_ok("Success")
    }

    pub fn msg_ok(message: &str) -> Self {
        Self {
            code: 0,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn err<E: AsRef<str>>(message: E) -> Self {
        Self::code_err(-1, message)
    }

    pub fn code_err<E: AsRef<str>>(code: i32, message: E) -> Self {
        Self {
            code,
            message: message.as_ref().to_string(),
            data: None,
        }
    }
}

impl<T: Serialize> ApiResponse<T> {
    /// 成功响应
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            message: "Success".to_string(),
            data: Some(data),
        }
    }
}