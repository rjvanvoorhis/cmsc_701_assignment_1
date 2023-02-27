use assignment_1::{
    args::{QueryMode, QuerysaArgs},
    reader::Reader,
    record::Record,
    search::Span,
    suffix_array::SuffixArray,
};
use clap::Parser;
use eyre::{Result, WrapErr};
use std::{
    fmt::Write as FmtWrite,
    fs::File,
    io::{BufReader, BufWriter, Write},
    time::{Duration, Instant},
};

fn format_output_line(suffix_array: &SuffixArray, record: &Record, result: Option<Span>) -> String {
    let mut line: String = record.header().to_string();
    match result {
        None => write!(&mut line, ", 0").unwrap(),
        Some((start, end)) => {
            write!(&mut line, ", {}", end - start).unwrap();
            suffix_array.suffix_array[start..end]
                .iter()
                .for_each(|&idx| write!(&mut line, ", {idx}").unwrap());
        }
    }
    line
}

pub fn main() -> Result<()> {
    let args = QuerysaArgs::parse();
    let buf_reader = BufReader::new(
        File::open(&args.index).wrap_err(format!("Could not open index file {:?}", &args.index))?,
    );
    let suffix_array: SuffixArray =
        bincode::deserialize_from(buf_reader).wrap_err("Failed to deserialize suffix array")?;
    let reader: Reader = Reader::from_file(&args.queries)
        .wrap_err(format!("Could not find query file {:?}", &args.queries))?;
    let mut total: Duration = Duration::default();
    let mut record_count = 0_usize;
    let mut writer = match args.output {
        Some(filepath) => {
            let writer: BufWriter<File> = BufWriter::new(
                File::create(&filepath)
                    .wrap_err(format!("Could not create output file {:?}", &filepath))?,
            );
            Some(writer)
        }
        None => None,
    };
    for result in reader {
        let record: Record = result?;
        let now: Instant = Instant::now();
        let res: Option<Span> = match args.query_mode {
            QueryMode::Naive => suffix_array.naive_search(record.sequence()),
            QueryMode::Simpaccel => suffix_array.simple_accelerant_search(record.sequence()),
        };
        let delta: Duration = Instant::now() - now;
        total += delta;
        if let Some(ref mut writer) = writer {
            writeln!(
                writer,
                "{}",
                format_output_line(&suffix_array, &record, res)
            )?;
        }
        record_count += 1;
    }
    if let Some(mut writer) = writer {
        writer.flush()?;
    }
    println!("Took {total:?} to find matches in {record_count} queries");
    Ok(())
}
