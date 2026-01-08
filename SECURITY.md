# Security Policy

## Supported Versions

We actively maintain odos-sdk with a focus on the latest releases:

- **Latest release**: Full support with active development
- **Recent releases**: Security fixes and critical bug fixes
- **Older releases**: Upgrade recommended

**Support Policy:**

- Pre-stable releases: Latest and previous minor versions supported
- Stable releases: Current and previous major versions supported
- Security fixes prioritized based on severity and affected versions

To check the current version and release notes, see [Releases](https://github.com/semiotic-ai/odos-sdk/releases) or [crates.io](https://crates.io/crates/odos-sdk).

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please report it responsibly.

### How to Report

**DO NOT** open a public GitHub issue for security vulnerabilities.

Instead, please report security vulnerabilities through:

- **GitHub Security Advisories**: [Report a vulnerability](https://github.com/semiotic-ai/odos-sdk/security/advisories/new)

This is the preferred and most secure method for reporting vulnerabilities.

Include in your report:

1. **Description**: Clear description of the vulnerability
2. **Impact**: What could an attacker achieve?
3. **Reproduction**: Step-by-step instructions to reproduce
4. **Version**: Affected version(s) of odos-sdk
5. **Proposed Fix**: (Optional) Suggestions for fixing the issue

### What to Expect

1. **Acknowledgment**: We will acknowledge receipt within 48 hours
2. **Assessment**: We will assess the report and determine severity within 5 business days
3. **Updates**: We will provide regular updates on our progress
4. **Fix Timeline**:
   - **Critical**: Patch within 7 days
   - **High**: Patch within 30 days
   - **Medium**: Patch within 90 days
   - **Low**: Addressed in next regular release
5. **Disclosure**: We will coordinate with you on public disclosure timing

### Security Advisories

Security advisories will be published through:

- GitHub Security Advisories
- RUSTSEC advisory database
- Release notes and CHANGELOG.md

## Security Best Practices for Users

### API Key Security

**CRITICAL**: Never expose your Odos API key in public repositories, logs, or error messages.

#### Secure Key Storage

```rust
use odos_sdk::{OdosClient, ClientConfig, ApiKey};
use std::env;

// ✅ GOOD: Load from environment variable
let api_key = env::var("ODOS_API_KEY")
    .ok()
    .and_then(|k| k.parse::<ApiKey>().ok());

let config = ClientConfig {
    api_key,
    ..Default::default()
};
let client = OdosClient::with_config(config)?;
```

```rust
// ❌ BAD: Hardcoded API key
let config = ClientConfig {
    api_key: Some("your-api-key".parse().unwrap()), // NEVER DO THIS
    ..Default::default()
};
```

#### Environment Variables

Use `dotenvy` to load API keys from `.env` files:

```rust
use dotenvy::dotenv;

// Load .env file
dotenv().ok();

// Access API key
let api_key = std::env::var("ODOS_API_KEY")
    .ok()
    .and_then(|k| k.parse().ok());
```

**Important**: Add `.env` to your `.gitignore`:

```gitignore
.env
.env.local
.env.*.local
```

### Rate Limiting

The SDK implements client-side retry logic, but you should also implement application-level rate limiting:

```rust
use odos_sdk::RetryConfig;

// Conservative retry configuration for production
let retry_config = RetryConfig {
    max_retries: 3,
    initial_backoff_ms: 200,
    retry_server_errors: true,
    retry_predicate: None,
};

let client = OdosClient::with_retry_config(retry_config)?;
```

### Input Validation

Always validate user inputs before constructing API requests:

```rust
use odos_sdk::QuoteRequest;
use alloy_primitives::{Address, U256};

// Validate addresses
let token_address = address_str.parse::<Address>()
    .map_err(|_| "Invalid token address")?;

// Validate amounts (prevent overflow)
let amount = U256::try_from(amount_str)
    .map_err(|_| "Invalid amount")?;

// Validate slippage (prevent excessive slippage)
let slippage = slippage_percent.max(0.1).min(50.0);

let quote = QuoteRequest::builder()
    .chain_id(chain_id)
    .input_tokens(vec![(token_address, amount).into()])
    // ... rest of request
    .build();
```

### Error Handling

Never expose sensitive information in error messages:

```rust
use odos_sdk::OdosError;

match client.quote(&request).await {
    Ok(quote) => {
        // Handle success
    }
    Err(e) => {
        // ✅ GOOD: Log internally, show generic message to users
        tracing::error!("Quote request failed: {}", e);
        return Err("Failed to get quote. Please try again.".into());
    }
}
```

```rust
// ❌ BAD: Exposing internal error details to users
match client.quote(&request).await {
    Err(e) => panic!("Error: {}", e), // May leak trace IDs, URLs, etc.
}
```

### Timeout Configuration

Configure appropriate timeouts to prevent resource exhaustion:

```rust
use std::time::Duration;

let config = ClientConfig {
    timeout: Duration::from_secs(30),
    connect_timeout: Duration::from_secs(10),
    ..Default::default()
};
```

### TLS/HTTPS

The SDK always uses HTTPS for API communication. The underlying `reqwest` client:

- Validates TLS certificates by default
- Uses system certificate store
- Enforces secure cipher suites

Never disable certificate validation in production.

## Dependency Security

### Regular Updates

We regularly audit and update dependencies. Users should:

1. **Update frequently**: Run `cargo update` regularly
2. **Monitor advisories**: Use `cargo audit` to check for vulnerabilities
3. **Review changes**: Read CHANGELOG.md before updating

### Dependency Auditing

Check for known vulnerabilities:

```bash
# Install cargo-audit
cargo install cargo-audit

# Run audit
cargo audit

# Check for specific advisories
cargo audit --deny warnings
```

### Minimal Dependencies

We maintain a minimal dependency footprint to reduce attack surface. See `DEPENDENCIES.md` for detailed rationale for each dependency.

## Cryptographic Operations

This SDK does not perform cryptographic operations directly. It relies on:

- **alloy ecosystem**: For Ethereum-related cryptography (signing, address derivation)
- **reqwest + rustls**: For TLS/HTTPS connections

Users performing transaction signing should:

1. Use hardware wallets when possible
2. Never expose private keys in logs or error messages
3. Use `alloy-signer` best practices for key management

## Network Security

### API Endpoints

The SDK connects to:

- `https://api.odos.xyz` (Public API)
- `https://enterprise-api.odos.xyz` (Enterprise API)

**Important**: The SDK does not support custom endpoints to prevent MitM attacks via endpoint injection.

### Connection Pool Security

The SDK uses connection pooling for performance. Default settings:

- Maximum 20 connections
- 90-second idle timeout
- Automatic cleanup of stale connections

### Proxy Support

If using HTTP proxies, ensure:

- Proxy connection uses TLS
- Proxy is trusted and properly configured
- Sensitive data is not logged by proxy

## Incident Response

If you suspect your application using odos-sdk has been compromised:

1. **Isolate**: Immediately isolate affected systems
2. **Rotate Keys**: Rotate any API keys that may have been exposed
3. **Investigate**: Determine scope of compromise
4. **Notify**: [Report via GitHub Security Advisories](https://github.com/semiotic-ai/odos-sdk/security/advisories/new) if the SDK itself is implicated
5. **Update**: Update to latest patched version if vulnerability was in SDK

## Security Checklist for Production

Before deploying applications using odos-sdk to production:

- [ ] API keys loaded from environment variables, never hardcoded
- [ ] `.env` files added to `.gitignore`
- [ ] Input validation implemented for all user inputs
- [ ] Error messages do not expose sensitive information
- [ ] Appropriate timeouts configured
- [ ] Rate limiting implemented at application level
- [ ] TLS certificate validation enabled (default)
- [ ] Dependencies audited with `cargo audit`
- [ ] Logging configured to exclude sensitive data
- [ ] Monitoring and alerting configured for error rates
- [ ] Incident response plan documented

## Additional Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [RUSTSEC Advisory Database](https://rustsec.org/)
- [Odos API Documentation](https://docs.odos.xyz/)

## Contact

For security-related questions or concerns:

- **Security Reports**: [GitHub Security Advisories](https://github.com/semiotic-ai/odos-sdk/security/advisories/new)
- **General Questions**: [GitHub Discussions](https://github.com/semiotic-ai/odos-sdk/discussions)
- **Project Issues**: <https://github.com/semiotic-ai/odos-sdk/issues>
