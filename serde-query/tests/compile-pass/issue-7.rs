use serde_query::DeserializeQuery;

#[derive(Debug, DeserializeQuery)]
pub struct DeviceRoot {
    #[query(".device.friendlyName")]
    pub device_name: String,
}

fn main() {}
