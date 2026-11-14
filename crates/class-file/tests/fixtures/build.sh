#!/usr/bin/env bash
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
lombok="${LOMBOK_JAR:-$HOME/.m2/repository/org/projectlombok/lombok/1.18.30/lombok-1.18.30.jar}"

rm -rf "$here/classes" "$here/fixtures.jar"
mkdir -p "$here/classes/shop" "$here/classes/lib"

javac --release 17 -cp "$lombok" -processorpath "$lombok" \
  -d "$here/classes" $(find "$here/src/shop" -name '*.java' | sort)
javac --release 17 -d "$here/classes" $(find "$here/src/lib" -name '*.java' | sort)

jar --create --file "$here/fixtures.jar" --date 2024-01-01T00:00:00Z \
  -C "$here/classes" lib
rm -rf "$here/classes/lib"
