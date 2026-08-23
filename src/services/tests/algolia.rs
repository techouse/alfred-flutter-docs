use super::*;
use flate2::{Compression, write::GzEncoder};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::{self, JoinHandle};
use ureq::tls::{RootCerts, TlsConfig};

fn config() -> AlgoliaSearchConfig {
    AlgoliaSearchConfig {
        application_id: "app".into(),
        api_key: "key".into(),
        index_name: "flutter".into(),
    }
}

#[test]
fn endpoint_uses_flutter_index_route() -> Result<()> {
    let base = Url::parse("http://localhost/api/")?;
    let client = AlgoliaSearch::with_base_url(config(), base.clone())?;
    assert_eq!(
        client.endpoint(&base)?.as_str(),
        "http://localhost/api/1/indexes/flutter/query"
    );
    Ok(())
}

#[test]
fn hosts_are_dsn_then_numbered_read_hosts() -> Result<()> {
    let client = AlgoliaSearch::new(config())?;
    assert_eq!(
        client
            .read_hosts
            .iter()
            .map(Url::as_str)
            .collect::<Vec<_>>(),
        vec![
            "https://app-dsn.algolia.net/",
            "https://app-1.algolianet.com/",
            "https://app-2.algolianet.com/",
            "https://app-3.algolianet.com/"
        ]
    );
    Ok(())
}

#[test]
fn custom_agent_constructor_keeps_local_test_hosts_isolated() -> Result<()> {
    let agent = super::platform_agent(CONNECT_TIMEOUT, SEARCH_TIMEOUT);
    let client = AlgoliaSearch::with_read_hosts_and_agent(
        config(),
        vec![Url::parse("http://127.0.0.1:1/")?],
        agent,
    )?;
    assert_eq!(client.read_hosts.len(), 1);
    Ok(())
}

#[test]
fn request_body_matches_flutter_contract() -> Result<()> {
    let client = AlgoliaSearch::with_base_url(config(), Url::parse("http://localhost/")?)?;
    let body: Value = serde_json::from_str(&client.request_body("container")?)?;
    assert_eq!(
        body,
        json!({
            "query": "container",
            "attributesToRetrieve": ["name", "qualifiedName", "href", "type", "enclosedBy"],
            "distinct": 1,
            "page": 0,
            "hitsPerPage": 20
        })
    );
    Ok(())
}

#[test]
fn terminal_and_retryable_statuses_are_classified() -> Result<()> {
    let endpoint = Url::parse("http://localhost/")?;
    assert!(matches!(
        classify_response_body_error(
            &endpoint,
            404,
            ureq::Error::Io(std::io::Error::other("bad"))
        ),
        AttemptFailure::Terminal(_)
    ));
    assert!(matches!(
        classify_response_body_error(
            &endpoint,
            503,
            ureq::Error::Io(std::io::Error::other("bad"))
        ),
        AttemptFailure::Retryable(_)
    ));
    Ok(())
}

#[test]
fn malformed_4xx_body_is_terminal_without_failover() -> Result<()> {
    let (first_url, first_server) = serve_once(401, b"not-gzip", Some("gzip"))?;
    let client =
        AlgoliaSearch::with_read_hosts_and_agent(config(), vec![first_url], local_agent())?;
    let error = client
        .query("container")
        .expect_err("4xx body errors must be terminal");
    first_server
        .join()
        .map_err(|_| anyhow::anyhow!("server panicked"))?;
    assert!(error.to_string().contains("HTTP status 401"));
    assert!(!error.to_string().contains("after trying"));
    Ok(())
}

#[test]
fn malformed_5xx_body_retries_the_next_host() -> Result<()> {
    let (first_url, first_server) = serve_once(503, b"not-gzip", Some("gzip"))?;
    let (second_url, second_server) = serve_once(200, br#"{"hits":[]}"#, None)?;
    let client = AlgoliaSearch::with_read_hosts_and_agent(
        config(),
        vec![first_url, second_url],
        local_agent(),
    )?;
    let hits = client.query("container")?;
    first_server
        .join()
        .map_err(|_| anyhow::anyhow!("server panicked"))?;
    second_server
        .join()
        .map_err(|_| anyhow::anyhow!("server panicked"))?;
    assert!(hits.is_empty());
    Ok(())
}

#[test]
fn oversized_5xx_body_retries_the_next_host() -> Result<()> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&vec![b'x'; (MAX_RESPONSE_BYTES + 1) as usize])?;
    let (first_url, first_server) = serve_once(503, &encoder.finish()?, Some("gzip"))?;
    let client =
        AlgoliaSearch::with_read_hosts_and_agent(config(), vec![first_url], local_agent())?;
    let error = client
        .query("container")
        .expect_err("oversized 5xx responses must be retryable");
    first_server
        .join()
        .map_err(|_| anyhow::anyhow!("server panicked"))?;
    assert!(error.to_string().contains("exceeds 2097152 bytes"));
    Ok(())
}

fn local_agent() -> Agent {
    Agent::config_builder()
        .proxy(None)
        .tls_config(
            TlsConfig::builder()
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_global(Some(SEARCH_TIMEOUT))
        .build()
        .into()
}

fn serve_once(status: u16, body: &[u8], encoding: Option<&str>) -> Result<(Url, JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let body = body.to_vec();
    let encoding = encoding.map(str::to_owned);
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("test server must accept");
        read_request(&mut stream);
        let content_encoding = encoding
            .as_deref()
            .map(|value| format!("Content-Encoding: {value}\r\n"))
            .unwrap_or_default();
        let response = format!(
            "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{content_encoding}Connection: close\r\n\r\n",
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("write headers");
        stream.write_all(&body).expect("write body");
        stream.flush().expect("flush response");
    });
    Ok((Url::parse(&format!("http://{address}/"))?, server))
}

fn read_request(stream: &mut TcpStream) {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        let bytes = stream.read(&mut buffer).expect("read request");
        if bytes == 0 {
            break;
        }
        request.extend_from_slice(&buffer[..bytes]);
        let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        let headers = String::from_utf8_lossy(&request[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("Content-Length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .unwrap_or(0);
        if request.len() >= header_end + 4 + content_length {
            break;
        }
    }
}
