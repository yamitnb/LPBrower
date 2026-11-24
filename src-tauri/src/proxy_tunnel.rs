//! High-performance local loopback HTTP proxy forwarder (`127.0.0.1:<port>`) with:
//! 1. Preemptive `Proxy-Authorization: Basic ...` injection (eliminating 407 double-RTTs on HTTP/HTTPS proxies)
//! 2. Pre-authenticated warm socket pooling for SOCKS5/HTTP/HTTPS upstreams (cutting 1–3 international RTTs)
//! 3. Elimination of Chromium's `Socks5UdpAssociator` `UDP_ASSOCIATE` reconnect storms on non-UDP SOCKS5 proxies
//! 4. Cached upstream DNS resolution (IPv4-first) + `TCP_NODELAY` + 64 KiB bidirectional zero-copy streaming.

use crate::space::proxy::{ProxyEntry, ProxyKind};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

const WARM_POOL_TARGET: usize = 3;
const WARM_SOCKET_TTL: Duration = Duration::from_secs(12);
const DNS_CACHE_TTL: Duration = Duration::from_secs(120);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(6);
const HEADER_TIMEOUT: Duration = Duration::from_secs(15);
const COPY_BUF_SIZE: usize = 65_536;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TunnelKey {
    kind: ProxyKind,
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl TunnelKey {
    fn of(e: &ProxyEntry) -> Self {
        Self {
            kind: e.kind.clone(),
            host: e.host.clone(),
            port: e.port,
            username: e.username.clone(),
            password: e.password.clone(),
        }
    }
}

enum UpstreamConn {
    Tcp(TcpStream),
    Tls(Box<tokio_rustls::client::TlsStream<TcpStream>>),
    /// Connected TCP stream that has already completed SOCKS5 greeting + RFC 1929 auth.
    Socks5(TcpStream),
}

struct Tunnel {
    entry: ProxyEntry,
    local_url: String,
    basic_auth_line: Option<String>,
    dns_cache: Mutex<Option<(Instant, Vec<SocketAddr>)>>,
    warm_pool: Mutex<VecDeque<(Instant, UpstreamConn)>>,
    warming: AtomicUsize,
}

fn tunnels() -> &'static Mutex<HashMap<TunnelKey, Arc<Tunnel>>> {
    static MAP: OnceLock<Mutex<HashMap<TunnelKey, Arc<Tunnel>>>> = OnceLock::new();
    MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

fn rustls_client_config() -> Arc<rustls::ClientConfig> {
    static CFG: OnceLock<Arc<rustls::ClientConfig>> = OnceLock::new();
    CFG.get_or_init(|| {
        let roots = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };
        let mut cfg = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .expect("ring TLS")
        .with_root_certificates(roots)
        .with_no_client_auth();
        cfg.alpn_protocols = vec![b"http/1.1".to_vec()];
        Arc::new(cfg)
    })
    .clone()
}

/// Ensure a local loopback tunnel is listening for `entry` and pre-warming upstream sockets.
/// Returns `"http://127.0.0.1:<port>"`.
pub async fn ensure_tunnel(entry: &ProxyEntry) -> Result<String> {
    let key = TunnelKey::of(entry);
    if let Some(existing) = tunnels().lock().unwrap_or_else(|e| e.into_inner()).get(&key).cloned() {
        existing.prewarm();
        return Ok(existing.local_url.clone());
    }

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind local proxy tunnel")?;
    let local_port = listener.local_addr()?.port();
    let local_url = format!("http://127.0.0.1:{local_port}");

    let basic_auth_line = if entry.username.is_empty() && entry.password.is_empty() {
        None
    } else {
        let encoded = STANDARD.encode(format!("{}:{}", entry.username, entry.password));
        Some(format!("Proxy-Authorization: Basic {encoded}\r\n"))
    };

    let tunnel = Arc::new(Tunnel {
        entry: entry.clone(),
        local_url: local_url.clone(),
        basic_auth_line,
        dns_cache: Mutex::new(None),
        warm_pool: Mutex::new(VecDeque::with_capacity(WARM_POOL_TARGET + 1)),
        warming: AtomicUsize::new(0),
    });

    {
        let mut g = tunnels().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(raced) = g.get(&key).cloned() {
            drop(listener);
            raced.prewarm();
            return Ok(raced.local_url.clone());
        }
        g.insert(key, tunnel.clone());
    }

    let accept_tunnel = tunnel.clone();
    tokio::spawn(async move {
        loop {
            let Ok((client, _)) = listener.accept().await else {
                tokio::time::sleep(Duration::from_millis(20)).await;
                continue;
            };
            let _ = client.set_nodelay(true);
            let t = accept_tunnel.clone();
            tokio::spawn(async move {
                let _ = t.handle_client(client).await;
            });
        }
    });

    tunnel.prewarm();
    Ok(local_url)
}

impl Tunnel {
    /// Pre-open up to `WARM_POOL_TARGET` ready upstream connections in the background.
    fn prewarm(self: &Arc<Self>) {
        let have = {
            let mut pool = self.warm_pool.lock().unwrap_or_else(|e| e.into_inner());
            pool.retain(|(at, conn)| at.elapsed() < WARM_SOCKET_TTL && conn_still_idle(conn));
            pool.len()
        };
        let in_flight = self.warming.load(Ordering::Relaxed);
        let needed = WARM_POOL_TARGET.saturating_sub(have + in_flight);
        for _ in 0..needed {
            self.spawn_one_warm();
        }
    }

    fn maybe_replenish(self: &Arc<Self>) {
        let have = self.warm_pool.lock().map(|p| p.len()).unwrap_or(0);
        let in_flight = self.warming.load(Ordering::Relaxed);
        if have + in_flight < 2 {
            self.spawn_one_warm();
        }
    }

    fn spawn_one_warm(self: &Arc<Self>) {
        self.warming.fetch_add(1, Ordering::Relaxed);
        let t = self.clone();
        tokio::spawn(async move {
            let res = t.connect_upstream_fresh().await;
            t.warming.fetch_sub(1, Ordering::Relaxed);
            if let Ok(conn) = res {
                let mut pool = t.warm_pool.lock().unwrap_or_else(|e| e.into_inner());
                pool.retain(|(at, c)| at.elapsed() < WARM_SOCKET_TTL && conn_still_idle(c));
                if pool.len() < WARM_POOL_TARGET {
                    pool.push_back((Instant::now(), conn));
                }
            }
        });
    }

    fn take_warm(&self) -> Option<UpstreamConn> {
        let mut pool = self.warm_pool.lock().unwrap_or_else(|e| e.into_inner());
        while let Some((at, conn)) = pool.pop_front() {
            if at.elapsed() < WARM_SOCKET_TTL && conn_still_idle(&conn) {
                return Some(conn);
            }
        }
        None
    }

    async fn resolve_upstream(&self) -> Result<Vec<SocketAddr>> {
        if let Ok(ip) = self.entry.host.parse::<IpAddr>() {
            return Ok(vec![SocketAddr::new(ip, self.entry.port)]);
        }
        if let Ok(g) = self.dns_cache.lock() {
            if let Some((at, addrs)) = g.as_ref() {
                if at.elapsed() < DNS_CACHE_TTL && !addrs.is_empty() {
                    return Ok(addrs.clone());
                }
            }
        }
        let host_port = (self.entry.host.as_str(), self.entry.port);
        let resolved = timeout(CONNECT_TIMEOUT, tokio::net::lookup_host(host_port))
            .await
            .context("proxy DNS lookup timeout")??;
        let mut v4 = Vec::new();
        let mut v6 = Vec::new();
        for a in resolved {
            if a.is_ipv4() {
                v4.push(a);
            } else {
                v6.push(a);
            }
        }
        v4.extend(v6);
        if v4.is_empty() {
            anyhow::bail!("proxy host {} resolved to no addresses", self.entry.host);
        }
        if let Ok(mut g) = self.dns_cache.lock() {
            *g = Some((Instant::now(), v4.clone()));
        }
        Ok(v4)
    }

    fn invalidate_dns(&self) {
        if let Ok(mut g) = self.dns_cache.lock() {
            *g = None;
        }
    }

    async fn connect_tcp(&self) -> Result<TcpStream> {
        let addrs = self.resolve_upstream().await?;
        let mut last_err = None;
        for addr in addrs {
            match timeout(CONNECT_TIMEOUT, TcpStream::connect(addr)).await {
                Ok(Ok(tcp)) => {
                    let _ = tcp.set_nodelay(true);
                    return Ok(tcp);
                }
                Ok(Err(e)) => last_err = Some(anyhow::Error::from(e)),
                Err(_) => last_err = Some(anyhow::anyhow!("connect timeout to {addr}")),
            }
        }
        self.invalidate_dns();
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("unable to connect to proxy")))
    }

    async fn connect_upstream_fresh(&self) -> Result<UpstreamConn> {
        let mut tcp = self.connect_tcp().await?;
        match self.entry.kind {
            ProxyKind::Http => Ok(UpstreamConn::Tcp(tcp)),
            ProxyKind::Https => {
                let server_name = rustls::pki_types::ServerName::try_from(self.entry.host.clone())
                    .context("invalid TLS server name for HTTPS proxy")?;
                let tls = timeout(
                    CONNECT_TIMEOUT,
                    tokio_rustls::TlsConnector::from(rustls_client_config()).connect(server_name, tcp),
                )
                .await
                .context("HTTPS proxy TLS timeout")??;
                Ok(UpstreamConn::Tls(Box::new(tls)))
            }
            ProxyKind::Socks5 => {
                socks5_authenticate(&mut tcp, &self.entry).await?;
                Ok(UpstreamConn::Socks5(tcp))
            }
        }
    }

    async fn acquire_upstream(self: &Arc<Self>) -> Result<(UpstreamConn, bool)> {
        if let Some(warm) = self.take_warm() {
            self.maybe_replenish();
            return Ok((warm, true));
        }
        self.maybe_replenish();
        Ok((self.connect_upstream_fresh().await?, false))
    }

    async fn handle_client(self: Arc<Self>, mut client: TcpStream) -> Result<()> {
        let Some((hdr_bytes, leftover)) = read_http_header_block(&mut client, HEADER_TIMEOUT).await? else {
            return Ok(());
        };
        let hdr_str = String::from_utf8_lossy(&hdr_bytes);
        let mut lines = hdr_str.split("\r\n");
        let Some(req_line) = lines.next() else {
            return Ok(());
        };
        let mut parts = req_line.split_whitespace();
        let (Some(method), Some(target), Some(version)) = (parts.next(), parts.next(), parts.next()) else {
            return Ok(());
        };

        if method.eq_ignore_ascii_case("CONNECT") {
            self.handle_connect(&mut client, target, &leftover).await
        } else {
            self.handle_plain_http(&mut client, method, target, version, &hdr_str, &leftover)
                .await
        }
    }

    async fn handle_connect(
        self: &Arc<Self>,
        client: &mut TcpStream,
        target: &str,
        leftover: &[u8],
    ) -> Result<()> {
        match self.entry.kind {
            ProxyKind::Http | ProxyKind::Https => {
                let mut req = String::with_capacity(192);
                req.push_str("CONNECT ");
                req.push_str(target);
                req.push_str(" HTTP/1.1\r\nHost: ");
                req.push_str(target);
                req.push_str("\r\nProxy-Connection: keep-alive\r\n");
                if let Some(auth) = &self.basic_auth_line {
                    req.push_str(auth);
                }
                req.push_str("\r\n");

                let (mut conn, from_warm) = self.acquire_upstream().await?;
                let resp = match send_and_read_headers(&mut conn, req.as_bytes()).await {
                    Ok(Some(r)) => r,
                    _ if from_warm => {
                        conn = self.connect_upstream_fresh().await?;
                        send_and_read_headers(&mut conn, req.as_bytes())
                            .await?
                            .context("upstream proxy closed CONNECT without reply")?
                    }
                    Err(e) => return Err(e),
                    Ok(None) => anyhow::bail!("upstream proxy closed CONNECT without reply"),
                };

                let (up_hdr, up_leftover) = resp;
                let status_ok = {
                    let s = String::from_utf8_lossy(&up_hdr);
                    let first = s.lines().next().unwrap_or("");
                    first.starts_with("HTTP/1.1 200") || first.starts_with("HTTP/1.0 200")
                };
                if !status_ok {
                    client.write_all(&up_hdr).await?;
                    if !up_leftover.is_empty() {
                        client.write_all(&up_leftover).await?;
                    }
                    return Ok(());
                }

                client
                    .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                    .await?;
                if !up_leftover.is_empty() {
                    client.write_all(&up_leftover).await?;
                }
                relay_conn(client, &mut conn, leftover).await
            }
            ProxyKind::Socks5 => {
                let (host, port) = split_host_port(target, 443);
                let (mut conn, from_warm) = self.acquire_upstream().await?;
                let mut tcp = match conn {
                    UpstreamConn::Socks5(s) => s,
                    _ => unreachable!(),
                };
                if let Err(e) = socks5_connect_target(&mut tcp, &host, port).await {
                    if from_warm {
                        let fresh = self.connect_upstream_fresh().await?;
                        tcp = match fresh {
                            UpstreamConn::Socks5(s) => s,
                            _ => unreachable!(),
                        };
                        if socks5_connect_target(&mut tcp, &host, port).await.is_err() {
                            let _ = client
                                .write_all(b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Length: 0\r\n\r\n")
                                .await;
                            return Ok(());
                        }
                    } else {
                        let _ = client
                            .write_all(b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Length: 0\r\n\r\n")
                            .await;
                        return Err(e);
                    }
                }

                client
                    .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                    .await?;
                if !leftover.is_empty() {
                    tcp.write_all(leftover).await?;
                }
                let _ = tokio::io::copy_bidirectional_with_sizes(
                    client,
                    &mut tcp,
                    COPY_BUF_SIZE,
                    COPY_BUF_SIZE,
                )
                .await;
                Ok(())
            }
        }
    }

    async fn handle_plain_http(
        self: &Arc<Self>,
        client: &mut TcpStream,
        method: &str,
        target: &str,
        version: &str,
        raw_hdr: &str,
        leftover: &[u8],
    ) -> Result<()> {
        match self.entry.kind {
            ProxyKind::Http | ProxyKind::Https => {
                let mut out = String::with_capacity(raw_hdr.len() + 128);
                let mut lines = raw_hdr.split("\r\n");
                if let Some(first) = lines.next() {
                    out.push_str(first);
                    out.push_str("\r\n");
                }
                for line in lines {
                    if line.is_empty() {
                        continue;
                    }
                    let lower = line.to_ascii_lowercase();
                    if lower.starts_with("proxy-authorization:")
                        || lower.starts_with("proxy-connection:")
                        || lower.starts_with("connection:")
                    {
                        continue;
                    }
                    out.push_str(line);
                    out.push_str("\r\n");
                }
                if let Some(auth) = &self.basic_auth_line {
                    out.push_str(auth);
                }
                out.push_str("Proxy-Connection: close\r\nConnection: close\r\n\r\n");

                let (mut conn, from_warm) = self.acquire_upstream().await?;
                if write_conn(&mut conn, out.as_bytes()).await.is_err() && from_warm {
                    conn = self.connect_upstream_fresh().await?;
                    write_conn(&mut conn, out.as_bytes()).await?;
                }
                relay_conn(client, &mut conn, leftover).await
            }
            ProxyKind::Socks5 => {
                let (host, port, origin_path) = parse_plain_http_target(target, raw_hdr)?;
                let mut out = String::with_capacity(raw_hdr.len() + 64);
                out.push_str(method);
                out.push(' ');
                out.push_str(&origin_path);
                out.push(' ');
                out.push_str(version);
                out.push_str("\r\n");
                for line in raw_hdr.split("\r\n").skip(1) {
                    if line.is_empty() {
                        continue;
                    }
                    let lower = line.to_ascii_lowercase();
                    if lower.starts_with("proxy-authorization:")
                        || lower.starts_with("proxy-connection:")
                        || lower.starts_with("connection:")
                    {
                        continue;
                    }
                    out.push_str(line);
                    out.push_str("\r\n");
                }
                out.push_str("Connection: close\r\n\r\n");

                let (conn, from_warm) = self.acquire_upstream().await?;
                let mut tcp = match conn {
                    UpstreamConn::Socks5(s) => s,
                    _ => unreachable!(),
                };
                if socks5_connect_target(&mut tcp, &host, port).await.is_err() {
                    if !from_warm {
                        let _ = client
                            .write_all(b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Length: 0\r\n\r\n")
                            .await;
                        return Ok(());
                    }
                    let fresh = self.connect_upstream_fresh().await?;
                    tcp = match fresh {
                        UpstreamConn::Socks5(s) => s,
                        _ => unreachable!(),
                    };
                    if socks5_connect_target(&mut tcp, &host, port).await.is_err() {
                        let _ = client
                            .write_all(b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Length: 0\r\n\r\n")
                            .await;
                        return Ok(());
                    }
                }
                tcp.write_all(out.as_bytes()).await?;
                if !leftover.is_empty() {
                    tcp.write_all(leftover).await?;
                }
                let _ = tokio::io::copy_bidirectional_with_sizes(
                    client,
                    &mut tcp,
                    COPY_BUF_SIZE,
                    COPY_BUF_SIZE,
                )
                .await;
                Ok(())
            }
        }
    }
}

fn conn_still_idle(conn: &UpstreamConn) -> bool {
    let tcp = match conn {
        UpstreamConn::Tcp(s) | UpstreamConn::Socks5(s) => s,
        UpstreamConn::Tls(tls) => tls.get_ref().0,
    };
    let mut probe = [0u8; 1];
    match tcp.try_read(&mut probe) {
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => true,
        _ => false,
    }
}

async fn write_conn(conn: &mut UpstreamConn, data: &[u8]) -> std::io::Result<()> {
    match conn {
        UpstreamConn::Tcp(s) | UpstreamConn::Socks5(s) => s.write_all(data).await,
        UpstreamConn::Tls(s) => s.write_all(data).await,
    }
}

async fn send_and_read_headers(
    conn: &mut UpstreamConn,
    req: &[u8],
) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
    match conn {
        UpstreamConn::Tcp(s) | UpstreamConn::Socks5(s) => {
            s.write_all(req).await?;
            read_http_header_block(s, CONNECT_TIMEOUT).await
        }
        UpstreamConn::Tls(s) => {
            s.write_all(req).await?;
            read_http_header_block(s, CONNECT_TIMEOUT).await
        }
    }
}

async fn relay_conn(client: &mut TcpStream, conn: &mut UpstreamConn, leftover: &[u8]) -> Result<()> {
    match conn {
        UpstreamConn::Tcp(s) | UpstreamConn::Socks5(s) => {
            if !leftover.is_empty() {
                s.write_all(leftover).await?;
            }
            let _ = tokio::io::copy_bidirectional_with_sizes(
                client,
                s,
                COPY_BUF_SIZE,
                COPY_BUF_SIZE,
            )
            .await;
        }
        UpstreamConn::Tls(s) => {
            if !leftover.is_empty() {
                s.write_all(leftover).await?;
            }
            let _ = tokio::io::copy_bidirectional_with_sizes(
                client,
                s,
                COPY_BUF_SIZE,
                COPY_BUF_SIZE,
            )
            .await;
        }
    }
    Ok(())
}

async fn read_http_header_block<S>(
    stream: &mut S,
    budget: Duration,
) -> Result<Option<(Vec<u8>, Vec<u8>)>>
where
    S: AsyncRead + Unpin,
{
    let mut buf = Vec::with_capacity(1024);
    let mut tmp = [0u8; 2048];
    let deadline = Instant::now() + budget;
    loop {
        let rem = deadline.saturating_duration_since(Instant::now());
        if rem.is_zero() {
            anyhow::bail!("HTTP header read timed out");
        }
        let n = timeout(rem, stream.read(&mut tmp))
            .await
            .context("HTTP header read timed out")??;
        if n == 0 {
            if buf.is_empty() {
                return Ok(None);
            }
            anyhow::bail!("unexpected EOF before end of HTTP headers");
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            let end = pos + 4;
            let rest = buf.split_off(end);
            return Ok(Some((buf, rest)));
        }
        if buf.len() > 32_768 {
            anyhow::bail!("HTTP header block exceeds 32 KiB");
        }
    }
}

fn split_host_port(target: &str, default_port: u16) -> (String, u16) {
    if let Some(rest) = target.strip_prefix('[') {
        if let Some((v6, after)) = rest.split_once(']') {
            let port = after
                .strip_prefix(':')
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(default_port);
            return (v6.to_string(), port);
        }
    }
    if let Some((h, p)) = target.rsplit_once(':') {
        if let Ok(port) = p.parse::<u16>() {
            return (h.to_string(), port);
        }
    }
    (target.to_string(), default_port)
}

fn parse_plain_http_target(target: &str, raw_hdr: &str) -> Result<(String, u16, String)> {
    if target.starts_with("http://") || target.starts_with("HTTP://") {
        let parsed = url::Url::parse(target).context("invalid HTTP proxy URL")?;
        let host = parsed.host_str().context("missing host in HTTP URL")?.to_string();
        let port = parsed.port_or_known_default().unwrap_or(80);
        let mut path = parsed.path().to_string();
        if path.is_empty() {
            path.push('/');
        }
        if let Some(q) = parsed.query() {
            path.push('?');
            path.push_str(q);
        }
        return Ok((host, port, path));
    }
    for line in raw_hdr.split("\r\n").skip(1) {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("host") {
                let (h, p) = split_host_port(v.trim(), 80);
                let path = if target.is_empty() { "/" } else { target };
                return Ok((h, p, path.to_string()));
            }
        }
    }
    anyhow::bail!("missing Host header in plain HTTP request")
}

async fn socks5_authenticate<S>(stream: &mut S, entry: &ProxyEntry) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let has_creds = !entry.username.is_empty() || !entry.password.is_empty();
    if has_creds {
        stream.write_all(&[0x05, 0x02, 0x00, 0x02]).await?;
    } else {
        stream.write_all(&[0x05, 0x01, 0x00]).await?;
    }
    let mut resp = [0u8; 2];
    timeout(CONNECT_TIMEOUT, stream.read_exact(&mut resp))
        .await
        .context("SOCKS5 greeting timeout")??;
    if resp[0] != 0x05 {
        anyhow::bail!("invalid SOCKS5 version");
    }
    match resp[1] {
        0x00 => Ok(()),
        0x02 => {
            if !has_creds {
                anyhow::bail!("SOCKS5 proxy requires credentials");
            }
            if entry.username.len() > 255 || entry.password.len() > 255 {
                anyhow::bail!("SOCKS5 credentials exceed 255 bytes");
            }
            let mut auth = Vec::with_capacity(3 + entry.username.len() + entry.password.len());
            auth.push(0x01);
            auth.push(entry.username.len() as u8);
            auth.extend_from_slice(entry.username.as_bytes());
            auth.push(entry.password.len() as u8);
            auth.extend_from_slice(entry.password.as_bytes());
            stream.write_all(&auth).await?;
            let mut aresp = [0u8; 2];
            timeout(CONNECT_TIMEOUT, stream.read_exact(&mut aresp))
                .await
                .context("SOCKS5 auth timeout")??;
            if aresp[1] != 0x00 {
                anyhow::bail!("SOCKS5 credentials rejected");
            }
            Ok(())
        }
        other => anyhow::bail!("unsupported SOCKS5 auth method {other:#x}"),
    }
}

async fn socks5_connect_target<S>(stream: &mut S, host: &str, port: u16) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut req = Vec::with_capacity(7 + host.len());
    req.extend_from_slice(&[0x05, 0x01, 0x00]);
    if let Ok(v4) = host.parse::<Ipv4Addr>() {
        req.push(0x01);
        req.extend_from_slice(&v4.octets());
    } else if let Ok(v6) = host.parse::<Ipv6Addr>() {
        req.push(0x04);
        req.extend_from_slice(&v6.octets());
    } else {
        if host.len() > 255 {
            anyhow::bail!("SOCKS5 domain too long");
        }
        req.push(0x03);
        req.push(host.len() as u8);
        req.extend_from_slice(host.as_bytes());
    }
    req.extend_from_slice(&port.to_be_bytes());
    stream.write_all(&req).await?;

    let mut hdr = [0u8; 4];
    timeout(CONNECT_TIMEOUT, stream.read_exact(&mut hdr))
        .await
        .context("SOCKS5 CONNECT reply timeout")??;
    if hdr[0] != 0x05 || hdr[1] != 0x00 {
        anyhow::bail!("SOCKS5 CONNECT failed ({:#x})", hdr[1]);
    }
    let tail_len = match hdr[3] {
        0x01 => 4 + 2,
        0x04 => 16 + 2,
        0x03 => {
            let mut dlen = [0u8; 1];
            timeout(CONNECT_TIMEOUT, stream.read_exact(&mut dlen)).await??;
            dlen[0] as usize + 2
        }
        other => anyhow::bail!("unsupported SOCKS5 reply ATYP {other:#x}"),
    };
    let mut tail = vec![0u8; tail_len];
    timeout(CONNECT_TIMEOUT, stream.read_exact(&mut tail)).await??;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[tokio::test]
    async fn http_connect_injects_preemptive_basic_auth_without_407() {
        let upstream = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let up_port = upstream.local_addr().unwrap().port();
        let packets_seen = Arc::new(AtomicU32::new(0));
        let seen_clone = packets_seen.clone();

        tokio::spawn(async move {
            while let Ok((mut sock, _)) = upstream.accept().await {
                let Some((hdr, _)) = read_http_header_block(&mut sock, Duration::from_secs(2))
                    .await
                    .ok()
                    .flatten()
                else {
                    continue;
                };
                seen_clone.fetch_add(1, Ordering::SeqCst);
                let text = String::from_utf8_lossy(&hdr);
                assert!(
                    text.contains("Proxy-Authorization: Basic dXNlcjE6cGFzczE="),
                    "first packet must carry preemptive Basic auth, got: {text}"
                );
                sock.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                    .await
                    .unwrap();
                let mut ping = [0u8; 4];
                if sock.read_exact(&mut ping).await.is_ok() && &ping == b"ping" {
                    let _ = sock.write_all(b"pong").await;
                }
            }
        });

        let entry = ProxyEntry {
            id: "t1".into(),
            name: "test-http".into(),
            kind: ProxyKind::Http,
            host: "127.0.0.1".into(),
            port: up_port,
            username: "user1".into(),
            password: "pass1".into(),
            country: String::new(),
            notes: String::new(),
        };

        let url = ensure_tunnel(&entry).await.unwrap();
        let addr = url.strip_prefix("http://").unwrap();
        let mut client = TcpStream::connect(addr).await.unwrap();
        client
            .write_all(b"CONNECT example.com:443 HTTP/1.1\r\nHost: example.com:443\r\n\r\nping")
            .await
            .unwrap();

        let (resp_hdr, mut extra) = read_http_header_block(&mut client, Duration::from_secs(2))
            .await
            .unwrap()
            .unwrap();
        assert!(String::from_utf8_lossy(&resp_hdr).starts_with("HTTP/1.1 200"));
        while extra.len() < 4 {
            let mut b = [0u8; 4];
            let n = client.read(&mut b).await.unwrap();
            extra.extend_from_slice(&b[..n]);
        }
        assert_eq!(&extra[..4], b"pong");
        assert_eq!(packets_seen.load(Ordering::SeqCst), 1, "single CONNECT, zero 407 retry");
    }

    #[tokio::test]
    async fn socks5_connect_uses_preauthenticated_warm_pool() {
        let target_srv = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target_port = target_srv.local_addr().unwrap().port();
        tokio::spawn(async move {
            while let Ok((mut s, _)) = target_srv.accept().await {
                let mut buf = [0u8; 5];
                if s.read_exact(&mut buf).await.is_ok() && &buf == b"hello" {
                    let _ = s.write_all(b"world").await;
                }
            }
        });

        let socks_srv = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let socks_port = socks_srv.local_addr().unwrap().port();
        tokio::spawn(async move {
            while let Ok((mut sock, _)) = socks_srv.accept().await {
                tokio::spawn(async move {
                    let mut g = [0u8; 4];
                    if sock.read_exact(&mut g).await.is_err() {
                        return;
                    }
                    assert_eq!(&g, &[0x05, 0x02, 0x00, 0x02]);
                    sock.write_all(&[0x05, 0x02]).await.unwrap();
                    let mut ver_ulen = [0u8; 2];
                    sock.read_exact(&mut ver_ulen).await.unwrap();
                    let mut u = vec![0u8; ver_ulen[1] as usize];
                    sock.read_exact(&mut u).await.unwrap();
                    let mut plen = [0u8; 1];
                    sock.read_exact(&mut plen).await.unwrap();
                    let mut p = vec![0u8; plen[0] as usize];
                    sock.read_exact(&mut p).await.unwrap();
                    assert_eq!((&u[..], &p[..]), (&b"suser"[..], &b"spass"[..]));
                    sock.write_all(&[0x01, 0x00]).await.unwrap();

                    let mut cmd = [0u8; 4];
                    if sock.read_exact(&mut cmd).await.is_err() {
                        return;
                    }
                    assert_eq!(cmd[1], 0x01, "must be CONNECT, never UDP_ASSOCIATE");
                    let target_addr = match cmd[3] {
                        0x01 => {
                            let mut ip_port = [0u8; 6];
                            sock.read_exact(&mut ip_port).await.unwrap();
                            SocketAddr::from((
                                [ip_port[0], ip_port[1], ip_port[2], ip_port[3]],
                                u16::from_be_bytes([ip_port[4], ip_port[5]]),
                            ))
                        }
                        0x03 => {
                            let mut l = [0u8; 1];
                            sock.read_exact(&mut l).await.unwrap();
                            let mut h = vec![0u8; l[0] as usize + 2];
                            sock.read_exact(&mut h).await.unwrap();
                            let port = u16::from_be_bytes([h[l[0] as usize], h[l[0] as usize + 1]]);
                            SocketAddr::from(([127, 0, 0, 1], port))
                        }
                        _ => return,
                    };
                    let mut dst = TcpStream::connect(target_addr).await.unwrap();
                    sock.write_all(&[0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 0])
                        .await
                        .unwrap();
                    let _ = tokio::io::copy_bidirectional(&mut sock, &mut dst).await;
                });
            }
        });

        let entry = ProxyEntry {
            id: "t2".into(),
            name: "test-socks".into(),
            kind: ProxyKind::Socks5,
            host: "127.0.0.1".into(),
            port: socks_port,
            username: "suser".into(),
            password: "spass".into(),
            country: String::new(),
            notes: String::new(),
        };

        let url = ensure_tunnel(&entry).await.unwrap();
        // Give pre-warm 30ms so warm SOCKS5 handshakes complete before CONNECT.
        tokio::time::sleep(Duration::from_millis(30)).await;

        let addr = url.strip_prefix("http://").unwrap();
        let mut client = TcpStream::connect(addr).await.unwrap();
        client
            .write_all(
                format!("CONNECT localhost:{target_port} HTTP/1.1\r\nHost: localhost:{target_port}\r\n\r\nhello")
                    .as_bytes(),
            )
            .await
            .unwrap();
        let (resp_hdr, mut extra) = read_http_header_block(&mut client, Duration::from_secs(2))
            .await
            .unwrap()
            .unwrap();
        assert!(String::from_utf8_lossy(&resp_hdr).starts_with("HTTP/1.1 200"));
        while extra.len() < 5 {
            let mut b = [0u8; 5];
            let n = client.read(&mut b).await.unwrap();
            extra.extend_from_slice(&b[..n]);
        }
        assert_eq!(&extra[..5], b"world");
    }
}
