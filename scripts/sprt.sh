#!/bin/bash

# rm stderr.txt
# rm pgnout.txt
# rm nohup.out

/home/jeff/ob/Client/cutechess-ob \
-engine name=dev cmd=./titan \
-engine name=main cmd=./titan \
-games 2 -rounds 250000 \
-pgnout "pgnout.txt" \
-sprt elo0=0.0 elo1=3.0 alpha=0.05 beta=0.05 \
-each proto=uci tc=8+0.08 stderr=stderr.txt \
-openings order=random file="/home/jeff/ob/Books/Pohl.epd" format=epd \
-concurrency 6 \
-ratinginterval 10 \
# -debug \
