#!/bin/bash

rm stderr.txt
rm pgnout.txt
rm nohup.out

cutechess-cli \
-engine name=dev cmd=./Titan \
-engine name=main cmd=./Titan \
-games 2 -rounds 100 \
-pgnout "pgnout.txt" \
-each proto=uci tc=inf stderr=stderr.txt depth=18 \
-concurrency 1 \
-ratinginterval 10 \
# -debug \
# -sprt elo0=0.0 elo1=3.0 alpha=0.05 beta=0.05 \
