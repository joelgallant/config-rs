use error::*;
use source::Source;
use std::collections::HashMap;
use std::convert::TryInto;
use std::env;
use value::{Value, ValueKind};

#[derive(Clone, Debug)]
pub struct Environment {
    /// Optional prefix that will limit access to the environment to only keys that
    /// begin with the defined prefix.
    ///
    /// A prefix with a separator of `_` is tested to be present on each key before its considered
    /// to be part of the source environment.
    ///
    /// For example, the key `CONFIG_DEBUG` would become `DEBUG` with a prefix of `config`.
    prefix: Option<String>,

    /// Optional character sequence that separates each key segment in an environment key pattern.
    /// Consider a nested configuration such as `redis.password`, a separator of `_` would allow
    /// an environment key of `REDIS_PASSWORD` to match.
    separator: Option<String>,

    /// Ignore empty env values (treat as unset).
    ignore_empty: bool,

    /// Parse numbers if they're detected.
    parse_numbers: bool,
}

impl Environment {
    pub fn new() -> Self {
        Environment::default()
    }

    pub fn with_prefix(s: &str) -> Self {
        Environment {
            prefix: Some(s.into()),
            ..Environment::default()
        }
    }

    pub fn prefix(mut self, s: &str) -> Self {
        self.prefix = Some(s.into());
        self
    }

    pub fn separator(mut self, s: &str) -> Self {
        self.separator = Some(s.into());
        self
    }

    pub fn ignore_empty(mut self, ignore: bool) -> Self {
        self.ignore_empty = ignore;
        self
    }

    pub fn parse_numbers(mut self, parse_numbers: bool) -> Self {
        self.parse_numbers = parse_numbers;
        self
    }
}

impl Default for Environment {
    fn default() -> Environment {
        Environment {
            prefix: None,
            separator: None,
            ignore_empty: false,
            parse_numbers: false,
        }
    }
}

impl Source for Environment {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<HashMap<String, Value>> {
        let mut m = HashMap::new();
        let uri: String = "the environment".into();

        let separator = match self.separator {
            Some(ref separator) => separator,
            _ => "",
        };

        // Define a prefix pattern to test and exclude from keys
        let prefix_pattern = self.prefix.as_ref().map(|prefix| prefix.clone() + "_");

        for (key, value) in env::vars() {
            // Treat empty environment variables as unset
            if self.ignore_empty && value.is_empty() {
                continue;
            }

            let mut key = key.to_string();

            // Check for prefix
            if let Some(ref prefix_pattern) = prefix_pattern {
                if key
                    .to_lowercase()
                    .starts_with(&prefix_pattern.to_lowercase())
                {
                    // Remove this prefix from the key
                    key = key[prefix_pattern.len()..].to_string();
                } else {
                    // Skip this key
                    continue;
                }
            }

            // If separator is given replace with `.`
            if !separator.is_empty() {
                key = key.replace(separator, ".");
            }

            let value = if self.parse_numbers {
                if let Ok(parsed) = value.parse() {
                    ValueKind::Integer(parsed)
                } else if let Ok(parsed) = value.parse() {
                    ValueKind::Float(parsed)
                } else {
                    ValueKind::String(value)
                }
            } else {
                ValueKind::String(value)
            };

            m.insert(key.to_lowercase(), Value::new(Some(&uri), value));
        }

        Ok(m)
    }
}
