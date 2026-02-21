class Bookmon < Formula
  desc "A command-line tool for tracking your reading progress"
  homepage "https://github.com/benedicteb/bookmon"
  url "https://github.com/benedicteb/bookmon/archive/refs/tags/v1.0.68.tar.gz"
  sha256 "e83ad19c0dad0ac0a496034a388bcdaf62cb6858fc28d68f2c19b40985868097"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "--path", "."
  end

  test do
    system "#{bin}/bookmon", "--version"
  end
end
