//! module for parse btwatch2 info

/// result
#[allow(dead_code)]
pub(crate) struct Measurement {
    pub(crate) relay: u8,
    pub(crate) voltage: f32,
    pub(crate) current: f32,
    pub(crate) power: f32,
}

/// parser
pub(crate) fn parse_manufacturer_data(value: &Vec<u8>) -> Measurement {
    let raw_relay_status = value[0];
    let raw_voltage = u16::from_le_bytes([value[1], value[2]]);
    let raw_current = u16::from_le_bytes([value[3], value[4]]);
    let raw_power = u32::from_le_bytes([value[5], value[6], value[7], 0]);

    return Measurement {
        relay: raw_relay_status,
        voltage: raw_voltage as f32 / 10_f32,
        current: raw_current as f32 / 1000_f32,
        power: raw_power as f32 / 1000_f32,
    };
}
