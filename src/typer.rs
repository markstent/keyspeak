use anyhow::Result;

/// Type text at the current cursor position in whatever app is focused.
pub fn type_at_cursor(text: &str) -> Result<()> {
    if text.is_empty() {
        return Ok(());
    }

    use enigo::{Enigo, Keyboard, Settings};
    let mut e = Enigo::new(&Settings::default()).map_err(|e| {
        anyhow::anyhow!(
            "Cannot inject text: {:?}\n\
             Fix: System Settings → Privacy & Security \
             → Accessibility → ✅ KeySpeak (or Terminal)",
            e
        )
    })?;

    e.text(text)
        .map_err(|e| anyhow::anyhow!("Typing failed: {:?}", e))?;
    Ok(())
}
