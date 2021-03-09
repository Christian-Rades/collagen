use image::GenericImageView;
mod blockdb;
use blockdb::BlockDb;


fn main() {
    let img = image::open("./markus-spiske-o8IfF0RUTTs-unsplash.jpg")
        .unwrap()
        .into_rgb8();
    let (width, height) = img.dimensions();

    let mut sub_imgs = Vec::new();

    for x in (0..width - 32).step_by(32) {
        for y in (0..height - 32).step_by(32) {
            sub_imgs.push(img.view(x, y, 32, 32))
        }
    }

    let bldb = BlockDb::new(sub_imgs, |img| avg_color(img).into());

    let img2 = image::open("./alex-motoc-ruzQQ5M1Ow0-unsplash.jpg")
        .unwrap()
        .into_rgb8();
    let (width, height) = img2.dimensions();
    let mut out_img: image::RgbImage = image::ImageBuffer::new(width, height);

    for x in (0..width - 32).step_by(32) {
        for y in (0..height - 32).step_by(32) {
            let avg = avg_color(&img2.view(x, y, 32, 32));
            let new_block = bldb.find_closest_pos(avg.into()).unwrap();
            image::imageops::replace(&mut out_img, new_block, x, y);
        }
    }

    out_img.save("out.png").unwrap();
}

#[derive(Debug)]
struct Pos {
    r: u64,
    g: u64,
    b: u64,
}

impl From<Pos> for [i64; 3] {
    fn from(p: Pos) -> Self {
        [p.r as i64, p.g as i64, p.b as i64]
    }
}

fn avg_color(img: &image::SubImage<&image::RgbImage>) -> Pos {
    let mut out = Pos { r: 0, g: 0, b: 0 };

    let mut count = 0;
    for p in img.pixels().map(|(_, _, p)| p) {
        count += 1;
        let (r, g, b) = (p[0], p[1], p[2]);
        out.r += r as u64;
        out.g += g as u64;
        out.b += b as u64;
    }

    out.r /= count;
    out.g /= count;
    out.b /= count;

    return out;
}
