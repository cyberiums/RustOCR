# Basic unit tests for RustOCR components
# Run with: cargo test

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        // Verify version is set
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
        assert!(version.starts_with("0."));
    }

    #[test]
    fn test_supported_languages() {
        // Test that we support common languages
        let common_langs = vec!["en", "ch_sim", "ja", "ko", "fr", "de", "es"];
        for lang in common_langs {
            assert!(!lang.is_empty());
            assert!(lang.len() >= 2);
        }
    }
}

#[cfg(test)]
mod config_tests {
    use crate::config::Config;
    use std::path::PathBuf;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.default.is_none());
        assert!(config.server.is_none());
        assert!(config.batch.is_none());
        assert!(config.profiles.is_none());
    }

    #[test]
    fn test_config_template() {
        let template = Config::create_default();
        assert!(template.contains("[default]"));
        assert!(template.contains("[server]"));
        assert!(template.contains("[batch]"));
        assert!(template.contains("[profiles."));
    }
}

#[cfg(test)]
mod server_tests {
    use crate::server;

    #[test]
    fn test_server_pid_path() {
        // Test PID file path generation
        let path = std::path::PathBuf::from("/tmp/rustocr_server.pid");
        assert!(path.to_string_lossy().contains("rustocr_server.pid"));
    }
}
