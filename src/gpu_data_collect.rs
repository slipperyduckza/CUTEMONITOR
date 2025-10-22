use machine_info::Machine;

pub fn get_gpu_status() -> String {
    if crate::what_cpu_check::is_virtual_machine() {
        "Virtual environment detected".to_string()
    } else {
        let machine = Machine::new();
        let graphics = machine.graphics_status();
        if let Some(usage) = graphics.first() {
            format!(
                "GPU Utilization: {}%\nGPU Memory usage: {} MB\nTemperature: {}°C",
                usage.gpu,
                usage.memory_used / 1024 / 1024,
                usage.temperature
            )
        } else {
            "No GPU detected".to_string()
        }
    }
}