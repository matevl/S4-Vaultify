use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::Rng; // For handling current date and time
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Timecode {
    pub code: String,
    pub time: SystemTime, // Current system time (standard)
    pub email: String,
}

impl Timecode {
    pub fn new(code: String, email: String) -> Self {
        Self {
            code,
            time: SystemTime::now(), // Capture system time when sending
            email,
        }
    }

    pub fn is_valid(&self) -> bool {
        match SystemTime::now().duration_since(self.time) {
            Ok(duration) => duration < Duration::from_secs(600),
            Err(_) => false, // In case system clock changed
        }
    }

    pub fn timestamp(&self) -> Option<u64> {
        match self.time.duration_since(UNIX_EPOCH) {
            Ok(dur) => Some(dur.as_secs()),
            Err(_) => None,
        }
    }
}

// Function to generate a random 6-digit numeric code
pub fn generate_code() -> String {
    let mut rng = rand::rng();
    (0..6)
        .map(|_| rng.random_range(0..10).to_string())
        .collect()
}

// Function to send an email containing the verification code
pub fn send_email(to: &str, code: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create the email message with subject and body
    let email = Message::builder()
        .from("vaultify.do.not.reply@gmail.com".parse()?) // Sender address
        .to(to.parse()?) // Recipient address
        .subject("Your 2FA verification code") // Email subject
        .body(format!("Here is your verification code : {}", code))?; // Email body

    // Set up the credentials (email + app-specific password)
    let creds = Credentials::new(
        "vaultify.do.not.reply@gmail.com".to_string(),
        "rvsf dykg gzpw patc".to_string(), // App password from Gmail
    );

    // Set up the SMTP transport to send the email through Gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")?
        .credentials(creds)
        .build();

    // Send the email
    mailer.send(&email)?;
    Ok(())
}

// This is the main function that handles everything:
// it generates the code, sends it by email, and returns a Timecode struct
pub fn final_send(email_address: &str) -> Result<Timecode, Box<dyn std::error::Error>> {
    let code = generate_code(); // Generate the random code

    // Try to send the email and handle errors if they occur
    if let Err(e) = send_email(email_address, &code) {
        eprintln!("Error while sending email: {}", e);
        return Err(e); // Forward the error
    } else {
        // If the email is sent successfully, return the Timecode struct
        Ok(Timecode::new(code, email_address.to_string()))
    }
}

// Function to retrieve the stored verification code for an email
pub fn get_stored_code(email: &str) -> Option<String> {
    // Implement this function to retrieve the stored verification code for the given email
    // For example, you can use a HashMap or a database to store and retrieve the codes
    None
}
