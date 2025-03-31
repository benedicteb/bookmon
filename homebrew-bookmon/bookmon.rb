class Bookmon < Formula
  desc "A command-line tool for tracking your reading progress"
  homepage "https://github.com/benedicteb/bookmon"
  url "git@github.com:benedicteb/bookmon.git", tag: "v1.0.38"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "--path", "."
  end

  test do
    system "#{bin}/bookmon", "--version"
  end
end 