use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};

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
    pub account_keys: Vec<String>,
    pub instructions: Vec<IxView>,
}

#[derive(Debug)]
pub struct IxView {
    pub program_id_index: usize,
    pub accounts: Vec<usize>,
    pub data_bytes: Vec<u8>,
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
    last_send: Arc<Mutex<Option<Instant>>>,
    min_gap: Duration,
}

impl RpcClient {
    pub fn new(url: &str, rate_per_sec: u32) -> Result<Self> {
        let rate = rate_per_sec.max(1);
        let min_gap = Duration::from_millis(1000 / rate as u64);
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
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

    async fn rate_gate(&self) {
        let mut last = self.last_send.lock().await;
        let now = Instant::now();
        if let Some(prev) = *last {
            let elapsed = now.duration_since(prev);
            if elapsed < self.min_gap {
                tokio::time::sleep(self.min_gap - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }

    async fn raw_call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let _permit = self.sem.acquire().await.map_err(|e| RpcError::Decode(e.to_string()))?;
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
        let v = self.raw_call("getSlot", json!([])).await?;
        v.as_u64().ok_or_else(|| RpcError::Decode("getSlot: not u64".into()))
    }

    pub async fn get_vote_accounts(&self) -> Result<Vec<VoteAccountInfo>, RpcError> {
        let v = self.raw_call("getVoteAccounts", json!([])).await?;
        let parsed: VoteAccountsResponse = serde_json::from_value(v)
            .map_err(|e| RpcError::Decode(format!("getVoteAccounts: {e}")))?;
        let mut out = parsed.current;
        out.extend(parsed.delinquent);
        Ok(out)
    }

    /// Returns Ok(None) if the slot returned -32015 (txn version not supported)
    /// or -32004/-32007 (slot not found / skipped). Other errors propagate.
    pub async fn get_block_with_votes(&self, slot: u64) -> Result<Option<BlockData>, RpcError> {
        let params = json!([
            slot,
            {
                "encoding": "json",
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
                    tokio::time::sleep(backoffs[attempt]).await;
                }
                Err(e) => {
                    last_err = Some(e);
                    if attempt >= backoffs.len() {
                        break;
                    }
                    tokio::time::sleep(backoffs[attempt]).await;
                }
            }
        }
        Err(last_err.unwrap_or_else(|| RpcError::Decode("all retries exhausted".into())))
    }
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
            let msg = match t.pointer("/transaction/message") {
                Some(m) => m,
                None => continue,
            };
            let account_keys: Vec<String> = msg
                .get("accountKeys")
                .and_then(|x| x.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(|x| x.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let mut ixs = Vec::new();
            if let Some(arr) = msg.get("instructions").and_then(|x| x.as_array()) {
                for ix in arr {
                    let program_id_index = match ix.get("programIdIndex").and_then(|x| x.as_u64()) {
                        Some(n) => n as usize,
                        None => continue,
                    };
                    let accounts: Vec<usize> = ix
                        .get("accounts")
                        .and_then(|x| x.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|n| n.as_u64().map(|x| x as usize))
                                .collect()
                        })
                        .unwrap_or_default();
                    let data_str = ix.get("data").and_then(|x| x.as_str()).unwrap_or("");
                    let data_bytes = bs58::decode(data_str).into_vec().unwrap_or_default();
                    ixs.push(IxView {
                        program_id_index,
                        accounts,
                        data_bytes,
                    });
                }
            }
            transactions.push(TxView {
                account_keys,
                instructions: ixs,
            });
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
                            "accountKeys": ["AAA", "BBB", "Vote111111111111111111111111111111111111111"],
                            "instructions": [
                                {"programIdIndex": 2, "accounts": [0, 1], "data": "3DTZbgwnSZQjL"}
                            ]
                        }
                    }
                }
            ]
        });
        let bd = parse_block(v).unwrap();
        assert_eq!(bd.parent_slot, 100);
        assert_eq!(bd.transactions.len(), 1);
        assert_eq!(bd.transactions[0].account_keys.len(), 3);
        assert_eq!(bd.transactions[0].instructions[0].program_id_index, 2);
    }
}
