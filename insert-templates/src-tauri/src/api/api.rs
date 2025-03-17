use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::form_urlencoded;

#[derive(Debug, Serialize, Deserialize)]
pub enum TemplateCategory {
    #[serde(rename = "MARKETING")]
    Marketing,
    #[serde(rename = "UTILITY")]
    Utility,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TemplateType {
    #[serde(rename = "TEXT")]
    Text,
    #[serde(rename = "IMAGE")]
    Image,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateRequest {
    pub element_name: String,
    pub language_code: String,
    pub content: String,
    pub category: TemplateCategory,
    pub app_id: String,
    pub vertical: String,
    pub template_type: TemplateType,
    pub example: String,
    pub example_header: Option<String>,
    pub media_id: Option<String>,
    pub media_url: Option<String>,
}

impl GupshupClient {
    pub fn new(api_key: &str, session_cookie: &str) -> Self {
        GupshupClient {
            client: Client::new(),

        }
    }
}