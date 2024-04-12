set -ue
set -o pipefail

rm -f out.txt && time cargo run --release -- ../data/opgave.invoer | tee out.txt
diff out.txt ../data/opgave.uitvoer
rm -f out.txt && time cargo run --release -- ../data/voorbeeld.invoer | tee out.txt
diff out.txt ../data/voorbeeld.uitvoer
rm -f out.txt && time cargo run --release -- ../data/wedstrijd.invoer | tee out.txt
diff out.txt ../data/wedstrijd.uitvoer
