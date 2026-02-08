use tokio::io::AsyncWriteExt as _;

pub async fn compress(in_data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut encoder = async_compression::tokio::write::BrotliEncoder::with_quality(
        Vec::new(),
        async_compression::Level::Best,
    );
    // https://github.com/DioxusLabs/dioxus/blob/09c1de7574abb36b11a2c8c825ac30d7398de948/packages/core/src/tasks.rs#L288
    for chunk in in_data.chunks(10 * 1024).enumerate() {
        encoder.write_all(chunk.1).await?; // hangs, move to worker?
        #[cfg(target_arch = "wasm32")]
        sleep(std::time::Duration::from_millis(0)).await;
    }
    encoder.shutdown().await?;
    Ok(encoder.into_inner())
}

pub async fn decompress(in_data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut decoder = async_compression::tokio::write::BrotliDecoder::new(Vec::new());
    decoder.write_all(in_data).await?;
    decoder.shutdown().await?;
    Ok(decoder.into_inner())
}
