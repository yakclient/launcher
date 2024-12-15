use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{create_dir_all, write, File};
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
pub struct PersistedData {
    content: Arc<Mutex<HashMap<String, Value>>>,
}

impl PersistedData {
    pub fn new_empty() -> PersistedData {
        PersistedData {
            content: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn put_value<T, S: AsRef<str>>(&self, name: S, value: T) -> &Self
    where
        T: serde::Serialize,
    {
        let value = serde_json::to_value(&value).expect("Failed to serialize");
        self.content
            .lock()
            .unwrap()
            .insert(name.as_ref().to_string(), value);
        self
    }

    pub fn read_value<T, S: AsRef<str>>(&self, name: S) -> Option<T>
    where
        T: DeserializeOwned,
    {
        if let Some(value) = self.content.lock().unwrap().get(name.as_ref()) {
            Some(serde_json::from_value(value.clone()).expect("Failed to deserialize"))
        } else {
            None
        }
    }

    pub fn persist_to<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = path.as_ref();

        let json = serde_json::to_string_pretty(&self).expect("Unable to serialize data to JSON");
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        write(path, json)?;
        Ok(())
    }

    pub fn read_from<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Self::new_empty());
        };
        let file = File::open(path)?;

        let data: PersistedData =
            serde_json::from_reader(file).expect("Unable to deserialize data to JSON");

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_read_data() {
        let mut data = PersistedData::new_empty();
        let expected_ret = "Hey how are you?".to_string();
        let read = data
            .put_value("test", expected_ret.clone())
            .read_value("test");

        assert!(read.is_some());
        assert_eq!(read, Some(expected_ret));
    }

    #[test]
    fn test_persist() {
        let mut data = PersistedData::new_empty();
        let expected_ret = "Hey how are you?".to_string();
        data.put_value("test", expected_ret.clone());

        data.persist_to("tests/config.json").unwrap();
        let data = PersistedData::read_from("tests/config.json").unwrap();
        let read: String = data.read_value("test").unwrap();

        assert_eq!(read, expected_ret);
    }
}
