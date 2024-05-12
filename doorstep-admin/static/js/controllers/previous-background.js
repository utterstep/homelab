import { Controller } from "https://esm.sh/@hotwired/stimulus@3.2.2";

window.Stimulus.register(
    "previous-background",
    class extends Controller {
        static outlets = ["background-canvas"];

        static values = {
            filename: String,
            isCurrent: Boolean,
        };

        connect() {
            if (this.isCurrentValue) {
                this.loadToCanvas();
            }
        }

        loadToCanvas(e) {
            if (e) {
                e.preventDefault();
            }

            const filename = this.filenameValue;
            const canvas = this.backgroundCanvasOutlet;

            const url = `/admin/background/files/${filename}`;
            canvas.loadImg(url);
        }
    },
);
