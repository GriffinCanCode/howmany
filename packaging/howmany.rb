class Howmany < Formula
  desc "Fast, intelligent code analysis tool with parallel processing and visualizations"
  homepage "https://github.com/GriffinCanCode/howmany"
  url "https://github.com/GriffinCanCode/howmany/archive/refs/tags/v3.0.0.tar.gz"
  sha256 "fc7b30d3154982c2feed943af158b2e906e89d9399449af81cb82faf348dfc4e"
  license "MIT"
  head "https://github.com/GriffinCanCode/howmany.git", branch: "main"

  depends_on "rust" => :build

  # The tarball root is the crate root: this repository is `howmany-core`, and
  # the sibling directories it sits beside locally are separate repositories.
  def install
    system "cargo", "install", *std_cargo_args
  end

  # A formula must not reach outside the prefix, and an editor's configuration
  # is about as far outside it as you can get -- so the wiring is named here
  # rather than performed. Anyone who skips this is offered the same thing by
  # the tool itself the first time they run it.
  def caveats
    <<~EOS
      To see line counts in your editor as you type:
        howmany init

      That installs the VS Code extension and writes a Neovim autoload file.
      Nothing is written without asking; `howmany init --dry-run` shows what
      would change.
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/howmany --version")
    system bin/"howmany", "--help"

    (testpath/"test.rs").write <<~RUST
      fn main() {
          println!("Hello, world!");
      }
    RUST

    output = shell_output("#{bin}/howmany #{testpath} --output json --no-interactive")
    assert_match "total_files", output
    assert_match "total_lines", output

    # The editor integration is only as good as the subcommands it rests on,
    # and a bottle that ships without them looks identical until someone tries.
    # A build machine has no editor installed, so this asserts that init runs
    # and reports on the editor it was asked about -- not that it found one.
    assert_match "VS Code", shell_output("#{bin}/howmany init --dry-run --editor vscode")
  end
end
