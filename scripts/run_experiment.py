import sys
import json
import subprocess
import pathlib
import dataclasses
import re

BUILDSA_REGEX = re.compile(
    r"the suffix array took (\d+\.\d+)(.*?)NEWLINE.*file has size: (\d+)"
)
QUERYSA_REGEX = re.compile(r"Took (\d+\.\d+)(.*?) to find matches in (\d+)")
QUERY_FILE_REGEX = re.compile(r"(.*?)-(\d+)-queries-size-(\d+)-to-(\d+)")


@dataclasses.dataclass
class QueryOptions:
    strategy: str = "exact-match"
    queries: int = 1000000
    min_length: int = 5
    max_length: int = 8


RELEASE = pathlib.Path(__file__).parent.parent.absolute() / "target" / "release"
RUNS = pathlib.Path(__file__).parent.absolute() / "runs"
SAVES = pathlib.Path(__file__).parent.parent.absolute() / "saves"
QUERIES = pathlib.Path(__file__).parent.parent.absolute() / "queries"
REFERENCE = pathlib.Path(__file__).parent.parent.absolute() / "references"
BUILD_SA = RELEASE / "buildsa"
BUILD_QUERIES = RELEASE / "buildquery"
QUERY_SA = RELEASE / "querysa"


def run_command(args, verbose=True):
    result = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if result.returncode:
        print(result.stderr.decode("utf-8"))
        sys.exit(result.returncode)
    output = (result.stdout or b"").decode("utf-8")
    if verbose and output:
        print(output)
    return output


def build_sa(reference_file, k=0):
    outfile = str(SAVES / "out.bin")
    args = [BUILD_SA, str(reference_file), outfile]
    if k:
        args.insert(1, f"--preftab={k}")

    return outfile, run_command(args)


def build_query(reference_file, options=QueryOptions()):
    outfile = str(
        QUERIES
        / f"{options.strategy}-{options.queries}-queries-size-{options.min_length}-to-{options.max_length}.fasta"
    )
    args = [
        BUILD_QUERIES,
        str(reference_file),
        outfile,
        options.strategy,
        f"--queries={options.queries}",
        f"--min-length={options.min_length}",
        f"--max-length={options.max_length}",
    ]
    return outfile, run_command(args)


def run_queries(save_file, query_file, query_mode):
    args = [QUERY_SA, str(save_file), str(query_file), query_mode, "--quiet"]
    return None, run_command(args)


def run_sub_experiment(reference_file, k=0, runs=10, verbose=True):
    save_file, build_sa_results = build_sa(reference_file, k)
    query_files = []
    for options in [
        QueryOptions(
            strategy="exact-match", min_length=3, max_length=3, queries=100000
        ),
        QueryOptions(
            strategy="exact-match", min_length=5, max_length=5, queries=100000
        ),
        QueryOptions(
            strategy="exact-match", min_length=8, max_length=8, queries=100000
        ),
        QueryOptions(
            strategy="exact-match", min_length=12, max_length=12, queries=100000
        ),
        QueryOptions(
            strategy="exact-match", min_length=20, max_length=20, queries=100000
        ),
        QueryOptions(
            strategy="exact-match", min_length=30, max_length=30, queries=100000
        ),
        QueryOptions(
            strategy="exact-match", min_length=3, max_length=30, queries=100000
        ),
        QueryOptions(
            strategy="exact-match", min_length=3, max_length=30, queries=100000
        ),
        QueryOptions(strategy="perturb", min_length=5, max_length=30, queries=100000),
    ]:
        if k and k < options.max_length:
            continue
        query_file, _ = build_query(reference_file, options)
        query_files.append(query_file)
    query_results = {}
    for query_mode in ["naive", "simpaccel"]:
        query_results[query_mode] = []
        if verbose:
            print(f"Using query_mode {query_mode}")
        for query_file in query_files:
            for _ in range(runs):
                _, result = run_queries(save_file, query_file, query_mode)
                query_results[query_mode].append(
                    {"query_file": query_file, "output": result}
                )
    return {"buildsa": build_sa_results, "querysa": query_results, "preftab": k}


def run_experiment(reference_file):
    reference_file = pathlib.Path(reference_file)
    size = reference_file.stat().st_size
    results = {"reference_name": reference_file.name, "file_size": size, "results": []}
    reference_file = str(pathlib.Path(reference_file).absolute())
    preftable_sizes = [0, *range(3, 10)]
    for k in preftable_sizes:
        results["results"].append(run_sub_experiment(reference_file, k))
    return results


def update_buildsa(item):
    buildsa = item["buildsa"]
    buildsa = buildsa.replace("\n", "NEWLINE")
    sa_time, sa_unit, size = BUILDSA_REGEX.search(buildsa).groups()
    item["buildsa"] = {
        "preftab": item["preftab"],
        "suffix_array_construction_time": float(sa_time),
        "suffix_array_construction_time_unit": sa_unit,
        "file_size": int(size),
        "output": item["buildsa"],
    }


def format_results(results):
    for item in results["results"]:
        update_buildsa(item)
        for query_mode in ["naive", "simpaccel"]:
            for query_item in item["querysa"][query_mode]:
                update_querysa(query_item)


def update_querysa(item):
    query_time, query_units, queries = QUERYSA_REGEX.search(item["output"]).groups()
    query_file = pathlib.Path(item["query_file"]).name
    query_strategy, _, min_size, max_size = QUERY_FILE_REGEX.search(query_file).groups()
    item.update(
        {
            "query_time": float(query_time),
            "query_time_units": query_units,
            "queries": int(queries),
            "query_strategy": query_strategy,
            "min_query_length": int(min_size),
            "max_query_length": int(max_size),
        }
    )


def main():
    for reference in ["ecoli.fasta", "mouse_chr_16.fasta", "human_chr_1.fasta"]:
        # for reference in ["mouse_chr_16.fasta", "human_chr_1.fasta"]:
        run_file = RUNS / f"{reference.split('.', 1)[0]}-run-post-refactor.json"
        result = run_experiment(REFERENCE / reference)
        format_results(result)
        run_file.write_text(json.dumps(result, indent=4))


def verify_methods(reference):
    query_file, _ = build_query(reference, QueryOptions(min_length=5, max_length=50))
    next_file = "b.txt"
    current_file = "a.txt"
    prev = None
    for k in [0, *range(3,14)]:
        save_file, _ = build_sa(reference, k)
        for query_mode in ["naive", "simpaccel"]:
            run_command(
                [QUERY_SA, save_file, str(query_file), query_mode, current_file]
            )
            if prev != None:
                print(
                    f"Running diff with output from query_mode={query_mode} and k={k}"
                )
                diff = run_command(["diff", current_file, prev])
                print(f"ERROR! found diff:\n{diff}" if diff else "Texts match...")
                next_file = prev
            prev = current_file
            current_file = next_file


if __name__ == "__main__":
    # verify_methods(REFERENCE / "ecoli.fasta")
    main()
