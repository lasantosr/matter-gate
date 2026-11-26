use alloc::vec;

use super::*;
use crate::{
    error::EndpointError,
    model::{
        AttributeId, AttributeValue, Cluster, ClusterId, DataVersion, DeviceTypeEntry, DeviceTypeId, Endpoint,
        EndpointId, MatterStatus, Nullable, Optional,
    },
};

#[test]
fn decode_list_success() {
    let val = AttributeValue::List(vec![AttributeValue::U8(1), AttributeValue::U8(2)]);
    let decoded = decode_list(val, |item| match item {
        AttributeValue::U8(v) => Ok(v),
        _ => Err(MatterStatus::InvalidDataType),
    })
    .unwrap();
    assert_eq!(decoded, vec![1, 2]);
}

#[test]
fn decode_list_item_error() {
    let val = AttributeValue::List(vec![AttributeValue::U8(1), AttributeValue::U16(2)]);
    let res = decode_list(val, |item| match item {
        AttributeValue::U8(v) => Ok(v),
        _ => Err(MatterStatus::InvalidDataType),
    });
    assert_eq!(res, Err(MatterStatus::InvalidDataType));
}

#[test]
fn decode_list_not_a_list() {
    let val = AttributeValue::Bool(true);
    let res = decode_list(val, Ok::<_, MatterStatus>);
    assert_eq!(res, Err(MatterStatus::InvalidDataType));
}

#[test]
fn decode_nullable_works() {
    let res_null = decode_nullable(AttributeValue::Null, |_| Ok::<_, MatterStatus>(42_u32)).unwrap();
    assert_eq!(res_null, Nullable::Null);

    let res_some = decode_nullable(AttributeValue::U32(100), |item| match item {
        AttributeValue::U32(v) => Ok(v),
        _ => Err(MatterStatus::InvalidDataType),
    })
    .unwrap();
    assert_eq!(res_some, Nullable::NonNull(100));
}

#[test]
fn decode_nullable_list_works() {
    let res_null: Nullable<Vec<u8>> =
        decode_nullable_list(AttributeValue::Null, |_| Ok::<_, MatterStatus>(0_u8)).unwrap();
    assert_eq!(res_null, Nullable::Null);

    let list_val = AttributeValue::List(vec![AttributeValue::U8(10), AttributeValue::U8(20)]);
    let res_list = decode_nullable_list(list_val, |item| match item {
        AttributeValue::U8(v) => Ok(v),
        _ => Err(MatterStatus::InvalidDataType),
    })
    .unwrap();
    assert_eq!(res_list, Nullable::NonNull(vec![10, 20]));

    let err = decode_nullable_list(AttributeValue::U32(42), |_| Ok::<_, MatterStatus>(0_u8));
    assert_eq!(err, Err(MatterStatus::InvalidDataType));
}

#[test]
fn validate_device_view_checks_presence_and_conformance() {
    let dt_id = DeviceTypeId::new(0x0100);
    let disallowed = [ClusterId::new(0x0406)];
    let mandatory_srv = [ClusterId::new(0x0006)];
    let mandatory_cli = [ClusterId::new(0x0019)];

    let empty = Endpoint::empty(EndpointId::new(1));
    assert_eq!(
        validate_device_view(&empty, dt_id, &disallowed, &mandatory_srv, &mandatory_cli),
        Err(EndpointError::EmptyDeviceTypes)
    );

    let wrong_dt = Endpoint::single_device(EndpointId::new(1), DeviceTypeId::new(0x0101), 1, vec![]);
    assert_eq!(
        validate_device_view(&wrong_dt, dt_id, &disallowed, &mandatory_srv, &mandatory_cli),
        Err(EndpointError::DeviceTypeMismatch {
            expected: dt_id,
            actual: DeviceTypeId::new(0x0101),
        })
    );

    let with_disallowed = Endpoint::single_device(
        EndpointId::new(1),
        dt_id,
        1,
        vec![
            Cluster::empty(ClusterId::new(0x0406)),
            Cluster::empty(ClusterId::new(0x0006)),
        ],
    )
    .with_client_list([ClusterId::new(0x0019)]);
    assert_eq!(
        validate_device_view(&with_disallowed, dt_id, &disallowed, &mandatory_srv, &mandatory_cli),
        Err(EndpointError::DisallowedCluster(ClusterId::new(0x0406)))
    );

    let missing_srv =
        Endpoint::single_device(EndpointId::new(1), dt_id, 1, vec![]).with_client_list([ClusterId::new(0x0019)]);
    assert_eq!(
        validate_device_view(&missing_srv, dt_id, &disallowed, &mandatory_srv, &mandatory_cli),
        Err(EndpointError::MissingMandatoryServerCluster(ClusterId::new(0x0006)))
    );

    let missing_cli = Endpoint::single_device(
        EndpointId::new(1),
        dt_id,
        1,
        vec![Cluster::empty(ClusterId::new(0x0006))],
    );
    assert_eq!(
        validate_device_view(&missing_cli, dt_id, &disallowed, &mandatory_srv, &mandatory_cli),
        Err(EndpointError::MissingMandatoryClientCluster(ClusterId::new(0x0019)))
    );

    let valid = Endpoint::single_device(
        EndpointId::new(1),
        dt_id,
        1,
        vec![Cluster::empty(ClusterId::new(0x0006))],
    )
    .with_client_list([ClusterId::new(0x0019)]);
    validate_device_view(&valid, dt_id, &disallowed, &mandatory_srv, &mandatory_cli).unwrap();
}

#[test]
fn update_descriptor_cluster_preserves_metadata() {
    let dt = DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 2);
    let server = Cluster::empty(ClusterId::new(0x0006));
    let client = ClusterId::new(0x0019);

    let mut prev = Cluster::empty(ClusterId::DESCRIPTOR).with_data_version(DataVersion::new(9));
    prev.set_attribute(
        AttributeId::new(0x0003),
        AttributeValue::List(vec![AttributeValue::U16(2)]),
    );
    prev.set_attribute(AttributeId::new(0x0005), AttributeValue::String("ep-1".into()));
    prev.set_attribute(AttributeId::CLUSTER_REVISION, AttributeValue::U16(3));

    let desc = update_descriptor_cluster(Some(&prev), &[dt], &[server], &[client]);
    assert_eq!(desc.id, ClusterId::DESCRIPTOR);
    assert_eq!(desc.data_version, Optional::Present(DataVersion::new(9)));
    assert_eq!(
        desc.attribute(AttributeId::new(0x0003)),
        Some(&AttributeValue::List(vec![AttributeValue::U16(2)]))
    );
    assert_eq!(
        desc.attribute(AttributeId::new(0x0005)),
        Some(&AttributeValue::String("ep-1".into()))
    );
    assert_eq!(
        desc.attribute(AttributeId::CLUSTER_REVISION),
        Some(&AttributeValue::U16(3))
    );
}

#[test]
fn update_descriptor_cluster_deduplicates_descriptor_and_handles_composed_device_types() {
    let dts = [
        DeviceTypeEntry::new(DeviceTypeId::new(0x0100), 3),
        DeviceTypeEntry::new(DeviceTypeId::new(0x0013), 2),
    ];
    let existing = Cluster::empty(ClusterId::DESCRIPTOR);
    let server_clusters = [
        existing.clone(),
        Cluster::empty(ClusterId::new(0x0006)),
        Cluster::empty(ClusterId::new(0x0039)),
    ];
    let client_clusters = [ClusterId::new(0x0019)];

    let mut prev = Cluster::empty(ClusterId::DESCRIPTOR).with_data_version(DataVersion::new(12));
    prev.set_attribute(AttributeId::new(0x0004), AttributeValue::List(vec![]));
    prev.set_attribute(AttributeId::ACCEPTED_COMMAND_LIST, AttributeValue::List(vec![]));
    prev.set_attribute(AttributeId::FEATURE_MAP, AttributeValue::U32(0x01));

    let desc = update_descriptor_cluster(Some(&prev), &dts, &server_clusters, &client_clusters);
    assert_eq!(desc.data_version, Optional::Present(DataVersion::new(12)));
    assert_eq!(
        desc.attribute(AttributeId::new(0x0004)),
        Some(&AttributeValue::List(vec![]))
    );
    assert_eq!(
        desc.attribute(AttributeId::ACCEPTED_COMMAND_LIST),
        Some(&AttributeValue::List(vec![]))
    );
    assert_eq!(
        desc.attribute(AttributeId::FEATURE_MAP),
        Some(&AttributeValue::U32(0x01))
    );

    let Some(AttributeValue::List(dt_list)) = desc.attribute(AttributeId::new(0x0000)) else {
        panic!("expected DeviceTypeList");
    };
    assert_eq!(dt_list.len(), 2);

    let Some(AttributeValue::List(srv_list)) = desc.attribute(AttributeId::new(0x0001)) else {
        panic!("expected ServerList");
    };
    // Descriptor (0x001D) appears only once even though it was present in `server_clusters`
    assert_eq!(
        srv_list,
        &vec![
            AttributeValue::from(ClusterId::DESCRIPTOR),
            AttributeValue::from(ClusterId::new(0x0006)),
            AttributeValue::from(ClusterId::new(0x0039)),
        ]
    );

    let Some(AttributeValue::List(cli_list)) = desc.attribute(AttributeId::new(0x0002)) else {
        panic!("expected ClientList");
    };
    assert_eq!(cli_list, &vec![AttributeValue::from(ClusterId::new(0x0019))]);
}
