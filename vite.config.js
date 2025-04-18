import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { viteStaticCopy } from 'vite-plugin-static-copy';
import path from 'path';

export default defineConfig({
  plugins: [
    react(),
    viteStaticCopy({
      targets: [
        { src: 'frontend/index.css', dest: '.' },
        { src: 'frontend/login.css', dest: '.' },
        { src: 'frontend/create.css', dest: '.' }
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