import "./app.css";
import App from "./App.svelte";
import init, { greet } from "cbf-rs-wasm";

await init();

console.log(greet("Joakim"));

const app = new App({
	target: document.getElementById("app")!,
})

export default app
