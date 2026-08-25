use rapidhash::{RapidHashMap as HashMap, RapidHashSet};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use toml;
use crate::helpers::show_fatal_error;

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct Config {
    pub compute_shaders: ShaderConfig,
    pub pixel_shaders: ShaderConfig,
    pub vertex_shaders: ShaderConfig,
    pub logging: bool,
    pub log_async: bool,
    pub dump_shaders: bool,
    pub next_dll: Option<String>,
}

impl Config {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn load() -> Self {
        let path = "IndirectX.toml";

        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                // File doesn't exist -> generate default config and save to disk
                let default_config = Config::new();
                let toml_string = toml::to_string_pretty(&default_config)
                    .expect("[IndirectX] Failed to serialize default config");

                if let Err(e) = std::fs::write(path, &toml_string) {
                    panic!("[IndirectX] Failed to write default config to {path}: {e:?}");
                }

                return default_config;
            }
            Err(err) => {
                // File exists but failed to read (permissions, lock, etc.) -> crash immediately
                panic!("[IndirectX] Critical error reading config file '{path}': {err:?}");
            }
        };

        // File exists -> parse it. If it fails, log the exact TOML error and exit
        let mut config: Config = match toml::from_str(&content) {
            Ok(cfg) => cfg,
            Err(err) => {
                let error_msg = format!(
                    "Failed to parse '{path}'.\n\nTOML Error:\n{err}\n\nPlease fix your config format!"
                );
                show_fatal_error("FATAL ERROR", &error_msg);
                std::process::exit(1);
            }
        };

        if config.next_dll.is_none() {
            let next_dll_path = "C:\\Windows\\System32\\d3d11.dll";
            config.next_dll = Some(next_dll_path.to_string());
        }

        config
    }

    pub fn save(&self) {
        let path = "IndirectX.toml";
        let toml_string = toml::to_string_pretty(self).unwrap();
        let _ = std::fs::write(path, toml_string);
    }
}

/// Custom ShaderHex type supporting unpadded, non-prefixed compact hex strings
/// as both values (Vec/HashSet) and TOML Map Keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ShaderHex(pub u64);

impl<'de> Deserialize<'de> for ShaderHex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ShaderHexVisitor;

        impl<'de> de::Visitor<'de> for ShaderHexVisitor {
            type Value = ShaderHex;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a compact hex string (e.g. \"e0c1b1fd5a68f4d9\") or a 64-bit integer")
            }

            fn visit_str<E>(self, value: &str) -> Result<ShaderHex, E>
            where
                E: de::Error,
            {
                let clean_hex = value.trim_start_matches("0x");
                u64::from_str_radix(clean_hex, 16)
                    .map(ShaderHex)
                    .map_err(|_| E::custom(format!("invalid hex string: '{value}'")))
            }

            fn visit_borrowed_str<E>(self, value: &'de str) -> Result<ShaderHex, E>
            where
                E: de::Error,
            {
                self.visit_str(value)
            }

            fn visit_string<E>(self, value: String) -> Result<ShaderHex, E>
            where
                E: de::Error,
            {
                self.visit_str(&value)
            }

            fn visit_u64<E>(self, value: u64) -> Result<ShaderHex, E>
            where
                E: de::Error,
            {
                Ok(ShaderHex(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<ShaderHex, E>
            where
                E: de::Error,
            {
                Ok(ShaderHex(value as u64))
            }
        }

        deserializer.deserialize_str(ShaderHexVisitor)
    }
}

impl Serialize for ShaderHex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Unpadded, non-prefixed compact hex output (e.g., "e0c1b1fd5a68f4d9")
        serializer.serialize_str(&format!("{:x}", self.0))
    }
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct ShaderConfig {
    pub skip: RapidHashSet<ShaderHex>,
    pub replace: RapidHashSet<ShaderHex>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cbv_patch: Option<HashMap<ShaderHex, Vec<CbvOverride>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CbvOverride {
    /// Slot index of the constant buffer (e.g., b0 = 0, b1 = 1)
    pub slot: u32,

    /// Byte offset within the CBuffer struct where mutation begins (must be 16-byte aligned for D3D11 floats/vectors)
    pub offset: usize,

    /// The payload/data to patch at the specified offset
    pub value: CbvValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CbvValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Int(i32),
    Uint(u32),
    Matrix4x4([[f32; 4]; 4]),
    Raw(Vec<u8>),
}

impl CbvValue {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            CbvValue::Float(v) => v.to_ne_bytes().to_vec(),
            CbvValue::Int(v) => v.to_ne_bytes().to_vec(),
            CbvValue::Uint(v) => v.to_ne_bytes().to_vec(),
            CbvValue::Vec2(v) => v.iter().flat_map(|f| f.to_ne_bytes()).collect(),
            CbvValue::Vec3(v) => v.iter().flat_map(|f| f.to_ne_bytes()).collect(),
            CbvValue::Vec4(v) => v.iter().flat_map(|f| f.to_ne_bytes()).collect(),
            CbvValue::Matrix4x4(m) => m.iter().flatten().flat_map(|f| f.to_ne_bytes()).collect(),
            CbvValue::Raw(bytes) => bytes.clone(),
        }
    }
}