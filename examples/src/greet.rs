use clickhouse_wasm_udf::clickhouse_udf;

#[clickhouse_udf]
fn greet(name: String) -> Result<String, String> {
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    Ok(format!("Hello, {name}!"))
}
