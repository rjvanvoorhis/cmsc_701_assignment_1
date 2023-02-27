use std::{
    fs::{metadata, File},
    io::BufWriter,
    time::Instant,
};

use assignment_1::{args::BuildsaArgs, reader::Reader, suffix_array::SuffixArray};
use clap::Parser;
use eyre::{eyre, Result, WrapErr};

pub fn main() -> Result<()> {
    let args: BuildsaArgs = BuildsaArgs::parse();
    let mut reader = Reader::from_file(&args.reference).wrap_err(format!(
        "The reference file {:?} does not exist",
        &args.reference
    ))?;
    let record = match reader.next() {
        Some(record) => record.wrap_err("could not parse record"),
        None => Err(eyre!(format!(
            "The reference file {:?} was empty",
            &args.reference
        ))),
    }?;
    let mut now: Instant = Instant::now();
    let mut suffix_array = SuffixArray::from_record(record);
    let mut delta = Instant::now() - now;
    println!("Constructing the suffix array took {delta:?}");
    if let Some(k) = args.preftab {
        println!("Building prefix table with k={k}");
        now = Instant::now();
        suffix_array.initialize_prefix_table(k);
        delta = Instant::now() - now;
        println!("Constructing the prefix table took {delta:?}")
    }
    let writer: BufWriter<File> = BufWriter::new(
        File::create(&args.output)
            .wrap_err(format!("Failed to create output file {:?}", &args.output))?,
    );
    bincode::serialize_into(writer, &suffix_array)?;
    let file_size = metadata(&args.output)?.len();
    println!(
        "The resulting file has size: {file_size} bytes or ~ {} MiB",
        file_size / 1024 / 1024
    );
    Ok(())
}
