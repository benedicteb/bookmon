class Bookmon < Formula
  desc "A command-line tool for tracking your reading progress"
  homepage "https://github.com/benedicteb/bookmon"
  url "https://github.com/benedicteb/bookmon/archive/refs/tags/v1.0.46.tar.gz"
  sha256 "27eca6c5cbefb67a4bfbc7b806e5f6ce07380f4afbb838e4b8d2c42aef0bb718"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "--path", "."
  end

  test do
    system "#{bin}/bookmon", "--version"
  end
end
