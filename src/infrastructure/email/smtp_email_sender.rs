use crate::infrastructure::email::{EmailSender, EmailMessage, EmailError};
use async_trait::async_trait;
use lettre::{
    Message, SmtpTransport, Transport,
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
};
use std::str::FromStr;

/// SMTP email sender configuration
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
}

#[allow(dead_code)]
impl SmtpConfig {
    /// Create a new SMTP configuration
    pub fn new(
        host: String,
        port: u16,
        username: String,
        password: String,
        from_email: String,
        from_name: String,
    ) -> Self {
        Self {
            host,
            port,
            username,
            password,
            from_email,
            from_name,
        }
    }

    /// Create SMTP config from environment variables
    pub fn from_env() -> Result<Self, EmailError> {
        Ok(Self {
            host: std::env::var("SMTP_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .map_err(|e| EmailError::ConfigError(format!("Invalid SMTP_PORT: {}", e)))?,
            username: std::env::var("SMTP_USERNAME")
                .unwrap_or_else(|_| "user@example.com".to_string()),
            password: std::env::var("SMTP_PASSWORD")
                .unwrap_or_else(|_| "password".to_string()),
            from_email: std::env::var("SMTP_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@example.com".to_string()),
            from_name: std::env::var("SMTP_FROM_NAME")
                .unwrap_or_else(|_| "URL Shortener".to_string()),
        })
    }
}

/// SMTP email sender implementation
pub struct SmtpEmailSender {
    config: SmtpConfig,
}

impl SmtpEmailSender {
    /// Create a new SMTP email sender
    pub fn new(config: SmtpConfig) -> Self {
        Self { config }
    }

    /// Create SMTP email sender from environment
    pub fn from_env() -> Result<Self, EmailError> {
        let config = SmtpConfig::from_env()?;
        Ok(Self::new(config))
    }

    /// Build the SMTP transport
    fn build_transport(&self) -> Result<SmtpTransport, EmailError> {
        let creds = Credentials::new(
            self.config.username.clone(),
            self.config.password.clone(),
        );

        let transport = SmtpTransport::relay(&self.config.host)
            .map_err(|e| EmailError::SmtpError(format!("Failed to create SMTP transport: {}", e)))?
            .port(self.config.port)
            .credentials(creds)
            .build();

        Ok(transport)
    }
}

#[async_trait]
impl EmailSender for SmtpEmailSender {
    async fn send_email(&self, message: EmailMessage) -> Result<(), EmailError> {
        // Parse from mailbox
        let from_mailbox = Mailbox::from_str(&format!(
            "{} <{}>",
            self.config.from_name, self.config.from_email
        ))
        .map_err(|e| EmailError::InvalidEmail(format!("Invalid from email: {}", e)))?;

        // Parse to mailbox
        let to_mailbox = Mailbox::from_str(&message.to)
            .map_err(|e| EmailError::InvalidEmail(format!("Invalid to email: {}", e)))?;

        // Build email message
        let email_builder = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(message.subject);

        // Add body (HTML if available, otherwise plain text)
        let email = if let Some(html_body) = message.html_body {
            email_builder
                .header(ContentType::TEXT_HTML)
                .body(html_body)
        } else {
            email_builder
                .header(ContentType::TEXT_PLAIN)
                .body(message.body)
        }
        .map_err(|e| EmailError::SendingFailed(format!("Failed to build email: {}", e)))?;

        // Build transport and send
        let transport = self.build_transport()?;
        
        transport
            .send(&email)
            .map_err(|e| EmailError::SendingFailed(format!("Failed to send email: {}", e)))?;

        tracing::info!("Email sent successfully to: {}", message.to);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smtp_config_creation() {
        let config = SmtpConfig::new(
            "smtp.example.com".to_string(),
            587,
            "user@example.com".to_string(),
            "password".to_string(),
            "noreply@example.com".to_string(),
            "Test Sender".to_string(),
        );

        assert_eq!(config.host, "smtp.example.com");
        assert_eq!(config.port, 587);
        assert_eq!(config.username, "user@example.com");
        assert_eq!(config.from_email, "noreply@example.com");
    }

    #[test]
    fn test_smtp_email_sender_creation() {
        let config = SmtpConfig::new(
            "smtp.example.com".to_string(),
            587,
            "user@example.com".to_string(),
            "password".to_string(),
            "noreply@example.com".to_string(),
            "Test Sender".to_string(),
        );

        let sender = SmtpEmailSender::new(config);
        assert_eq!(sender.config.host, "smtp.example.com");
    }
}
