# Minesweeper solver

Another take at Minesweeper implemented with Rust.
See [previous iteration](https://github.com/PetrGlad/robo-minesweeper) (implemented in Haskell). This implementation
does not enumerate consistent layouts - only estimated probabilities are used. This should improve performance by
removing combinatorial algorithms compared to `robo-minesweeper`.


# To do

* API between game and algorithm (supporting batch updates).
* Maybe multi-threaded or GPU version? :)


# Bugs

Sample
```
# # # # # # # # 
# 1 1         # 
# % 1         # 
# % 1   1 1 1 # 
# % 2 1 1 @ 1 # 
# % @ 1 2 2 2 # 
# 2 2 1 1 @ 1 # 
# # # # # # # # 

Failed, uncleared 4. Probe at (5, 1)
```
p at (2,1) and (5,1) should be 1.0

Sample
```
# # # # # # # # 
#   1 2 % # # # 
# 1 2 % % # # # 
# 3 % % # # # # 
# % % # # # # # 
# # # # # # # # 
# # # # # # # # 
# # # # # # # # 

Failed, uncleared 30. Probe at (2, 4)
```

p at (2,3) and (3,2) should be 1.0

Looks like border is not handled properly.

# Done

* Convert storage to arrays, sparse implementation is not strictly needed as we can use mutation now. For optimal
  presentation would need both sparse and regular presentations. Sparse representation is more complex (e.g. hash or 2d
  tree), so putting it aside until really needed.
* Consider rust-ndarray/ndarray to process field.
