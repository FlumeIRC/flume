use serde::{Deserialize, Serialize};

/// Configurable display format strings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormatsConfig {
    // Message display
    #[serde(default = "default_message")]
    pub message: String,
    #[serde(default = "default_message")]
    pub own_message: String,
    #[serde(default = "default_action")]
    pub action: String,
    #[serde(default = "default_notice")]
    pub notice: String,
    #[serde(default = "default_server_notice")]
    pub server_notice: String,
    #[serde(default = "default_system")]
    pub system: String,
    #[serde(default = "default_server")]
    pub server: String,

    // Events
    #[serde(default = "default_join")]
    pub join: String,
    #[serde(default = "default_part")]
    pub part: String,
    #[serde(default = "default_quit")]
    pub quit: String,
    #[serde(default = "default_kick")]
    pub kick: String,
    #[serde(default = "default_nick_change")]
    pub nick_change: String,
    #[serde(default = "default_topic")]
    pub topic: String,
    #[serde(default = "default_mode")]
    pub mode: String,

    // UI
    #[serde(default = "default_status_bar")]
    pub status_bar: String,
    #[serde(default = "default_buffer_entry")]
    pub buffer_entry: String,
    #[serde(default = "default_buffer_entry_unread")]
    pub buffer_entry_unread: String,
    #[serde(default = "default_buffer_entry_highlight")]
    pub buffer_entry_highlight: String,

    // Server notice routing rules
    #[serde(default)]
    pub snotice: Vec<SnoticeRuleConfig>,
}

impl Default for FormatsConfig {
    fn default() -> Self {
        Self {
            message: default_message(),
            own_message: default_message(),
            action: default_action(),
            notice: default_notice(),
            server_notice: default_server_notice(),
            system: default_system(),
            server: default_server(),
            join: default_join(),
            part: default_part(),
            quit: default_quit(),
            kick: default_kick(),
            nick_change: default_nick_change(),
            topic: default_topic(),
            mode: default_mode(),
            status_bar: default_status_bar(),
            buffer_entry: default_buffer_entry(),
            buffer_entry_unread: default_buffer_entry_unread(),
            buffer_entry_highlight: default_buffer_entry_highlight(),
            snotice: Vec::new(),
        }
    }
}

/// A server notice routing rule.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SnoticeRuleConfig {
    /// Regex pattern to match against the notice text.
    #[serde(rename = "match")]
    pub pattern: String,
    /// Format string using ${1}, ${2}, etc. for capture groups. None = raw text.
    #[serde(default)]
    pub format: Option<String>,
    /// Buffer name to route to. None = server buffer.
    #[serde(default)]
    pub buffer: Option<String>,
    /// If true, suppress (drop) the notice entirely.
    #[serde(default)]
    pub suppress: bool,
}

// Default format strings — match current hardcoded behavior
fn default_message() -> String { "<${nick}> ${text}".to_string() }
fn default_action() -> String { "* ${nick} ${text}".to_string() }
fn default_notice() -> String { "[notice] <${nick}> ${text}".to_string() }
fn default_server_notice() -> String { "[notice] ${text}".to_string() }
fn default_system() -> String { "-- ${text} --".to_string() }
fn default_server() -> String { "[${label}] ${text}".to_string() }
fn default_join() -> String { "--> ${nick} (${userhost}) has joined ${channel}".to_string() }
fn default_part() -> String { "<-- ${nick} has left ${channel}${?message| (${message})}".to_string() }
fn default_quit() -> String { "<-- ${nick} has quit${?message| (${message})}".to_string() }
fn default_kick() -> String { "<<< ${target} was kicked from ${channel} by ${nick}${?reason| (${reason})}".to_string() }
fn default_nick_change() -> String { "*** ${old_nick} is now known as ${new_nick}".to_string() }
fn default_topic() -> String { "${nick} changed topic of ${channel}: ${topic}".to_string() }
fn default_mode() -> String { "mode/${target} [${modes}] by ${nick}".to_string() }
fn default_status_bar() -> String { "${time} | [${nick}(${user_modes})] | [${channel}(${chan_modes})] | [${user_count}]".to_string() }
fn default_buffer_entry() -> String { " ${index}.${name}".to_string() }
fn default_buffer_entry_unread() -> String { " ${index}.${name}(${unread})".to_string() }
fn default_buffer_entry_highlight() -> String { " ${index}.${name}(${unread}!)".to_string() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = FormatsConfig::default();
        assert!(config.message.contains("${nick}"));
        assert!(config.join.contains("${channel}"));
        assert!(config.part.contains("${?message"));
        assert!(config.snotice.is_empty());
    }

    #[test]
    fn deserialize_with_snotice_rules() {
        let toml_str = r##"
message = "<${nick}> ${text}"

[[snotice]]
match = 'Client connecting: (\S+)'
format = "[connect] ${1}"
buffer = "snotice-connections"

[[snotice]]
match = 'Oper-up'
suppress = true
"##;
        let config: FormatsConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.snotice.len(), 2);
        assert_eq!(config.snotice[0].buffer, Some("snotice-connections".to_string()));
        assert!(!config.snotice[0].suppress);
        assert!(config.snotice[1].suppress);
    }

    #[test]
    fn deserialize_empty_defaults() {
        let config: FormatsConfig = toml::from_str("").unwrap();
        assert_eq!(config.message, default_message());
        assert_eq!(config.join, default_join());
    }
}
