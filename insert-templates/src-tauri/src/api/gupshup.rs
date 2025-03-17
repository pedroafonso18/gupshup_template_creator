use reqwest::{Client, Error as ReqwestError, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Error as IoError, ErrorKind};
use bytes::Bytes;
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TemplateCategory {
    #[serde(rename = "MARKETING")]
    Marketing,
    #[serde(rename = "UTILITY")]
    Utility,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl TemplateRequest {
    pub fn new(
        name: &str, 
        content: &str, 
        app_id: &str, 
        category: TemplateCategory,
        template_type: TemplateType,
        vertical: &str,
    ) -> Self {
        TemplateRequest {
            element_name: name.to_string(),
            language_code: "pt_BR".to_string(),
            content: content.to_string(),
            category,
            app_id: app_id.to_string(),
            vertical: vertical.to_string(),
            template_type,
            example: content.to_string(),
            example_header: None,
            media_id: None,
            media_url: None,
        }
    }

    pub fn with_media(mut self, media_id: &str, media_url: &str) -> Self {
        self.media_id = Some(media_id.to_string());
        self.media_url = Some(media_url.to_string());
        self
    }

    pub fn with_header_example(mut self, example_header: &str) -> Self {
        self.example_header = Some(example_header.to_string());
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GupshupResponse {
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaResponse {
    pub status: String,
    pub media: Option<MediaDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MediaDetails {
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub id: String,
    pub url: String,
}

pub struct GupshupClient {
    client: Client,
    base_url: String,
    api_key: String,
    session_cookie: String,
}

impl GupshupClient {
    pub fn new(api_key: &str, session_cookie: &str) -> Self {
        GupshupClient {
            client: Client::new(),
            base_url: "https://api.gupshup.io/wa/app".to_string(),
            api_key: api_key.to_string(),
            session_cookie: session_cookie.to_string(),
        }
    }

    pub async fn upload_media(&self, app_id: &str, file_name: &str, file_data: Vec<u8>) -> Result<MediaResponse, ReqwestError> {
        let url = format!("https://api.gupshup.io/wa/api/v1/app/{}/media", app_id);
        
        let part = reqwest::multipart::Part::bytes(file_data)
            .file_name(file_name.to_string())
            .mime_str("image/jpeg")?;
        
        let form = reqwest::multipart::Form::new().part("file", part);

        let response = self.client
            .post(&url)
            .header("Cookie", format!("session={}", self.session_cookie))
            .multipart(form)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            println!("Error uploading media: {}", error_text);
            // Create a proper error
            let io_err = IoError::new(ErrorKind::Other, format!("HTTP error: {}", error_text));
            return Err(ReqwestError::new(io_err.into()));
        }

        let media_response = response.json::<MediaResponse>().await?;
        Ok(media_response)
    }

    pub async fn create_template_with_image(&self, 
                                          app_id: &str, 
                                          mut template: TemplateRequest, 
                                          image_data: Option<Vec<u8>>,
                                          image_name: Option<String>) -> Result<GupshupResponse, ReqwestError> {
        if let Some(data) = image_data {
            let file_name = image_name.unwrap_or_else(|| "image.jpg".to_string());
            
            let media_response = self.upload_media(app_id, &file_name, data).await?;
            
            if media_response.status != "success" {
                // Create a proper error message
                let error_msg = media_response.media
                    .map_or("Unknown error".to_string(), 
                    |m| format!("Error with file {}", m.file_name));
                
                let io_err = IoError::new(ErrorKind::Other, 
                    format!("Failed to upload media: {}", error_msg));
                
                return Err(ReqwestError::new(io_err.into()));
            }
            
            if let Some(media_details) = media_response.media {
                template = template.with_media(&media_details.id, &media_details.url);
            }
        }
        
        self.create_template(app_id, template).await
    }

    pub async fn create_template(&self, app_id: &str, template: TemplateRequest) -> Result<GupshupResponse, ReqwestError> {
        let url = format!("{}/{}/template", self.base_url, app_id);
        
        let mut form = HashMap::new();
        form.insert("elementName", template.element_name);
        form.insert("languageCode", template.language_code);
        form.insert("content", template.content);
        form.insert("category", match template.category {
            TemplateCategory::Marketing => "MARKETING".to_string(),
            TemplateCategory::Utility => "UTILITY".to_string(),
        });
        form.insert("appId", template.app_id);
        form.insert("vertical", template.vertical);
        form.insert("templateType", match template.template_type {
            TemplateType::Text => "TEXT".to_string(),
            TemplateType::Image => "IMAGE".to_string(),
        });
        form.insert("example", template.example);
        form.insert("enableSample", "true".to_string());
        form.insert("allowTemplateCategoryChange", "true".to_string());
        form.insert("checkerApprovalRequired", "false".to_string());
        
        if let Some(example_header) = template.example_header {
            form.insert("exampleHeader", example_header);
        } else {
            form.insert("exampleHeader", "".to_string());
        }

        if let Some(media_id) = template.media_id {
            form.insert("mediaId", media_id);
        }

        if let Some(media_url) = template.media_url {
            form.insert("mediaUrl", media_url);
        }

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("session={}", self.session_cookie))
            .form(&form)
            .send()
            .await?;

        let gupshup_response = response.json::<GupshupResponse>().await?;
        Ok(gupshup_response)
    }
}
