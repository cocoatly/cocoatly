use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonOutput<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> JsonOutput<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }

    pub fn print(&self) where T: Serialize {
        let json = serde_json::to_string_pretty(self).unwrap();
        println!("{}", json);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationResult {
    pub operation: String,
    pub package: String,
    pub version: String,
    pub message: String,
}
