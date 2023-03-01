The project contains two binaries `buildsa` and `querysa`

To build the executables run the following command

```bash
cargo build --release
```

This will write the executables to `./target/release`. To run each program you can run

```
# Display the usage info for buildsa
./target/release/buildsa --help

# Display the usage info for querysa
./target/release/querysa --help
```

Each program has an associated help text. The following is the help for buildsa
```
Builds the suffix array for a given reference files and saves the result to disk

Usage: buildsa [OPTIONS] <REFERENCE> <OUTPUT>

Arguments:
  <REFERENCE>  The path to a FASTA file containing the reference sequence
  <OUTPUT>     The path to the file the suffix array will be saved to

Options:
  -p, --preftab <k>  Build a prefix table of size <k> for this reference sequence
  -h, --help         Print help
```

And for querysa
```
Find occurences of query strings in a reference sequence using the saved suffix array from buildsa

Usage: querysa [OPTIONS] <INDEX> <QUERIES> <QUERY_MODE> [OUTPUT]

Arguments:
  <INDEX>
          The path to the binary file generated in buildsa

  <QUERIES>
          The path to a FASTA file containing the queries to run

  <QUERY_MODE>
          Possible values:
          - naive:     bisect left and right with redundant comparisons
          - simpaccel: bisect left and right, skipping min lcp comparisons

  [OUTPUT]
          The path to the file the results are written to (not required if quiet flag is set)

Options:
  -q, --quiet
          run queries without writing the results to the output file

  -h, --help
          Print help (see a summary with '-h')
```

