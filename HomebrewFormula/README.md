# Homebrew formula

[`homebrewconfig.rb`](homebrewconfig.rb) builds homebrewconfig from the tagged
GitHub source release and installs the binary, man page and shell completions.

## Quick install (no tap)

```bash
brew install --formula \
  https://raw.githubusercontent.com/vincentlauriat/homebrewconfig/main/HomebrewFormula/homebrewconfig.rb
```

## Publishing a tap (recommended)

A tap is a separate GitHub repo named `homebrew-<name>`:

1. Create a repo `vincentlauriat/homebrew-tap`.
2. Copy `homebrewconfig.rb` into its `Formula/` directory and push.
3. Users then run:

   ```bash
   brew install vincentlauriat/tap/homebrewconfig
   ```

## Updating for a new release

For each new tag `vX.Y.Z`:

1. Bump `url` to the new tag.
2. Refresh `sha256`:

   ```bash
   curl -sL https://github.com/vincentlauriat/homebrewconfig/archive/refs/tags/vX.Y.Z.tar.gz \
     | shasum -a 256
   ```

3. `brew audit --strict --new homebrewconfig` and `brew test homebrewconfig`
   before pushing.
