use std::path::PathBuf;

use clap::Parser;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum QueryMode {
    /// bisect left and right with redundant comparisons
    Naive,
    /// bisect left and right, skipping min lcp comparisons
    Simpaccel,
}

#[derive(Debug, Parser)]
/// Builds the suffix array for a given reference files
/// and saves the result to disk
pub struct BuildsaArgs {
    #[arg(short, long, value_name="k", value_parser = clap::value_parser!(u16).range(1..100))]
    /// Build a prefix table of size <k> for this reference sequence
    pub preftab: Option<u16>,

    /// The path to a FASTA file containing the reference sequence
    pub reference: PathBuf,
    /// The path to the file the suffix array will be saved to
    pub output: PathBuf,
}

#[derive(Debug, Parser)]
/// Find occurences of query strings in a reference sequence using the saved suffix array from buildsa
pub struct QuerysaArgs {
    /// The path to the binary file generated in buildsa
    pub index: PathBuf,
    /// The path to a FASTA file containing the queries to run
    pub queries: PathBuf,

    #[arg(value_enum)]
    pub query_mode: QueryMode,

    #[arg(required_unless_present = "quiet")]
    /// The path to the file the results are written to (not required if quiet flag is set)
    pub output: Option<PathBuf>,

    #[arg(short, long)]
    /// run queries without writing the results to the output file
    pub quiet: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum SampleStrategy {
    ExactMatch,
    /// Generate queries as random substrings of the reference sequence but randomly modify 5% of the characters
    Perturb,
}

#[derive(Debug, Parser)]
/// build queries to be used in quersa program
pub struct BuildQueryArgs {
    /// The path to a FASTA file containing the reference sequence
    pub reference: PathBuf,

    /// The path to a FASTA file where the queries will be written to
    pub output: PathBuf,

    #[arg(value_enum)]
    /// Method to generate the queries
    pub strategy: SampleStrategy,

    /// The minimum length query to generate (defaults to 5)
    #[arg(long, value_parser = clap::value_parser!(u16).range(3..1000), default_value="5")]
    pub min_length: u16,

    #[arg(long, value_parser = clap::value_parser!(u16).range(3..1000), default_value="30")]
    /// The maximum length query to generate (default to 30)
    pub max_length: u16,

    /// The number of queries to generate (defaults to 100)
    #[arg(short, long, default_value = "100")]
    pub queries: usize,
}
