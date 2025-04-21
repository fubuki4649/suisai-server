//! This module provides a trait `JsonMap` to simplify accessing and deserializing values
//! from a `Json<Value>` object in Rocket. It enables retrieving keys and their associated 
//! values from the JSON payload with proper error handling.
use rocket::serde::json::{serde_json, Json, Value};
use serde::de::DeserializeOwned;

pub trait JsonMap {
    fn get_value<T>(&self, key: &str) -> anyhow::Result<T> where T: DeserializeOwned;
}

impl JsonMap for Json<Value> {
    /// Retrieves and deserializes the value associated with the given key from a `Json<Value>`.
    ///
    /// This utility function simplifies extraction of typed data from JSON payloads
    /// in Rocket requests, reducing boilerplate to a single line. It returns a deserialized
    /// value of the specified type or an error if the key is missing or the type conversion fails.
    ///
    /// # Example
    /// ```
    /// let name: String = json.get_value("name")?;
    /// ```
    ///
    /// # Errors
    /// Returns an `anyhow::Error` if the key does not exist or if deserialization fails.
    fn get_value<T>(&self, key: &str) -> anyhow::Result<T> where T: DeserializeOwned {

        if let Some(value) = self.get(key) {
            let result: T = serde_json::from_value(value.clone())?;
            return Ok(result)
        }

        Err(anyhow::anyhow!("Key \"{}\" not found in JSON", key))
    }
}