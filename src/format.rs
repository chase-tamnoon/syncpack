use serde_json;
use serde_json::Map;
use std;
use std::collections::HashSet;

/// Sort the keys in a JSON object, with the given keys first
pub fn sort_first(value: &mut serde_json::Value, order: &Vec<String>) {
  match value {
    serde_json::Value::Object(obj) => {
      let order_set: HashSet<_> = order.into_iter().collect();
      let mut sorted_obj: Map<String, serde_json::Value> = Map::new();
      let mut remaining_keys: Vec<_> = obj
        .keys()
        .filter(|k| !order_set.contains(*k))
        .cloned()
        .collect();

      remaining_keys.sort();

      for key in order.clone() {
        if let Some(val) = obj.remove(&key) {
          sorted_obj.insert(key, val);
        }
      }

      for key in remaining_keys {
        if let Some(val) = obj.remove(&key) {
          sorted_obj.insert(key, val);
        }
      }

      *value = serde_json::Value::Object(sorted_obj);
    }
    _ => {}
  }
}

/// Sort an array or object alphabetically
pub fn sort_alphabetically(value: &mut serde_json::Value) {
  match value {
    serde_json::Value::Object(obj) => {
      let mut entries: Vec<_> =
        obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
      entries.sort_by(|a, b| a.0.cmp(&b.0));
      let sorted_obj: Map<String, serde_json::Value> =
        entries.into_iter().collect();

      *value = serde_json::Value::Object(sorted_obj);
    }
    serde_json::Value::Array(arr) => {
      arr.sort_by(|a, b| {
        a.as_str()
          .unwrap_or("")
          .partial_cmp(b.as_str().unwrap_or(""))
          .unwrap_or(std::cmp::Ordering::Equal)
      });
    }
    _ => {}
  }
}
