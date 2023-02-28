use std::cmp::{min, Ordering};

#[derive(Debug, Clone, Copy)]
pub struct Comparison {
    pub lcp: usize,
    pub ordering: Ordering,
}

#[derive(Debug, Clone, Copy)]
pub struct Bound {
    pub index: usize,
    pub comparison: Comparison,
}

pub type Span = (u32, u32);

pub enum QueryMode {
    Naive,
    Simpaccel,
}

/// Compares the bytes of the reference sequence up to the end of the target prefix
///
/// This function will be used by both the simple_accelerant and naive searches.
/// While the naive search does not need to read the value of lcp using the same comparison
/// function should lessen the impact of my inexpert rust coding when comparing the two
/// implementations
///
/// ```rust
/// # use assignment_1::search::compare_bytes;
/// # use std::cmp::Ordering;
/// let result = compare_bytes("CTGGAAC".as_bytes(), "CTGA".as_bytes(), 0);
/// /// The sequence CTGGAAC is lexographically greater than the prefix CTGA
/// assert_eq!(result.ordering, Ordering::Greater);
/// /// The sequence and the prefix share the first 3 bytes
/// assert_eq!(result.lcp, 3);
/// ```
pub fn compare_bytes(sequence_bytes: &[u8], prefix_bytes: &[u8], offset: usize) -> Comparison {
    for (idx, byte) in prefix_bytes[offset..].iter().enumerate() {
        let ordering = sequence_bytes[idx + offset].cmp(byte);
        if ordering != Ordering::Equal {
            return Comparison {
                ordering,
                lcp: idx + offset,
            };
        }
    }
    Comparison {
        lcp: prefix_bytes.len(),
        ordering: Ordering::Equal,
    }
}

pub fn naive_bisect_by<F>(
    sequence_bytes: &[u8],
    prefix_bytes: &[u8],
    suffix_array: &[u32],
    span: &Span,
    mut f: F,
    // ) -> usize
) -> u32
where
    F: FnMut(&Ordering) -> bool,
{
    let (mut left, mut right) = span;
    while left < right {
        //let center: usize = (left + right) / 2;
        let center: u32 = (left + right) / 2;
        let comparison: Comparison = compare_bytes(
            //&sequence_bytes[suffix_array[center] as usize..],
            &sequence_bytes[suffix_array[center as usize] as usize..],
            prefix_bytes,
            0,
        );
        if f(&comparison.ordering) {
            left = center + 1;
        } else {
            right = center;
        }
    }
    left
}

pub fn simple_accelerant_bisect_by<F>(
    sequence_bytes: &[u8],
    prefix_bytes: &[u8],
    suffix_array: &[u32],
    left: &mut Bound,
    right: &mut Bound,
    mut f: F,
) where
    F: FnMut(&Ordering) -> bool,
{
    while left.index < right.index {
        let center = (left.index + right.index) / 2;
        let min_lcp = min(left.comparison.lcp, right.comparison.lcp);
        let comparison = compare_bytes(
            &sequence_bytes[suffix_array[center] as usize..],
            prefix_bytes,
            min_lcp,
        );
        if f(&comparison.ordering) {
            left.index = center + 1;
            left.comparison = comparison;
        } else {
            right.index = center;
            right.comparison = comparison;
        }
    }
}

pub fn simple_accelerant_search(
    sequence_bytes: &[u8],
    prefix_bytes: &[u8],
    suffix_array: &[u32],
    span: &Span,
) -> Option<Span> {
    let mut left_bound = Bound {
        index: span.0 as usize,
        // index: span.0,
        comparison: compare_bytes(
            &sequence_bytes[suffix_array[span.0 as usize] as usize..],
            // &sequence_bytes[suffix_array[span.0] as usize..],
            prefix_bytes,
            0,
        ),
    };
    if left_bound.comparison.ordering == Ordering::Greater {
        return None;
    }
    let mut right_bound = Bound {
        index: span.1 as usize,
        // index: span.1,
        comparison: compare_bytes(
            // &sequence_bytes[suffix_array[span.1 - 1] as usize..],
            &sequence_bytes[suffix_array[span.1 as usize - 1] as usize..],
            prefix_bytes,
            0,
        ),
    };
    let right_bound_copy = right_bound;
    simple_accelerant_bisect_by(
        sequence_bytes,
        prefix_bytes,
        suffix_array,
        &mut left_bound,
        &mut right_bound,
        |&x| x == Ordering::Less,
    );
    if left_bound.comparison.ordering != Ordering::Equal
        && right_bound.comparison.ordering != Ordering::Equal
    {
        return None;
    }
    let left = left_bound.index as u32;
    // let left = left_bound.index;
    right_bound = right_bound_copy;
    simple_accelerant_bisect_by(
        sequence_bytes,
        prefix_bytes,
        suffix_array,
        &mut left_bound,
        &mut right_bound,
        |&x| x != Ordering::Greater,
    );
    Some((left, left_bound.index as u32))
    // Some((left, left_bound.index))
}

pub fn naive_search(
    sequence_bytes: &[u8],
    prefix_bytes: &[u8],
    suffix_array: &[u32],
    span: &Span,
) -> Option<Span> {
    if compare_bytes(
        // &sequence_bytes[suffix_array[span.0] as usize..],
        &sequence_bytes[suffix_array[span.0 as usize] as usize..],
        prefix_bytes,
        0,
    )
    .ordering
        == Ordering::Greater
    {
        return None;
    }
    if compare_bytes(
        // &sequence_bytes[suffix_array[span.1 - 1] as usize..],
        &sequence_bytes[suffix_array[span.1 as usize - 1] as usize..],
        prefix_bytes,
        0,
    )
    .ordering
        == Ordering::Less
    {
        return None;
    }
    let left = naive_bisect_by(sequence_bytes, prefix_bytes, suffix_array, span, |&x| {
        x == Ordering::Less
    });
    if compare_bytes(
        // &sequence_bytes[suffix_array[left] as usize..],
        &sequence_bytes[suffix_array[left as usize] as usize..],
        prefix_bytes,
        0,
    )
    .ordering
        != Ordering::Equal
    {
        return None;
    }
    let right = naive_bisect_by(
        sequence_bytes,
        prefix_bytes,
        suffix_array,
        // &(left, span.1),
        &(left, span.1),
        |&x| x != Ordering::Greater,
    );
    Some((left, right))
    // Some((left as u32, right as u32))
}

#[cfg(test)]
mod search_tests {
    use super::*;
    use rand::{distributions::Slice, rngs::StdRng, Rng, SeedableRng};
    use suffix::SuffixTable;

    fn generate_query(size: u32) -> String {
        let nucleotides = ['A', 'C', 'T', 'G'];
        let nucleotide_distribution = Slice::new(&nucleotides).unwrap();
        let rng = StdRng::seed_from_u64(42);
        rng.sample_iter(&nucleotide_distribution)
            .take(size as usize)
            .collect()
    }

    fn generate_sequence(size: u32) -> String {
        let rng = StdRng::seed_from_u64(42);
        let nucleotides = ['A', 'C', 'T', 'G'];
        let nucleotide_distribution = Slice::new(&nucleotides).unwrap();
        rng.sample_iter(&nucleotide_distribution)
            .take(size as usize)
            .chain(['$'].iter())
            .collect()
    }

    fn very_naive_search(
        sequence_bytes: &[u8],
        prefix_bytes: &[u8],
        suffix_array: &[u32],
        span: &Span,
    ) -> Option<Span> {
        let (mut start, end) = *span;
        let mut found: bool = false;
        for idx in start..end {
            // let elem = suffix_array[idx];
            let elem = suffix_array[idx as usize];
            match (
                sequence_bytes[elem as usize..].starts_with(prefix_bytes),
                found,
            ) {
                (true, false) => {
                    start = idx;
                    found = true;
                }
                (false, true) => return Some((start, idx)),
                _ => {}
            }
        }
        if found {
            Some((start, end))
        } else {
            None
        }
    }

    fn get_suffix_array(sequence: &str) -> Vec<u32> {
        SuffixTable::new(sequence).table().to_owned()
    }

    #[test]
    fn test_compare_bytes_equal_up_to_prefix() {
        let sequence = "ATTGCTGGA$";
        let prefix: &str = "ATT";
        let result = compare_bytes(sequence.as_bytes(), prefix.as_bytes(), 0);
        assert!(result.ordering == Ordering::Equal);
        assert!(result.lcp == 3);
    }

    #[test]
    fn test_compare_bytes_no_common_prefix() {
        let sequence = "ATTGCTGGA$";
        let prefix: &str = "CGG";
        let result = compare_bytes(sequence.as_bytes(), prefix.as_bytes(), 0);
        assert!(result.ordering == Ordering::Less);
        assert!(result.lcp == 0);
    }

    #[test]
    fn test_compare_bytes_greater_than() {
        let sequence = "CTTGCTGGA$";
        let prefix: &str = "CTG";
        let result = compare_bytes(sequence.as_bytes(), prefix.as_bytes(), 0);
        assert!(result.ordering == Ordering::Greater);
        assert!(result.lcp == 2);
    }

    #[test]
    fn test_naive_search_no_match() {
        let prefix_bytes = "CAT".as_bytes();
        let sequence = "ACCTAT$".to_string();
        let suffix_array = get_suffix_array(&sequence);
        let result = naive_search(
            sequence.as_bytes(),
            prefix_bytes,
            &suffix_array,
            // &(0, suffix_array.len()),
            &(0, suffix_array.len() as u32),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn test_simpaccel_search_no_match() {
        let prefix_bytes = "CAT".as_bytes();
        let sequence = "ACCTAT$".to_string();
        let suffix_array = get_suffix_array(&sequence);
        let result = simple_accelerant_search(
            sequence.as_bytes(),
            prefix_bytes,
            &suffix_array,
            // &(0, suffix_array.len()),
            &(0, suffix_array.len() as u32),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn test_simpaccel_multi_match() {
        let prefix_bytes = "CAT".as_bytes();
        let sequence = "ACCATATCCCATTGAAGADC$".to_string();
        let suffix_array = get_suffix_array(&sequence);
        let result = simple_accelerant_search(
            sequence.as_bytes(),
            prefix_bytes,
            &suffix_array,
            // &(0, suffix_array.len()),
            &(0, suffix_array.len() as u32),
        );
        assert_eq!(result.unwrap(), (9, 11));
    }

    #[test]
    fn search_results_match() {
        let mut rng = StdRng::seed_from_u64(42);
        let sequence = generate_sequence(1000000);
        let sa = get_suffix_array(&sequence);
        for _ in 0..100 {
            let prefix = generate_query(rng.gen_range(1..10) as u32);
            // let span = (0_usize, sa.len());
            let span = (0_u32, sa.len() as u32);
            let baseline = very_naive_search(sequence.as_bytes(), prefix.as_bytes(), &sa, &span);
            let naive_result = naive_search(sequence.as_bytes(), prefix.as_bytes(), &sa, &span);
            let simpaccel_result =
                simple_accelerant_search(sequence.as_bytes(), prefix.as_bytes(), &sa, &span);
            if let Some(result) = baseline {
                assert_eq!(result, naive_result.unwrap());
                assert_eq!(result, simpaccel_result.unwrap());
            } else {
                assert_eq!(naive_result, None);
                assert_eq!(simpaccel_result, None);
            }
        }
    }
}
