import { resolve } from "path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
export default defineConfig({
    plugins: [react(), tailwindcss()],
    resolve: {
        alias: {
            "@": resolve(__dirname, "./src"),
        },
    },
    server: {
        port: 5173,
        host: true, // Listen on all interfaces (0.0.0.0)
        allowedHosts: ["studio", "localhost"],
        proxy: {
            "/trpc": {
                target: "http://localhost:3000",
                changeOrigin: true,
            },
            "/ws": {
                target: "http://localhost:3000",
                ws: true,
                changeOrigin: true,
            },
        },
    },
});
