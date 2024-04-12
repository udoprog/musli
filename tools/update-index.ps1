git checkout gh-pages benchmarks
git reset
cargo run -p tools --release -- report --index-output index.md
git checkout gh-pages
cp index.md benchmarks/index.md
rm index.md
git add benchmarks/index.md
git commit -m "Update index"
git push
git checkout main
