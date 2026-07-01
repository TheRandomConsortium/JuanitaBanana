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
  <small>Changed your mind? Open juanita://config and solve the equation.</small>
</div></body></html>"#
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
}
