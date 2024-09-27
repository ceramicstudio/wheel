class Wheel < Formula
    desc "ComposeDB and Ceramic Setup Tool"
    homepage "https://ceramic.network/"
    url "https://github.com/ceramicstudio/wheel/archive/refs/tags/v0.2.1.tar.gz"
    sha256 "4025dc804f1e1d98852c555049f314bb74f20f0a6ff60cdee15470e85ebb2dc9"
    license all_of: ["MIT", "Apache-2.0"]
    head "https://github.com/ceramicstudio/wheel.git", branch: "main"
  
    livecheck do
      url :stable
      regex(/^v?(\d+(?:\.\d+)+)$/i)
    end
  
    depends_on "curl" => :build
    depends_on "jq" => :build
    depends_on "node"
  
    def install
      system "./wheel.sh"
      system "./wheel"
      bin.install "wheel"
      ENV.deparallelize
    end
  
    test do
      # Testing the version
      assert_match "wheel-3box #{version}", shell_output("#{bin}/wheel --version")
    end
  end
  