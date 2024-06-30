import { defineConfig } from "vite";
import wasmPack from "vite-plugin-wasm-pack";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
	plugins: [wasmPack("../cbf-rs-wasm"), svelte()],
})
