use crate::unsubscribe::db::SmtpConfig;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub fn build_gdpr_notice(
    full_name: &str,
    national_id: &str,
    email_associated: &str,
    domain: &str,
) -> (String, String) {
    // Subject line
    let subject = format!("FORMAL NOTICE OF DATA ERASURE, REVOCATION OF CONSENT AND EXERCISE OF THE RIGHT TO BE FORGOTTEN (GDPR ARTICLE 17) - {}", domain);

    // Body template based on docs/GDPR_ARTICLE_17_TEMPLATE.md
    let body = format!(
        "TO: DATA PROTECTION OFFICER (DPO) / DATA PRIVACY DEPARTMENT\n\n\
        Pursuant to the General Data Protection Regulation (EU) 2016/679 (GDPR), I, {}, holding ID/PASSPORT NUMBER {}, identified on your platform via the email address {}, hereby serve you with this formal notification to terminate any and all data processing agreements, whether explicit, implicit, or derived from your standard \"Terms and Conditions.\"\n\n\
        Your current operational model, predicated on cognitive asymmetry and the systematic betrayal of user trust, constitutes a profound breach of the integrity expected of a data controller. Your role inversion—treating the data subject not as a customer, but as a mere asset for extractive modeling—is fundamentally incompatible with my digital sovereignty. Your business model is not an \"experience\"; it is a persistent violation of the autonomy guaranteed under European law.\n\n\
        Consequently, and in strict adherence to Article 17 (Right to Erasure/Right to be Forgotten) and Article 21 (Right to Object) of the GDPR, you are hereby instructed to execute the following actions immediately:\n\n\
        1. PERMANENT ERASURE: You are directed to delete, irreversibly and without delay, all personal data, behavioral patterns, historical session logs, and profile attributes associated with my identity from your internal databases and those of all downstream third-party processors or \"partners\" with whom my data has been shared.\n\n\
        2. REVOCATION OF CONSENT: Any previous consent given is formally revoked. You are prohibited from retaining, profiling, or processing my data for any purpose whatsoever.\n\n\
        3. CESSATION OF TRAFFIC: Any further attempt to initiate communication, re-targeting, or persistent tracking post-receipt of this notice shall be logged and documented as a flagrant violation of the GDPR.\n\n\
        Please expedite this request to the relevant legal department. You are reminded that administrative silence will be interpreted not as a procedural delay, but as willful negligence. Failure to comply within 72 hours of the receipt of this notice will compel me to escalate this matter to the relevant Supervisory Authority within the European Union, seeking maximum penalties as stipulated under Article 83 of the GDPR, without prejudice to potential civil litigation for damages resulting from your tortious data management practices.\n\n\
        I require no generic apology, nor any \"service improvement\" link. My digital presence within your ecosystem is a liability I intend to liquidate. Confirm receipt and execution of this erasure immediately.\n\n\
        Regards,\n\n\
        {}\n\n\
        (Exercising my absolute right to digital sovereignty against your extractive models)",
        full_name, national_id, email_associated, full_name
    );

    (subject, body)
}

pub fn send_gdpr_smtp(
    smtp: &SmtpConfig,
    to_email: &str,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    let email = Message::builder()
        .from(
            smtp.user
                .parse()
                .map_err(|e| format!("Invalid SMTP user address: {}", e))?,
        )
        .to(to_email
            .parse()
            .map_err(|e| format!("Invalid recipient address: {}", e))?)
        .subject(subject)
        .body(body.to_string())
        .map_err(|e| format!("Failed to build message: {}", e))?;

    let creds = Credentials::new(smtp.user.clone(), smtp.pass.clone());

    let mailer = SmtpTransport::relay(&smtp.server)
        .map_err(|e| format!("Failed to resolve SMTP server: {}", e))?
        .port(smtp.port)
        .credentials(creds)
        .build();

    mailer
        .send(&email)
        .map_err(|e| format!("Failed to send email: {}", e))?;
    Ok(())
}

pub fn build_mailto_link(to_email: &str, subject: &str, body: &str) -> String {
    format!(
        "mailto:{}?subject={}&body={}",
        urlencoding::encode(to_email),
        urlencoding::encode(subject),
        urlencoding::encode(body)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_gdpr_notice() {
        let (subject, body) =
            build_gdpr_notice("Juan Perez", "12345678X", "juan@example.com", "spammer.com");
        assert!(subject.contains("spammer.com"));
        assert!(body.contains("Juan Perez"));
        assert!(body.contains("12345678X"));
        assert!(body.contains("juan@example.com"));
        assert!(body.contains("Article 17"));
    }

    #[test]
    fn test_build_mailto_link() {
        let mailto = build_mailto_link("dpo@spammer.com", "Test Subject", "Test Body");
        assert!(
            mailto.starts_with("mailto:dpo%40spammer.com?subject=Test%20Subject&body=Test%20Body")
        );
    }
}
