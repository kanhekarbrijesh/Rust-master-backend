// src/_utils/security.rs

/// Defines the strategy used to mask or secure a sensitive value
#[derive(Debug, Clone, Copy)]
pub enum SecretMaskStrategy {
    /// Completely obscures the value (e.g., replaces everything with "********")
    #[allow(dead_code)]
    FullyRedacted,

    /// Masks everything except a few visible characters at the start and end (e.g., API keys, credit cards)
    /// Parameters: (visible_prefix_len, visible_suffix_len)
    #[allow(dead_code)]
    ExposeEdges(usize, usize),

    /// Parses database URI strings and strips out only the `username:password` segment
    DatabaseUri,
}

/// A centralized secure configuration wrapper for logs and diagnostics
pub struct SecureLogUtil;

impl SecureLogUtil {
    /// Secures a sensitive value dynamically based on its key name and selected strategy
    ///
    /// # Arguments
    /// * `key` - The name/label of the environment variable or field (used for descriptive warnings)
    /// * `value` - The raw secret string
    /// * `strategy` - How to securely transform this value
    pub fn mask_value(key: &str, value: &str, strategy: SecretMaskStrategy) -> String {
        if value.is_empty() {
            return format!("[{} IS EMPTY/MISSING]", key);
        }

        match strategy {
            SecretMaskStrategy::FullyRedacted => "[REDACTED]".to_string(),

            SecretMaskStrategy::ExposeEdges(prefix, suffix) => {
                let len = value.chars().count();
                // If the string is too short to safely expose edges, redact it completely
                if len <= (prefix + suffix) {
                    return "********".to_string();
                }

                let chars: Vec<char> = value.chars().collect();
                let prefix_str: String = chars[..prefix].iter().collect();
                let suffix_str: String = chars[len - suffix..].iter().collect();

                format!("{}********{}", prefix_str, suffix_str)
            }

            SecretMaskStrategy::DatabaseUri => {
                if let Some(scheme_end) = value.find("://") {
                    let scheme = &value[..scheme_end + 3]; // e.g., "mongodb+srv://"
                    let remainder = &value[scheme_end + 3..];

                    if let Some(auth_end) = remainder.find('@') {
                        // There is a username/password block present!
                        let cluster_details = &remainder[auth_end..]; // Includes the '@' and everything after
                        format!("{}{}{}", scheme, "******:******", cluster_details)
                    } else {
                        // It's a local connection with NO username/password credentials (safe to print!)
                        value.to_string()
                    }
                } else {
                    // Fallback if it doesn't even contain "://"
                    format!("[REDACTED {} - INVALID URI FORMAT]", key)
                }
            }
        }
    }
}
