import { Controller } from "https://esm.sh/@hotwired/stimulus@3.2.2";

window.Stimulus.register(
    "previous-background",
    class extends Controller {
        static outlets = ["background-canvas", "background-preview"];

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

        preview(e) {
            e.preventDefault();
            const filename = this.filenameValue;
            const preview = this.backgroundPreviewOutlet;

            const url = `/admin/background/files/${filename}`;
            preview.showPreview(url, e.clientX + 10, e.clientY + 10);
        }

        hidePreview(e) {
            e.preventDefault();
            const preview = this.backgroundPreviewOutlet;

            preview.hidePreview();
        }
    },
);
