use super::*;
use std::time::Duration;

#[test]
fn platform_agent_uses_platform_roots_and_bounded_timeouts() {
    let agent = platform_agent(Duration::from_secs(2), Duration::from_secs(5));
    assert!(matches!(
        agent.config().tls_config().root_certs(),
        ureq::tls::RootCerts::PlatformVerifier
    ));
    assert_eq!(
        agent.config().timeouts().connect,
        Some(Duration::from_secs(2))
    );
    assert_eq!(
        agent.config().timeouts().global,
        Some(Duration::from_secs(5))
    );
}
