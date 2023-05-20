use std::fs;
use serde::Deserialize;



#[derive(Deserialize, Debug)]
pub struct ConfigHandler {
    pub sql_connection_string: String,
    pub public_key: String,
    pub private_key: String
}

pub fn parse_config(relative_path: String) -> Result<ConfigHandler, serde_json::Error>{
    let config_text = fs::read_to_string(relative_path).unwrap();
    return serde_json::from_str::<ConfigHandler>(&config_text);
} 