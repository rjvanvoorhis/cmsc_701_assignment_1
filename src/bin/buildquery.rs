use std::{
    fmt::{Write as FmtWrite},
    fs::{File},
    io::{Write, BufWriter},
    iter::zip,
};

use assignment_1::{args::{BuildQueryArgs, SampleStrategy}, reader::Reader};
use clap::Parser;
use eyre::{ContextCompat, Result, WrapErr};
use rand::{
    distributions::{Uniform},
    prelude::Distribution,
    thread_rng, Rng, seq::IteratorRandom,
};


fn generate_exact_match_sequences(reference: &str, min_size: usize, max_size: usize, queries: usize) -> Vec<String> {
    let mut rng = thread_rng();
    let starts: Vec<usize> = Uniform::new(0, reference.len() - max_size).sample_iter(&mut rng).take(queries).collect();
    let offsets: Vec<usize> = Uniform::new_inclusive(min_size, max_size).sample_iter(&mut rng).take(queries).collect();
    zip(starts, offsets).map(|(start, offset)| reference[start..start+offset].to_string()).collect()
}


fn generate_perturbed_sequences(reference: &str, min_size: usize, max_size: usize, queries: usize) -> Vec<String> {
    let mut rng = thread_rng();
    let starts: Vec<usize> = Uniform::new(0, reference.len() - max_size).sample_iter(&mut rng).take(queries).collect();
    let offsets: Vec<usize> = Uniform::new_inclusive(min_size, max_size).sample_iter(&mut rng).take(queries).collect();
    zip(starts, offsets).map(|(start, offset)| {
        let mut buffer = String::new();
        reference[start..start+offset].chars().for_each(|x| {
            let num = rng.gen_range(0..100);
            let next_char = if num <= 5 {
                x
            } else {
                "ACTG".chars().choose(&mut rng).unwrap()
            };
            write!(&mut buffer, "{next_char}").unwrap();
        });
        buffer
    }).collect()
}


pub fn main() -> Result<()> {
    let args = BuildQueryArgs::parse();
    let mut reader = Reader::from_file(&args.reference).wrap_err(format!(
        "Could not open reference file {:?}",
        &args.reference
    ))?;
    let record = reader
        .next()
        .wrap_err("The reference file was empty")
        .unwrap()
        .wrap_err("Could not parse reference file")?;
    let queries = match args.strategy {
        SampleStrategy::ExactMatch => generate_exact_match_sequences(record.sequence(), args.min_length as usize, args.max_length as usize, args.queries),
        SampleStrategy::Perturb => generate_perturbed_sequences(record.sequence(), args.min_length as usize, args.max_length as usize, args.queries)
    };
    let mut writer: BufWriter<File> = BufWriter::new(File::create(&args.output)?);
    for (idx, query) in queries.iter().enumerate(){
        write!(&mut writer, ">query-{idx}\n{query}\n")?;
    }
    writer.flush()?;
    Ok(())
}
