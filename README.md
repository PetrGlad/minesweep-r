# Minesweeper solver

Another take at Minesweeper implemented with Rust.
See [previous iteration](https://github.com/PetrGlad/robo-minesweeper) (implemented in Haskell).

Status: just started. Nothing to see here yet.

# TODO

* Convert storage to arrays, sparse implementation is not strictly needed as we can use mutation now. For optimal presentation would need both sparse and regular presentations. Sparse representation is more complex (e.g. hash or 2d tree), so putting it aside until really needed.
* Consider rust-ndarray/ndarray to process field.