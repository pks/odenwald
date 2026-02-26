use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct SparseVector {
    pub map: HashMap<String, f64>,
}

impl SparseVector {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn from_kv(s: &str, kv_sep: char, pair_sep: char) -> Self {
        let mut map = HashMap::new();
        for pair in s.split(pair_sep) {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }
            if let Some((k, v)) = pair.split_once(kv_sep) {
                if let Ok(val) = v.trim().parse::<f64>() {
                    map.insert(k.trim().to_string(), val);
                }
            }
        }
        Self { map }
    }

    pub fn from_hash(h: &serde_json::Map<String, serde_json::Value>) -> Self {
        let mut map = HashMap::new();
        for (k, v) in h {
            if let Some(val) = v.as_f64() {
                map.insert(k.clone(), val);
            }
        }
        Self { map }
    }

    pub fn dot(&self, other: &SparseVector) -> f64 {
        let mut sum = 0.0;
        for (k, v) in &self.map {
            if let Some(ov) = other.map.get(k) {
                sum += v * ov;
            }
        }
        sum
    }

    pub fn to_json(&self) -> serde_json::Value {
        let map: serde_json::Map<String, serde_json::Value> = self
            .map
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::from(*v)))
            .collect();
        serde_json::Value::Object(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_kv() {
        let sv = SparseVector::from_kv("logp 2\nuse_house 0\nuse_shell 1", ' ', '\n');
        assert_eq!(sv.map["logp"], 2.0);
        assert_eq!(sv.map["use_house"], 0.0);
        assert_eq!(sv.map["use_shell"], 1.0);
    }

    #[test]
    fn test_dot() {
        let a = SparseVector::from_kv("x=1 y=2", '=', ' ');
        let b = SparseVector::from_kv("x=3 y=4 z=5", '=', ' ');
        assert_eq!(a.dot(&b), 11.0);
    }

    #[test]
    fn test_empty_dot() {
        let a = SparseVector::new();
        let b = SparseVector::from_kv("x=1", '=', ' ');
        assert_eq!(a.dot(&b), 0.0);
    }
}
