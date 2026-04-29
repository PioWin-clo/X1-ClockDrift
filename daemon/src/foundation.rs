//! Hardcoded list of X1 Labs Foundation validators.
//!
//! These are the official infrastructure nodes operated by X1 Labs. They
//! share infrastructure intentionally and have a baseline drift profile
//! that should not be confused with misconfigured validators or random
//! "farms". The cluster-detection algorithm explicitly skips them; the
//! dashboard surfaces them in a dedicated showcase section so their drift
//! is presented as operational baseline, not failure.
//!
//! Source: x1val.online + production RPC `getVoteAccounts` filtered by
//! 50–65M XNT activated stake and 0% commission. List frozen at deploy
//! time; updates require a code release. (At ~12 nodes this is small
//! enough that maintaining a registry table in the DB is overkill.)

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FoundationNode {
    pub identity: &'static str,
    pub vote_account: &'static str,
    pub label: &'static str,
}

pub const X1_LABS_FOUNDATION: &[FoundationNode] = &[
    FoundationNode {
        identity: "8LWKkcxFz4kWWExAVLfUAFLvoKVrWnqRawH1T7gHHeNg",
        vote_account: "6Wf81YuCHu3j7xJupCq5mxDWz8seuNkybyT9riVm5FeA",
        label: "X1 Labs (node8)",
    },
    FoundationNode {
        identity: "7ufaUVtQKzGu5tpFtii9Cg8kR4jcpjQSXwsF3oVPSMZA",
        vote_account: "9v8bGQk9JhUhbxGock4KAhUfzCe9VtJBX15fNDQY4mkw",
        label: "X1 Labs (node0)",
    },
    FoundationNode {
        identity: "5Rzytnub9yGTFHqSmauFLsAbdXFbehMwPBLiuEgKajUN",
        vote_account: "Hdcj25JfB7oPAwpCiedYRKUBYFBMGuw4GrQ9JZF5112",
        label: "X1 Labs (node1)",
    },
    FoundationNode {
        identity: "7J5wJaH55ZYjCCmCMt7Gb3QL6FGFmjz5U8b6NcbzfoTy",
        vote_account: "2Rr9ocgMFfVcuxtWhQ6s4dukCiViG6CuguFNzgoGmuRb",
        label: "X1 Labs (node4)",
    },
    FoundationNode {
        identity: "4V2QkkWce8bwTzvvwPiNRNQ4W433ZsGQi9aWU12Q8uBF",
        vote_account: "DWNBX8QrjefyeHY2VFwRvVWp8nPUEoLHXTm3S7bu1d7E",
        label: "X1 Labs (node2)",
    },
    FoundationNode {
        identity: "CkMwg4TM6jaSC5rJALQjvLc51XFY5pJ1H9f1Tmu5Qdxs",
        vote_account: "F16B8rLuY5B1S3Bj9Gyj7Hc9hxoFe894tWbU4t57uWDj",
        label: "X1 Labs (node3)",
    },
    FoundationNode {
        identity: "73RKDYK431DFw3bJXBN9ztk5UbdkWYyWCTTm7JLM7YUr",
        vote_account: "2EstMAjQXebLQtoWUP4KQfvk7KhnreBpbXjnxotZ5xGS",
        label: "X1 Labs (node6)",
    },
    FoundationNode {
        identity: "8gv2Vx7Go1hUAD2TQx2HEwn8JEb9FqtguutTMQ43wT2o",
        vote_account: "31MDFmh6QDYFRbytMLnJdUoGtznLMfojtrQzzav4to5r",
        label: "X1 Labs (node5)",
    },
    FoundationNode {
        identity: "Gv5kyHCneaRKNJPgyPreoiYnBVBm2XYqt981zYykcSSU",
        vote_account: "BYAt5rm4CXnp3C5Hgejri2wQdErMBRjDFVzqmFopDJRB",
        label: "X1 Labs (node9)",
    },
    FoundationNode {
        identity: "EXDQt1T1eQ4NjttSdxn1eNS3EkHDrmZ3ZrgZmMSbfYiy",
        vote_account: "2aoje61DYarSYEb4VgCStpkivfZqyo5DQ4vkY9iq3SiU",
        label: "X1 Labs (node10)",
    },
    FoundationNode {
        identity: "B9xaPxcm3qKe15EiK4j6Am5eGp3Mxkzaci6Ywbs4it7Q",
        vote_account: "9kg4suZeNNJ4ytZQUtzryJY8KQTFL5PvfMjnioTDZdXi",
        label: "X1 Labs (node11)",
    },
    FoundationNode {
        identity: "4Y9fnKcTJ3Kxj6744HZX8ubd89DPKibyKckGnPWGkfU3",
        vote_account: "HsMbWVLNxCRFokzaBzTRmsjc5Z4xQqd4W7YdajpQrZDE",
        label: "X1 Labs (node7)",
    },
];

/// Lookup a validator by its **vote account** pubkey. Returns the
/// matching `FoundationNode` if it's an X1 Labs node.
pub fn lookup_foundation(vote_account: &str) -> Option<&'static FoundationNode> {
    X1_LABS_FOUNDATION
        .iter()
        .find(|n| n.vote_account == vote_account)
}

/// Convenience predicate.
#[allow(dead_code)]
pub fn is_foundation(vote_account: &str) -> bool {
    lookup_foundation(vote_account).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_foundation_nodes_lookup() {
        let n = lookup_foundation("6Wf81YuCHu3j7xJupCq5mxDWz8seuNkybyT9riVm5FeA").unwrap();
        assert_eq!(n.label, "X1 Labs (node8)");
        assert_eq!(n.identity, "8LWKkcxFz4kWWExAVLfUAFLvoKVrWnqRawH1T7gHHeNg");
    }

    #[test]
    fn unknown_pubkey_returns_none() {
        assert!(lookup_foundation("NotAFoundationNode111111111111111111111111").is_none());
        assert!(!is_foundation("NotAFoundationNode111111111111111111111111"));
    }

    #[test]
    fn list_has_twelve_nodes() {
        assert_eq!(X1_LABS_FOUNDATION.len(), 12);
    }

    #[test]
    fn vote_accounts_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for n in X1_LABS_FOUNDATION {
            assert!(
                seen.insert(n.vote_account),
                "duplicate vote_account in foundation list: {}",
                n.vote_account
            );
        }
    }
}
