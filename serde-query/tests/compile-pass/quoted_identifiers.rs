#[derive(serde_query::Deserialize)]
struct Cluster {
    #[query(r#".["kubernetes_clusters"].id"#)]
    cluster_id: String,
    #[query(r#".["kubernetes_clusters"].name"#)]
    cluster_name: String,
}

fn assert_deserialize<'de, D: serde::Deserialize<'de>>() {}

fn main() {
    assert_deserialize::<Cluster>();
}