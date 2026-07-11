use std::time::Duration;

use moe_research_net::reqwest_client::ReqwestNetworkClient;
use moe_research_net::{Header, NetworkClient, NetworkRequest};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

struct CapturedHttpProxyRequest {
    request_line: String,
}

struct CapturedSocksProxyRequest {
    address_type: u8,
    destination_host: String,
    destination_port: u16,
    request_line: String,
}

struct ProxyResponse {
    content_type: &'static str,
    body: Vec<u8>,
}

#[tokio::test]
async fn http_proxy_routes_json_requests() {
    let (proxy_url, captured) = start_http_proxy(ProxyResponse {
        content_type: "application/json",
        body: br#"{"ok":true}"#.to_vec(),
    })
    .await;
    let client = build_client(Some(&proxy_url));

    let response = client
        .send_json(json_request("http://example.invalid/proxy-json"))
        .await
        .expect("proxied JSON request succeeds");

    assert_eq!(response.body["ok"], true);
    assert!(
        captured_http_request(captured)
            .await
            .request_line
            .starts_with("GET http://example.invalid/proxy-json ")
    );
}

#[tokio::test]
async fn http_proxy_routes_sse_requests() {
    let (proxy_url, captured) = start_http_proxy(ProxyResponse {
        content_type: "text/event-stream",
        body: b"event: complete\ndata: {\"ok\":true}\n\n".to_vec(),
    })
    .await;
    let client = build_client(Some(&proxy_url));

    let mut stream = client
        .send_sse(sse_request("http://example.invalid/proxy-events"))
        .await
        .expect("proxied SSE request starts");
    let event = stream
        .next_event()
        .await
        .expect("SSE event read succeeds")
        .expect("SSE event is present");

    assert_eq!(event.event, "complete");
    assert_eq!(event.data, r#"{"ok":true}"#);
    assert!(
        captured_http_request(captured)
            .await
            .request_line
            .starts_with("GET http://example.invalid/proxy-events ")
    );
}

#[tokio::test]
async fn http_proxy_routes_binary_requests() {
    let body = vec![0, 1, 2, 3, 255];
    let (proxy_url, captured) = start_http_proxy(ProxyResponse {
        content_type: "application/octet-stream",
        body: body.clone(),
    })
    .await;
    let client = build_client(Some(&proxy_url));

    let bytes = client
        .send_bytes(bytes_request("http://example.invalid/asset.tar.gz"))
        .await
        .expect("proxied binary request succeeds");

    assert_eq!(bytes, body);
    assert!(
        captured_http_request(captured)
            .await
            .request_line
            .starts_with("GET http://example.invalid/asset.tar.gz ")
    );
}

#[tokio::test]
async fn socks5_proxy_routes_ipv4_json_requests() {
    let (proxy_url, captured) = start_socks5_proxy(ProxyResponse {
        content_type: "application/json",
        body: br#"{"ok":true}"#.to_vec(),
    })
    .await;
    let client = build_client(Some(&proxy_url));

    let response = client
        .send_json(json_request("http://127.0.0.1:9/socks-json"))
        .await
        .expect("SOCKS5 JSON request succeeds");

    assert_eq!(response.body["ok"], true);
    let captured = captured_socks_request(captured).await;
    assert_eq!(captured.address_type, 0x01);
    assert_eq!(captured.destination_host, "127.0.0.1");
    assert_eq!(captured.destination_port, 9);
    assert!(captured.request_line.starts_with("GET /socks-json "));
}

#[tokio::test]
async fn socks5_proxy_routes_ipv4_binary_requests() {
    let body = b"socks binary asset".to_vec();
    let (proxy_url, captured) = start_socks5_proxy(ProxyResponse {
        content_type: "application/octet-stream",
        body: body.clone(),
    })
    .await;
    let client = build_client(Some(&proxy_url));

    let bytes = client
        .send_bytes(bytes_request("http://127.0.0.1:9/socks-asset.tar.gz"))
        .await
        .expect("SOCKS5 binary request succeeds");

    assert_eq!(bytes, body);
    let captured = captured_socks_request(captured).await;
    assert_eq!(captured.address_type, 0x01);
    assert_eq!(captured.destination_host, "127.0.0.1");
    assert!(
        captured
            .request_line
            .starts_with("GET /socks-asset.tar.gz ")
    );
}

#[tokio::test]
async fn socks5h_proxy_sends_domain_name_to_proxy_for_sse() {
    let (proxy_url, captured) = start_socks5_proxy(ProxyResponse {
        content_type: "text/event-stream",
        body: b"event: complete\ndata: {\"ok\":true}\n\n".to_vec(),
    })
    .await;
    let proxy_url = proxy_url.replacen("socks5://", "socks5h://", 1);
    let client = build_client(Some(&proxy_url));

    let mut stream = client
        .send_sse(sse_request("http://example.invalid/socks-events"))
        .await
        .expect("SOCKS5h SSE request starts");
    let event = stream
        .next_event()
        .await
        .expect("SSE event read succeeds")
        .expect("SSE event is present");

    assert_eq!(event.event, "complete");
    let captured = captured_socks_request(captured).await;
    assert_eq!(captured.address_type, 0x03);
    assert_eq!(captured.destination_host, "example.invalid");
    assert_eq!(captured.destination_port, 80);
    assert!(captured.request_line.starts_with("GET /socks-events "));
}

fn build_client(proxy_url: Option<&str>) -> ReqwestNetworkClient {
    ReqwestNetworkClient::new(5_000, 0, 1, "moe-research-tests/0.0.0", proxy_url)
        .expect("build Reqwest network client")
}

fn json_request(url: &str) -> NetworkRequest {
    NetworkRequest {
        method: "GET".to_owned(),
        url: url.to_owned(),
        headers: vec![Header {
            name: "accept".to_owned(),
            value: "application/json".to_owned(),
        }],
        body: None,
        inactivity_timeout_ms: None,
    }
}

fn sse_request(url: &str) -> NetworkRequest {
    NetworkRequest {
        method: "GET".to_owned(),
        url: url.to_owned(),
        headers: vec![Header {
            name: "accept".to_owned(),
            value: "text/event-stream".to_owned(),
        }],
        body: None,
        inactivity_timeout_ms: None,
    }
}

fn bytes_request(url: &str) -> NetworkRequest {
    NetworkRequest {
        method: "GET".to_owned(),
        url: url.to_owned(),
        headers: Vec::new(),
        body: None,
        inactivity_timeout_ms: None,
    }
}

async fn start_http_proxy(
    response: ProxyResponse,
) -> (String, oneshot::Receiver<CapturedHttpProxyRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind HTTP proxy");
    let address = listener.local_addr().expect("HTTP proxy address");
    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept HTTP proxy request");
        let request_line = read_http_request(&mut stream).await;
        let _ = sender.send(CapturedHttpProxyRequest { request_line });
        write_http_response(&mut stream, &response).await;
    });

    (format!("http://{address}"), receiver)
}

async fn start_socks5_proxy(
    response: ProxyResponse,
) -> (String, oneshot::Receiver<CapturedSocksProxyRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind SOCKS5 proxy");
    let address = listener.local_addr().expect("SOCKS5 proxy address");
    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept SOCKS5 request");
        let (address_type, destination_host, destination_port) =
            read_socks5_connect(&mut stream).await;
        let request_line = read_http_request(&mut stream).await;
        let _ = sender.send(CapturedSocksProxyRequest {
            address_type,
            destination_host,
            destination_port,
            request_line,
        });
        write_http_response(&mut stream, &response).await;
    });

    (format!("socks5://{address}"), receiver)
}

async fn read_socks5_connect(stream: &mut TcpStream) -> (u8, String, u16) {
    let mut greeting = [0_u8; 2];
    stream
        .read_exact(&mut greeting)
        .await
        .expect("read SOCKS5 greeting");
    assert_eq!(greeting[0], 0x05);
    let mut methods = vec![0_u8; usize::from(greeting[1])];
    stream
        .read_exact(&mut methods)
        .await
        .expect("read SOCKS5 methods");
    assert!(methods.contains(&0x00));
    stream
        .write_all(&[0x05, 0x00])
        .await
        .expect("accept SOCKS5 no-auth method");

    let mut request = [0_u8; 4];
    stream
        .read_exact(&mut request)
        .await
        .expect("read SOCKS5 CONNECT header");
    assert_eq!(request[0], 0x05);
    assert_eq!(request[1], 0x01);
    assert_eq!(request[2], 0x00);

    let address_type = request[3];
    let destination_host = match address_type {
        0x01 => {
            let mut octets = [0_u8; 4];
            stream
                .read_exact(&mut octets)
                .await
                .expect("read IPv4 target");
            std::net::Ipv4Addr::from(octets).to_string()
        }
        0x03 => {
            let mut length = [0_u8; 1];
            stream
                .read_exact(&mut length)
                .await
                .expect("read domain length");
            let mut domain = vec![0_u8; usize::from(length[0])];
            stream
                .read_exact(&mut domain)
                .await
                .expect("read target domain");
            String::from_utf8(domain).expect("target domain is UTF-8")
        }
        other => panic!("unexpected SOCKS5 address type {other}"),
    };
    let mut port = [0_u8; 2];
    stream
        .read_exact(&mut port)
        .await
        .expect("read target port");

    stream
        .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
        .await
        .expect("accept SOCKS5 CONNECT");

    (address_type, destination_host, u16::from_be_bytes(port))
}

async fn read_http_request(stream: &mut TcpStream) -> String {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 1024];
    while !bytes.windows(4).any(|window| window == b"\r\n\r\n") {
        let read = stream.read(&mut buffer).await.expect("read HTTP request");
        assert_ne!(read, 0, "HTTP request ended before headers");
        bytes.extend_from_slice(&buffer[..read]);
    }

    String::from_utf8(bytes)
        .expect("HTTP request headers are UTF-8")
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned()
}

async fn write_http_response(stream: &mut TcpStream, response: &ProxyResponse) {
    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.content_type,
        response.body.len()
    );
    stream
        .write_all(headers.as_bytes())
        .await
        .expect("write HTTP response headers");
    stream
        .write_all(&response.body)
        .await
        .expect("write HTTP response body");
    stream.shutdown().await.expect("close HTTP response");
}

async fn captured_http_request(
    receiver: oneshot::Receiver<CapturedHttpProxyRequest>,
) -> CapturedHttpProxyRequest {
    tokio::time::timeout(Duration::from_secs(2), receiver)
        .await
        .expect("HTTP proxy captured request before timeout")
        .expect("HTTP proxy task completed")
}

async fn captured_socks_request(
    receiver: oneshot::Receiver<CapturedSocksProxyRequest>,
) -> CapturedSocksProxyRequest {
    tokio::time::timeout(Duration::from_secs(2), receiver)
        .await
        .expect("SOCKS5 proxy captured request before timeout")
        .expect("SOCKS5 proxy task completed")
}
