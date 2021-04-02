#[derive(serde_query::Deserialize)]
struct NodesDocument {
    #[query(".nodes")]
    nodes: Vec<Node>,
    #[query(".hash")]
    version: i64,
}

#[derive(serde_query::Deserialize)]
struct Node {
    #[query(".nodeinfo.node_id")]
    node_id:String,
    #[query(".nodeinfo.hostname")]
    hostname: String,
    #[query(".nodeinfo.hardware.model")]
    hardwaremodel: String,
    #[query(".nodeinfo.software.firmware.base")]
    basefirmware: String,
    #[query(".nodeinfo.software.firmware.release")]
    releasefirmware: String,
    #[query(".nodeinfo.network.adresses")]
    adresses: Vec<String>,
}

fn main() {}