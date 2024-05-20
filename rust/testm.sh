set -ue

rm -f out.txt
cargo build --release
time ./target/release/rust ../data/maarten.invoer | tee out.txt
diff out.txt ../data/maarten.uitvoer
echo "All correct"
