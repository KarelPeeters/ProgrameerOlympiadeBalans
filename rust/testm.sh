set -ue
set -o pipefail

rm -f out.txt && time cargo run --release -- ../data/maarten.invoer | tee out.txt
diff out.txt ../data/maarten.uitvoer