use async_trait::async_trait;
use thiserror::Error;

/// Email message structure
#[derive(Debug, Clone)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub html_body: Option<String>,
}

#[allow(dead_code)]
impl EmailMessage {
    /// Create a new email message
    pub fn new(to: String, subject: String, body: String) -> Self {
        Self {
            to,
            subject,
            body,
            html_body: None,
        }
    }

    /// Create a new email message with HTML body
    pub fn new_with_html(to: String, subject: String, body: String, html_body: String) -> Self {
        Self {
            to,
            subject,
            body,
            html_body: Some(html_body),
        }
    }

    /// Create a password reset email
    pub fn password_reset(to: String, reset_link: String, expires_in_hours: i64) -> Self {
        let subject = "Password Reset Request".to_string();

        let body = format!(
            "You have requested to reset your password.\n\n\
             Click the link below to reset your password:\n\
             {}\n\n\
             This link will expire in {} hours.\n\n\
             If you did not request this password reset, please ignore this email.\n\n\
             Best regards,\n\
             URL Shortener Team",
            reset_link, expires_in_hours
        );

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <title>Password Reset Request</title>
                <style>
                    body {{
                        font-family: Arial, sans-serif;
                        line-height: 1.6;
                        color: #333;
                        max-width: 600px;
                        margin: 0 auto;
                        padding: 20px;
                    }}
                    .container {{
                        background-color: #f9f9f9;
                        border-radius: 5px;
                        padding: 30px;
                        border: 1px solid #ddd;
                    }}
                    .button {{
                        display: inline-block;
                        padding: 12px 24px;
                        background-color: #007bff;
                        color: white;
                        text-decoration: none;
                        border-radius: 4px;
                        margin: 20px 0;
                    }}
                    .button:hover {{
                        background-color: #0056b3;
                    }}
                    .warning {{
                        color: #856404;
                        background-color: #fff3cd;
                        border: 1px solid #ffeaa7;
                        padding: 10px;
                        border-radius: 4px;
                        margin: 20px 0;
                    }}
                    .footer {{
                        margin-top: 30px;
                        padding-top: 20px;
                        border-top: 1px solid #ddd;
                        font-size: 12px;
                        color: #666;
                    }}
                </style>
            </head>
            <body>
                <div class="container">
                    <h2>Password Reset Request</h2>
                    <p>You have requested to reset your password.</p>
                    <p>Click the button below to reset your password:</p>
                    <a href="{}" class="button">Reset Password</a>
                    <div class="warning">
                        <strong>⚠️ Important:</strong> This link will expire in {} hours.
                    </div>
                    <p>If you did not request this password reset, please ignore this email.</p>
                    <div class="footer">
                        <p>Best regards,<br>URL Shortener Team</p>
                        <p><small>This is an automated message. Please do not reply to this email.</small></p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            reset_link, expires_in_hours
        );

        Self::new_with_html(to, subject, body, html_body)
    }

    /// Create an account deletion confirmation email
    pub fn account_deletion_confirmation(
        to: String,
        confirmation_link: String,
        expires_in_hours: i64,
    ) -> Self {
        let subject = "Confirm Account Deletion".to_string();

        let body = format!(
            "You have requested to delete your account.\n\n\
             This is a permanent action and cannot be undone. All your data including URLs and analytics will be permanently deleted.\n\n\
             Click the link below to confirm account deletion:\n\
             {}\n\n\
             This link will expire in {} hours.\n\n\
             If you did not request account deletion, please ignore this email and your account will remain active.\n\n\
             Best regards,\n\
             URL Shortener Team",
            confirmation_link, expires_in_hours
        );

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <title>Confirm Account Deletion</title>
                <style>
                    body {{
                        font-family: Arial, sans-serif;
                        line-height: 1.6;
                        color: #333;
                        max-width: 600px;
                        margin: 0 auto;
                        padding: 20px;
                    }}
                    .container {{
                        background-color: #f9f9f9;
                        border-radius: 5px;
                        padding: 30px;
                        border: 1px solid #ddd;
                    }}
                    .button {{
                        display: inline-block;
                        padding: 12px 24px;
                        background-color: #dc3545;
                        color: white;
                        text-decoration: none;
                        border-radius: 4px;
                        margin: 20px 0;
                    }}
                    .button:hover {{
                        background-color: #c82333;
                    }}
                    .warning {{
                        color: #721c24;
                        background-color: #f8d7da;
                        border: 1px solid #f5c6cb;
                        padding: 15px;
                        border-radius: 4px;
                        margin: 20px 0;
                    }}
                    .footer {{
                        margin-top: 30px;
                        padding-top: 20px;
                        border-top: 1px solid #ddd;
                        font-size: 12px;
                        color: #666;
                    }}
                </style>
            </head>
            <body>
                <div class="container">
                    <h2>⚠️ Confirm Account Deletion</h2>
                    <p>You have requested to delete your account.</p>
                    <div class="warning">
                        <strong>Warning:</strong> This is a permanent action and cannot be undone.<br>
                        All your data including:
                        <ul>
                            <li>Shortened URLs</li>
                            <li>Analytics data</li>
                            <li>Profile information</li>
                        </ul>
                        will be permanently deleted.
                    </div>
                    <p>Click the button below to confirm account deletion:</p>
                    <a href="{}" class="button">Confirm Account Deletion</a>
                    <div class="warning">
                        <strong>⏱️ Important:</strong> This link will expire in {} hours.
                    </div>
                    <p>If you did not request account deletion, please ignore this email and your account will remain active.</p>
                    <div class="footer">
                        <p>Best regards,<br>URL Shortener Team</p>
                        <p><small>This is an automated message. Please do not reply to this email.</small></p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            confirmation_link, expires_in_hours
        );

        Self::new_with_html(to, subject, body, html_body)
    }
}

/// Email sender trait for sending emails
#[async_trait]
#[allow(dead_code)]
pub trait EmailSender: Send + Sync {
    /// Send an email message
    async fn send_email(&self, message: EmailMessage) -> Result<(), EmailError>;

    /// Send multiple email messages
    async fn send_bulk_emails(&self, messages: Vec<EmailMessage>) -> Result<(), EmailError> {
        for message in messages {
            self.send_email(message).await?;
        }
        Ok(())
    }
}

/// Email sending errors
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum EmailError {
    #[error("SMTP error: {0}")]
    SmtpError(String),

    #[error("Invalid email address: {0}")]
    InvalidEmail(String),

    #[error("Email configuration error: {0}")]
    ConfigError(String),

    #[error("Email sending failed: {0}")]
    SendingFailed(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_message_creation() {
        let message = EmailMessage::new(
            "test@example.com".to_string(),
            "Test Subject".to_string(),
            "Test Body".to_string(),
        );

        assert_eq!(message.to, "test@example.com");
        assert_eq!(message.subject, "Test Subject");
        assert_eq!(message.body, "Test Body");
        assert!(message.html_body.is_none());
    }

    #[test]
    fn test_password_reset_email() {
        let message = EmailMessage::password_reset(
            "user@example.com".to_string(),
            "https://example.com/reset?token=abc123".to_string(),
            24,
        );

        assert_eq!(message.to, "user@example.com");
        assert_eq!(message.subject, "Password Reset Request");
        assert!(message.body.contains("reset your password"));
        assert!(message.body.contains("24 hours"));
        assert!(message.html_body.is_some());

        let html = message.html_body.unwrap();
        assert!(html.contains("Reset Password"));
        assert!(html.contains("24 hours"));
    }
}
