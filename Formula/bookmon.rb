class Bookmon < Formula
  desc "A command-line tool for tracking your reading progress"
  homepage "https://github.com/benedicteb/bookmon"
  url "https://github.com/benedicteb/bookmon/archive/refs/tags/v1.0.43.tar.gz"
  sha256 "4330e59a1216e7a6ca524d8def4f50136f7bb4d16a17fdb7307c99e5d6ed28cc"
  sha256 "sha-placeholder"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "--path", "."
  end

  test do
    system "#{bin}/bookmon", "--version"
  end
end
