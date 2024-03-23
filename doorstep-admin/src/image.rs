use eyre::{Result, WrapErr};
use image::{imageops, io::Reader as ImageReader, Luma};

#[tracing::instrument(skip(image))]
pub fn image_to_bitmap(image: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let img = ImageReader::new(std::io::Cursor::new(image))
        .with_guessed_format()
        .wrap_err("Failed to guess image format")?
        .decode()
        .wrap_err("Failed to decode image")?
        .into_luma8();

    let img = imageops::resize(&img, width, height, imageops::FilterType::Nearest);

    let mut data: Vec<u8> = Vec::with_capacity(
        (width * height / 2) as usize + 1, // +1 for odd widths
    );

    for y in 0..img.height() {
        let mut byte = 0u8;
        for x in 0..img.width() {
            let Luma([l]) = img.get_pixel(x, y);
            if x % 2 == 0 {
                byte = l >> 4;
            } else {
                byte |= l & 0xF0;
                data.push(byte);
            }
        }
        // For odd widths, push the last byte too
        if img.width() % 2 != 0 {
            data.push(byte);
        }
    }

    Ok(data)
}
