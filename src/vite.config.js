import { resolve } from 'path'; //Add this import to ensure access to the __dirname and resolve() procedure
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
// https://vitejs.dev/config/
export default defineConfig(async () => ({
    plugins: [react()],

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
        port: 3000,
        strictPort: true,
        watch: {
            // 3. tell vite to ignore watching `src-tauri`
            ignored: ["**/src-tauri/**"],
        },
    },
    //Add this build with the rollupOptions and input to reference the html files
    build: {
        rollupOptions: {
            input: {
                main: resolve( __dirname, 'index.html'),
                new_map: resolve( __dirname , "src/windows/new-map/window.html"),
                import_map: resolve( __dirname , "src/windows/import-map/window.html"),
                settings: resolve( __dirname , "src/windows/settings/window.html"),
                about: resolve( __dirname , "src/windows/about/window.html")
            }
        },
        publicDir: resolve(__dirname, "public"),
    }
}));