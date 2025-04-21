use rocket::serde::json::{serde_json, Json, Value};
use serde::de::DeserializeOwned;

pub trait JsonMap {
    fn get_value<T>(&self, key: &str) -> anyhow::Result<T> where T: DeserializeOwned;
}

impl JsonMap for Json<Value> {
    fn get_value<T>(&self, key: &str) -> anyhow::Result<T> where T: DeserializeOwned {

        if let Some(value) = self.get(key) {
            let result: T = serde_json::from_value(value.clone())?;
            return Ok(result)
        }

        Err(anyhow::anyhow!("Key \"{}\" not found in JSON", key))
    }
}