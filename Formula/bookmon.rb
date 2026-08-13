class Bookmon < Formula
  desc "A command-line tool for tracking your reading progress"
  homepage "https://github.com/benedicteb/bookmon"
  url "https://github.com/benedicteb/bookmon/archive/refs/tags/v1.0.76.tar.gz"
  sha256 "5b5d36f317b61a3a4af8da853a7a3dbd5ade81ac9479f6853fc01a26c58fbfe0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "--path", "."
  end

  test do
    system "#{bin}/bookmon", "--version"
  end
end
