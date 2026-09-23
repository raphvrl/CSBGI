use glam::Vec2;

use crate::assets::data::ImageData;

pub const DEFAULT_ALBEDO: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

#[derive(Clone, Copy)]
pub struct VoxelMaterial<'a> {
    pub color_factor: [f32; 4],
    pub color_image: Option<&'a ImageData>,
}

pub fn pack_albedo(color: [f32; 4]) -> u32 {
    let to_u8 = |value: f32| -> u32 { (value.clamp(0.0, 1.0) * 255.0 + 0.5) as u32 };

    let r = to_u8(color[0]);
    let g = to_u8(color[1]);
    let b = to_u8(color[2]);
    let a = to_u8(color[3]);

    r | (g << 8) | (b << 16) | (a << 24)
}

pub fn sample_albedo(material: VoxelMaterial<'_>, uv: Vec2) -> u32 {
    let texture_color = material
        .color_image
        .map(|image| sample_image(image, uv))
        .unwrap_or([255, 255, 255, 255]);

    pack_albedo([
        f32::from(texture_color[0]) / 255.0 * material.color_factor[0],
        f32::from(texture_color[1]) / 255.0 * material.color_factor[1],
        f32::from(texture_color[2]) / 255.0 * material.color_factor[2],
        f32::from(texture_color[3]) / 255.0 * material.color_factor[3],
    ])
}

fn sample_image(image: &ImageData, uv: Vec2) -> [u8; 4] {
    if image.width == 0 || image.height == 0 {
        return [255, 255, 255, 255];
    }

    let wrap = |coordinate: f32| {
        let wrapped = coordinate.fract();
        if wrapped < 0.0 { wrapped + 1.0 } else { wrapped }
    };

    let x = (wrap(uv.x) * image.width as f32) as u32 % image.width;
    let y = (wrap(uv.y) * image.height as f32) as u32 % image.height;
    let offset = ((y * image.width + x) * 4) as usize;

    image
        .rgba8
        .get(offset..offset + 4)
        .and_then(|pixel| pixel.try_into().ok())
        .unwrap_or([255, 255, 255, 255])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn albedo_is_packed_as_rgba8() {
        assert_eq!(pack_albedo([1.0, 0.5, 0.0, 1.0]), 0xff00_80ff);
    }

    #[test]
    fn albedo_channels_are_clamped() {
        assert_eq!(pack_albedo([-1.0, 2.0, 0.5, 0.0]), 0x0080_ff00);
    }

    #[test]
    fn texture_albedo_is_multiplied_by_color_factor() {
        let image = ImageData {
            width: 1,
            height: 1,
            rgba8: vec![255, 0, 0, 255],
        };

        let material = VoxelMaterial {
            color_factor: [0.5, 1.0, 1.0, 1.0],
            color_image: Some(&image),
        };

        assert_eq!(sample_albedo(material, Vec2::new(0.5, 0.5)), 0xff00_0080);
    }

    #[test]
    fn malformed_image_data_falls_back_to_white() {
        let image = ImageData {
            width: 1,
            height: 1,
            rgba8: vec![255, 0, 0],
        };

        let material = VoxelMaterial {
            color_factor: DEFAULT_ALBEDO,
            color_image: Some(&image),
        };

        assert_eq!(sample_albedo(material, Vec2::splat(0.5)), 0xffff_ffff);
    }
}
