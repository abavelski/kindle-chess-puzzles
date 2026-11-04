use chessnut_ble_probe::{
    discovery::{validate_database, GattRecord},
    ipc::*,
};
fn database() -> Vec<GattRecord> {
    [
        (POSITION_SERVICE, POSITION_UUID, 0x10),
        (COMMAND_SERVICE, WRITE_UUID, 0x0c),
        (COMMAND_SERVICE, RESPONSE_UUID, 0x10),
    ]
    .map(|(service, uuid, properties)| GattRecord {
        service: service.into(),
        uuid: uuid.into(),
        properties,
        handle: 1,
    })
    .to_vec()
}
fn reversed(uuid: &str) -> String {
    let bytes: Vec<_> = uuid
        .replace('-', "")
        .as_bytes()
        .chunks(2)
        .map(|c| std::str::from_utf8(c).unwrap().to_owned())
        .rev()
        .collect();
    let raw = bytes.join("");
    format!(
        "{}-{}-{}-{}-{}",
        &raw[..8],
        &raw[8..12],
        &raw[12..16],
        &raw[16..20],
        &raw[20..]
    )
}
#[test]
fn discovery_matches_all_bytes_parent_services_and_properties_in_either_order() {
    assert_eq!(validate_database(&database()).unwrap(), "forward");
    let mut reversed_db = database();
    for record in &mut reversed_db {
        record.service = reversed(&record.service);
        record.uuid = reversed(&record.uuid);
    }
    assert_eq!(validate_database(&reversed_db).unwrap(), "reverse");
    for index in 0..3 {
        let mut records = database();
        records[index].properties = 0;
        assert!(validate_database(&records).is_err());
        let mut records = database();
        records[index].service = POSITION_UUID.into();
        assert!(validate_database(&records).is_err());
        let mut records = database();
        records[index].uuid.replace_range(7..8, "f");
        assert!(validate_database(&records).is_err());
    }
}
#[test]
fn discovery_refuses_mixed_order_duplicates_missing_and_invalid_records() {
    let mut records = database();
    records[0].uuid = reversed(&records[0].uuid);
    assert!(validate_database(&records).is_err());
    let mut records = database();
    records.push(records[0].clone());
    assert!(validate_database(&records).is_err());
    assert!(validate_database(&database()[..2]).is_err());
    let mut records = database();
    records[0].handle = 0;
    assert!(validate_database(&records).is_err());
}
