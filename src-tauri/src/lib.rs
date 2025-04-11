use serde::{Deserialize, Serialize};

mod config;
mod database;
mod api;

use crate::config::config::load;
use database::connect;
use database::fetch;
use api::gupshup::{TemplateCategory, TemplateRequest, TemplateType, GupshupClient};

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
            create_template,
            create_template_for_all_connections
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
    println!("Starting fetch_all_connections_data");
    let mut env = load();
    if let Some(db_url) = params.db_url {
        if !db_url.is_empty() {
            env.db_url = db_url;
        }
    }

    println!("Connecting to database with URL: {}", env.db_url);
    let mut db_conn = connect::connect_db(&env.db_url)
        .await
        .map_err(|e| {
            println!("Database connection error: {}", e);
            format!("Failed to connect to DB: {}", e)
        })?;

    println!("Connection successful, fetching data...");
    let data = fetch::fetch_connections(&mut db_conn)
        .await
        .map_err(|e| {
            println!("Error fetching connections: {}", e);
            format!("Failed to fetch connections: {}", e)
        })?;

    println!("Fetched {} connections", data.len());

    let results: Vec<ConnectionsCompleteResult> = data.into_iter().map(|item| { ConnectionsCompleteResult {
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
    
    println!("Returning {} connection results", results.len());
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
    println!("Starting create_template for app_id: {}", params.app_id);
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
    
    let template_request = if let Some(header_text) = params.header_text {
        template_request.with_header_example(&header_text)
    } else {
        template_request
    };
    
    let template_request = if let (Some(media_id), Some(media_url), None) = (&params.media_id, &params.media_url, &params.image_data) {
        template_request.with_media(media_id, media_url)
    } else {
        template_request
    };
    
    println!("Creating template '{}' of type {} for app_id {}", 
        params.template_name, params.template_type, params.app_id);
    
    let result = if let Some(image_data) = params.image_data {
        println!("Template has image, image size: {} bytes", image_data.len());
        client.create_template_with_image(&params.app_id, template_request, Some(image_data), params.image_name)
            .await?
    } else {
        println!("Creating text-only template");
        client.create_template(&params.app_id, template_request)
            .await?
    };
    
    println!("Template creation result: {}", result.status);
    
    match result.status.as_str() {
        "success" => Ok("Template created successfully".to_string()),
        _ => Err(result.message.unwrap_or("Unknown error".to_string())),
    }
}

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
    println!("Starting create_template_for_all_connections");
    let env = load();
    
    println!("Connecting to database to retrieve connections");
    let mut db_conn = connect::connect_db(&env.db_url)
        .await
        .map_err(|e| {
            println!("Database connection error: {}", e);
            format!("Failed to connect to DB: {}", e)
        })?;
    
    println!("Fetching connections from database");
    let connections = fetch::fetch_connections(&mut db_conn)
        .await
        .map_err(|e| {
            println!("Error fetching connections: {}", e);
            format!("Failed to fetch connections: {}", e)
        })?;
    
    println!("Found {} total connections", connections.len());
    let connections_with_app_id: Vec<_> = connections.into_iter()
        .filter(|conn| conn.app_id.is_some() && !conn.app_id.as_ref().unwrap().is_empty())
        .collect();
    
    println!("Found {} connections with valid app_id", connections_with_app_id.len());
    
    if connections_with_app_id.is_empty() {
        return Err("No connections found with valid app_id".to_string());
    }
    
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
    
    let mut successful = 0;
    let total = connections_with_app_id.len();
    let mut successful_app_ids = Vec::new();
    
    println!("Starting template creation for {} connections", connections_with_app_id.len());
    
    let mut skipped = 0;
    let mut skipped_app_ids = Vec::new();
    
    for (index, connection) in connections_with_app_id.iter().enumerate() {
        let app_id = connection.app_id.as_ref().unwrap();
        println!("[{}/{}] Processing app_id: {}", index + 1, total, app_id);

        let template_request = TemplateRequest::new(
            &params.template_name,
            &params.content,
            &app_id,
            category.clone(),
            template_type.clone(),
            &params.vertical,
        );
        
        let template_request = if let Some(ref header_text) = params.header_text {
            template_request.with_header_example(header_text)
        } else {
            template_request
        };
        
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
        
        if result.status != "success" {
            // Check if the error is about an existing template
            let error_message = result.message.unwrap_or("Unknown error".to_string());
            if error_message.contains("Template Already exists with same namespace and elementName and languageCode") {
                println!("Template already exists for app_id: {}, skipping", app_id);
                skipped += 1;
                skipped_app_ids.push(app_id.clone());
                continue; // Skip to the next connection
            }
            
            // For other errors, fail the operation
            return Err(format!(
                "Failed to create template for app_id {}: {}", 
                app_id, 
                error_message
            ));
        }
        
        println!("Template created successfully for app_id: {}", app_id);
        successful += 1;
        successful_app_ids.push(app_id.clone());
    }
    
    println!("Bulk template creation completed: {}/{} successful, {} skipped (already exist)", 
             successful, total, skipped);
    
    Ok(BulkCreateResult {
        successful,
        total,
        app_ids: successful_app_ids,
    })
}