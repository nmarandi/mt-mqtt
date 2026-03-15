use std::collections::HashMap;

/// Authentication module for username/password validation with secure password hashing
#[derive(Debug, Clone)]
pub struct Authenticator {
    // Username -> bcrypt password hash
    credentials: HashMap<String, String>,
    // ACL: username -> allowed topics (pub/sub patterns)
    acl: HashMap<String, TopicPermissions>,
    // If true, allow anonymous connections (no username/password)
    allow_anonymous: bool,
}

#[derive(Debug, Clone)]
pub struct TopicPermissions {
    pub publish: Vec<String>,
    pub subscribe: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum AuthResult {
    Success,
    InvalidCredentials,
    NotAuthorized,
}

impl Authenticator {
    pub fn new(allow_anonymous: bool) -> Self {
        Self {
            credentials: HashMap::new(),
            acl: HashMap::new(),
            allow_anonymous,
        }
    }

    /// Add a user with username and password (password will be hashed using bcrypt)
    pub fn add_user(&mut self, username: String, password: String) {
        // Hash the password with bcrypt (cost factor 12 is a good balance of security and performance)
        let hash = bcrypt::hash(password, 12).expect("Failed to hash password");
        self.credentials.insert(username, hash);
    }

    /// Add a user with a pre-hashed password (useful for loading from database)
    /// The hash should be a valid bcrypt hash string
    pub fn add_user_with_hash(&mut self, username: String, password_hash: String) {
        self.credentials.insert(username, password_hash);
    }

    /// Set topic permissions for a user
    pub fn set_permissions(&mut self, username: String, permissions: TopicPermissions) {
        self.acl.insert(username, permissions);
    }

    /// Authenticate a connection using bcrypt password verification
    pub fn authenticate(&self, username: Option<&str>, password: Option<&[u8]>) -> AuthResult {
        match (username, password) {
            (None, None) => {
                // No credentials provided
                if self.allow_anonymous {
                    AuthResult::Success
                } else {
                    AuthResult::InvalidCredentials
                }
            }
            (Some(user), Some(pass)) => {
                // Check if user exists and password matches using bcrypt
                if let Some(stored_hash) = self.credentials.get(user) {
                    let pass_str = String::from_utf8_lossy(pass);
                    // Use bcrypt's constant-time verification to prevent timing attacks
                    match bcrypt::verify(pass_str.as_ref(), stored_hash) {
                        Ok(true) => AuthResult::Success,
                        _ => AuthResult::InvalidCredentials,
                    }
                } else {
                    AuthResult::InvalidCredentials
                }
            }
            _ => {
                // Username provided but no password, or vice versa
                AuthResult::InvalidCredentials
            }
        }
    }

    /// Check if a user can publish to a topic
    pub fn can_publish(&self, username: Option<&str>, topic: &str) -> bool {
        // If anonymous and allowed, check if there are any ACL restrictions
        if username.is_none() && self.allow_anonymous {
            // Anonymous users can publish to any topic if no ACL is defined
            return true;
        }

        if let Some(user) = username {
            if let Some(perms) = self.acl.get(user) {
                return Self::topic_matches_patterns(topic, &perms.publish);
            }
            // If user exists but has no ACL, allow all
            if self.credentials.contains_key(user) {
                return true;
            }
        }

        false
    }

    /// Check if a user can subscribe to a topic
    pub fn can_subscribe(&self, username: Option<&str>, topic: &str) -> bool {
        // If anonymous and allowed, check if there are any ACL restrictions
        if username.is_none() && self.allow_anonymous {
            // Anonymous users can subscribe to any topic if no ACL is defined
            return true;
        }

        if let Some(user) = username {
            if let Some(perms) = self.acl.get(user) {
                return Self::topic_matches_patterns(topic, &perms.subscribe);
            }
            // If user exists but has no ACL, allow all
            if self.credentials.contains_key(user) {
                return true;
            }
        }

        false
    }

    /// Check if a topic matches any of the allowed patterns
    /// Supports wildcards: + (single level), # (multi level)
    fn topic_matches_patterns(topic: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            if Self::topic_matches_pattern(topic, pattern) {
                return true;
            }
        }
        false
    }

    /// Check if a topic matches a single pattern
    fn topic_matches_pattern(topic: &str, pattern: &str) -> bool {
        let topic_parts: Vec<&str> = topic.split('/').collect();
        let pattern_parts: Vec<&str> = pattern.split('/').collect();

        let mut topic_idx = 0;
        let mut pattern_idx = 0;

        while pattern_idx < pattern_parts.len() {
            let pattern_part = pattern_parts[pattern_idx];

            if pattern_part == "#" {
                // Multi-level wildcard must be last segment and matches zero or more levels
                // Verify it's the last segment
                if pattern_idx != pattern_parts.len() - 1 {
                    return false; // Invalid pattern: # must be last
                }
                // Matches everything remaining (including nothing)
                return true;
            } else if topic_idx >= topic_parts.len() {
                // Pattern has more parts but topic is exhausted
                return false;
            } else if pattern_part == "+" {
                // Single-level wildcard matches exactly one level
                topic_idx += 1;
                pattern_idx += 1;
            } else if pattern_part == topic_parts[topic_idx] {
                // Exact match
                topic_idx += 1;
                pattern_idx += 1;
            } else {
                // No match
                return false;
            }
        }

        // Both must be fully consumed (all pattern parts matched and all topic parts consumed)
        topic_idx == topic_parts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anonymous_allowed() {
        let auth = Authenticator::new(true);
        assert_eq!(auth.authenticate(None, None), AuthResult::Success);
    }

    #[test]
    fn test_anonymous_not_allowed() {
        let auth = Authenticator::new(false);
        assert_eq!(auth.authenticate(None, None), AuthResult::InvalidCredentials);
    }

    #[test]
    fn test_valid_credentials() {
        let mut auth = Authenticator::new(false);
        auth.add_user("user1".to_string(), "pass1".to_string());

        assert_eq!(auth.authenticate(Some("user1"), Some(b"pass1")), AuthResult::Success);
    }

    #[test]
    fn test_invalid_password() {
        let mut auth = Authenticator::new(false);
        auth.add_user("user1".to_string(), "pass1".to_string());

        assert_eq!(auth.authenticate(Some("user1"), Some(b"wrong")), AuthResult::InvalidCredentials);
    }

    #[test]
    fn test_invalid_username() {
        let mut auth = Authenticator::new(false);
        auth.add_user("user1".to_string(), "pass1".to_string());

        assert_eq!(auth.authenticate(Some("wrong"), Some(b"pass1")), AuthResult::InvalidCredentials);
    }

    #[test]
    fn test_acl_publish_exact() {
        let mut auth = Authenticator::new(false);
        auth.add_user("user1".to_string(), "pass1".to_string());
        auth.set_permissions(
            "user1".to_string(),
            TopicPermissions {
                publish: vec!["home/temp".to_string()],
                subscribe: vec![],
            },
        );

        assert!(auth.can_publish(Some("user1"), "home/temp"));
        assert!(!auth.can_publish(Some("user1"), "home/humidity"));
    }

    #[test]
    fn test_acl_publish_wildcard_single() {
        let mut auth = Authenticator::new(false);
        auth.add_user("user1".to_string(), "pass1".to_string());
        auth.set_permissions(
            "user1".to_string(),
            TopicPermissions {
                publish: vec!["home/+/temp".to_string()],
                subscribe: vec![],
            },
        );

        assert!(auth.can_publish(Some("user1"), "home/room1/temp"));
        assert!(auth.can_publish(Some("user1"), "home/room2/temp"));
        assert!(!auth.can_publish(Some("user1"), "home/room1/humidity"));
    }

    #[test]
    fn test_acl_publish_wildcard_multi() {
        let mut auth = Authenticator::new(false);
        auth.add_user("user1".to_string(), "pass1".to_string());
        auth.set_permissions(
            "user1".to_string(),
            TopicPermissions {
                publish: vec!["home/#".to_string()],
                subscribe: vec![],
            },
        );

        // Multi-level wildcard should match zero or more levels
        assert!(auth.can_publish(Some("user1"), "home"));
        assert!(auth.can_publish(Some("user1"), "home/temp"));
        assert!(auth.can_publish(Some("user1"), "home/room1/temp"));
        assert!(auth.can_publish(Some("user1"), "home/room1/sensors/temp"));
        assert!(!auth.can_publish(Some("user1"), "office/temp"));
    }

    #[test]
    fn test_acl_subscribe() {
        let mut auth = Authenticator::new(false);
        auth.add_user("user1".to_string(), "pass1".to_string());
        auth.set_permissions(
            "user1".to_string(),
            TopicPermissions {
                publish: vec![],
                subscribe: vec!["sensors/#".to_string()],
            },
        );

        assert!(auth.can_subscribe(Some("user1"), "sensors/temp"));
        assert!(auth.can_subscribe(Some("user1"), "sensors/room/temp"));
        assert!(!auth.can_subscribe(Some("user1"), "actuators/light"));
    }

    #[test]
    fn test_no_acl_allows_all() {
        let mut auth = Authenticator::new(false);
        auth.add_user("user1".to_string(), "pass1".to_string());
        // No ACL set for user1

        assert!(auth.can_publish(Some("user1"), "any/topic"));
        assert!(auth.can_subscribe(Some("user1"), "any/topic"));
    }
}
