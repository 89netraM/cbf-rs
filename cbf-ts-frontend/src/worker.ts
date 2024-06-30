import init, { Image, Analysis } from "cbf-rs-wasm";

const initialization = init();

self.onmessage = async e => {
    await initialization;
    const { index, file } = e.data;
    const image = new Uint8Array(await file.arrayBuffer());
    const img = Image.load(image);
    const width = img.width;
    const analyzer = Analysis.init();
    analyzer.analyze(img);
    const raw = analyzer.raw;
    const scaled = analyzer.localScaled;
    analyzer.free();
    img.free();
    self.postMessage({ index, width, raw, scaled });
};
