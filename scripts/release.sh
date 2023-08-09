#!/bin/bash

set -e

toml_set () {
    out=$(toml set "$1" "$2" "$3")
    echo "$out" > "$1"
}

toml_get () {
    toml get --raw "$1" "$2"
}

current_version=$(toml_get Cargo.toml package.version)

read -p "Current version is $current_version, enter new version: " version
tag="v${version}"

read -p "Creating new release for $tag. Do you want to continue? [Y/n] " prompt

if [[ $prompt == "y" || $prompt == "Y" || $prompt == "yes" || $prompt == "Yes" ]]; then
    echo "Updating Cargo.toml files and CHANGELOG for release..."
else
    echo "Cancelled"
    exit 1
fi

toml_set Cargo.toml package.version "$version"
toml_set bindings/python/Cargo.toml package.version "$version"
VERSION="$version" python scripts/prepare_changelog.py

read -p "Updated files, please check for errors. Do you want to continue? [Y/n] " prompt

if [[ $prompt == "y" || $prompt == "Y" || $prompt == "yes" || $prompt == "Yes" ]]; then
    git add -A
    git commit -m "(chore) Bump version to $tag for release" || true && git push
    git tag "$tag" -m "$tag"
    git push --tags
else
    echo "Cancelled"
    exit 1
fi
