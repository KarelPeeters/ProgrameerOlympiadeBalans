set -ue

cargo build --release
rm -f out.txt && time ./target/release/rust ../data/opgave.invoer | tee out.txt && diff out.txt ../data/opgave.uitvoer
rm -f out.txt && time ./target/release/rust ../data/voorbeeld.invoer | tee out.txt && diff out.txt ../data/voorbeeld.uitvoer
rm -f out.txt && time ./target/release/rust ../data/wedstrijd.invoer | tee out.txt && diff out.txt ../data/wedstrijd.uitvoer
