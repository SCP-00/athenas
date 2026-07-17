/// TUI — Terminal dashboard for Athena Laboratory.
///
/// Muestra en tiempo real:
/// - Hardware status
/// - Queue de experimentos
/// - Live experiment progress
/// - Knowledge Base stats
///
/// No usa librerías externas (solo `colored` que ya está en dependencias).
/// Diseñado para SSH, servidores headless, y laboratorios remotos.
use std::path::Path;
use std::time::SystemTime;

use crate::runtime::experiment_queue::{ExperimentQueue, ExperimentStatus};
use crate::runtime::knowledge_base::KnowledgeBase;

/// Renderizar el dashboard completo en terminal
pub fn render_dashboard() {
    let state_dir = Path::new(".state");

    // Hardware
    let hw = crate::runtime::hardware::detect_hardware();

    // Queue
    let queue = ExperimentQueue::load(state_dir);
    let queue_counts = queue.count_by_status();

    // Knowledge Base
    let kb = KnowledgeBase::load(state_dir);

    // GPU status
    let gpu_util = get_gpu_util();
    let gpu_temp = get_gpu_temp();
    let vram_used = get_vram_used();
    let vram_total = get_vram_total();

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║              ATHENA — Laboratorio Autónomo                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // ── Hardware ──
    println!("┌─ Hardware ─────────────────────────────────────────────────┐");
    if let Some(gpu) = hw.gpu.first() {
        let gpu_bar = progress_bar(gpu_util, 20);
        let vram_pct = if vram_total > 0.0 { (vram_used / vram_total) * 100.0 } else { 0.0 };
        let vram_bar = progress_bar(vram_pct, 20);
        println!("│ GPU    : {} ({:.0} GB)                            │", gpu.model, gpu.vram_gb);
        println!("│ Util   : {gpu_bar} {gpu_util:.0}%  Temp: {gpu_temp:.0}°C              │");
        println!("│ VRAM   : {vram_bar} {vram_used:.1}/{vram_total:.0} GB                 │");
    }
    println!("│ RAM    : Available {:.1} GB / {:.0} GB           │", hw.memory.available_gb, hw.memory.total_gb);
    println!("│ CPU    : {} ({} threads)                               │", hw.cpu.model, hw.cpu.threads);
    println!("└──────────────────────────────────────────────────────────┘");
    println!();

    // ── Queue ──
    println!("┌─ Experiment Queue ─────────────────────────────────────────┐");
    let all = queue.list(None);
    if all.is_empty() {
        println!("│  Queue is empty. Use: ath queue add --model <path>        │");
    } else {
        let queued = queue_counts.get("Queued").copied().unwrap_or(0);
        let running = queue_counts.get("Running").copied().unwrap_or(0);
        let completed = queue_counts.get("Completed").copied().unwrap_or(0);
        let failed = queue_counts.get("Failed").copied().unwrap_or(0);
        println!("│  ⏳ Queued: {queued}  🔄 Running: {running}  ✅ Done: {completed}  ❌ Failed: {failed}  │");
        // Show first few experiments
        for exp in all.iter().take(3) {
            let icon = match exp.status {
                ExperimentStatus::Queued => "⏳",
                ExperimentStatus::Running => "🔄",
                ExperimentStatus::Completed => "✅",
                ExperimentStatus::Failed(_) => "❌",
                ExperimentStatus::Blocked(_) => "⛔",
                ExperimentStatus::Cancelled => "🚫",
            };
            let phases = format!("{}/{}", exp.completed_phases.len(), 
                exp.completed_phases.len() + exp.failed_phases.len());
            let model_short = Path::new(&exp.model_path)
                .file_name().map(|s| s.to_string_lossy()).unwrap_or_default();
            println!("│  {icon} {} — {} — {}                │", 
                &exp.id[..exp.id.len().min(16)], model_short, phases);
        }
        if all.len() > 3 {
            println!("│  ... and {} more                                       │", all.len() - 3);
        }
    }
    println!("└──────────────────────────────────────────────────────────┘");
    println!();

    // ── Knowledge Base ──
    println!("┌─ Knowledge Base ──────────────────────────────────────────┐");
    let total_revs = kb.total_revisions();
    let questions = kb.questions();
    println!("│  📚 {} AnswerRevisions, {} questions                    │", total_revs, questions.len());
    for q in questions.iter().take(3) {
        if let Some(latest) = kb.latest(q) {
            let status_icon = match latest.status.as_str() {
                "active" => "🟢",
                "superseded" => "🟡",
                "outdated" => "🔴",
                _ => "⚪",
            };
            let answer_short = if latest.answer.len() > 40 {
                format!("{}...", &latest.answer[..40])
            } else {
                latest.answer.clone()
            };
            println!("│  {status_icon} Rev.{} — {}        │", latest.revision, answer_short);
        }
    }
    if questions.len() > 3 {
        println!("│  ... and {} more questions                              │", questions.len() - 3);
    }
    println!("└──────────────────────────────────────────────────────────┘");
    println!();

    // ── Quick Commands ──
    println!("┌─ Quick Commands ──────────────────────────────────────────┐");
    println!("│  ath queue add --model <path>   — Enqueue experiment      │");
    println!("│  ath queue list                 — View queue              │");
    println!("│  ath queue process              — Process next experiment │");
    println!("│  ath study list                 — List scientific studies │");
    println!("│  ath study SP-005               — Run health check        │");
    println!("│  ath phase list                 — List all 11 phases      │");
    println!("└──────────────────────────────────────────────────────────┘");
    println!();
}

/// Renderizar una view simplificada del dashboard (para refresco rápido)
pub fn render_status_line() -> String {
    let state_dir = Path::new(".state");
    let queue = ExperimentQueue::load(state_dir);
    let counts = queue.count_by_status();

    let queued = counts.get("Queued").copied().unwrap_or(0);
    let running = counts.get("Running").copied().unwrap_or(0);
    let completed = counts.get("Completed").copied().unwrap_or(0);
    let failed = counts.get("Failed").copied().unwrap_or(0);

    let gpu_util = get_gpu_util();
    let vram_used = get_vram_used();
    let vram_total = get_vram_total();

    format!(
        "ATHENA | GPU: {gpu_util:.0}% | VRAM: {vram_used:.1}/{vram_total:.0}GB | Queue: ⏳{queued} 🔄{running} ✅{completed} ❌{failed}"
    )
}

// ── GPU helpers ──

fn get_gpu_util() -> f64 {
    std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=utilization.gpu", "--format=csv,noheader,nounits"])
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0.0)
}

fn get_gpu_temp() -> f64 {
    std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=temperature.gpu", "--format=csv,noheader,nounits"])
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0.0)
}

fn get_vram_used() -> f64 {
    std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=memory.used", "--format=csv,noheader,nounits"])
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<f64>().ok().map(|mb| mb / 1024.0))
        .unwrap_or(0.0)
}

fn get_vram_total() -> f64 {
    std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
        .output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<f64>().ok().map(|mb| mb / 1024.0))
        .unwrap_or(0.0)
}

/// Generar barra de progreso en texto (█▓▒░)
fn progress_bar(pct: f64, width: usize) -> String {
    if pct.is_nan() || pct < 0.0 {
        return "░".repeat(width);
    }
    let pct = pct.min(100.0);
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let filled_chars = "█".repeat(filled);
    let empty_chars = "░".repeat(width.saturating_sub(filled));
    format!("{filled_chars}{empty_chars}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar() {
        let bar = progress_bar(50.0, 10);
        assert_eq!(bar.chars().count(), 10);
        assert!(bar.contains('█'));
        assert!(bar.contains('░'));
    }

    #[test]
    fn test_progress_bar_full() {
        let bar = progress_bar(100.0, 5);
        assert_eq!(bar, "█████");
    }

    #[test]
    fn test_progress_bar_empty() {
        let bar = progress_bar(0.0, 5);
        assert_eq!(bar, "░░░░░");
    }

    #[test]
    fn test_status_line_format() {
        let line = render_status_line();
        assert!(line.contains("ATHENA"));
    }
}
