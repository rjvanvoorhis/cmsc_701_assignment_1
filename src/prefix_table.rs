use itertools::Itertools;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug)]
pub struct CompactPrefixTable(Vec<Option<(usize, usize)>>);

impl CompactPrefixTable {
    pub fn get(&self, idx: usize) -> Option<(usize, usize)> {
        if let Some(pair) = self.0.get(idx) {
            return *pair;
        }
        None
    }

    pub fn k(&self) -> u16 {
        (self.0.len() as f32).log(4.0) as u16
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(from = "CompactPrefixTable")]
pub struct PrefixTable(u16, HashMap<String, (usize, usize)>);

impl PrefixTable {
    pub fn get(&self, k: &str) -> Option<(usize, usize)> {
        if let Some(pair) = self.1.get(k) {
            return Some(*pair);
        }
        None
    }

    pub fn k(&self) -> u16 {
        self.0
    }

    pub fn new(k: u16) -> Self {
        Self(k, HashMap::new())
    }

    pub fn insert(&mut self, k: String, v: (usize, usize)) {
        self.1.insert(k, v);
    }
}

impl From<CompactPrefixTable> for PrefixTable {
    fn from(other: CompactPrefixTable) -> Self {
        let k = other.k();
        let mut table: HashMap<String, (usize, usize)> = HashMap::new();
        for (idx, chars) in (0..k)
            .map(|_| "ACGT".chars())
            .multi_cartesian_product()
            .enumerate()
        {
            let key: String = chars.iter().collect();
            if let Some(value) = other.0[idx] {
                table.insert(key, value);
            }
        }
        Self(k, table)
    }
}

impl Serialize for PrefixTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        CompactPrefixTable::from(self).serialize(serializer)
    }
}

impl<'a> From<&'a PrefixTable> for CompactPrefixTable {
    fn from(other: &'a PrefixTable) -> Self {
        let mut table: Vec<Option<(usize, usize)>> = Vec::new();
        if let Some(key) = other.1.keys().next() {
            let k = key.len();
            for chars in (0..k).map(|_| "ACGT".chars()).multi_cartesian_product() {
                let prefix: String = chars.iter().collect();
                table.push(other.get(&prefix))
            }
        }
        // let k = (other.0.len() as f32).log(4.0) as u16;
        Self(table)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn create_sample_vector() -> Vec<Option<(usize, usize)>> {
        let mut sample_vector: Vec<Option<(usize, usize)>> = Vec::new();
        sample_vector.push(Some((0, 1)));
        sample_vector.push(Some((1, 4)));
        sample_vector.push(None);
        sample_vector.push(Some((4, 5)));
        sample_vector
    }

    #[test]
    fn convert_from_compact() {
        let sample_vector = create_sample_vector();
        let original = CompactPrefixTable(sample_vector);
        let original_pair = original.get(0).unwrap();
        let prefix_table = PrefixTable::from(original);
        assert_eq!(original_pair, prefix_table.get("A").unwrap());
    }

    #[test]
    fn deserialize_to_compact() {
        let sample_vector = create_sample_vector();

        let original = CompactPrefixTable(sample_vector);
        let original_bytes = bincode::serialize(&original).unwrap();
        let compact_copy: CompactPrefixTable = bincode::deserialize(&original_bytes).unwrap();
        assert_eq!(original, compact_copy);
    }

    #[test]
    fn deserialize_to_prefix_table() {
        let sample_vector = create_sample_vector();

        let original = CompactPrefixTable(sample_vector);
        let original_bytes = bincode::serialize(&original).unwrap();
        let converted_prefix_table: PrefixTable = PrefixTable::from(original);
        let deserialized_prefix_table: PrefixTable = bincode::deserialize(&original_bytes).unwrap();
        assert_eq!(converted_prefix_table, deserialized_prefix_table);
    }
}
