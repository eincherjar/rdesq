use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::timeout;

use crate::models::{PingTarget, PingResult};

const MAX_CONCURRENT: usize = 50;
const PING_TIMEOUT_MS: u64 = 800;

pub async fn ping_hosts(targets: Vec<PingTarget>) -> Vec<PingResult> {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
    let mut handles = Vec::with_capacity(targets.len());

    for target in targets {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        handles.push(tokio::spawn(async move {
            let _permit = permit;
            let start = Instant::now();
            let addr = format!("{}:{}", target.host, target.port);
            let result = timeout(
                std::time::Duration::from_millis(PING_TIMEOUT_MS),
                TcpStream::connect(&addr),
            )
            .await;
            let latency = start.elapsed().as_millis() as u64;
            let reachable = result.is_ok() && result.unwrap().is_ok();
            PingResult {
                host: target.host,
                port: target.port,
                protocol: target.protocol,
                reachable,
                latency,
            }
        }));
    }

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        if let Ok(r) = handle.await {
            results.push(r);
        }
    }
    results
}
