#!/bin/sh

CMD="flamegraph -o my_flamegraph.svg -- ./target/release/HGEexample"
konsole --noclose -e $CMD
firefox --new-tab ./my_flamegraph.svg
exit
