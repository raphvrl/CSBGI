use glam::Vec3;

pub const DEFAULT_NORMAL: u32 = 0x0080_ff80;

pub fn pack_normal(normal: Vec3) -> u32 {
    let normal = normal.normalize_or_zero();

    if normal == Vec3::ZERO {
        return DEFAULT_NORMAL;
    }

    let x = encode_component(normal.x);
    let y = encode_component(normal.y);
    let z = encode_component(normal.z);

    x | (y << 8) | (z << 16)
}

pub fn unpack_normal(packed: u32) -> Vec3 {
    let normal = Vec3::new(
        decode_component(packed & 0xff),
        decode_component((packed >> 8) & 0xff),
        decode_component((packed >> 16) & 0xff),
    );

    normal.normalize_or_zero()
}

fn encode_component(value: f32) -> u32 {
    (((value.clamp(-1.0, 1.0) * 0.5) + 0.5) * 255.0).round() as u32
}

fn decode_component(value: u32) -> f32 {
    value as f32 / 255.0 * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_normal_round_trips_direction() {
        let normal = Vec3::new(0.35, 0.8, -0.45).normalize();
        let unpacked = unpack_normal(pack_normal(normal));

        assert!(normal.dot(unpacked) > 0.9999);
    }

    #[test]
    fn zero_normal_uses_default_direction() {
        let normal = unpack_normal(pack_normal(Vec3::ZERO));

        assert!(normal.dot(Vec3::Y) > 0.9999);
    }
}
