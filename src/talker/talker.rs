use std::env;
use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::IntoResponse,
    Json, Router, routing,
};
use serde::{Serialize, Deserialize};
use tokio::net::TcpListener;
use crate::config::Config;
use crate::special_ops::shader_manager::{get_created_shaders, get_shader_stats, get_shader_bytecode, reload_replacement_shaders};
use crate::log;
use crate::CONFIG;

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/health", routing::get(|| async { "OK" }))
        .route("/", routing::get(get_process_details))
        .route("/api/v0/shaders/get_shaders_active", routing::get(|| async {
            Json(get_created_shaders())
        }))
        .route("/api/v0/shaders/get_shader_stats", routing::get(|| async {
            Json(get_shader_stats())
        }))
        .route("/api/v0/get_config", routing::get(||async {
            let config = CONFIG.get().unwrap().load();
            let config = config.as_ref();
            Json((*config).clone())
        }))
        .route("/api/v0/get_shader_bytecode", routing::get(shader_bytecode))
        .route("/api/v0/set_config", routing::post(set_config))
        .route("/api/v0/save_config",routing::post(save_config))
        .route("/api/v0/reload_replacement_shaders", routing::get(||async {
            reload_replacement_shaders();
            Json("OK")
        }));

    let listener = TcpListener::bind("0.0.0.0:2157").await?;
    log!("[+] IndirectX Axum server listening on http://0.0.0.0:2157");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_process_details() -> String {
    let process_name: std::path::PathBuf = env::current_exe().unwrap_or_default();
    let process_name = process_name.into_os_string().into_string().unwrap_or(String::from("No exe name found"));
    process_name
}

async fn set_config(Json(payload): Json<Config>) -> impl IntoResponse {
    let current_config = CONFIG.get().unwrap();
    current_config.store(payload.into());
    (StatusCode::OK, "Config updated successfully")
} 

async fn save_config(Json(payload): Json<Config>) -> impl IntoResponse {
    let current_config = CONFIG.get().unwrap();
    current_config.store(payload.into());
    let updated_config = current_config.load();
    updated_config.save();
    (StatusCode::OK, "Config saved successfully")
} 

#[derive(Serialize, Deserialize)]
pub struct BytecodeQuery {
    pub stage: String,
    pub hash: String, // Accepts both "0x1452..." hex or raw "1452..."
}

async fn shader_bytecode(Query(query): Query<BytecodeQuery>) -> impl IntoResponse {
    // 1. Clean and parse hash string (handles both hexadecimal and decimal query inputs)
    let hash_str = query.hash.trim();
    let hash = if let Some(hex_stripped) = hash_str.strip_prefix("0x") {
        u64::from_str_radix(hex_stripped, 16)
    } else if let Ok(parsed_u64) = hash_str.parse::<u64>() {
        Ok(parsed_u64)
    } else {
        u64::from_str_radix(hash_str, 16)
    };

    let hash = match hash {
        Ok(h) => h,
        Err(_) => {
            log!("[!] [/get_shader_bytecode] Invalid hash format: {}", query.hash);
            return (StatusCode::BAD_REQUEST, "Invalid shader hash query parameter").into_response();
        }
    };

    // 2. Query stored bytecode from ShaderTracker
    if let Some(bytecode) = get_shader_bytecode(&query.stage, hash) {
        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/octet-stream"),
                (
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{:#018x}.dxbc\"", hash).as_str(),
                ),
            ],
            bytecode, // Axum converts Vec<u8> into raw binary response body
        )
            .into_response()
    } else {
        log!(
            "[!] [/get_shader_bytecode] Shader not found for stage '{}' and hash {:#018x}",
            query.stage, hash
        );
        (StatusCode::NOT_FOUND, "Shader bytecode not found").into_response()
    }
}