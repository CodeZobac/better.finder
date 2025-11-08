#[cfg(test)]
mod tests {
    use super::super::theme::*;
    use crate::settings::Theme;

    #[test]
    fn test_resolve_theme_light() {
        let result = resolve_theme(Theme::Light);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Theme::Light);
    }

    #[test]
    fn test_resolve_theme_dark() {
        let result = resolve_theme(Theme::Dark);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Theme::Dark);
    }

    #[test]
    fn test_resolve_theme_system() {
        // Should not panic and return a valid theme
        let result = resolve_theme(Theme::System);
        assert!(result.is_ok());
        let theme = result.unwrap();
        assert!(theme == Theme::Light || theme == Theme::Dark);
    }

    #[test]
    fn test_detect_system_theme_does_not_panic() {
        // Should not panic even if registry access fails
        let result = detect_system_theme();
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_system_theme_returns_valid_theme() {
        let result = detect_system_theme();
        assert!(result.is_ok());
        let theme = result.unwrap();
        assert!(theme == Theme::Light || theme == Theme::Dark);
    }
}
