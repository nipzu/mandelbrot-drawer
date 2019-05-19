import { CanvasBuffer, MandelbrotRenderer, initialize } from "mandelbrot/mandelbrot";
import { memory } from "mandelbrot/mandelbrot_bg";
import "../css/style.css";

const canvas = document.getElementById("mandelbrot-canvas");
const ctx = canvas.getContext("2d");

const zoom_factor = 1.1;

var zoom = 1.0;
var view_x = -0.3;
var view_y = 0.0;
var max_iterations = 30000;

initialize();
var canvas_buffer = CanvasBuffer.new();
var mandelbrot_rendered = MandelbrotRenderer.new(zoom, view_x, view_y, max_iterations);

var width;
var height;

var mouse_pressed = false;

var next_render;

const change_zoom = event => {
    let mouse_window_x = event.clientX/width - 0.5;
    let mouse_window_y = event.clientY/height - 0.5;
    if (!mouse_pressed) {
        if (width > height) {
            mouse_window_x *= width/height
        } else {
            mouse_window_y *= height/width;
        }

        mandelbrot_rendered.change_view(2*mouse_window_x, 2*mouse_window_y);

        const delta = Math.sign(event.deltaY);
        const factor = (delta > 0) ? 1/zoom_factor : zoom_factor/1;
        mandelbrot_rendered.change_zoom(factor);

        mandelbrot_rendered.change_view(-2*mouse_window_x, -2*mouse_window_y);

        zoom_canvas(event);
    }
}

const zoom_canvas = event => {
    const delta = Math.sign(event.deltaY);
    let delta_zoom = (delta > 0) ? 1/zoom_factor : zoom_factor/1;
    
    const last_left = parseFloat(canvas.style.left) / 100;
    const last_top = parseFloat(canvas.style.top) / 100;
    const last_right = last_left + parseFloat(canvas.style.width) / 100;
    const last_bottom = last_top + parseFloat(canvas.style.height) / 100;

    const mouse_x = event.clientX/width;
    const mouse_y = event.clientY/height;

    const new_left = mouse_x - delta_zoom*(mouse_x - last_left);
    const new_top = mouse_y - delta_zoom*(mouse_y - last_top);
    const new_right = mouse_x + delta_zoom*(last_right - mouse_x);
    const new_bottom = mouse_y + delta_zoom*(last_bottom - mouse_y);

    canvas.style.left = (new_left*100).toString() + "%";
    canvas.style.top = (new_top*100).toString() + "%";

    canvas.style.width = ((new_right - new_left)*100).toString() + "%";
    canvas.style.height  = ((new_bottom - new_top)*100).toString() + "%";

    clearTimeout(next_render);
    next_render = setTimeout(() => { draw_image(); }, 500);
}

const start_mouse = event => {
    if (event.button === 0) {
        clearTimeout(next_render);
        mouse_pressed = true;
    }
}

const stop_mouse = event => {
    if (event.button === 0) {
        mouse_pressed = false;
        clearTimeout(next_render);
        next_render = setTimeout(() => { draw_image(); }, 500);
    }
}

const move_mouse = event => {
    if (mouse_pressed) {
        if (width > height) {
            mandelbrot_rendered.change_view(-2*event.movementX/height, -2*event.movementY/height);
        } else {
            mandelbrot_rendered.change_view(-2*event.movementX/width, -2*event.movementY/width);
        }

        let last_left = parseFloat(canvas.style.left) / 100;
        let last_top = parseFloat(canvas.style.top) / 100;

        transform_canvas(last_left + event.movementX/width, last_top + event.movementY/height);
    }
}

const resize_canvas = () => {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    width = canvas.width;
    height = canvas.height;

    canvas_buffer.resize(width, height);

    draw_image();
}

const transform_canvas = (x, y) => {
    canvas.style.left = (x*100).toString() + "%";
    canvas.style.top = (y*100).toString() + "%";
}

const draw_image = () => {
    mandelbrot_rendered.render(canvas_buffer);
    
    canvas.style.top = "0%";
    canvas.style.left = "0%";
    canvas.style.width = "100%";
    canvas.style.height = "100%";

    draw_canvas();
}

const draw_canvas = () => {
    const buffer = canvas_buffer.get_buffer();
    const buffer_length = canvas_buffer.get_buffer_length();

    const canvas_data = new Uint8ClampedArray(memory.buffer, buffer, buffer_length);

    const image_data = new ImageData(canvas_data, width, height);
    
    ctx.clearRect(0, 0, width, height);
    ctx.putImageData(image_data, 0, 0);
}

window.addEventListener("resize", resize_canvas);
window.addEventListener("wheel", change_zoom);
window.addEventListener("mousedown", start_mouse);
window.addEventListener("mouseup", stop_mouse);
window.addEventListener("mousemove", move_mouse);

resize_canvas();