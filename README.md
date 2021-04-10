# Minesweeper solver

Another take at Minesweeper implemented with Rust.
See [previous iteration](https://github.com/PetrGlad/robo-minesweeper) (implemented in Haskell). This implementation
does not enumerate consistent layouts - only estimated probabilities are used. This should improve performance by
removing combinatorial algorithms compared to `robo-minesweeper`.

# To do

* API between game and algorithm (supporting batch updates).
* Maybe multi-threaded or GPU version? :)

# Done

* Convert storage to arrays, sparse implementation is not strictly needed as we can use mutation now. For optimal
  presentation would need both sparse and regular presentations. Sparse representation is more complex (e.g. hash or 2d
  tree), so putting it aside until really needed.
* Consider rust-ndarray/ndarray to process field.
