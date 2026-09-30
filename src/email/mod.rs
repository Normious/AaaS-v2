use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::config::Config;

pub mod templates;

pub struct EmailSender {
    mailer: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from_email: String,
    from_name: String,
    enabled: bool,
}

impl EmailSender {
    pub fn from_config(config: &Config) -> Self {
        if !config.smtp_enabled {
            return Self {
                mailer: None,
                from_email: config.smtp_from_email.clone(),
                from_name: config.smtp_from_name.clone(),
                enabled: false,
            };
        }

        let creds = Credentials::new(
            config.smtp_username.clone(),
            config.smtp_password.clone(),
        );

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
            .ok()
            .map(|b| b.port(config.smtp_port).credentials(creds).build());

        Self {
            mailer,
            from_email: config.smtp_from_email.clone(),
            from_name: config.smtp_from_name.clone(),
            enabled: true,
        }
    }

    pub async fn send_verification(
        &self,
        to: &str,
        name: Option<&str>,
        verify_url: &str,
    ) -> anyhow::Result<()> {
        self.send_html(to, "Verify your email", templates::verification_email(name, verify_url))
            .await
    }

    pub async fn send_password_reset(
        &self,
        to: &str,
        name: Option<&str>,
        reset_url: &str,
    ) -> anyhow::Result<()> {
        self.send_html(
            to,
            "Reset your password",
            templates::password_reset_email(name, reset_url),
        )
        .await
    }

    pub async fn send_email_changed(
        &self,
        to: &str,
        name: Option<&str>,
        verify_url: &str,
    ) -> anyhow::Result<()> {
        self.send_html(
            to,
            "Confirm your new email",
            templates::verification_email(name, verify_url),
        )
        .await
    }

    async fn send_html(&self, to: &str, subject: &str, html: String) -> anyhow::Result<()> {
        if !self.enabled || self.mailer.is_none() {
            tracing::warn!(to = %to, subject = %subject, "SMTP disabled; email not sent");
            return Ok(());
        }

        let from = format!("{} <{}>", self.from_name, self.from_email);
        let email = Message::builder()
            .from(from.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html)?;

        self.mailer.as_ref().unwrap().send(email).await?;
        Ok(())
    }
}
