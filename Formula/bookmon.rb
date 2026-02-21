class Bookmon < Formula
  desc "A command-line tool for tracking your reading progress"
  homepage "https://github.com/benedicteb/bookmon"
  url "https://github.com/benedicteb/bookmon/archive/refs/tags/v1.0.69.tar.gz"
  sha256 "6cd97d9f8144c3888e237062bfea1b7f9c3f2452cc453497f75bd6667ad3d4f7"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "--path", "."
  end

  test do
    system "#{bin}/bookmon", "--version"
  end
end
