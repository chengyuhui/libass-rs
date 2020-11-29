use std::{env, error::Error, fs::File, io::BufWriter};

use libass::{DefaultFontProvider, Layer, Library};

fn draw_layer(layer: Layer, dst: &mut [u8]) {
    // RGBA order
    let mut color = layer.color.to_be_bytes();
    color[3] = 255 - color[3]; // Inverse alpha

    for y in 0..layer.height as usize {
        for x in 0..layer.width as usize {
            let k = layer.bitmap[y * layer.width as usize + x] as u16;

            let dst_x = x + layer.x as usize;
            let dst_y = y + layer.y as usize;
            let dst_p = (dst_y * 1920 + dst_x) * 4;

            for i in 0..4 {
                let dst_off = dst_p + i;
                let dst_orig = dst[dst_off] as u16;
                dst[dst_off] = ((k * color[i] as u16 + (255 - k) * dst_orig) / 255) as u8;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: {} <image file> <subtitle file> <time>", args[0]);
        return Ok(());
    }

    let img_file = File::create(&args[1])?;
    let img_writer = BufWriter::new(img_file);
    let mut encoder = png::Encoder::new(img_writer, 1920, 1080);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);

    let sub_file = &args[2];
    let timestamp: i64 = args[3].parse()?;

    let lib = Library::new()?;
    let mut renderer = lib.new_renderer()?;
    renderer.set_frame_size(1920, 1080);
    renderer.set_fonts(
        None,
        "sans-serif",
        DefaultFontProvider::Autodetect,
        None,
        false,
    );

    let mut track = lib.new_track_from_file(sub_file, "UTF-8")?;
    let frame = renderer.render_frame(&mut track, timestamp);
    let image = frame.0.unwrap();

    let mut framebuffer = vec![0u8; 1920 * 1080 * 4];

    for layer in image {
        draw_layer(layer, &mut framebuffer);
    }

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&framebuffer)?;
    Ok(())
}
