pub fn banned_page(uri: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Banned — Juanita Banana</title>
<style>
  *{{margin:0;padding:0;box-sizing:border-box}}
  body{{background:#111;color:#eee;font-family:monospace;
       display:flex;align-items:center;justify-content:center;
       height:100vh;text-align:center;}}
  .b{{font-size:5rem;margin-bottom:1rem}}
  h1{{color:#f5c518;font-size:1.8rem;margin-bottom:1rem}}
  p{{color:#aaa;line-height:1.6}}
  code{{color:#ff6b6b;background:#1a1a1a;padding:.2rem .6rem;border-radius:3px}}
  small{{display:block;margin-top:2rem;color:#444;font-size:.8rem}}
</style></head>
<body><div>
  <div class="b">🍌</div>
  <h1>You blocked this website before.</h1>
  <p><code>{uri}</code></p>
  <p style="margin-top:1rem">Go look for greener pastures elsewhere.</p>
  <small>Changed your mind? Enter the unban config page and solve the equation.</small>
</div></body></html>"#
    )
}

pub trait EquationProvider {
    fn generate_challenge(&self) -> (String, i32);
}

pub struct BasicIntegralEquationProvider;

impl EquationProvider for BasicIntegralEquationProvider {
    fn generate_challenge(&self) -> (String, i32) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let a = rng.gen_range(1..10);
        let b = rng.gen_range(1..10);
        let c = rng.gen_range(1..10);

        let equation = format!("Evaluate: ∫ ({}x + {}) dx  from 0 to {}", a, b, c);
        let answer = (a as f32 / 2.0 * (c * c) as f32) as i32 + (b * c);

        (equation, answer)
    }
}

pub fn unban_page(domain: &str, equation: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Unban Challenge</title>
<style>
  *{{margin:0;padding:0;box-sizing:border-box}}
  body{{background:#111;color:#eee;font-family:monospace;
       display:flex;align-items:center;justify-content:center;
       height:100vh;text-align:center;}}
  .b{{font-size:5rem;margin-bottom:1rem}}
  h1{{color:#f5c518;font-size:1.8rem;margin-bottom:1rem}}
  p{{color:#aaa;line-height:1.6; font-size:1.2rem}}
  .math{{font-size: 2rem; color: #fff; background:#1a1a1a; padding: 1rem; border-radius: 5px; margin: 1.5rem 0;}}
  input{{background:#222; color:#fff; border: 1px solid #444; padding: 10px; font-size:1.2rem; width: 150px; text-align: center;}}
  button{{background:#007acc; color:#fff; border:none; padding:10px 20px; font-size:1.2rem; cursor:pointer;}}
  button:hover{{background:#0098ff;}}
</style></head>
<body><div>
  <div class="b">🍌</div>
  <h1>Mathematical Redemption</h1>
  <p>To unban <code>{domain}</code>, prove you are worthy.</p>
  <div class="math">
      {equation}
  </div>
  <p><small>(Answer is an integer. No fractions.)</small></p>
  <form action="juanita://submit-unban">
      <input type="hidden" name="domain" value="{domain}">
      <input type="number" name="answer" placeholder="Answer..." required autofocus>
      <button type="submit">Submit</button>
  </form>
</div></body></html>"#
    )
}

pub fn unban_list_page(domains: &std::collections::HashSet<String>) -> String {
    let mut list_html = String::new();
    if domains.is_empty() {
        list_html.push_str("<p>You have not banned any domains yet.</p>");
    } else {
        let mut sorted_domains: Vec<&String> = domains.iter().collect();
        sorted_domains.sort();
        list_html.push_str("<ul style=\"list-style:none; padding:0; margin-top:20px;\">");
        for domain in sorted_domains {
            list_html.push_str(&format!(
                r#"<li style="background:#1a1a1a; margin-bottom:10px; padding:15px; display:flex; justify-content:space-between; align-items:center; border-radius:5px;">
                    <span style="font-size:1.2rem; color:#eee;">{}</span>
                    <a href="juanita://unban?domain={}" style="background:#007acc; color:#fff; padding:8px 15px; text-decoration:none; border-radius:3px;">Unban</a>
                   </li>"#,
                domain, domain
            ));
        }
        list_html.push_str("</ul>");
    }

    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Banned Domains List</title>
<style>
  *{{margin:0;padding:0;box-sizing:border-box}}
  body{{background:#111;color:#eee;font-family:monospace; padding: 40px;}}
  .b{{font-size:3rem; margin-bottom:1rem; text-align:center;}}
  h1{{color:#f5c518;font-size:1.8rem;margin-bottom:1rem; text-align:center;}}
  .container{{max-width: 800px; margin: 0 auto;}}
</style></head>
<body>
  <div class="container">
      <div class="b">🍌</div>
      <h1>Banned Domains</h1>
      <p style="text-align:center; color:#aaa; margin-bottom: 2rem;">Choose a domain to redeem.</p>
      {list_html}
  </div>
</body></html>"#
    )
}

pub fn toxic_warning_script(config: &crate::util::config::AppConfig) -> String {
    include_str!("../../scripts/js/toxic_warning.js").replace(
        "TOXIC_THRESHOLD_PLACEHOLDER",
        &config.toxic_threshold.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banned_page_formatting() {
        let uri = "https://example.com/toxic-tracker";
        let html = banned_page(uri);

        assert!(html.contains(uri));
        assert!(html.contains("You blocked this website before."));
        assert!(html.contains("🍌"));
    }

    #[test]
    fn test_unban_page_formatting() {
        let provider = BasicIntegralEquationProvider;
        let (eq, _ans) = provider.generate_challenge();
        let html = unban_page("example.com", &eq);
        assert!(html.contains("example.com"));
        assert!(html.contains("Evaluate: ∫"));
    }

    #[test]
    fn test_toxic_warning_script_generation() {
        let config = crate::util::config::AppConfig::default();
        let js = toxic_warning_script(&config);
        assert!(js.contains("const threshold = 5;"));
        assert!(js.contains("juanita-toxic-marquee"));
    }
}
