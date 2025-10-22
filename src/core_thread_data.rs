use sysinfo::System;

pub fn get_core_usages() -> Vec<f32> {
    let mut sys = System::new_all();
    sys.refresh_cpu_all();
    let cores = System::physical_core_count().unwrap_or(1);
    (0..cores).map(|i| sys.cpus()[i].cpu_usage()).collect()
}

pub fn get_thread_usages() -> Vec<f32> {
    let mut sys = System::new_all();
    sys.refresh_cpu_all();
    sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect()
}