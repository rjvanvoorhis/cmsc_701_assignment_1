import pathlib
import typing

def chunk(lst: typing.Sized, chunk_size: int=80) -> typing.Iterable[str]:
    total = len(lst)
    start, end = (0, min(chunk_size, total))
    while end <= total:
        yield lst[start:end]
        start = end
        end += chunk_size

def clean_sequence(path: pathlib.Path) -> str:
    head, *tail = path.read_text().split("\n")
    chunk_size = len(tail[0])
    tail = "\n".join(chunk("".join(tail).replace("N", ""), chunk_size=chunk_size))
    return "\n".join((head, tail))


if __name__ == "__main__":
    import sys
    if len(sys.argv) < 2:
        raise ValueError("A reference file is required")
    infile, outfile = map(pathlib.Path, (*sys.argv[1:], sys.argv[1])[0:2])
    cleaned = clean_sequence(infile)
    outfile.write_text(cleaned)
