if [[ "$TRAVIS_BRANCH" == "master" ]]; then
	zip -r0 latest target/release/rustodon target/release/rustodonctl static/ migrations/ scripts/ appspec.yml
	mkdir -p buildartifacts; mv latest.zip buildartifacts/latest.zip
fi

# Check in the results of the Gulp compilation
git config --global user.email "travis@travis-ci.org"
git config --global user.name "Travis CI"
git checkout -b travis-ran-gulp
git add static
git commit --message "Travis build: $TRAVIS_BUILD_NUMBER"
git remote add origin https://${GH_TOKEN}@github.com/rustodon/rustodon.git > /dev/null 2>&1
git push --quiet --set-upstream origin travis-ran-gulp
