use std::{fs::File, io::BufReader};

use assignment_1::{
    reader::Reader,
    record::Record,
    search::{naive_search, simple_accelerant_search, Span},
    suffix_array::SuffixArray,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

enum QueryMode {
    Naive,
    Simpaccel,
}

fn get_suffix_array(filename: &str) -> SuffixArray {
    bincode::deserialize_from(BufReader::new(
        File::open(filename).expect("file must exist"),
    ))
    .unwrap()
}

fn get_records(filename: &str) -> Vec<Record> {
    let reader = Reader::from_file(filename).expect("reader file must exist");
    reader.into_iter().filter_map(|r| r.ok()).collect()
}

fn raw_search_function_harness(query_mode: QueryMode, sa: &SuffixArray, records: &Vec<Record>) {
    let f = match query_mode {
        QueryMode::Simpaccel => simple_accelerant_search,
        QueryMode::Naive => naive_search,
    };
    let sequence_bytes = sa.sequence.as_bytes();
    let span: Span = (0, sequence_bytes.len());
    records.iter().for_each(|record| {
        f(
            sequence_bytes,
            record.sequence().as_bytes(),
            &sa.suffix_array,
            &span,
        );
    });
}

fn naive_search_harness(sa: &SuffixArray, records: &Vec<Record>) {
    records.iter().for_each(|record: &Record| {
        sa.naive_search(record.sequence());
    })
}

fn simpaccel_harness(sa: &SuffixArray, records: &Vec<Record>) {
    records.iter().for_each(|record: &Record| {
        sa.simple_accelerant_search(record.sequence());
    })
}

fn search_harness(query_mode: QueryMode, sa: &SuffixArray, records: &Vec<Record>) {
    let f = match query_mode {
        QueryMode::Naive => naive_search_harness,
        QueryMode::Simpaccel => simpaccel_harness,
    };
    f(sa, records)
}

fn raw_search_criterion(c: &mut Criterion) {
    let sa = get_suffix_array(
        "./benches/data/ecoli_sa.bin",
    );
    let records: Vec<Record> = get_records(
        "./benches/data/mixed_queries.fasta",
    );

    c.bench_function("raw naive search", |b| {
        b.iter(|| raw_search_function_harness(QueryMode::Naive, &sa, &records))
    });
    c.bench_function("raw simpaccel search", |b| {
        b.iter(|| raw_search_function_harness(QueryMode::Simpaccel, &sa, &records))
    });
}

fn prefix_table_criterion(c: &mut Criterion) {
    let mut sa = get_suffix_array("./benches/data/ecoli_sa.bin");
    let records: Vec<Record> = get_records("./benches/data/mixed_queries.fasta");

    c.bench_function("naive search - no prefix table", |b| {
        b.iter(|| search_harness(black_box(QueryMode::Naive), &sa, &records))
    });
    c.bench_function("simpaccel search - no prefix table", |b| {
        b.iter(|| search_harness(black_box(QueryMode::Simpaccel), &sa, &records))
    });

    for k in [1, 2, 3, 5, 8, 12] {
        sa.initialize_prefix_table(k);
        c.bench_function(format!("naive search k={}", k).as_str(), |b| {
            b.iter(|| search_harness(black_box(QueryMode::Naive), &sa, &records))
        });
        c.bench_function(format!("simpaccel search k={}", k).as_str(), |b| {
            b.iter(|| search_harness(black_box(QueryMode::Simpaccel), &sa, &records))
        });
    }
}

criterion_group!(benches, raw_search_criterion, prefix_table_criterion);
criterion_main!(benches);
