//! Bounded GATT snapshots, validated independently of native structure layouts.
use crate::ipc::{COMMAND_SERVICE, POSITION_SERVICE, POSITION_UUID, RESPONSE_UUID, WRITE_UUID};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GattRecord {
    pub service: String,
    pub uuid: String,
    pub properties: u8,
    pub handle: u16,
}
fn normalized(uuid: &str, reverse: bool) -> Option<String> {
    if [4, 8].contains(&uuid.len()) && uuid.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Some(uuid.to_ascii_lowercase());
    }
    if uuid.len() != 36 || !uuid.is_ascii() {
        return None;
    }
    for (i, b) in uuid.bytes().enumerate() {
        if if [8, 13, 18, 23].contains(&i) {
            b != b'-'
        } else {
            !b.is_ascii_hexdigit()
        } {
            return None;
        }
    }
    let raw = uuid.replace('-', "").to_ascii_lowercase();
    let raw = if reverse {
        raw.as_bytes()
            .chunks_exact(2)
            .rev()
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect::<String>()
    } else {
        raw
    };
    Some(format!(
        "{}-{}-{}-{}-{}",
        &raw[..8],
        &raw[8..12],
        &raw[12..16],
        &raw[16..20],
        &raw[20..]
    ))
}
pub fn validate_database(records: &[GattRecord]) -> Result<&'static str, &'static str> {
    if records.is_empty()
        || records.len() > 20
        || records.iter().any(|r| {
            r.handle == 0
                || normalized(&r.service, false).is_none()
                || normalized(&r.uuid, false).is_none()
        })
    {
        return Err("invalid_or_oversized_database");
    }
    for (reverse, order) in [(false, "forward"), (true, "reverse")] {
        let required = [
            (POSITION_SERVICE, POSITION_UUID, 0x10),
            (COMMAND_SERVICE, WRITE_UUID, 0x0c),
            (COMMAND_SERVICE, RESPONSE_UUID, 0x10),
        ];
        if required.iter().all(|(service, uuid, properties)| {
            let found: Vec<_> = records
                .iter()
                .filter(|r| {
                    normalized(&r.service, reverse).as_deref() == Some(service)
                        && normalized(&r.uuid, reverse).as_deref() == Some(uuid)
                })
                .collect();
            found.len() == 1 && found[0].properties & properties == *properties
        }) {
            return Ok(order);
        }
    }
    Err("required_uuid_service_or_properties_mismatch")
}
