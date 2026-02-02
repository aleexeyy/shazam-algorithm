import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');
  const target = env.VITE_DEV_PROXY_TARGET || env.VITE_API_URL || 'http://localhost:8000';

  return {
    plugins: [react()],
    server: {
      proxy: {
        '/ping': { target, changeOrigin: true },
        '/healthz': { target, changeOrigin: true },
        '/songs': { target, changeOrigin: true },
        '/upload-song': { target, changeOrigin: true },
        '/recognize-song': { target, changeOrigin: true },
      },
    },
  };
});
