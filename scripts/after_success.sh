if [[ "$TRAVIS_BRANCH" == "master" ]]; then
	zip -r0 latest target/release/rustodon target/release/rustodonctl static/ migrations/ scripts/ appspec.yml
	mkdir -p buildartifacts; mv latest.zip buildartifacts/latest.zip
fi
