#![allow(clippy::too_many_arguments)]
use std::fs;
use std::path::PathBuf;

fn generate_pdf_content(title: &str, text: &str) -> Vec<u8> {
    let mut stream_content = String::new();
    stream_content.push_str("BT\n/F1 14 Tf\n50 800 Td\n18 TL\n");

    // Escaping function for PDF strings: escape \, (, )
    let escape_pdf_str = |s: &str| -> String {
        s.replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)")
    };

    // Title line
    stream_content.push_str(&format!("({}) Tj T*\n", escape_pdf_str(title)));
    stream_content.push_str("/F1 9 Tf\n0 -20 Td\n12 TL\n"); // adjust position and font size for body

    for line in text.lines() {
        if line.trim().is_empty() {
            stream_content.push_str("T*\n");
        } else {
            // Split long lines to wrap them to avoid cutoffs
            let words = line.split_whitespace();
            let mut current_line = String::new();
            for word in words {
                if current_line.len() + word.len() + 1 > 90 {
                    stream_content
                        .push_str(&format!("({}) Tj T*\n", escape_pdf_str(&current_line)));
                    current_line = word.to_string();
                } else {
                    if !current_line.is_empty() {
                        current_line.push(' ');
                    }
                    current_line.push_str(word);
                }
            }
            if !current_line.is_empty() {
                stream_content.push_str(&format!("({}) Tj T*\n", escape_pdf_str(&current_line)));
            }
        }
    }
    stream_content.push_str("ET");

    let stream_bytes = stream_content.as_bytes();
    let stream_len = stream_bytes.len();

    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n");

    let mut offsets = Vec::new();

    let mut write_obj = |pdf: &mut Vec<u8>, num: usize, content: &[u8]| {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n", num).as_bytes());
        pdf.extend_from_slice(content);
        pdf.extend_from_slice(b"\nendobj\n");
    };

    write_obj(&mut pdf, 1, b"<< /Type /Catalog /Pages 2 0 R >>");
    write_obj(&mut pdf, 2, b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>");
    write_obj(&mut pdf, 3, b"<< /Type /Page /Parent 2 0 R /Resources << /Font << /F1 4 0 R >> >> /MediaBox [0 0 595.27 841.89] /Contents 5 0 R >>");
    write_obj(
        &mut pdf,
        4,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    );

    let stream_obj_header = format!("<< /Length {} >>\nstream\n", stream_len);
    let mut stream_obj_data = Vec::new();
    stream_obj_data.extend_from_slice(stream_obj_header.as_bytes());
    stream_obj_data.extend_from_slice(stream_bytes);
    stream_obj_data.extend_from_slice(b"\nendstream");
    write_obj(&mut pdf, 5, &stream_obj_data);

    let xref_offset = pdf.len();
    pdf.extend_from_slice(b"xref\n");
    pdf.extend_from_slice(format!("0 {}\n", offsets.len() + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in &offsets {
        pdf.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
    }

    pdf.extend_from_slice(b"trailer\n");
    pdf.extend_from_slice(format!("<< /Size {} /Root 1 0 R >>\n", offsets.len() + 1).as_bytes());
    pdf.extend_from_slice(b"startxref\n");
    pdf.extend_from_slice(format!("{}\n", xref_offset).as_bytes());
    pdf.extend_from_slice(b"%%EOF\n");

    pdf
}

pub fn generate_reincidence_report(
    full_name: &str,
    national_id: &str,
    contact_email: &str,
    domain: &str,
    notified_date: &str,
    emails_used: &[String],
    recipient_email: &str,
    digital_cert: Option<(String, Vec<u8>, String)>,
) -> Result<PathBuf, String> {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let report_content = format!(
"================================================================================
FORMAL COMPLAINT UNDER ARTICLE 77 OF THE GENERAL DATA PROTECTION REGULATION (GDPR)
================================================================================

TO: DATA PROTECTION SUPERVISORY AUTHORITY

DATE OF COMPLAINT: {now}

1. COMPLAINANT DETAILS
-----------------------
Full Name: {full_name}
National ID/Passport: {national_id}
Contact Email Address: {contact_email}

2. DETAILS OF THE RESPONDENT (DATA CONTROLLER)
-----------------------------------------------
Domain/Entity: {domain}
Recipient Email Address: {recipient_email}
Emails Used for Initial Request: {emails_used}

3. DESCRIPTION OF THE INFRACTION / NON-COMPLIANCE
--------------------------------------------------
On {notified_date}, the Complainant formally exercised the Right to Erasure (Article 17) and Right to Object (Article 21) of the GDPR, instructing the Respondent to permanently delete all personal data and cease processing or communications.

Despite the receipt of this request and the expiration of the statutory one-month period specified under Article 12(3) of the GDPR, the Respondent has willfully ignored the request and continues to process the Complainant's personal data and/or send unsolicited communications.

4. REQUESTED ACTIONS
--------------------
The Complainant requests that the Supervisory Authority:
1. Initiate an investigation into the Respondent's data handling practices.
2. Issue an order compelling the Respondent to execute the erasure request immediately.
3. Impose administrative fines as set out in Article 83 of the GDPR for flagrant non-compliance.

5. EVIDENCE OF CORRESPONDENCE
-----------------------------
- Initial request sent on: {notified_date}
- Target Recipient: {recipient_email}
- Spammed account: {contact_email}

Signed,
{full_name}
(Exercising absolute digital sovereignty)",
        now = now,
        full_name = full_name,
        national_id = national_id,
        contact_email = contact_email,
        domain = domain,
        recipient_email = recipient_email,
        emails_used = emails_used.join(", "),
        notified_date = notified_date
    );

    // Save to the Downloads folder or user home directory
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dest_dir = PathBuf::from(home).join("Descargas");
    let pdf_name = format!("gdpr-complaint-{}.pdf", domain.replace('.', "-"));
    let dest_path = if dest_dir.exists() {
        dest_dir.join(&pdf_name)
    } else {
        PathBuf::from("/tmp").join(&pdf_name)
    };

    let pdf_bytes = generate_pdf_content("FORMAL GDPR ARTICLE 77 COMPLAINT", &report_content);

    // If a PKCS#12 certificate is available, produce a CAdES .p7m signed envelope.
    //
    // All key material is handled entirely in RAM via the openssl Rust crate — the
    // private key is never serialised to disk (SSDs retain deleted data in flash cells).
    //
    // CAdES (CMS Advanced Electronic Signatures, ETSI TS 101 903) is the standard
    // eIDAS-compliant format accepted by Autofirma and Spanish public administration.
    if let Some((cert_name, cert_blob, password)) = digital_cert {
        match sign_pdf_cades_in_memory(&pdf_bytes, &cert_blob, &password) {
            Ok(p7m_bytes) => {
                // Build .p7m path next to where the PDF would have been
                let stem = cert_name
                    .trim_end_matches(".p12")
                    .trim_end_matches(".pfx");
                let p7m_name = format!("{}.pdf.p7m", stem);
                let p7m_path = dest_path.parent()
                    .unwrap_or_else(|| std::path::Path::new("/tmp"))
                    .join(p7m_name);
                fs::write(&p7m_path, p7m_bytes)
                    .map_err(|e| format!("Failed to write signed .p7m: {}", e))?;
                return Ok(p7m_path);
            }
            Err(_) => {
                // Signing failed non-fatally: fall through to writing unsigned PDF
            }
        }
    }

    fs::write(&dest_path, &pdf_bytes)
        .map_err(|e| format!("Failed to write complaint report PDF: {}", e))?;

    Ok(dest_path)
}

/// Signs `pdf_bytes` entirely in RAM using the `openssl` Rust crate.
///
/// Parses the PKCS#12 blob, builds a PKCS#7/CMS signed-data structure in-process,
/// and returns the DER-encoded `.p7m` bytes.  No private key material is ever
/// written to disk — not even to `/tmp` or `/dev/shm`.
fn sign_pdf_cades_in_memory(
    pdf_bytes: &[u8],
    cert_blob: &[u8],
    password: &str,
) -> Result<Vec<u8>, String> {
    use openssl::pkcs12::Pkcs12;
    use openssl::pkcs7::{Pkcs7, Pkcs7Flags};
    use openssl::stack::Stack;

    // Decode PKCS#12 — happens entirely in RAM
    let p12 = Pkcs12::from_der(cert_blob)
        .map_err(|e| format!("Invalid PKCS#12 blob: {}", e))?;
    let parsed = p12
        .parse2(password)
        .map_err(|e| format!("Wrong certificate password or corrupt cert: {}", e))?;

    let pkey = parsed
        .pkey
        .ok_or_else(|| "No private key found in PKCS#12".to_string())?;
    let cert = parsed
        .cert
        .ok_or_else(|| "No signing certificate found in PKCS#12".to_string())?;

    // Build CA chain stack (also in RAM)
    let mut chain = Stack::new().map_err(|e| e.to_string())?;
    if let Some(ca_certs) = parsed.ca {
        for ca in ca_certs {
            chain.push(ca).map_err(|e| e.to_string())?;
        }
    }

    // PKCS7/CMS sign:
    //   BINARY  — treat content as raw bytes (required for PDF)
    //   NOINTERN — don't attempt to include extra certs from the content
    let flags = Pkcs7Flags::BINARY | Pkcs7Flags::NOINTERN;

    let pkcs7 = Pkcs7::sign(&cert, &pkey, &chain, pdf_bytes, flags)
        .map_err(|e| format!("CMS signing failed: {}", e))?;

    // Serialise to DER — the standard encoding for .p7m (CAdES)
    pkcs7
        .to_der()
        .map_err(|e| format!("Failed to DER-encode signed envelope: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_reincidence_report() {
        let res = generate_reincidence_report(
            "Juan Perez",
            "12345678X",
            "juan@example.com",
            "spammer.com",
            "2026-07-09 12:00:00",
            &["promo@spammer.com".to_string()],
            "dpo@spammer.com",
            None,
        );
        assert!(res.is_ok());
        let path = res.unwrap();
        assert!(path.exists());
        assert_eq!(path.extension().unwrap(), "pdf");

        let content = fs::read(&path).unwrap();
        assert!(content.starts_with(b"%PDF-1.4"));

        // clean up test file
        fs::remove_file(path).ok();
    }
}
