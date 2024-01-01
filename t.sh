#!/bin/bash

rm stderr.txt
rm pgnout.txt
rm nohup.out

cutechess-cli \
-engine name=dev cmd=./Titan \
-engine name=main cmd=./Titan \
-games 1 -rounds 1 \
-pgnout "pgnout.txt" \
-sprt elo0=0.0 elo1=3.0 alpha=0.05 beta=0.05 \
-each proto=uci tc=8+0.08 stderr=stderr.txt \
-concurrency 1 \
-ratinginterval 10 \
-debug \
