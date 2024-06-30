<script lang="ts">
	import init, { Image } from "cbf-rs-wasm";
	import AnalysisWorker from "./worker?worker";

	let canvas: HTMLCanvasElement | null = null;
	let bufferBuffer: Float64Array | null = null;

	let scale = "linear";
	let rows = 0;
	let height = 1;
	$: scale, height, realRenderAnalysisImage();

	async function openFile(
		e: Event & { currentTarget: EventTarget & HTMLInputElement },
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
		ctx.clearRect(0, 0, canvas.width, canvas.height);
		let isFirst = true;

		const workers = new Array<AnalysisWorker>();
		let completed = 0;
		for (let i = 0; i < navigator.hardwareConcurrency; i++) {
			const worker = new AnalysisWorker();
			worker.onmessage = (e) => {
				const { index, width, raw, scaled } = e.data;
				if (isFirst) {
					canvas!.width = width / 2;
					canvas!.height = files.length * height;
					rows = files.length;
					bufferBuffer = new Float64Array((width / 2) * files.length);
					isFirst = false;
				}
				bufferBuffer!.set(new Float64Array(raw), (index * width) / 2);
				const rowImageData = new ImageData(
					new Uint8ClampedArray(scaled),
					width / 2,
					1,
				);
				for (
					let rowDuplicate = 0;
					rowDuplicate < height;
					rowDuplicate++
				) {
					ctx.putImageData(
						rowImageData,
						0,
						index * height + rowDuplicate,
					);
				}
				if (++completed === files.length) {
					for (const worker of workers) {
						worker.terminate();
					}
					realRenderAnalysisImage();
				}
			};
			workers.push(worker);
		}
		for (let i = 0; i < files.length; i++) {
			workers[i % workers.length].postMessage({
				index: i,
				file: files[i],
			});
		}
	}

	async function readImage(file: File): Promise<Image> {
		const buffer = new Uint8Array(await file.arrayBuffer());
		return Image.load(buffer);
	}

	function realRenderAnalysisImage(): void {
		if (canvas == null || bufferBuffer == null) {
			return;
		}

		const ctx = canvas.getContext("2d")!;
		canvas.height = rows * height;
		const imageBuffer = new Uint8ClampedArray(
			canvas.width * canvas.height * 4,
		);
		let min = Number.MAX_SAFE_INTEGER;
		let max = Number.MIN_SAFE_INTEGER;
		for (let i = 0; i < bufferBuffer.length; i++) {
			min = Math.min(min, bufferBuffer[i]);
			max = Math.max(max, bufferBuffer[i]);
		}
		for (let i = 0; i < bufferBuffer.length; i++) {
			let value = bufferBuffer[i];
			if (scale === "linear") {
				value = (bufferBuffer[i] - min) / (max - min);
			} else if (scale === "circle") {
				value = Math.sqrt(
					1 - Math.pow((bufferBuffer[i] - min) / (max - min) - 1, 2),
				);
			} else if (scale === "log") {
				value =
					(Math.log(bufferBuffer[i]) - Math.log(min)) /
					(Math.log(max) - Math.log(min));
			}
			value *= 255;
			const row =
				Math.floor(i / canvas.width) * canvas.width * (height - 1) * 4;
			for (let j = 0; j < height; j++) {
				imageBuffer[row + i * 4 + j * canvas.width * 4] = 255 - value;
				imageBuffer[row + i * 4 + 1 + j * canvas.width * 4] =
					255 - value;
				imageBuffer[row + i * 4 + 2 + j * canvas.width * 4] =
					255 - value;
				imageBuffer[row + i * 4 + 3 + j * canvas.width * 4] = 255;
			}
		}
		const imageData = new ImageData(
			imageBuffer,
			canvas.width,
			canvas.height,
		);
		ctx.putImageData(imageData, 0, 0);
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
			<input
				id="file"
				type="file"
				accept=".cbf"
				multiple
				on:change={openFile}
			/>
			<br />
			<label>
				<span>Scale</span>
				<select bind:value={scale}>
					<option value="linear">Linear</option>
					<option value="circle">Circle</option>
					<option value="log">Logarithmic</option>
				</select>
			</label>
			<br />
			<label>
				<span>Height</span>
				<input
					type="number"
					min="1"
					max="10"
					step="1"
					bind:value={height}
				/>
			</label>
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
