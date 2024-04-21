set -ue

cargo build --release
echo "Testing opgave"
rm -f out.txt && time ./target/release/rust ../data/opgave.invoer | tee out.txt && diff out.txt ../data/opgave.uitvoer
echo "Testing voorbeeld"
rm -f out.txt && time ./target/release/rust ../data/voorbeeld.invoer | tee out.txt && diff out.txt ../data/voorbeeld.uitvoer
echo "Testing wedstrijd"
rm -f out.txt && time ./target/release/rust ../data/wedstrijd.invoer | tee out.txt && diff out.txt ../data/wedstrijd.uitvoer
