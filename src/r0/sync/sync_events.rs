//! [GET /_matrix/client/r0/sync](https://matrix.org/docs/spec/client_server/r0.4.0.html#get-matrix-client-r0-sync)

use std::collections::HashMap;

use ruma_api_macros::ruma_api;
use ruma_events::{
    collections::{all, only},
    stripped,
};
use ruma_identifiers::RoomId;
use serde::{Deserialize, Serialize};

use crate::r0::filter::FilterDefinition;

ruma_api! {
    metadata {
        description: "Get all new events from all rooms since the last sync or a given point of time.",
        method: GET,
        name: "sync",
        path: "/_matrix/client/r0/sync",
        rate_limited: false,
        requires_authentication: true,
    }

    request {
        /// A filter represented either as its full JSON definition or the ID of a saved filter.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub filter: Option<Filter>,
        /// A point in time to continue a sync from.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub since: Option<String>,
        /// Controls whether to include the full state for all rooms the user is a member of.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub full_state: Option<bool>,
        /// Controls whether the client is automatically marked as online by polling this API.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub set_presence: Option<SetPresence>,
        /// The maximum time to poll in milliseconds before returning this request.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ruma_api(query)]
        pub timeout: Option<u64>,
    }

    response {
        /// The batch token to supply in the `since` param of the next `/sync` request.
        pub next_batch: String,
        /// Updates to rooms.
        pub rooms: Rooms,
        /// Updates to the presence status of other users.
        pub presence: Presence,
    }

}

/// Whether to set presence or not during sync.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SetPresence {
    /// Do not set the presence of the user calling this API.
    #[serde(rename = "offline")]
    Offline,
}

/// A filter represented either as its full JSON definition or the ID of a saved filter.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Filter {
    // The filter definition needs to be (de)serialized twice because it is a URL-encoded JSON
    // string. Since #[ruma_api(query)] only does the latter and this is a very uncommon
    // setup, we implement it through custom serde logic for this specific enum variant rather than
    // adding another ruma_api attribute.
    //
    // On the deserialization side, because this is an enum with #[serde(untagged)], serde will
    // try the variants in order (https://serde.rs/enum-representations.html). That means because
    // FilterDefinition is the first variant, JSON decoding is attempted first which is almost
    // functionally equivalent to looking at whether the first symbol is a '{' as the spec says.
    // (there are probably some corner cases like leading whitespace)
    #[serde(with = "filter_def_serde")]
    /// A complete filter definition serialized to JSON.
    FilterDefinition(FilterDefinition),
    /// The ID of a filter saved on the server.
    FilterId(String),
}

mod filter_def_serde {
    use serde::{de::Error as _, ser::Error as _, Deserialize, Deserializer, Serializer};

    use crate::r0::filter::FilterDefinition;

    pub fn serialize<S>(filter_def: &FilterDefinition, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = serde_json::to_string(filter_def).map_err(S::Error::custom)?;
        serializer.serialize_str(&string)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<FilterDefinition, D::Error>
    where
        D: Deserializer<'de>,
    {
        let filter_str = <&str>::deserialize(deserializer)?;
        serde_json::from_str(filter_str).map_err(D::Error::custom)
    }
}

/// Updates to rooms.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Rooms {
    /// The rooms that the user has left or been banned from.
    pub leave: HashMap<RoomId, LeftRoom>,
    /// The rooms that the user has joined.
    pub join: HashMap<RoomId, JoinedRoom>,
    /// The rooms that the user has been invited to.
    pub invite: HashMap<RoomId, InvitedRoom>,
}

/// Historical updates to left rooms.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeftRoom {
    /// The timeline of messages and state changes in the room up to the point when the user
    /// left.
    pub timeline: Timeline,
    /// The state updates for the room up to the start of the timeline.
    pub state: State,
}

/// Updates to joined rooms.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JoinedRoom {
    /// Counts of unread notifications for this room.
    pub unread_notifications: UnreadNotificationsCount,
    /// The timeline of messages and state changes in the room.
    pub timeline: Timeline,
    /// Updates to the state, between the time indicated by the `since` parameter, and the start
    /// of the `timeline` (or all state up to the start of the `timeline`, if `since` is not
    /// given, or `full_state` is true).
    pub state: State,
    /// The private data that this user has attached to this room.
    pub account_data: AccountData,
    /// The ephemeral events in the room that aren't recorded in the timeline or state of the
    /// room. e.g. typing.
    pub ephemeral: Ephemeral,
}

/// unread notifications count
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnreadNotificationsCount {
    /// The number of unread notifications for this room with the highlight flag set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_count: Option<u64>,
    /// The total number of unread notifications for this room.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_count: Option<u64>,
}

/// Events in the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Timeline {
    /// True if the number of events returned was limited by the `limit` on the filter.
    pub limited: bool,
    /// A token that can be supplied to to the `from` parameter of the
    /// `/rooms/{roomId}/messages` endpoint.
    pub prev_batch: String,
    /// A list of events.
    pub events: Vec<all::RoomEvent>,
}

/// State events in the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State {
    /// A list of state events.
    pub events: Vec<only::StateEvent>,
}

/// The private data that this user has attached to this room.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountData {
    /// A list of events.
    pub events: Vec<only::Event>,
}

/// Ephemeral events not recorded in the timeline or state of the room.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ephemeral {
    /// A list of events.
    pub events: Vec<only::Event>,
}

/// Updates to the rooms that the user has been invited to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvitedRoom {
    /// The state of a room that the user has been invited to.
    pub invite_state: InviteState,
}

/// The state of a room that the user has been invited to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InviteState {
    /// A list of state events.
    pub events: Vec<stripped::StrippedState>,
}

/// Updates to the presence status of other users.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Presence {
    /// A list of events.
    pub events: Vec<only::Event>,
}
