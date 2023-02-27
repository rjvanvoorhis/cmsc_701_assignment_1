use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use suffix::SuffixTable;

use crate::{
    prefix_table::PrefixTable,
    record::Record,
    search::{naive_search, simple_accelerant_search, Span},
};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct SuffixArray {
    pub sequence: String,
    pub suffix_array: Vec<u32>,
    prefix_table: Option<PrefixTable>,
}

#[derive(Debug)]
pub struct Comparison {
    pub lcp: usize,
    pub ordering: Ordering,
}

pub enum QueryMode {
    Naive,
    Simpaccel,
}

impl SuffixArray {
    fn build_prefix_table(&self, k: u16) -> PrefixTable {
        let mut last_prefix: Option<&str> = None;
        let mut start: usize = 0;
        let mut prefix_table: PrefixTable = PrefixTable::new(k);
        let max_elem = self.suffix_array.len() - k as usize;
        for (idx, ele) in self
            .suffix_array
            .iter()
            .map(|&x| x as usize)
            .filter(|&x| x < max_elem)
            .enumerate()
        {
            let prefix = &self.sequence[ele..ele + k as usize];
            if let Some(previous) = last_prefix {
                if prefix != previous {
                    prefix_table.insert(previous.to_string(), (start, idx));
                    last_prefix = Some(prefix);
                    start = idx;
                }
            } else {
                last_prefix = Some(prefix);
            }
        }
        prefix_table
    }

    pub fn initialize_prefix_table(&mut self, k: u16) {
        match &self.prefix_table {
            Some(table) => {
                if table.k() != k {
                    self.prefix_table = Some(self.build_prefix_table(k))
                }
            }
            _ => self.prefix_table = Some(self.build_prefix_table(k)),
        }
    }

    pub fn from_record(record: Record) -> Self {
        let Record { mut sequence, .. } = record;
        if !sequence.ends_with('$') {
            sequence.push('$');
        }
        let (text, table) = SuffixTable::new(sequence).into_parts();
        Self {
            suffix_array: table.into_owned(),
            sequence: text.into_owned(),
            prefix_table: None,
        }
    }

    fn get_start_span(&self, prefix: &str) -> Option<(usize, usize)> {
        if let Some(table) = &self.prefix_table {
            let k = table.k() as usize;
            if prefix.len() < table.k() as usize {
                return Some((0, self.suffix_array.len()));
            }
            return table.get(&prefix[..k]);
        }
        Some((0, self.suffix_array.len()))
    }

    fn prefix_out_of_bounds(
        &self,
        sequence_bytes: &[u8],
        prefix_bytes: &[u8],
        span: &Span,
    ) -> bool {
        let view = &self.suffix_array[span.0..span.1];
        view.is_empty()
            || prefix_bytes < &sequence_bytes[*view.first().unwrap() as usize..]
            || prefix_bytes > &sequence_bytes[*view.last().unwrap() as usize..]
    }

    pub fn naive_search(&self, prefix: &str) -> Option<Span> {
        let span;
        if let Some(start_span) = self.get_start_span(prefix) {
            span = start_span;
        } else {
            return None;
        }
        let sequence_bytes = self.sequence.as_bytes();
        let prefix_bytes = prefix.as_bytes();
        if self.prefix_out_of_bounds(sequence_bytes, prefix_bytes, &span) {
            return None;
        }
        // if self.suffix_array[span.0..span.1].is_empty() {
        //     return None;
        // }
        naive_search(sequence_bytes, prefix_bytes, &self.suffix_array, &span)
    }

    pub fn simple_accelerant_search(&self, prefix: &str) -> Option<Span> {
        let span;
        if let Some(start_span) = self.get_start_span(prefix) {
            span = start_span;
        } else {
            return None;
        }
        let sequence_bytes = self.sequence.as_bytes();
        let prefix_bytes = prefix.as_bytes();
        if self.suffix_array[span.0..span.1].is_empty() {
            return None;
        }
        simple_accelerant_search(sequence_bytes, prefix_bytes, &self.suffix_array, &span)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn get_suffix_array(sequence: &str) -> SuffixArray {
        let record: Record = Record {
            sequence: sequence.to_string(),
            header: String::from("test"),
        };
        return SuffixArray::from_record(record);
    }

    #[test]
    fn naive_and_accelerated_search_produce_the_same_result() {
        let sa: SuffixArray = get_suffix_array("AGGTGGCAATGCGCGCTCATCGCCTTGCAT$");
        let prefix: &str = "GCA";
        let (naive_start, naive_end) = sa.naive_search(&prefix).unwrap();
        assert_eq!(naive_end - naive_start, 2);
        sa.suffix_array[naive_start..naive_end]
            .iter()
            .for_each(|idx| {
                assert!(
                    String::from_utf8((&sa.sequence.as_bytes()[*idx as usize..]).to_vec())
                        .unwrap()
                        .starts_with(prefix)
                );
            });
        let (accelerated_start, accelerated_end) = sa.simple_accelerant_search(&prefix).unwrap();
        assert_eq!(naive_start, accelerated_start);
        assert_eq!(accelerated_end, naive_end);
    }
}
