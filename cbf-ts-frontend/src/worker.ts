import init, { Image, Analysis } from "cbf-rs-wasm";

const initialization = init();

self.onmessage = async e => {
    await initialization;
    const { index, file } = e.data;
    const image = new Uint8Array(await file.arrayBuffer());
    const img = Image.load(image);
    const width = img.width;
    const buffer = new ArrayBuffer(img.width / 2 * 4);
    const analyzer = Analysis.init();
    analyzer.analyze(img);
    analyzer.writeImage(new Uint8Array(buffer));
    analyzer.free();
    img.free();
    self.postMessage({ index, width, buffer });
};
