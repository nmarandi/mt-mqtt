# Authentication & Authorization

## Overview

MT-MQTT includes a flexible authentication and authorization module that provides:
- **Username/password authentication** - Validate client credentials
- **Topic-based ACL (Access Control List)** - Fine-grained publish/subscribe permissions
- **Wildcard pattern matching** - Use MQTT wildcards in permission rules
- **Anonymous access control** - Configurable anonymous connections

## Architecture

The authentication module (`src/auth.rs`) is standalone and can be integrated with the broker or used independently.

> **⚠️ Security Warning**: The current implementation stores passwords in cleartext in memory. This is a security risk in production environments. Future versions will implement password hashing (e.g., Argon2, bcrypt) to protect credentials. For production use, passwords should be hashed before storage and validated using constant-time comparison.

### Components

```rust
pub struct Authenticator {
    credentials: HashMap<String, String>,       // username -> password (CLEARTEXT - not production-ready)
    acl: HashMap<String, TopicPermissions>,     // username -> permissions
    allow_anonymous: bool,                       // allow connections without credentials
}

pub struct TopicPermissions {
    pub publish: Vec<String>,      // Topics user can publish to
    pub subscribe: Vec<String>,    // Topics user can subscribe to
}

pub enum AuthResult {
    Success,
    InvalidCredentials,
    NotAuthorized,
}
```

## Basic Usage

### Creating an Authenticator

```rust
use mt_mqtt::auth::Authenticator;

// Create authenticator (anonymous connections NOT allowed)
let mut auth = Authenticator::new(false);

// Allow anonymous connections
let mut auth = Authenticator::new(true);
```

### Adding Users

```rust
// Add users with passwords
auth.add_user("sensor_device".to_string(), "secret123".to_string());
auth.add_user("admin".to_string(), "admin_pass".to_string());
auth.add_user("monitor".to_string(), "monitor123".to_string());
```

### Setting Permissions

```rust
use mt_mqtt::auth::TopicPermissions;

// Sensor device: can publish to sensors/*, subscribe to config/*
auth.set_permissions("sensor_device".to_string(), TopicPermissions {
    publish: vec!["sensors/#".to_string()],
    subscribe: vec!["config/+/sensor".to_string()],
});

// Admin: full access
auth.set_permissions("admin".to_string(), TopicPermissions {
    publish: vec!["#".to_string()],
    subscribe: vec!["#".to_string()],
});

// Monitor: read-only access to all topics
auth.set_permissions("monitor".to_string(), TopicPermissions {
    publish: vec![],  // Cannot publish
    subscribe: vec!["#".to_string()],  // Can subscribe to everything
});
```

## Authentication

### Validating Credentials

```rust
use mt_mqtt::auth::AuthResult;

// Valid credentials
let result = auth.authenticate(Some("sensor_device"), Some(b"secret123"));
assert_eq!(result, AuthResult::Success);

// Invalid password
let result = auth.authenticate(Some("sensor_device"), Some(b"wrong"));
assert_eq!(result, AuthResult::InvalidCredentials);

// Invalid username
let result = auth.authenticate(Some("unknown"), Some(b"password"));
assert_eq!(result, AuthResult::InvalidCredentials);

// Anonymous (if allowed)
let result = auth.authenticate(None, None);
// Success if allow_anonymous=true, InvalidCredentials otherwise
```

## Authorization (ACL)

### Checking Publish Permissions

```rust
// Check if user can publish to a topic
assert!(auth.can_publish(Some("sensor_device"), "sensors/temperature"));
assert!(auth.can_publish(Some("sensor_device"), "sensors/room1/humidity"));
assert!(!auth.can_publish(Some("sensor_device"), "actuators/light"));

// Admin can publish anywhere
assert!(auth.can_publish(Some("admin"), "any/topic/here"));
```

### Checking Subscribe Permissions

```rust
// Check if user can subscribe to a topic
assert!(auth.can_subscribe(Some("sensor_device"), "config/device/sensor"));
assert!(!auth.can_subscribe(Some("sensor_device"), "config/other/device"));

// Monitor can subscribe to anything
assert!(auth.can_subscribe(Some("monitor"), "any/topic/here"));
```

## Wildcard Patterns

ACL rules support MQTT wildcard patterns:

### Single-Level Wildcard (+)

Matches exactly one level in the topic hierarchy:

```rust
auth.set_permissions("user".to_string(), TopicPermissions {
    publish: vec!["home/+/temperature".to_string()],
    subscribe: vec![],
});

// Matches:
auth.can_publish(Some("user"), "home/living_room/temperature");  // ✓
auth.can_publish(Some("user"), "home/bedroom/temperature");      // ✓

// Does NOT match:
auth.can_publish(Some("user"), "home/temperature");              // ✗
auth.can_publish(Some("user"), "home/room/sub/temperature");     // ✗
auth.can_publish(Some("user"), "office/room/temperature");       // ✗
```

### Multi-Level Wildcard (#)

Matches zero or more levels in the topic hierarchy (must be last):

```rust
auth.set_permissions("user".to_string(), TopicPermissions {
    publish: vec!["sensors/#".to_string()],
    subscribe: vec![],
});

// Matches:
auth.can_publish(Some("user"), "sensors/temp");                  // ✓
auth.can_publish(Some("user"), "sensors/room1/temp");            // ✓
auth.can_publish(Some("user"), "sensors/room1/zone2/temp");      // ✓

// Does NOT match:
auth.can_publish(Some("user"), "actuators/light");               // ✗
```

### Combined Wildcards

```rust
auth.set_permissions("user".to_string(), TopicPermissions {
    publish: vec!["home/+/sensors/#".to_string()],
    subscribe: vec![],
});

// Matches:
auth.can_publish(Some("user"), "home/living_room/sensors/temp");        // ✓
auth.can_publish(Some("user"), "home/bedroom/sensors/humidity/data");   // ✓

// Does NOT match:
auth.can_publish(Some("user"), "home/sensors/temp");                    // ✗
auth.can_publish(Some("user"), "office/room1/sensors/temp");            // ✗
```

## Default Behavior

### No ACL Defined

If a user exists but has no ACL rules, they can publish and subscribe to any topic:

```rust
auth.add_user("new_user".to_string(), "password".to_string());
// No set_permissions() call

// User can access any topic
assert!(auth.can_publish(Some("new_user"), "any/topic"));
assert!(auth.can_subscribe(Some("new_user"), "any/topic"));
```

### Anonymous Access

When `allow_anonymous=true`, users without credentials can access any topic (unless ACL restricts):

```rust
let mut auth = Authenticator::new(true);

// Anonymous users can access anything
assert!(auth.can_publish(None, "any/topic"));
assert!(auth.can_subscribe(None, "any/topic"));
```

## Integration with Broker

### Option 1: Standalone Broker with Auth (Future)

```rust
use mt_mqtt::broker::Broker;
use mt_mqtt::auth::Authenticator;

let mut auth = Authenticator::new(false);
auth.add_user("user".to_string(), "pass".to_string());

let broker = Broker::with_authenticator(auth);
// Broker will check auth on CONNECT
```

### Option 2: Manual Integration (Current)

Since authentication is not fully integrated with the broker yet, you can use the module independently:

```rust
use mt_mqtt::auth::{Authenticator, AuthResult};

let mut auth = Authenticator::new(false);
auth.add_user("sensor".to_string(), "secret".to_string());

// In your connection handler:
fn handle_connect(username: Option<&str>, password: Option<&[u8]>) -> bool {
    let result = auth.authenticate(username, password);
    result == AuthResult::Success
}
```

## Security Best Practices

### 1. Strong Passwords

```rust
// Good
auth.add_user("device1".to_string(), "xK9$mP2#vL5@qR8&".to_string());

// Bad
auth.add_user("device1".to_string(), "password".to_string());
```

### 2. Principle of Least Privilege

Grant only the permissions needed:

```rust
// Good: Specific permissions
auth.set_permissions("sensor".to_string(), TopicPermissions {
    publish: vec!["sensors/temp".to_string()],
    subscribe: vec!["config/sensor".to_string()],
});

// Bad: Overly permissive
auth.set_permissions("sensor".to_string(), TopicPermissions {
    publish: vec!["#".to_string()],
    subscribe: vec!["#".to_string()],
});
```

### 3. Disable Anonymous Access in Production

```rust
// Production
let auth = Authenticator::new(false);

// Development only
let auth = Authenticator::new(true);
```

### 4. Use TLS/SSL

Always use TLS when deploying with authentication to protect credentials in transit (feature planned).

### 5. Separate Credentials by Device Type

```rust
// IoT devices
auth.add_user("sensor_001".to_string(), "sensor_secret_001".to_string());
auth.set_permissions("sensor_001".to_string(), TopicPermissions {
    publish: vec!["devices/sensors/#".to_string()],
    subscribe: vec!["config/sensors/#".to_string()],
});

// Backend services
auth.add_user("analytics_service".to_string(), "service_secret".to_string());
auth.set_permissions("analytics_service".to_string(), TopicPermissions {
    publish: vec!["analytics/#".to_string()],
    subscribe: vec!["devices/#".to_string()],
});

// Admin tools
auth.add_user("admin_user".to_string(), "admin_secret".to_string());
auth.set_permissions("admin_user".to_string(), TopicPermissions {
    publish: vec!["#".to_string()],
    subscribe: vec!["#".to_string()],
});
```

## Testing

The authentication module includes comprehensive unit tests:

### Running Tests

```bash
# Run all auth tests
cargo test auth::tests

# Run specific test
cargo test auth::tests::test_valid_credentials
```

### Test Coverage

- ✅ `test_anonymous_allowed` - Anonymous connections when enabled
- ✅ `test_anonymous_not_allowed` - Anonymous connections rejected when disabled
- ✅ `test_valid_credentials` - Valid username/password authentication
- ✅ `test_invalid_password` - Invalid password rejection
- ✅ `test_invalid_username` - Invalid username rejection
- ✅ `test_acl_publish_exact` - Exact topic matching
- ✅ `test_acl_publish_wildcard_single` - Single-level wildcard (+)
- ✅ `test_acl_publish_wildcard_multi` - Multi-level wildcard (#)
- ✅ `test_acl_subscribe` - Subscribe permissions
- ✅ `test_no_acl_allows_all` - Default allow-all behavior

## Example Configurations

### IoT Sensor Network

```rust
let mut auth = Authenticator::new(false);

// Temperature sensors
auth.add_user("temp_sensor_1".to_string(), "temp_secret_1".to_string());
auth.set_permissions("temp_sensor_1".to_string(), TopicPermissions {
    publish: vec!["sensors/temperature/#".to_string()],
    subscribe: vec!["config/temperature/+".to_string()],
});

// Humidity sensors
auth.add_user("humid_sensor_1".to_string(), "humid_secret_1".to_string());
auth.set_permissions("humid_sensor_1".to_string(), TopicPermissions {
    publish: vec!["sensors/humidity/#".to_string()],
    subscribe: vec!["config/humidity/+".to_string()],
});

// Data aggregator
auth.add_user("aggregator".to_string(), "agg_secret".to_string());
auth.set_permissions("aggregator".to_string(), TopicPermissions {
    publish: vec!["analytics/#".to_string()],
    subscribe: vec!["sensors/#".to_string()],
});
```

### Smart Home

```rust
let mut auth = Authenticator::new(false);

// Light controller
auth.add_user("light_ctrl".to_string(), "light_pass".to_string());
auth.set_permissions("light_ctrl".to_string(), TopicPermissions {
    publish: vec!["lights/+/status".to_string()],
    subscribe: vec!["lights/+/command".to_string()],
});

// Thermostat
auth.add_user("thermostat".to_string(), "thermo_pass".to_string());
auth.set_permissions("thermostat".to_string(), TopicPermissions {
    publish: vec!["hvac/status".to_string(), "sensors/temperature".to_string()],
    subscribe: vec!["hvac/setpoint".to_string()],
});

// Mobile app
auth.add_user("mobile_app".to_string(), "app_pass".to_string());
auth.set_permissions("mobile_app".to_string(), TopicPermissions {
    publish: vec!["lights/+/command".to_string(), "hvac/setpoint".to_string()],
    subscribe: vec!["#".to_string()],  // Read-only for everything else
});
```

## Performance Considerations

- **Credential lookup**: O(1) HashMap lookup
- **ACL checks**: O(n) where n = number of patterns (typically small)
- **Pattern matching**: Efficient string splitting and comparison
- **Memory usage**: ~100 bytes per user + patterns

The authentication module has minimal performance impact on broker operations.

## Limitations

- Passwords stored in memory (hashing not yet implemented)
- No password policy enforcement (length, complexity)
- No rate limiting on authentication attempts
- No audit logging
- No integration with external auth systems (LDAP, OAuth, etc.)

## Future Enhancements

Planned improvements:
- [ ] Password hashing (bcrypt/argon2)
- [ ] Integration with broker (automatic auth on CONNECT)
- [ ] External authentication providers
- [ ] Rate limiting
- [ ] Audit logging
- [ ] Dynamic user management API
- [ ] Password policies
- [ ] Token-based authentication
