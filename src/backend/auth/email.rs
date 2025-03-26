
use rand::Rng;
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use std::time::{Duration, Instant};

pub struct Timecode
{
    pub code : String,
    pub time : Instant,
}
impl Timecode
{
    pub fn new(code: String) -> Self {
        Self {
            code,
            time: Instant::now(),
        }
    }
    pub fn is_valid(&self) -> bool {
        self.time.elapsed() < Duration::from_secs(600)
    }
}

pub fn generate_code() -> String {
    let mut rng = rand::thread_rng();
    (0..6).map(|_| rng.gen_range(0..10).to_string()).collect()
}

pub fn send_email(to: &str, code: &str) -> Result<(), Box<dyn std::error::Error>> {
    let email = Message::builder()
        .from("vaultify.do.not.reply@gmail.com".parse()?)
        .to(to.parse()?)
        .subject("Ton code de vérification 2FA")
        .body(format!("Voici ton code mouille mouille : {}", code))?;

    let creds = Credentials::new(
        "vaultify.do.not.reply@gmail.com".to_string(),
        "rvsf dykg gzpw patc".to_string(),
    );

    let mailer = SmtpTransport::relay("smtp.gmail.com")?
        .credentials(creds)
        .build();

    mailer.send(&email)?;
    Ok(())
}
