# Titan

This project is a chess engine that was developed as a learning experience and passion project. It is capable of using the UCI interface to communicate with most chess GUIs, and well past capable of beating me for better or for worse...

The Makefile supports two options. The first entry is capable of being built on stable, and utilizes compiler autovectorization for neural network updates and evaluation. The second option requires both an AVX512 capable cpu and the nightly compiler, as SIMD intrinsics in rust have not stabilized. I expect this will segfault on a non-AVX512 capable cpu.

This project requires giving credit to a few projects. 
- LeelaChessZero for the data required to train the neural network used for evaluation https://github.com/LeelaChessZero/lc0
- Bullet for training the network used for evaulation https://github.com/jw1912/bullet/tree/main
- Primer for filtering and converting the downloaded data from Leela https://github.com/PGG106/Primer
