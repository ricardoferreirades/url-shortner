pub mod email_sender;
pub mod smtp_email_sender;

pub use email_sender::{EmailError, EmailMessage, EmailSender};
pub use smtp_email_sender::SmtpEmailSender;
