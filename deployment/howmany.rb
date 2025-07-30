class Howmany < Formula
  desc "Fast, intelligent code analysis tool with parallel processing and visualizations"
  homepage "https://github.com/GriffinCanCode/howmany"
  url "https://github.com/GriffinCanCode/howmany/archive/refs/tags/v2.0.0.tar.gz"
  sha256 "7af42a9ea6aa74a2c42c697ddb04992ff46c58610e7bafa3d30f27461e945753"
  license "Griffin-1.0"
  head "https://github.com/GriffinCanCode/howmany.git", branch: "main"

  depends_on "rust" => :build

  def install
    cd "howmany-core" do
      system "cargo", "install", *std_cargo_args
    end
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