//! [POST /_matrix/client/r0/keys/claim](https://matrix.org/docs/spec/client_server/r0.6.0#post-matrix-client-r0-keys-claim)

use std::collections::BTreeMap;

use std::time::Duration;

use ruma_api::ruma_api;
use ruma_identifiers::{DeviceId, UserId};
use serde_json::Value as JsonValue;

use super::{AlgorithmAndDeviceId, KeyAlgorithm, OneTimeKey};

ruma_api! {
    metadata: {
        description: "Claims one-time keys for use in pre-key messages.",
        method: POST,
        name: "claim_keys",
        path: "/_matrix/client/r0/keys/claim",
        rate_limited: false,
        requires_authentication: true,
    }

    request: {
        /// The time (in milliseconds) to wait when downloading keys from remote servers.
        /// 10 seconds is the recommended default.
        #[serde(
            with = "ruma_serde::duration::opt_ms",
            default,
            skip_serializing_if = "Option::is_none",
        )]
        pub timeout: Option<Duration>,

        /// The keys to be claimed.
        pub one_time_keys: BTreeMap<UserId, BTreeMap<Box<DeviceId>, KeyAlgorithm>>,
    }

    response: {
        /// If any remote homeservers could not be reached, they are recorded here.
        /// The names of the properties are the names of the unreachable servers.
        pub failures: BTreeMap<String, JsonValue>,

        /// One-time keys for the queried devices.
        pub one_time_keys: BTreeMap<UserId, OneTimeKeys>,
    }

    error: crate::Error
}

/// The one-time keys for a given device.
pub type OneTimeKeys = BTreeMap<Box<DeviceId>, BTreeMap<AlgorithmAndDeviceId, OneTimeKey>>;
