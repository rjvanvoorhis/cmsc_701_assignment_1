use eyre::{eyre, Report, Result};
use itertools::Itertools;
use serde::{ser::SerializeTupleVariant, Deserialize, Serialize};
use std::{collections::HashMap, iter::zip};

use crate::search::Span;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum PrefixTable {
    Sparse(u16, HashMap<String, Span>),
    Dense(Vec<Option<Span>>),
}

impl PrefixTable {
    pub fn new_dense(k: u16) -> Self {
        Self::Dense(vec![None; 4_usize.pow(k as u32)])
    }

    pub fn new_sparse(k: u16) -> Self {
        Self::Sparse(k, HashMap::new())
    }

    pub fn k(&self) -> u16 {
        match self {
            Self::Dense(table) => (table.len() as f32).log(4.0) as u16,
            Self::Sparse(k, _) => *k,
        }
    }

    pub fn get(&self, k: &str) -> Option<Span> {
        match self {
            Self::Sparse(_, table) => table.get(k).copied(),
            Self::Dense(table) => {
                let index = prefix_to_index(k).unwrap();
                if let Some(pair) = table.get(index) {
                    return *pair;
                }
                None
            }
        }
    }

    pub fn insert(&mut self, k: String, v: Span) {
        match self {
            Self::Dense(table) => {
                let index = prefix_to_index(&k).unwrap();
                table.insert(index, Some(v))
            }
            Self::Sparse(_, table) => {
                table.insert(k, v);
            }
        }
    }

    pub fn to_dense(other: Self) -> Self {
        match other {
            Self::Dense(_) => other,
            Self::Sparse(k, mut table) => {
                let dense = (0..k)
                    .map(|_| "ACGT".chars())
                    .multi_cartesian_product()
                    .map(|chars| {
                        let prefix: String = chars.iter().collect();
                        table.remove(&prefix)
                    })
                    .collect::<Vec<Option<Span>>>();
                Self::Dense(dense)
            }
        }
    }

    pub fn clone_dense(other: &Self) -> Self {
        match other {
            Self::Dense(_) => other.clone(),
            Self::Sparse(k, table) => {
                let dense = (0..*k)
                    .map(|_| "ACGT".chars())
                    .multi_cartesian_product()
                    .map(|chars| {
                        let prefix: String = chars.iter().collect();
                        table.get(&prefix).copied()
                    })
                    .collect::<Vec<Option<Span>>>();
                Self::Dense(dense)
            }
        }
    }

    pub fn to_sparse(other: Self) -> Self {
        let k = other.k();
        match other {
            Self::Sparse(_, _) => other,
            Self::Dense(table) => {
                let mut sparse: HashMap<String, Span> = HashMap::new();
                zip(
                    (0..k)
                        .map(|_| "ACGT".chars())
                        .multi_cartesian_product()
                        .map(|x| x.iter().collect::<String>()),
                    table.into_iter(),
                )
                .for_each(|(prefix, span)| {
                    if let Some(value) = span {
                        sparse.insert(prefix, value);
                    }
                });
                Self::Sparse(k, sparse)
            }
        }
    }
}

impl Serialize for PrefixTable {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Sparse(k, table) => {
                if *k < 12 {
                    let dense = Self::clone_dense(self);
                    return dense.serialize(serializer);
                }
                let mut state =
                    serializer.serialize_tuple_variant("PrefixTable", 0, "Sparse", 2)?;
                state.serialize_field(k)?;
                state.serialize_field(table)?;
                state.end()
            }
            Self::Dense(table) => {
                let mut state = serializer.serialize_tuple_variant("PrefixTable", 1, "Dense", 1)?;
                state.serialize_field(table)?;
                state.end()
            }
        }
    }
}

fn nucleotide_to_int(nucleotide: &char) -> Result<usize> {
    match nucleotide {
        'A' => Ok(0),
        'C' => Ok(1),
        'G' => Ok(2),
        'T' => Ok(3),
        _ => Err(eyre!("Received unexpected nucleotide: {nucleotide}")),
    }
}

fn nucleotide_to_value(nucleotide: &char) -> Result<char> {
    match nucleotide {
        'A' => Ok('0'),
        'C' => Ok('1'),
        'G' => Ok('2'),
        'T' => Ok('3'),
        _ => Err(eyre!("Received unexpected nucleotide: {nucleotide}")),
    }
}

/// Convert a prefix like AAC to it's position in the prefix array
/// ```
/// # use assignment_1::prefix_table::prefix_to_index;
/// assert_eq!(prefix_to_index("AAC").unwrap(), 1);
/// assert_eq!(prefix_to_index("ATC").unwrap(), 13);
pub fn prefix_to_index(prefix: &str) -> Result<usize> {
    let result = prefix
        .chars()
        .map(|x| nucleotide_to_value(&x))
        .collect::<Result<String>>()?;
    Ok(usize::from_str_radix(&result, 4)?)
}

/// Convert a prefix like AAC to it's position in the prefix array (alternate implementation)
/// ```
/// # use assignment_1::prefix_table::{prefix_to_index_custom as prefix_to_index};
/// assert_eq!(prefix_to_index("AAC").unwrap(), 1);
/// assert_eq!(prefix_to_index("ATC").unwrap(), 13);
pub fn prefix_to_index_custom(prefix: &str) -> Result<usize> {
    let mut result = 0_usize;
    prefix
        .chars()
        .rev()
        .enumerate()
        .try_for_each(|(idx, nucleotide)| {
            result += 4_usize.pow(idx as u32) * nucleotide_to_int(&nucleotide)?;
            Ok::<(), Report>(())
        })?;
    Ok(result)
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    fn test_serialize_deserialize_sparse() {
        let mut table = PrefixTable::new_sparse(2);
        table.insert(String::from("AA"), (0, 1));
        table.insert(String::from("AC"), (1, 3));
        table.insert(String::from("TT"), (3, 5));
        let table_bytes = bincode::serialize(&table).unwrap();
        let copied: PrefixTable = bincode::deserialize(&table_bytes).unwrap();
        let sparse = PrefixTable::to_sparse(copied);
        assert_eq!(table, sparse);
    }

    #[test]
    fn test_serialize_large_k() {
        let mut table = PrefixTable::new_sparse(20);
        table.insert(String::from("AAAAAAAAAAAAAAAAAAAA"), (0, 3));
        table.insert(String::from("TTTTTTTTTTTTTTTTTTTT"), (3, 5));
        let table_bytes = bincode::serialize(&table).unwrap();
        let copied: PrefixTable = bincode::deserialize(&table_bytes).unwrap();
        assert_eq!(copied, table);
    }
}
