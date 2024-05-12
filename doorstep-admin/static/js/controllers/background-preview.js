import { Controller } from "https://esm.sh/@hotwired/stimulus@3.2.2";

window.Stimulus.register(
    "background-preview",
    class extends Controller {
        static targets = ["preview"];

        static values = {
            width: Number,
            height: Number,
        };

        connect() {
            this.shown = false;
            this.url = null;

            const ratio = this.widthValue / this.heightValue;
            const width = 250;
            const height = width / ratio;

            this.element.style.width = `${width}px`;
            this.element.style.height = `${height}px`;
        }

        showPreview(url, x, y) {
            if (!(this.shown && url === this.url)) {
                this.shown = true;
                this.previewTarget.src = url;
                this.element.style.display = "block";
            }

            this.element.style.left = `${x}px`;
            this.element.style.top = `${y}px`;
        }

        hidePreview() {
            this.shown = false;
            this.url = null;

            this.element.style.display = "none";
        }
    },
);
