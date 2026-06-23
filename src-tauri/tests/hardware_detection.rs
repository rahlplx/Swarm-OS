use swarm_os_lib::hardware::profiler::detect_hardware_default;

#[test]
fn real_hardware_detection() {
    let profile = detect_hardware_default();

    assert!(profile.cpu_cores > 0, "must detect at least 1 CPU core");
    assert!(profile.ram_total_bytes > 0, "must detect non-zero RAM");
    assert!(!profile.cpu_name.is_empty(), "CPU name must not be empty");
    assert!(!profile.os.is_empty(), "OS info must not be empty");
}

#[test]
fn capability_score_from_real_hardware() {
    let profile = detect_hardware_default();
    let score = swarm_os_lib::hardware::capability::compute_capability(&profile);

    assert!(score.total > 0.0, "score must be positive on any machine");
    assert!(score.ram_score > 0.0, "RAM score must be positive");
    assert!(score.cpu_score > 0.0, "CPU score must be positive");
}
