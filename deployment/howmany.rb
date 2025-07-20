class Howmany < Formula
  desc "Fast, intelligent code analysis tool with parallel processing and visualizations"
  homepage "https://github.com/GriffinCanCode/howmany"
  url "https://github.com/GriffinCanCode/howmany/archive/refs/tags/v0.3.2.tar.gz"
  sha256 "0b7ec2d0ec31c53c2e0f22ed5943d36aa47fa8a96d00c5a8737ec07bc3c85dad"
  license "Griffin-1.0"
  head "https://github.com/GriffinCanCode/howmany.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/howmany", "--version"
    system "#{bin}/howmany", "--help"
    
    # Create a simple test file to analyze
    (testpath/"test.rs").write <<~EOS
      fn main() {
          println!("Hello, world!");
      }
    EOS
    
    # Test basic functionality
    output = shell_output("#{bin}/howmany #{testpath} --output json --no-interactive")
    assert_match "total_files", output
    assert_match "total_lines", output
  end
end 