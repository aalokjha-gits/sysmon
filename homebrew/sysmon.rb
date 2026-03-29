class Sysmon < Formula
  desc "Lightweight system monitoring dashboard with web UI"
  homepage "https://github.com/aalokjha-gits/sysmon"
  version "0.2.0"
  license "MIT"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/aalokjha-gits/sysmon/releases/download/v#{version}/sysmon-universal"
    sha256 "2a1ab9c46b630473b7ed14c5490769ae1800098e6da6f0f286bea873b52bdcac"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/aalokjha-gits/sysmon/releases/download/v#{version}/sysmon-x86_64-apple-darwin"
    sha256 "37d14864cd34505953a1b5831667b925394ba6e70fe339dab31f8d68c368bdf7"
  end

  def install
    if build.head?
      system "cargo", "build", "--release"
      bin.install "target/release/sysmon"
    else
      bin.install Dir.glob("sysmon*").first => "sysmon"
    end
  end

  service do
    run [opt_bin/"sysmon", "--no-browser", "--port", "8989"]
    keep_alive true
    log_path var/"log/sysmon.log"
    error_log_path var/"log/sysmon.log"
    working_dir var
  end

  def caveats
    <<~EOS
      To run interactively:
        sysmon

      To run as a background service:
        brew services start sysmon
        # or
        sysmon service install

      To check status:
        brew services info sysmon
        # or
        sysmon service status

      Daemon port: 8989 (http://127.0.0.1:8989)
      Configuration: ~/.config/sysmon/config.toml
    EOS
  end

  test do
    assert_match "sysmon", shell_output("#{bin}/sysmon --version")
  end
end
