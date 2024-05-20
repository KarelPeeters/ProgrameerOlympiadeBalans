set -ue

rm -f out.txt
g++ ../notes/karel9.cpp -o ../notes/karel9 -O3
time cat ../data/maarten5.invoer | ../notes/karel9 > out.txt
diff out.txt ../data/maarten5.uitvoer
echo "All correct"
