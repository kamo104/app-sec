use lettre::message::header::ContentType;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tracing::{debug, error};

pub struct EmailSender {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
}

impl EmailSender {
    pub fn new_mailhog() -> Self {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::unencrypted_localhost();

        Self {
            transport,
            from_email: "noreply@appsec.local".to_string(),
        }
    }

    pub async fn send_verification_email(
        &self,
        to_email: &str,
        verification_link: &str,
    ) -> anyhow::Result<()> {
        let body = format!(
            "Welcome to our app! Please verify your email by clicking the following link: \n\n{}",
            verification_link
        );

        let email = Message::builder()
            .from(self.from_email.parse()?)
            .to(to_email.parse()?)
            .header(ContentType::TEXT_PLAIN)
            .subject("Verify your email")
            .body(body)?;

        match self.transport.send(email).await {
            Ok(_) => {
                debug!("Email sent successfully to {}", to_email);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send email to {}: {:?}", to_email, e);
                Err(anyhow::anyhow!("Failed to send email: {:?}", e))
            }
        }
    }

    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        reset_link: &str,
    ) -> anyhow::Result<()> {
        let body = format!(
            "You have requested to reset your password. Please click the following link to set a new password: \n\n{}",
            reset_link
        );

        let email = Message::builder()
            .from(self.from_email.parse()?)
            .to(to_email.parse()?)
            .header(ContentType::TEXT_PLAIN)
            .subject("Reset your password")
            .body(body)?;

        match self.transport.send(email).await {
            Ok(_) => {
                debug!("Password reset email sent successfully to {}", to_email);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send reset email to {}: {:?}", to_email, e);
                Err(anyhow::anyhow!("Failed to send email: {:?}", e))
            }
        }
    }
}
