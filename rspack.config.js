const path = require("path");
const { CopyRspackPlugin } = require("@rspack/core");

class VersionEmitPlugin {
  apply(compiler) {
    compiler.hooks.thisCompilation.tap("VersionEmitPlugin", (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: "VersionEmitPlugin",
          stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
        },
        (assets) => {
          const envVersion = process.env.VERSION;
          let versionContent =
            envVersion && envVersion.trim() ? envVersion.trim() : null;
          if (!versionContent) {
            try {
              const fs = require("fs");
              const rootVersionPath = path.resolve(__dirname, "VERSION");
              if (fs.existsSync(rootVersionPath)) {
                versionContent = fs
                  .readFileSync(rootVersionPath, "utf8")
                  .trim();
              }
            } catch (_) {}
          }
          if (!versionContent) {
            versionContent = "dev";
          }
          compilation.emitAsset(
            "VERSION",
            new compiler.webpack.sources.RawSource(versionContent + "\n")
          );
        }
      );
    });
  }
}

module.exports = {
  mode: "production",
  target: "node",
  entry: {
    engine: path.resolve(__dirname, "engine.js"),
  },
  experiments: {
    asyncWebAssembly: true,
  },
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "engine.js",
    library: { type: "commonjs2" },
  },
  plugins: [
    new CopyRspackPlugin({
      patterns: [
        {
          from: "definition.yml",
          to: "definition.yml",
          noErrorOnMissing: true,
        },
        { from: "LICENSE", to: "LICENSE", noErrorOnMissing: true },
        { from: "pkg/*.wasm", to: "[name][ext]" },
      ],
    }),
    new VersionEmitPlugin(),
  ],
  resolve: {
    extensions: [".js", ".wasm"],
  },
};
