use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::response::Response;
use lettre::transport::smtp::Error;
use lettre::{Message, AsyncSmtpTransport, Tokio1Executor, AsyncTransport};
use crate::config::EmailConfig;

pub async fn send_email_verification(dest_address: &Mailbox, email_config: &EmailConfig) -> Result<Response, Error> {
    
    let email = Message::builder()
        .from(email_config.from_mailbox.clone())
        .reply_to(email_config.replay_to_mailbox.clone())
        .to(dest_address.clone())
        .subject(email_config.subject.clone())
        .body(email_config.body.clone())
        .unwrap();
    
    let creds = Credentials::new(email_config.smtp_username.clone(), email_config.smtp_password.clone());

    let mailer  = AsyncSmtpTransport::<Tokio1Executor>::relay(&email_config.server_domain)
        .unwrap()
        .credentials(creds)
        .build();
    
    Ok(mailer.send(email).await?)
        
}