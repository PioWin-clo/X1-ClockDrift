use anyhow::{Context, Result};
use rand::Rng;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};

const USER_AGENT: &str =
    "x1-clockdrift/0.1 (+https://github.com/PioWin-clo/X1-ClockDrift)";

/// Use 'confirmed' commitment to reduce data staleness from ~14s
/// (default 'finalized') to ~1s. Both commitment levels still produce
/// consistent drift measurements because chain_time and t_local refer
/// to the same chain state — fresher just means less lag, not different
/// drift values. Empirically validated against Sentinel production data.
const COMMITMENT: &str = "confirmed";

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct VoteAccountInfo {
    #[serde(rename = "votePubkey")]
    pub vote_pubkey: String,
    #[serde(rename = "nodePubkey")]
    pub node_pubkey: String,
    #[serde(rename = "activatedStake")]
    pub activated_stake: i64,
}

#[derive(Debug, Deserialize)]
struct VoteAccountsResponse {
    current: Vec<VoteAccountInfo>,
    delinquent: Vec<VoteAccountInfo>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct BlockData {
    pub parent_slot: u64,
    pub block_time: Option<i64>,
    pub transactions: Vec<TxView>,
}

#[derive(Debug)]
pub struct TxView {
    pub instructions: Vec<Value>,
}

#[derive(Debug)]
pub enum RpcError {
    Rpc { code: i64, message: String },
    Http(reqwest::Error),
    Decode(String),
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcError::Rpc { code, message } => {
                write!(f, "rpc error code {code}: {message}")
            }
            RpcError::Http(e) => write!(f, "http: {e}"),
            RpcError::Decode(s) => write!(f, "decode: {s}"),
        }
    }
}

impl std::error::Error for RpcError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RpcError::Http(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for RpcError {
    fn from(e: reqwest::Error) -> Self {
        RpcError::Http(e)
    }
}

pub struct RpcClient {
    http: reqwest::Client,
    url: String,
    sem: Arc<Semaphore>,
    /// Tracks the time of the next allowed RPC issue. Updated atomically
    /// inside `rate_gate` (under a short Mutex lock) before sleeping
    /// outside the lock. Pre-v0.4.1 this held the timestamp of the last
    /// completed call and the mutex was held during sleep — that
    /// serialized concurrent callers despite the Semaphore.
    last_send: Arc<Mutex<Option<Instant>>>,
    min_gap: Duration,
}

impl RpcClient {
    pub fn new(url: &str, rate_per_sec: u32) -> Result<Self> {
        let rate = rate_per_sec.max(1);
        let min_gap = Duration::from_millis(1000 / rate as u64);
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .user_agent(USER_AGENT)
            .build()
            .context("building reqwest client")?;
        Ok(Self {
            http,
            url: url.to_string(),
            sem: Arc::new(Semaphore::new(rate as usize)),
            last_send: Arc::new(Mutex::new(None)),
            min_gap,
        })
    }

    /// Reserve a unique time slot under the lock, then sleep until that
    /// slot WITHOUT holding the lock. Concurrent callers each get a
    /// monotonically-increasing slot allocated in O(microseconds), and
    /// then sleep concurrently — preserving the rate budget while
    /// allowing the Semaphore to actually do its job. Pre-v0.4.1 this
    /// function held the lock across the sleep, which serialized all
    /// callers.
    async fn rate_gate(&self) {
        let next_allowed = {
            let mut last = self.last_send.lock().await;
            let now = Instant::now();
            // Each new caller's slot is at least `min_gap` after the
            // previous reservation, but never earlier than now.
            let next = match *last {
                Some(prev) => prev.max(now) + self.min_gap,
                None => now,
            };
            *last = Some(next);
            next
        };
        let now = Instant::now();
        if next_allowed > now {
            tokio::time::sleep(next_allowed - now).await;
        }
    }

    async fn raw_call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let _permit = self
            .sem
            .acquire()
            .await
            .map_err(|e| RpcError::Decode(e.to_string()))?;
        self.rate_gate().await;

        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let resp = self.http.post(&self.url).json(&body).send().await?;
        let status = resp.status();
        let v: Value = resp.json().await?;
        if let Some(err) = v.get("error") {
            let code = err.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
            let message = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown")
                .to_string();
            return Err(RpcError::Rpc { code, message });
        }
        if !status.is_success() {
            return Err(RpcError::Decode(format!("http status {status}")));
        }
        v.get("result")
            .cloned()
            .ok_or_else(|| RpcError::Decode("missing result".into()))
    }

    #[allow(dead_code)]
    pub async fn get_slot(&self) -> Result<u64, RpcError> {
        let v = self
            .raw_call("getSlot", json!([{ "commitment": COMMITMENT }]))
            .await?;
        v.as_u64()
            .ok_or_else(|| RpcError::Decode("getSlot: not u64".into()))
    }

    pub async fn get_vote_accounts(&self) -> Result<Vec<VoteAccountInfo>, RpcError> {
        let v = self
            .raw_call("getVoteAccounts", json!([{ "commitment": COMMITMENT }]))
            .await?;
        let parsed: VoteAccountsResponse = serde_json::from_value(v)
            .map_err(|e| RpcError::Decode(format!("getVoteAccounts: {e}")))?;
        let mut out = parsed.current;
        out.extend(parsed.delinquent);
        Ok(out)
    }

    /// Returns Ok(None) for skipped slots and unsupported txn versions; other
    /// errors propagate. Uses `jsonParsed` encoding so vote instructions arrive
    /// pre-decoded from the RPC. Uses `confirmed` commitment (v0.4.1) for
    /// fresher data — note this does NOT change drift measurement
    /// accuracy, only data staleness.
    pub async fn get_block_with_votes(&self, slot: u64) -> Result<Option<BlockData>, RpcError> {
        let params = json!([
            slot,
            {
                "commitment": COMMITMENT,
                "encoding": "jsonParsed",
                "transactionDetails": "full",
                "rewards": false,
                "maxSupportedTransactionVersion": 1
            }
        ]);

        let mut last_err: Option<RpcError> = None;
        let backoffs = [Duration::from_secs(1), Duration::from_secs(4)];

        for attempt in 0..=backoffs.len() {
            match self.raw_call("getBlock", params.clone()).await {
                Ok(v) => {
                    if v.is_null() {
                        return Ok(None);
                    }
                    let parsed = parse_block(v).map_err(RpcError::Decode)?;
                    return Ok(Some(parsed));
                }
                Err(RpcError::Rpc { code, ref message }) => {
                    if code == -32015 {
                        tracing::warn!(slot, code, "txn version not supported, skipping slot");
                        return Ok(None);
                    }
                    if code == -32004 || code == -32007 || code == -32009 {
                        tracing::debug!(slot, code, "block missing/skipped");
                        return Ok(None);
                    }
                    let is_retryable = code == -32005 || code == 429 || code >= 500;
                    last_err = Some(RpcError::Rpc {
                        code,
                        message: message.clone(),
                    });
                    if !is_retryable || attempt >= backoffs.len() {
                        break;
                    }
                    tokio::time::sleep(backoff_with_jitter(backoffs[attempt])).await;
                }
                Err(e) => {
                    last_err = Some(e);
                    if attempt >= backoffs.len() {
                        break;
                    }
                    tokio::time::sleep(backoff_with_jitter(backoffs[attempt])).await;
                }
            }
        }
        Err(last_err.unwrap_or_else(|| RpcError::Decode("all retries exhausted".into())))
    }
}

/// Add 0–100 ms uniform jitter to a base backoff. Prevents thundering-herd
/// retries when many concurrent calls hit the same transient RPC error.
fn backoff_with_jitter(base: Duration) -> Duration {
    let jitter_ms: u64 = rand::thread_rng().gen_range(0..100);
    base + Duration::from_millis(jitter_ms)
}

fn parse_block(v: Value) -> Result<BlockData, String> {
    let parent_slot = v
        .get("parentSlot")
        .and_then(|x| x.as_u64())
        .ok_or_else(|| "missing parentSlot".to_string())?;
    let block_time = v.get("blockTime").and_then(|x| x.as_i64());

    let mut transactions = Vec::new();
    if let Some(txs) = v.get("transactions").and_then(|x| x.as_array()) {
        for t in txs {
            let ixs_value = t.pointer("/transaction/message/instructions");
            let instructions = match ixs_value.and_then(|v| v.as_array()) {
                Some(arr) => arr.clone(),
                None => continue,
            };
            transactions.push(TxView { instructions });
        }
    }

    Ok(BlockData {
        parent_slot,
        block_time,
        transactions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_block_basic() {
        let v = json!({
            "parentSlot": 100,
            "blockTime": 1700000000,
            "transactions": [
                {
                    "transaction": {
                        "message": {
                            "accountKeys": ["AAA", "BBB"],
                            "instructions": [
                                {
                                    "parsed": {"type": "towersync", "info": {}},
                                    "program": "vote",
                                    "programId": "Vote111111111111111111111111111111111111111"
                                }
                            ]
                        }
                    }
                }
            ]
        });
        let bd = parse_block(v).unwrap();
        assert_eq!(bd.parent_slot, 100);
        assert_eq!(bd.block_time, Some(1700000000));
        assert_eq!(bd.transactions.len(), 1);
        assert_eq!(bd.transactions[0].instructions.len(), 1);
        assert_eq!(
            bd.transactions[0].instructions[0]["program"]
                .as_str()
                .unwrap(),
            "vote"
        );
    }

    #[test]
    fn parse_block_skips_txns_without_instructions() {
        let v = json!({
            "parentSlot": 1,
            "transactions": [
                {"transaction": {}},
                {"transaction": {"message": {"instructions": []}}}
            ]
        });
        let bd = parse_block(v).unwrap();
        assert_eq!(bd.transactions.len(), 1);
        assert!(bd.transactions[0].instructions.is_empty());
    }

    #[test]
    fn backoff_jitter_within_bounds() {
        let base = Duration::from_secs(2);
        for _ in 0..50 {
            let d = backoff_with_jitter(base);
            assert!(d >= base);
            assert!(d < base + Duration::from_millis(100));
        }
    }

    /// Regression guard for the v0.4.1 rate_gate fix. Pre-fix, the mutex
    /// was held across the sleep, so 10 concurrent calls at 5/sec
    /// serialized into ~10 × 200 ms = 2 s of *total wall time*… but
    /// arrived sequentially, NOT concurrently. With the fix, callers
    /// reserve their slot in microseconds and sleep concurrently —
    /// the LAST call still completes around t≈1.8 s (9 × 200 ms gap),
    /// but each individual sleep happens in parallel rather than in
    /// a queue. The bound below catches the pre-fix behaviour, which
    /// would have been ~2 × 10 × 200 ms = ~4 s under heavy contention
    /// (each call waits for prior call's mutex AND its own sleep).
    #[tokio::test]
    async fn rate_gate_allows_concurrency() {
        let client = Arc::new(RpcClient::new("http://localhost:1", 5).unwrap());
        let start = Instant::now();
        let mut handles = Vec::new();
        for _ in 0..10 {
            let c = client.clone();
            handles.push(tokio::spawn(async move { c.rate_gate().await }));
        }
        for h in handles {
            h.await.unwrap();
        }
        let elapsed = start.elapsed();
        // Lower bound: 9 gaps × 200 ms = 1.8 s (the LAST caller's slot).
        // First call has no wait; second waits 200 ms; tenth waits 1800 ms.
        // We give 200 ms slack on either side for scheduler jitter.
        assert!(
            elapsed >= Duration::from_millis(1600),
            "rate limit too loose; expected ≥1.6s got {elapsed:?}"
        );
        // Upper bound: catches the pre-fix serialization. With mutex
        // held during sleep, total time would compound to ~2.0 s + queue
        // delays, often >2.5 s.
        assert!(
            elapsed < Duration::from_millis(2500),
            "rate limit serialized callers; expected <2.5s got {elapsed:?}"
        );
    }
}
