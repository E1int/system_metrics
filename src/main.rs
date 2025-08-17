use std::{env, path::PathBuf};

use amdgpu_sysfs::gpu_handle::GpuHandle;
use axum::{Json, Router, routing::get};
use bytesize::ByteSize;
use serde::Serialize;
use sysinfo::System;

#[derive(Serialize)]
struct Response {
    cpu_usage_cores: Vec<f32>,
    cpu_usage_total: f32,
    gpu_usage: f32,
    gpu_vram_total: String,
    gpu_vram_usage: f32,
    gpu_vram_used: String,
    ram_total: String,
    ram_usage: f32,
    ram_used: String,
    swap_total: String,
    swap_usage: f32,
    swap_used: String,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let port = &args[1];
    let address = format!("127.0.0.1:{port}");
    println!("Listening on {}", address);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    let app = Router::new().route("/", get(root));
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> Json<Response> {
    let mut s = System::new();
    s.refresh_memory();

    s.refresh_cpu_usage();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    s.refresh_cpu_usage();

    let gpu_path = PathBuf::from("/sys/class/drm/card1/device");
    let gpu_handle = GpuHandle::new_from_path(gpu_path).unwrap();
    let gpu_vram_total = gpu_handle.get_total_vram().unwrap();
    let gpu_vram_used = gpu_handle.get_used_vram().unwrap();

    let response = Response {
        cpu_usage_cores: s.cpus().iter().map(|c| c.cpu_usage()).collect(),
        cpu_usage_total: s.global_cpu_usage() / 100.0,
        gpu_usage: gpu_handle.get_busy_percent().unwrap() as f32 / 100.0,
        gpu_vram_total: ByteSize::b(gpu_vram_total).display().iec().to_string(),
        gpu_vram_usage: gpu_vram_used as f32 / gpu_vram_total as f32,
        gpu_vram_used: ByteSize::b(gpu_vram_used).display().iec().to_string(),
        ram_total: ByteSize::b(s.total_memory()).display().iec().to_string(),
        ram_usage: s.used_memory() as f32 / s.total_memory() as f32,
        ram_used: ByteSize::b(s.used_memory()).display().iec().to_string(),
        swap_total: ByteSize::b(s.total_swap()).display().iec().to_string(),
        swap_usage: s.used_swap() as f32 / s.total_swap() as f32,
        swap_used: ByteSize::b(s.used_swap()).display().iec().to_string(),
    };

    Json(response)
}
