use serde::{Deserialize, Deserializer, Serialize, Serializer};
use windows::Win32::Graphics::Dxgi::Common::*;
use toml;

fn serialize_dxgi_format<S>(format: &DXGI_FORMAT, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i32(format.0)
}

fn deserialize_dxgi_format<'de, D>(deserializer: D) -> Result<DXGI_FORMAT, D::Error>
where
    D: Deserializer<'de>,
{
    let value = i32::deserialize(deserializer)?;
    Ok(DXGI_FORMAT(value))
}

#[derive(Deserialize, Serialize)]
pub struct TextureConfig {
    pub initial_resolution: (u32, u32),
    pub target_resolution: (u32, u32),
    #[serde(serialize_with = "serialize_dxgi_format", deserialize_with = "deserialize_dxgi_format")]
    pub format: DXGI_FORMAT,
}

#[derive(Deserialize, Serialize)]
pub struct ViewportConfig {
    pub initial_resolution: (u32, u32),
    pub target_resolution: (u32, u32),
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub texture: TextureConfig,
    pub viewport: ViewportConfig,
    pub logging: bool,
    pub log_async: bool,
    pub next_dll: Option<String>,
}

impl Config {
    fn new() -> Self {
        Self {
            texture: TextureConfig {
                initial_resolution: (2048, 2048),
                target_resolution: (512, 512),
                format: DXGI_FORMAT_R32_TYPELESS,
            },
            viewport: ViewportConfig {
                initial_resolution: (2048, 2048),
                target_resolution: (512, 512),
            },
            logging: true,
            log_async: true,
            next_dll: None,
        }
    }

    pub fn load() -> Self {
        let path = "IndirectX.toml";
        let content = std::fs::read_to_string(path).unwrap_or_else(|_| " ".to_string());    
        let mut config: Config = toml::from_str(&content).unwrap_or_else(|_| {
            let temp = Config::new();
            let toml_string = toml::to_string(&temp).unwrap();
            std::fs::write(path, toml_string).unwrap();
            temp
        });
        
        if config.next_dll.is_none() {
            let next_dll_path = "C:\\Windows\\System32\\d3d11.dll";
            config.next_dll = Some(next_dll_path.to_string());
        }
        config
    }


}
