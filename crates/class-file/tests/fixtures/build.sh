#!/usr/bin/env bash
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
lombok="${LOMBOK_JAR:-$HOME/.m2/repository/org/projectlombok/lombok/1.18.30/lombok-1.18.30.jar}"
modules="$(mktemp -d)"
work="$(mktemp -d)"
trap 'rm -rf "$modules" "$work"' EXIT

rm -rf "$here/classes" "$here/fixtures.jar"
mkdir -p "$here/classes/shop" "$here/classes/lib"

javac --release 17 -Xpkginfo:always -cp "$lombok" -processorpath "$lombok" \
  -d "$here/classes" $(find "$here/src/shop" -name '*.java' | sort)
javac --release 17 -d "$here/classes" $(find "$here/src/lib" -name '*.java' | sort)
javac --release 17 -d "$modules" "$here/src/module-info.java"

jar --create --file "$here/fixtures.jar" --date 2024-01-01T00:00:00Z \
  -C "$here/classes" lib -C "$modules" module-info.class
rm -rf "$here/classes/lib"

javac --release 17 -d "$work/current" $(find "$here/jdk-src/current" -name '*.java' | sort)
javac --release 17 -d "$work/legacy" $(find "$here/jdk-src/legacy" -name '*.java' | sort)

stage="$work/stage"
put() {
  mkdir -p "$stage/$1/$2/$(dirname "$4")"
  cp "$work/$3/$4.class" "$stage/$1/$2/$4.sig"
}
put 8 sample.base legacy sample/util/Box
put 9AB sample.base current sample/util/Box
put 89AB sample.base current sample/util/Pair
put 89AB sample.base current sample/util/Tagged
put B sample.base current sample/util/Fresh
put 8 sample.desktop current sample/awt/Canvas
printf 'garbage' > "$stage/B/sample.base/sample/util/Corrupt.sig"
printf 'garbage' > "$stage/B/sample.base/module-info.sig"
mkdir -p "$stage/C"
printf 'sample.base\nsample.desktop\n' > "$stage/C/system-modules"

rm -rf "$here/jdk"
mkdir -p "$here/jdk/lib"
jar --create --no-manifest --file "$here/jdk/lib/ct.sym" --date 2024-01-01T00:00:00Z -C "$stage" .
printf 'JAVA_VERSION="12.0.1"\n' > "$here/jdk/release"
