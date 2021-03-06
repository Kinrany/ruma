//! Types for the *m.room.encryption* event.

use js_int::UInt;
use ruma_events_macros::StateEventContent;
use serde::{Deserialize, Serialize};

use crate::{Algorithm, StateEvent};

/// Defines how messages sent in this room should be encrypted.
pub type EncryptionEvent = StateEvent<EncryptionEventContent>;

/// The payload for `EncryptionEvent`.
#[derive(Clone, Debug, Deserialize, Serialize, StateEventContent)]
#[non_exhaustive]
#[ruma_event(type = "m.room.encryption")]
pub struct EncryptionEventContent {
    /// The encryption algorithm to be used to encrypt messages sent in this room.
    ///
    /// Must be `m.megolm.v1.aes-sha2`.
    pub algorithm: Algorithm,

    /// How long the session should be used before changing it.
    ///
    /// `uint!(604800000)` (a week) is the recommended default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_period_ms: Option<UInt>,

    /// How many messages should be sent before changing the session.
    ///
    /// `uint!(100)` is the recommended default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_period_msgs: Option<UInt>,
}

impl EncryptionEventContent {
    /// Creates a new `EncryptionEventContent` with the given algorithm.
    pub fn new(algorithm: Algorithm) -> Self {
        Self { algorithm, rotation_period_ms: None, rotation_period_msgs: None }
    }
}
