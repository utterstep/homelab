import { Controller } from "https://esm.sh/@hotwired/stimulus@3.2.2";

window.Stimulus.register(
    "background-canvas",
    class extends Controller {
        static targets = ["canvas", "lineWidth", "nameInput"];

        static values = {
            width: Number,
            height: Number,
        };

        connect() {
            const canvas = this.canvasTarget;
            const ctx = canvas.getContext("2d");

            // Get the device pixel ratio, falling back to 1.
            this.dpr = window.devicePixelRatio || 1;
            // Give the canvas pixel dimensions of their CSS
            // size * the device pixel ratio.
            canvas.width = this.widthValue;
            canvas.height = this.heightValue;

            canvas.style.width = `${this.widthValue / this.dpr}px`;
            canvas.style.height = `${this.heightValue / this.dpr}px`;
            // Scale all drawing operations by the dpr, so you
            // don't have to worry about the difference.
            // ctx.scale(dpr, dpr);

            this.ctx = ctx;
            this.ctx.strokeStyle = "#000000";
            this.ctx.lineWidth = 8;
            this.ctx.lineCap = "round";

            this.initDrawing();
        }

        async save() {
            // add timestamp to the filename to avoid overwriting
            const name = this.nameInputTarget.value;
            const timestamp = Math.round(new Date().getTime() / 1000);
            // if name contains previous timestamp, remove it
            const nameParts = name.split("-");
            if (nameParts.length > 1) {
                const lastPart = nameParts.pop();
                if (lastPart.match(/^\d{10+}$/)) {
                    nameParts.push(timestamp);
                }
            }
            const filename = `${nameParts.join("-")}-${timestamp}.png`;

            this.canvasTarget.toBlob(async (pngBlob) => {
                const formData = new FormData();
                formData.append("background", pngBlob, filename);

                const response = await fetch("/admin/background/update/", {
                    method: "POST",
                    body: formData,
                });
                const hash = response.headers.get("X-Background-Hash");
                console.log("Background saved, hash: ", hash);

                Turbo.visit(window.location);
            }, "image/png");
        }

        clearCanvas() {
            this.ctx.clearRect(0, 0, this.widthValue, this.heightValue);
        }

        changeLineWidth(e) {
            const lineWidth = e.target.value;
            this.ctx.lineWidth = lineWidth;

            this.lineWidthTarget.textContent = `${lineWidth}px`;
        }

        loadImg(url) {
            const img = new Image();

            img.src = url;
            img.onload = () => {
                this.ctx.drawImage(img, 0, 0);
            };

            this.nameInputTarget.value = url.split("/").pop().split(".")[0];
        }

        initDrawing() {
            let pressedPointer = false;
            let x;
            let y;

            const drawLineOnCanvas = (x_start, y_start, x_end, y_end, ctx) => {
                ctx.beginPath();
                ctx.moveTo(x_start, y_start);
                ctx.lineTo(x_end, y_end);
                ctx.stroke();
            };

            const startDrawing = (e) => {
                e.preventDefault();

                pressedPointer = true;

                x = e.offsetX * this.dpr;
                y = e.offsetY * this.dpr;
            };

            const drawLine = (e) => {
                e.preventDefault();

                if (pressedPointer) {
                    if (!e.touches) {
                        this.canvasTarget.style.cursor = "crosshair";
                    }
                    const xM = e.offsetX * this.dpr;
                    const yM = e.offsetY * this.dpr;
                    drawLineOnCanvas(x, y, xM, yM, this.ctx);
                    x = xM;
                    y = yM;
                }
            };

            const stopDrawing = (e) => {
                e.preventDefault();

                pressedPointer = false;
                this.canvasTarget.style.cursor = "pointer";
            };

            // pointer events
            this.canvasTarget.addEventListener("pointerdown", startDrawing);
            this.canvasTarget.addEventListener("pointermove", drawLine);
            this.canvasTarget.addEventListener("pointerup", stopDrawing);

            // on touch events – prevent scrolling
            const dropEvent = (e) => e.preventDefault();

            this.canvasTarget.addEventListener("touchstart", dropEvent);
            this.canvasTarget.addEventListener("touchmove", dropEvent);
            this.canvasTarget.addEventListener("touchend", dropEvent);
        }
    },
);
