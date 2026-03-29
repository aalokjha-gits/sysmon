class Sysmon < Formula
  desc "Lightweight system monitoring dashboard with web UI"
  homepage "https://github.com/aalokjha-gits/sysmon"
  version "0.1.0"
  license "MIT"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/aalokjha-gits/sysmon/releases/download/v#{version}/sysmon-universal"
    sha256 "58215a16af627a8fb1f03272a5efd1f26b13d8fe1e72bff3515eddace2af3663"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/aalokjha-gits/sysmon/releases/download/v#{version}/sysmon-x86_64-apple-darwin"
    sha256 "66aea17382d3f4cb6aaa4b5eaa51b344285d78048a60f665ad2d3665fd0d318f"
  end

  def install
    if build.head?
      system "cargo", "build", "--release"
      bin.install "target/release/sysmon"
    else
      bin.install Dir.glob("sysmon*").first => "sysmon"
    end
  end

  def caveats
    <<~EOS
      To start sysmon:
        sysmon

      To start on a custom port:
        sysmon --port 8080

      To start without opening browser:
        sysmon --no-browser

      Configuration: ~/.config/sysmon/config.toml
    EOS
  end

  test do
    assert_match "sysmon", shell_output("#{bin}/sysmon --version")
  end
end
