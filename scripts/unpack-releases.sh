#!/usr/bin/env bash

# This script is used to move a group of zipped and nested
# release artifacts, and is used in the GitHub workflow so
# that we can upload all artifacts to the release easier

CWD="$PWD"

# We should have gotten RELEASES_DIR as the first arg to this script
RELEASES_DIR="$1"
if [ -z "$RELEASES_DIR" ]; then
    echo "Usage: $0 <RELEASES_DIR>"
    exit 1
fi
if [ ! -d "$RELEASES_DIR" ]; then
    echo "Releases directory '$RELEASES_DIR' does not exist"
    exit 1
fi

# Navigate into the releases dir and print out verbose info about it
cd "$RELEASES_DIR"
echo ""
echo "Releases dir:"
ls -lhrt

# Look for and move out zip files into a common directory
echo ""
echo "Searching for zipped releases..."
for DIR in * ; do
	if [ -d "$DIR" ]; then
		cd "$DIR"
		for FILE in * ; do
			if [ ! -d "$FILE" ]; then
				if [ "$FILE" = "release.zip" ]; then
					echo "Found zipped release '$DIR'"
					mv "$FILE" "../$DIR.zip"
					rm -rf "../$DIR/"
				fi
			fi
		done
		cd ..
	fi
done

# Finally, print out verbose info about the releases dir again,
# so that anyone inspecting the script output can see that the
# zipped releases have been moved out successfully
echo ""
echo "Releases dir:"
ls -lhrt

# Go back to cwd
cd "$CWD"
