# Copyright 2019 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).

# We use this to establish the build root, rather than `./pants`, because we cannot safely use the
# latter as the sentinel filename per https://github.com/pantsbuild/pants/pull/8105.
files(
  name = 'build_root',
  source = "BUILD_ROOT",
)

files(
  name = 'build_tools',
  source = 'BUILD.tools',
  dependencies = [
    ':scalajs_3rdparty_directory',
  ],
)

files(
  name = '3rdparty_directory',
  sources = rglobs('3rdparty/*'),
)

files(
  name = 'scalajs_3rdparty_directory',
  sources = rglobs('contrib/scalajs/3rdparty/*'),
)

files(
  name = 'pants_ini',
  source = 'pants.ini',
)

# NB: This is used for integration tests. This is generated automatically via `./pants` and
# `build-support/bin/bootstrap_pants_pex.sh`.
files(
  name = 'pants_pex',
  source = 'pants.pex',
)
