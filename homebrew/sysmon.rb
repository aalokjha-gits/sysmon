class Sysmon < Formula
  desc "Lightweight system monitoring dashboard with web UI"
  homepage "https://github.com/aalokjha-gits/sysmon"
  version "0.1.0"
  license "MIT"

  # Pre-built binaries
  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/aalokjha-gits/sysmon/releases/download/v#{version}/sysmon-universal"
    sha256 "58215a16af627a8fb1f03272a5efd1f26b13d8fe1e72bff3515eddace2af3663"
  elsif OS.mac? && Hardware::CPU.intel?
    url "https://github.com/aalokjha-gits/sysmon/releases/download/v#{version}/sysmon-x86_64-apple-darwin"
    sha256 "66aea17382d3f4cb6aaa4b5eaa51b344285d78048a60f665ad2d3665fd0d318f"
  end

  # Build from source
  head "https://github.com/aalokjha-gits/sysmon.git", branch: "main"

  depends_on "rust" => :build
  depends_on "node" => :build if build.head?

  def install
    if build.head? || build.from_source?
      # Build from source
      system "cargo", "build", "--release"
      bin.install "target/release/sysmon"

      # Install shell completions
      generate_completions_from_executable(bin/"sysmon", "completions")
    else
      # Install pre-built binary
      bin.install "sysmon" => "sysmon"
    end

    # Create config directory
    (etc/"sysmon").mkpath

    # Install example config if not present
    unless (etc/"sysmon/config.toml").exist?
      (etc/"sysmon").install "config.example.toml" => "config.toml" if File.exist?("config.example.toml")
    end
  end

  def post_install
    # Create default config directory for user
    (var/"log/sysmon").mkpath
  end

  def caveats
    <<~EOS
      sysmon has been installed!

      To start sysmon:
        sysmon

      To start on a custom port:
        sysmon --port 8080

      To start without opening browser:
        sysmon --no-browser

      Configuration file location:
        ~/.config/sysmon/config.toml

      Web UI will be available at:
        http://localhost:3030

      Logs are stored in:
        #{var}/log/sysmon/
    EOS
  end

  service do
    run [opt_bin/"sysmon", "--no-browser"]
    keep_alive true
    log_path var/"log/sysmon/sysmon.log"
    error_log_path var/"log/sysmon/sysmon.error.log"
    environment_variables PATH: std_service_path_env
  end

  test do
    # Test that the binary works
    assert_match "sysmon", shell_output("#{bin}/sysmon --version")

    # Test that the config is valid
    system "#{bin}/sysmon", "--help"
  end
end
