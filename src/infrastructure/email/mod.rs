pub mod email_sender;
pub mod smtp_email_sender;

pub use email_sender::{EmailSender, EmailMessage, EmailError};
pub use smtp_email_sender::SmtpEmailSender;

