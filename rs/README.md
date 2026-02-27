# odenwald

Rust implementation of the Odenwald SCFG (synchronous context-free grammar) machine translation decoder.

## Build

```
cargo build --release
```

## Usage

```
odenwald -g <grammar> -w <weights> [-i <input>] [-l] [-p]
```

- `-g, --grammar` — grammar file (required)
- `-w, --weights` — weights file (required)
- `-i, --input` — input file (default: stdin)
- `-l, --add-glue` — add glue rules
- `-p, --add-pass-through` — add pass-through rules

Output: `translation ||| log_score` per input line.

## Examples

```
cargo run -- -g ../example/toy/grammar -w ../example/toy/weights.toy -i ../example/toy/in -l
# → i saw a small shell ||| -0.5

cargo run -- -g ../example/toy-reorder/grammar -w ../example/toy-reorder/weights.toy -i ../example/toy-reorder/in -l
# → he reads the book ||| -1.5
```

## Tests

```
cargo test
```
