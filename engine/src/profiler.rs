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

pub fn get_unified_usage() -> Result<u64, hypomnesis::HypomnesisError> {
    let snap = Snapshot::now(0)?;
    
    let mb_vram = snap.gpu
            .filter(|g| g.is_per_process)
            .map(|g| g.used_bytes)
            .unwrap_or(0);
        let mb_vram = ((mb_vram as f64 / (1024.0 * 1024.0)) * 10.0).round() as u64;

    let mb_ram: u64 = (((snap.ram_bytes as f64 / (1024.0 * 1024.0)) * 10.0).round() / 10.0) as u64;

    let mb_unified: u64 = mb_ram + mb_vram;
    
    Ok(mb_unified)
}