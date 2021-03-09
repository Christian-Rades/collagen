{
  pkgs ? (import <nixpkgs>){}
}:
with pkgs;

rustPlatform.buildRustPackage rec {
  pname = "collagen";
  version = "0.1.0";

  src = ./.;

  cargoSha256 = "1ybwhdgjxjy50x8393qc6vldsdwg9bmr8l9qqz5jhjqgyx6ry7xz";
}
