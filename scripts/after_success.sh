if [[ "$TRAVIS_BRANCH" == "master" ]]; then
	cargo build --release; else cargo build
	zip -r0 latest target/release/rustodon static/ migrations/ scripts/ appspec.yml
	mkdir -p buildartifacts; mv latest.zip buildartifacts/latest.zip
fi
