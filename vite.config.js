import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { viteStaticCopy } from 'vite-plugin-static-copy';
import path from 'path';
import glob from 'fast-glob'; // npm i fast-glob
import fs from 'fs';

function cleanOldCSS() {
  return {
    name: 'clean-old-css',
    apply: 'build',
    buildStart() {
      const files = glob.sync('static/assets/*.css');
      for (const file of files) {
        const absolutePath = path.resolve(__dirname, file);
        fs.unlinkSync(absolutePath);
      }
      console.log('[vite] Cleaned old CSS files from /static ✅');
    }
  };
}

export default defineConfig({
  plugins: [cleanOldCSS(),
    react(),
    viteStaticCopy({
      targets: [
        { src: 'frontend/index.css', dest: '.' },
        { src: 'frontend/login.css', dest: '.' },
        { src: 'frontend/create.css', dest: '.' },
        { src: 'frontend/home.css', dest: '.' },
      ]
    })
  ],
  build: {
    outDir: 'static',
    emptyOutDir: false,
    cssCodeSplit: false,
    rollupOptions: {
      input: {
        app: path.resolve(__dirname, 'frontend/main.jsx')
      },
      output: {
        entryFileNames: 'app.bundle.js'
      }
    }
  }
});