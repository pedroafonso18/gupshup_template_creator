use serde::{Deserialize, Serialize};

mod config;
mod database;
mod api;  // Add this line

use crate::config::config::load;
use database::connect;
use database::fetch;
use api::gupshup::{TemplateCategory, TemplateRequest, TemplateType, GupshupClient};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            fetch_all_connections_data, 
            create_template,  // Add this line
            create_template_for_all_connections  // Add this new command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


#[derive(Serialize, Deserialize)]
struct Params {
    db_url: Option<String>,
}

#[derive(Serialize)]
struct ConnectionsCompleteResult {
    id: i32,
    source_name: Option<String>,
    disparos_dia: i32,
    qualidade: Option<String>,
    restriction: Option<String>,
    issues: i32,
    msg_limit: Option<String>,
    app_id: Option<String>,
    ultima_issue_dia: Option<String>,
    facebook_id: Option<String>,
    facebook_token: Option<String>,
}

#[tauri::command]
async fn fetch_all_connections_data(params: Params) -> Result<Vec<ConnectionsCompleteResult>, String> {
    let mut env = load();
    if let Some(db_url) = params.db_url {
        if !db_url.is_empty() {
            env.db_url = db_url;
        }
    }

    let private_client = connect::connect_db(&env.db_url)
        .await
        .map_err(|e| format!("Failed to connect to DB: {}", e))?;

    let data = fetch::fetch_connections(&private_client)
        .await
        .map_err(|e| format!("Failed to fetch connections: {}", e))?;

    let results = data.into_iter().map(|item| { ConnectionsCompleteResult {
            id: item.id,
            source_name: item.source_name,
            disparos_dia: item.disparos_dia,
            qualidade: item.qualidade,
            restriction: item.restriction,
            issues: item.issues,
            msg_limit: item.msg_limit,
            app_id: item.app_id,
            ultima_issue_dia: item.ultima_issue_dia,
            facebook_id: item.facebook_id,
            facebook_token: item.facebook_token,
        }
    }).collect();
    Ok(results)
}

#[derive(Serialize, Deserialize)]
struct CreateTemplateParams {
    template_name: String,
    app_id: String,
    content: String,
    category: String,
    template_type: String,
    vertical: String,
    media_id: Option<String>,
    media_url: Option<String>,
    header_text: Option<String>,
    image_data: Option<Vec<u8>>,
    image_name: Option<String>,
}

#[tauri::command]
async fn create_template(params: CreateTemplateParams) -> Result<String, String> {
    let env = load();
    
    let category = match params.category.as_str() {
        "MARKETING" => TemplateCategory::Marketing,
        "UTILITY" => TemplateCategory::Utility,
        _ => return Err("Invalid category. Must be 'MARKETING' or 'UTILITY'".to_string()),
    };
    
    let template_type = match params.template_type.as_str() {
        "TEXT" => TemplateType::Text,
        "IMAGE" => TemplateType::Image,
        _ => return Err("Invalid template type. Must be 'TEXT' or 'IMAGE'".to_string()),
    };
    
    let client = GupshupClient::new(&env.apikey, &env.cookie);
    
    let template_request = TemplateRequest::new(
        &params.template_name,
        &params.content,
        &params.app_id,
        category,
        template_type,
        &params.vertical,
    );
    
    // If header text is provided, add it to the template
    let template_request = if let Some(header_text) = params.header_text {
        template_request.with_header_example(&header_text)
    } else {
        template_request
    };
    
    // For direct image URL usage without uploading
    let template_request = if let (Some(media_id), Some(media_url), None) = (&params.media_id, &params.media_url, &params.image_data) {
        template_request.with_media(media_id, media_url)
    } else {
        template_request
    };
    
    let result = if let Some(image_data) = params.image_data {
        // If image data is provided, upload it and create template
        client.create_template_with_image(&params.app_id, template_request, Some(image_data), params.image_name)
            .await?
    } else {
        // Otherwise create template without uploading image
        client.create_template(&params.app_id, template_request)
            .await?
    };
    
    match result.status.as_str() {
        "success" => Ok("Template created successfully".to_string()),
        _ => Err(result.message.unwrap_or("Unknown error".to_string())),
    }
}

// Add a new command to create templates for all connections
#[derive(Serialize, Deserialize)]
struct BulkCreateTemplateParams {
    template_name: String,
    content: String,
    category: String,
    template_type: String,
    vertical: String,
    header_text: Option<String>,
    image_data: Option<Vec<u8>>,
    image_name: Option<String>,
}

#[derive(Serialize)]
struct BulkCreateResult {
    successful: usize,
    total: usize,
    app_ids: Vec<String>,
}

#[tauri::command]
async fn create_template_for_all_connections(
    params: BulkCreateTemplateParams
) -> Result<BulkCreateResult, String> {
    let env = load();
    
    // Get database client
    let db_client = connect::connect_db(&env.db_url)
        .await
        .map_err(|e| format!("Failed to connect to DB: {}", e))?;
    
    // Fetch all connections
    let connections = fetch::fetch_connections(&db_client)
        .await
        .map_err(|e| format!("Failed to fetch connections: {}", e))?;
    
    // Filter connections that have app_ids
    let connections_with_app_id: Vec<_> = connections.into_iter()
        .filter(|conn| conn.app_id.is_some() && !conn.app_id.as_ref().unwrap().is_empty())
        .collect();
    
    if connections_with_app_id.is_empty() {
        return Err("No connections found with valid app_id".to_string());
    }
    
    // Convert template parameters
    let category = match params.category.as_str() {
        "MARKETING" => TemplateCategory::Marketing,
        "UTILITY" => TemplateCategory::Utility,
        _ => return Err("Invalid category. Must be 'MARKETING' or 'UTILITY'".to_string()),
    };
    
    let template_type = match params.template_type.as_str() {
        "TEXT" => TemplateType::Text,
        "IMAGE" => TemplateType::Image,
        _ => return Err("Invalid template type. Must be 'TEXT' or 'IMAGE'".to_string()),
    };
    
    // Create client for API calls
    let client = GupshupClient::new(&env.apikey, &env.cookie);
    
    let mut successful = 0;
    let total = connections_with_app_id.len();
    let mut successful_app_ids = Vec::new();
    
    // Process each connection
    for connection in connections_with_app_id {
        let app_id = connection.app_id.unwrap(); // Safe because of filter
        
        // Create template request
        let template_request = TemplateRequest::new(
            &params.template_name,
            &params.content,
            &app_id,
            category.clone(),
            template_type.clone(),
            &params.vertical,
        );
        
        // Add header if specified
        let template_request = if let Some(ref header_text) = params.header_text {
            template_request.with_header_example(header_text)
        } else {
            template_request
        };
        
        // Process template request (with or without image)
        let result = if let Some(ref image_data) = params.image_data {
            client.create_template_with_image(
                &app_id,
                template_request,
                Some(image_data.clone()),
                params.image_name.clone()
            )
            .await
            .map_err(|e| format!("Failed to create template for app_id {}: {}", app_id, e))?
        } else {
            client.create_template(&app_id, template_request)
                .await
                .map_err(|e| format!("Failed to create template for app_id {}: {}", app_id, e))?
        };
        
        // Check if template creation was successful
        if result.status != "success" {
            return Err(format!(
                "Failed to create template for app_id {}: {}", 
                app_id, 
                result.message.unwrap_or("Unknown error".to_string())
            ));
        }
        
        // Track success
        successful += 1;
        successful_app_ids.push(app_id);
    }
    
    Ok(BulkCreateResult {
        successful,
        total,
        app_ids: successful_app_ids,
    })
}