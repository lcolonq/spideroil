{
  inputs = {
    teleia.url = "github:lcolonq/teleia";
    nixpkgs.follows = "teleia/nixpkgs";
  };

  outputs = inputs@{ self, nixpkgs, ... }:
    let
      system = "x86_64-linux";
      spideroil = inputs.teleia.native.build ./. "spideroil";
      wasm = inputs.teleia.wasm.build ./. "spideroil";
    in {
      devShells.${system} = {
        default = inputs.teleia.shell;
        windows = inputs.teleia.windows.shell;
      };
    };
}
