use std::fs;
use lettre::message::Mailbox;
use serde::Deserialize;
use std::path::PathBuf;


#[derive(Deserialize, Debug)]
pub struct ConfigHandler {
    pub sql_connection_string: String,
    pub server_address: String,
    pub server_port: u16,
    pub cert: PathBuf,
    pub key: PathBuf,
    pub password_requirments: PasswordRequirements,
    pub require_email_verification: bool,
    pub email_config: Option<EmailConfig>
}

#[derive(Deserialize, Debug)]
pub struct PasswordRequirements {
    pub minimum_size: usize,
    pub maximum_size: usize,
    pub forbidden_characters: String
}

#[derive(Deserialize, Debug)]
pub struct EmailConfig {
    pub smtp_username: String,
    pub smtp_password: String,
    pub server_domain: String,
    pub from_mailbox: Mailbox,
    pub replay_to_mailbox: Mailbox,
    pub subject: String,
    pub header: String,
    //pub body_template_path: PathBuf,
    pub body: String
}

impl ConfigHandler {
    pub fn parse_config(relative_path: String) -> Result<ConfigHandler, serde_json::Error>{
        let config_text = fs::read_to_string(relative_path).unwrap();
        serde_json::from_str::<ConfigHandler>(&config_text)
    } 
}

