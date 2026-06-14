class Homebrewconfig < Formula
  desc "TUI and CLI for configuring Homebrew environment variables"
  homepage "https://github.com/vincentlauriat/homebrewconfig"
  url "https://github.com/vincentlauriat/homebrewconfig/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "3d66265cdb32308351d07cdae27a33d70e4fa15a06601b5ed7bf25b10b6a142d"
  license "MIT"
  head "https://github.com/vincentlauriat/homebrewconfig.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
    man1.install "man/homebrewconfig.1"
    bash_completion.install "completions/homebrewconfig.bash" => "homebrewconfig"
    zsh_completion.install "completions/_homebrewconfig"
    fish_completion.install "completions/homebrewconfig.fish"
  end

  test do
    assert_match "homebrewconfig #{version}", shell_output("#{bin}/homebrewconfig --version")
    # The non-interactive path prints the managed export block.
    output = shell_output("#{bin}/homebrewconfig --set HOMEBREW_NO_ANALYTICS=1 --dry-run")
    assert_match "homebrewconfig BEGIN", output
    assert_match "export HOMEBREW_NO_ANALYTICS=1", output
  end
end
