use hypomnesis::Snapshot;

pub fn get_ram_usage() -> Result<u64, hypomnesis::HypomnesisError> {
    let snap = Snapshot::now(0)?;

    let mb_ram: u64 = (((snap.ram_bytes as f64 / (1024.0 * 1024.0)) * 10.0).round() / 10.0) as u64;

    Ok(mb_ram)
}

pub fn get_vram_usage() -> Result<Option<u64>, hypomnesis::HypomnesisError> {
    let snap = Snapshot::now(0)?;

    let mb_vram: Option<u64> = snap
        .gpu_device
        .map(|gpu| ((gpu.used_bytes as f64 / 1_048_576.0) * 10.0).round() as u64 / 10);

    Ok(mb_vram)
}
