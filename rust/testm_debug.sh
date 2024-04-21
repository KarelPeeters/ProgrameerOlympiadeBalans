set -ue

rm -f out.txt
cargo build
time ./target/debug/rust ../data/maarten.invoer | tee out.txt
diff out.txt ../data/maarten.uitvoer
