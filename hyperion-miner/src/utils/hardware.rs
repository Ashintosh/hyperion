use num_cpus;

pub fn detect_optimal_threads() -> usize {
    let cpu_count = num_cpus::get();
    
    // Leave one core free for system tasks
    if cpu_count > 2 {
        cpu_count - 1
    } else {
        cpu_count
    }
}

pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        cpu_cores: num_cpus::get(),
        optimal_threads: detect_optimal_threads(),
    }
}

#[derive(Debug)]
pub struct SystemInfo {
    pub cpu_cores: usize,
    pub optimal_threads: usize,
}