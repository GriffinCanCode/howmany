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
  end
end
