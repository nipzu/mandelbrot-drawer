const bytes = fetch("../../../target/wasm32-unknown-unknown/release/mandelbrot.wasm");
//import { CanvasBuffer, MandelbrotRenderer } from "mandelbrot/mandelbrot";
//import { memory } from "mandelbrot/mandelbrot_bg";

Renderer
  .then(m => {
    var mandelbrot_renderer = MandelbrotRenderer.new(1.0, -0.3, 0.0, 10, 2);
    var canvas_buffer = CanvasBuffer.new();
  })
  .catch(console.error);

var rendering = false;
var mandelbrot_renderer = MandelbrotRenderer.new(1.0, -0.3, 0.0, 10, 2);
var canvas_buffer = CanvasBuffer.new();

onmessage = event => {
    switch(event.data.action) {
        case "render":
            mandelbrot_renderer.render(canvas_buffer);
            const buffer = canvas_buffer.get_buffer();
            const buffer_length = canvas_buffer.get_buffer_length();

            const canvas_data = new Uint8ClampedArray(memory.buffer, buffer, buffer_length);

            const image_data = new ImageData(canvas_data, width, height);

            postMessage(image_data);
            break;
        case "change_zoom":
            mandelbrot_renderer.change_zoom(event.data.arguments[0]);
            break;
        case "change_view":
            mandelbrot_renderer.change_view(event.data.arguments[0], event.data.arguments[1]);
            break;
        case "resize":
            canvas_buffer.resize(event.data.arguments[0], event.data.arguments[1]);
            break;
    }
}