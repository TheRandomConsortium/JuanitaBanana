pub fn banned_page(uri: &str) -> String {
    let shared_css = crate::browsing::internal::SHARED_CSS.as_str();
    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Banned — Juanita Banana</title>
<style>
  {shared_css}
</style>
</head>
<body class="jb-page">
<div class="jb-container">
  <div style="font-size: 5rem; margin-bottom: var(--jb-space-md);">🍌</div>
  <div class="jb-title-group">
      <h1 class="jb-title">You blocked this website before.</h1>
      <div class="jb-subtitle">Go look for greener pastures elsewhere.</div>
  </div>
  <div class="jb-value-box" style="margin-bottom: var(--jb-space-2xl);">
      <code>{uri}</code>
  </div>
  <div class="jb-card-alert" style="text-align: center;">
      <div class="jb-card-text">Changed your mind? Open <a href="juanita://unban" class="jb-text-yellow">juanita://unban</a> and solve the equation.</div>
  </div>
  <nav class="jb-nav">
      <a class="jb-nav-link" href="juanita://home">Home</a>
      <a class="jb-nav-link" href="juanita://config">Settings</a>
  </nav>
</div></body></html>"#,
        shared_css = shared_css,
        uri = uri
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
    let shared_css = crate::browsing::internal::SHARED_CSS.as_str();
    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Unban Challenge</title>
<style>
  {shared_css}
</style>
</head>
<body class="jb-page">
<div class="jb-container">
  <div style="font-size: 5rem; margin-bottom: var(--jb-space-md);">🍌</div>
  <div class="jb-title-group">
      <h1 class="jb-title">Mathematical Redemption</h1>
      <div class="jb-subtitle">To unban <code>{domain}</code>, prove you are worthy.</div>
  </div>
  
  <div class="jb-card" style="text-align: center;">
      <div class="jb-value-box" style="font-size: 1.5rem; margin-bottom: var(--jb-space-lg);">
          {equation}
      </div>
      <div class="jb-text-secondary" style="font-size: 0.85em; margin-bottom: var(--jb-space-xl);">(Answer is an integer. No fractions.)</div>
      <form action="juanita://submit-unban" style="display: flex; gap: 10px; justify-content: center;">
          <input type="hidden" name="domain" value="{domain}">
          <input type="number" name="answer" placeholder="Answer..." class="jb-input" style="width: 150px; text-align: center;" required autofocus>
          <button type="submit" class="jb-btn jb-btn-primary">Submit</button>
      </form>
  </div>
</div></body></html>"#,
        shared_css = shared_css,
        domain = domain,
        equation = equation
    )
}

pub fn unban_list_page(domains: &std::collections::HashSet<String>) -> String {
    let shared_css = crate::browsing::internal::SHARED_CSS.as_str();
    let mut list_html = String::new();
    if domains.is_empty() {
        list_html.push_str("<div class='jb-card' style='text-align:center;'><p class='jb-text-secondary'>You have not banned any domains yet.</p></div>");
    } else {
        let mut sorted_domains: Vec<&String> = domains.iter().collect();
        sorted_domains.sort();
        list_html.push_str("<div class='jb-card'>");
        for domain in sorted_domains {
            list_html.push_str(&format!(
                r#"<div class="jb-card-item" style="display:flex; justify-content:space-between; align-items:center; margin-bottom: 10px;">
                    <span style="font-size:1.1rem; color:var(--jb-text-primary);">{domain}</span>
                    <a href="juanita://unban?domain={domain}" class="jb-btn jb-btn-primary" style="padding: 6px 16px; font-size: 0.85em;">Unban</a>
                   </div>"#,
                domain = domain
            ));
        }
        list_html.push_str("</div>");
    }

    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>Banned Domains List</title>
<style>
  {shared_css}
</style>
</head>
<body class="jb-page">
  <div class="jb-container">
      <div style="font-size: 3.5rem; margin-bottom: var(--jb-space-md);">🍌</div>
      <div class="jb-title-group">
          <h1 class="jb-title">Banned Domains</h1>
          <div class="jb-subtitle">Choose a domain to redeem.</div>
      </div>
      {list_html}
  </div>
</body></html>"#,
        shared_css = shared_css,
        list_html = list_html
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
