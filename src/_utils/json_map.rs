//! This module provides a trait `JsonMap` to simplify accessing and deserializing values
//! from an `axum::Json<Value>` object. It enables retrieving keys and their associated 
//! values from the JSON payload with proper error handling.

use axum::Json;
use inflector::Inflector;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub trait JsonMap {
    fn get_value<T>(&self, key: &str) -> anyhow::Result<T> where T: DeserializeOwned;
}

impl JsonMap for Json<Value> {
    /// Retrieves and deserializes the value associated with the given key from an `axum::Json<Value>`.
    /// 
    /// **The `key` is automatically renamed to camelCase**
    ///
    /// This utility function simplifies extraction of typed data from JSON payloads
    /// in requests, reducing boilerplate to a single line. It returns a deserialized
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

        let camel_key = key.to_camel_case();
        
        if let Some(value) = self.get(&camel_key) {
            let result: T = serde_json::from_value(value.clone())?;
            return Ok(result)
        }

        Err(anyhow::anyhow!("Key \"{camel_key}\" not found in JSON"))
    }
}