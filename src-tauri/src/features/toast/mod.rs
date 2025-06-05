use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ToastType {
    Success,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToastMessage {
    #[serde(rename = "type")]
    ty: ToastType,
    message: String,
}

impl ToastMessage {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            ty: ToastType::Success,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            ty: ToastType::Error,
            message: message.into(),
        }
    }
}
