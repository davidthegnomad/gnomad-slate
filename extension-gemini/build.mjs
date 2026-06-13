import { build } from "vite";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";
import fs from "fs";

const __dirname = dirname(fileURLToPath(import.meta.url));

async function main() {
  try {
    // 1. Build frontend React pages
    console.log("Building Gnomad Slate frontend pages...");
    await build({
      configFile: resolve(__dirname, "vite.config.ts"),
    });

    // 2. Build service worker (background.ts)
    console.log("Building Gnomad Slate service worker...");
    await build({
      build: {
        emptyOutDir: false, // Keep the HTML files built in step 1
        outDir: resolve(__dirname, "../dist-extension-gemini"),
        minify: false, // Easy debug and store review compliance
        lib: {
          entry: resolve(__dirname, "src/background.ts"),
          name: "background",
          formats: ["iife"],
          fileName: () => "background.js",
        },
      },
    });

    // 3. Copy manifest.json to dist-extension
    console.log("Copying manifest.json...");
    fs.copyFileSync(
      resolve(__dirname, "manifest.json"),
      resolve(__dirname, "../dist-extension-gemini/manifest.json")
    );

    // 4. Copy icons folder to dist-extension
    console.log("Copying icons...");
    const distIconsDir = resolve(__dirname, "../dist-extension-gemini/icons");
    if (!fs.existsSync(distIconsDir)) {
      fs.mkdirSync(distIconsDir, { recursive: true });
    }
    const sourceIconsDir = resolve(__dirname, "public/icons");
    if (fs.existsSync(sourceIconsDir)) {
      const files = fs.readdirSync(sourceIconsDir);
      for (const file of files) {
        fs.copyFileSync(resolve(sourceIconsDir, file), resolve(distIconsDir, file));
      }
    }

    console.log("\n🚀 Extension build completed successfully!");
    console.log("Output folder: dist-extension-gemini/");
  } catch (error) {
    console.error("Extension build failed:", error);
    process.exit(1);
  }
}

main();
