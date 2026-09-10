use std::path::Path;
use tokio::fs::{remove_file, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use xxhash_rust::xxh3::Xxh3;

/// Streams a file from `src` to `dest`, hashing its contents as it goes.
/// Only reads the source file once, doing both operations in a single pass.
///
/// If `no_preserve` is set, the source file is deleted after transfer.
/// Returns the xxh3-128 content hash and the number of bytes transferred.
pub async fn hash_and_transfer(src: &Path, dest: &Path, no_preserve: bool) -> Result<(String, u64), std::io::Error> {
    let mut src_file = File::open(src).await?;
    let mut dest_file = File::create(dest).await?;
    let mut hasher = Xxh3::new();
    let mut total_bytes = 0u64;
    let mut buf = vec![0u8; 128 * 1024]; // 128KB chunks

    loop {
        let n = match src_file.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                let _ = remove_file(dest).await;
                return Err(e);
            }
        };
        hasher.update(&buf[..n]);
        if let Err(e) = dest_file.write_all(&buf[..n]).await {
            let _ = remove_file(dest).await;
            return Err(e);
        }
        total_bytes += n as u64;
    }

    if let Err(e) = dest_file.flush().await {
        let _ = remove_file(dest).await;
        return Err(e);
    }

    if no_preserve {
        remove_file(src).await?;
    }

    Ok((format!("{:032x}", hasher.digest128()), total_bytes))
}
