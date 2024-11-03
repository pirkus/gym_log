use bson::serde_helpers::deserialize_hex_string_from_object_id;
use bson::serde_helpers::serialize_hex_string_as_object_id;
use serde::{Deserialize, Serialize};

pub mod handlers;

#[derive(Serialize, Deserialize, Debug)]
pub struct Exercise {
    #[serde(serialize_with = "serialize_hex_string_as_object_id", deserialize_with = "deserialize_hex_string_from_object_id")]
    pub _id: String,
    pub name: String,
    pub desc: String,
}
