#!/bin/bash

rm stderr.txt
rm pgnout.txt
rm nohup.out

nohup cutechess-cli \
-engine name=dev cmd=./target/release/chess-engine stderr=stderr.txt \
-engine name=main cmd=./main \
-games 2 -rounds 50000 \
-pgnout "pgnout.txt" \
-sprt elo0=-3.0 elo1=0.0 alpha=0.005 beta=0.005 \
-each proto=uci tc=8+0.08 \
-openings order=random file="book.pgn" format=pgn \
-concurrency 6 \
-ratinginterval 10 \
