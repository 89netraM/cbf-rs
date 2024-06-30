<script lang="ts">
	import init, { Image, Analysis } from "cbf-rs-wasm";

	let canvas: HTMLCanvasElement | null = null;

	async function openFile(
		e: Event & { currentTarget: EventTarget & HTMLInputElement }
	): Promise<void> {
		const files = e.currentTarget.files;
		if (files == null || files.length === 0) {
			alert("No file selected");
			return;
		}
		
		if (files.length === 1) {
			await showImage(files[0]);
		} else {
			await showAnalysis(Array.from(files));
		}
	}

	async function showImage(file: File): Promise<void> {
		if (canvas == null) {
			return;
		}

		const image = await readImage(file);

		canvas.width = image.width;
		canvas.height = image.height;

		const ctx = canvas.getContext("2d")!;
		const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
		image.writeImage(imageData.data);
		ctx.putImageData(imageData, 0, 0);
		image.free();
	}

	async function showAnalysis(files: ReadonlyArray<File>): Promise<void> {
		if (canvas == null) {
			return;
		}

		const ctx = canvas.getContext("2d")!;
		let imageData: ImageData | null = null;

		const analysis = Analysis.init();
		for (const file of files) {
			const image = await readImage(file);
			analysis.analyze(image);
			
			if (imageData == null) {
				canvas.width = image.width / 2;
				canvas.height = files.length;
				
				imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
			}
			image.free();
			analysis.writeImage(imageData.data);
			ctx.putImageData(imageData, 0, 0);
		}
		analysis.free();
	}

	async function readImage(file: File): Promise<Image> {
		const buffer = new Uint8Array(await file.arrayBuffer());
		return Image.load(buffer);
	}
</script>

<div>
	<header>
		<h1>CBF</h1>
	</header>

	<main>
		<canvas bind:this={canvas} />
	</main>

	<aside>
		{#await init()}
			<p>Loading...</p>
		{:then}
			<label for="file">Load <code>.cbf</code></label>
			<input id="file" type="file" accept=".cbf" multiple on:change={openFile} />
		{:catch error}
			<p>Something went wrong: {error.message}</p>
		{/await}
	</aside>
</div>

<style lang="scss">
	div {
		width: 100vw;
		height: 100vh;
		display: grid;
		grid-template-columns: 3fr 1fr;
		grid-template-rows: 3rem 1fr;
		grid-template-areas:
			"header header"
			"main aside";

		header {
			text-align: center;
			border-bottom: var(--border) 1px solid;
			background-color: var(--level1);
			grid-area: header;

			h1 {
				margin: 0px;
			}
		}

		main {
			grid-area: main;

			display: flex;
			justify-content: center;
			align-items: center;

			canvas {
				max-width: 100%;
				max-height: calc(100vh - 3rem - 4px);
			}
		}

		aside {
			grid-area: aside;
			border-left: var(--border) 1px solid;
			background-color: var(--level1);

			padding: 1rem;

			display: flex;
			flex-direction: column;
			align-items: flex-start;
			gap: 0.5rem;

			#file {
				display: none;
			}
		}
	}
</style>
