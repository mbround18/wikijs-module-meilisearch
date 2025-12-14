import { defineConfig } from "vite";
import { viteStaticCopy } from "vite-plugin-static-copy";
import path from "path";
import fs from "fs";

function getVersion() {
  const envVersion = process.env.VERSION;
  if (envVersion && envVersion.trim()) return envVersion.trim();
  try {
    const rootVersionPath = path.resolve(__dirname, "VERSION");
    if (fs.existsSync(rootVersionPath)) {
      return fs.readFileSync(rootVersionPath, "utf8").trim();
    }
  } catch {}
  return "dev";
}

export default defineConfig({
  build: {
    target: "node24",
    outDir: "dist",
    lib: {
      entry: path.resolve(__dirname, "engine.js"),
      name: "engine",
      formats: ["cjs"],
      fileName: () => "engine.js",
    },
    rollupOptions: {
      external: [], // Bundle all dependencies
      output: {
        entryFileNames: "engine.js",
        assetFileNames: "[name][extname]",
      },
    },
    emptyOutDir: true,
  },
  server: {
    host: false,
  },
  plugins: [
    viteStaticCopy({
      targets: [
        { src: "definition.yml", dest: "." },
        { src: "LICENSE", dest: "." },
        { src: "pkg", dest: "." },
      ],
    }),
    {
      name: "emit-version",
      generateBundle(_, bundle) {
        this.emitFile({
          type: "asset",
          fileName: "VERSION",
          source: getVersion() + "\n",
        });
      },
    },
  ],
  resolve: {
    extensions: [".js", ".wasm"],
  },
});
