set -ue

rm -f out.txt
cargo build --release
time ./target/release/rust ../data/maarten5.invoer > out.txt
diff out.txt ../data/maarten5.uitvoer
echo "All correct"
