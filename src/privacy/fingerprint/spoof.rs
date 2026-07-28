// ── Anti-Fingerprinting Module ────────────────────────────────
//
// Here live all the spoofing strategies.
// They are injected before the page's JS executes via
// WebKit's UserContentManager, into ALL frames (including
// iframes) to prevent sub-frame bypass.
//
// Do NOT inject this into a single top-frame only — trackers
// spin up invisible iframes to read the clean OS navigator.

/// JS payload injected into EVERY page and EVERY sub-frame
/// before any page script executes.
pub fn anti_fingerprint_script() -> &'static str {
    include_root_str!(@scripts, "anti_fingerprint.js")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anti_fingerprint_script_contains_overrides() {
        let script = anti_fingerprint_script();

        // Prototype toString Protection
        assert!(script.contains("Function.prototype.toString"));
        assert!(script.contains("[native code]"));
        assert!(script.contains("_makeNative"));
        assert!(script.contains("_defineGetter"));
        assert!(script.contains("_overrideMethod"));

        // Viewport
        assert!(script.contains("screen.width"));
        assert!(script.contains("window.innerHeight"));

        // GPU
        assert!(script.contains("Juanita Banana GPU"));
        assert!(script.contains("Juanita Banana Graphics API"));

        // Navigator
        assert!(script.contains("JuanitaBanana/0.1"));
        assert!(script.contains("webdriver"));

        // Timezone
        assert!(script.contains("Intl.DateTimeFormat"));
        assert!(script.contains("Europe/London"));

        // Battery
        assert!(script.contains("navigator.getBattery"));
        assert!(script.contains("charging: true"));

        // Fonts
        assert!(script.contains("CanvasRenderingContext2D.prototype"));
        assert!(script.contains("FontFaceSet.prototype"));

        // Sensors
        assert!(script.contains("DeviceMotionEvent.prototype overrides"));
        assert!(script.contains("DeviceOrientationEvent.prototype overrides"));
        assert!(script.contains("window.Accelerometer = createSensorMock"));
        assert!(script.contains("window.Gyroscope = createSensorMock"));
    }
}
