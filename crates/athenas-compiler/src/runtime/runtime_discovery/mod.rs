pub mod capability;
pub mod prober;

pub use capability::RuntimeCapabilities;
pub use prober::RuntimeProber;

/// Run a full runtime discovery and return sorted list by capability score
pub fn discover_runtimes() -> Vec<RuntimeCapabilities> {
    RuntimeProber::probe_all()
}

/// Display discovered runtimes in a human-readable table
pub fn display_runtimes(runtimes: &[RuntimeCapabilities]) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "╔══════════════════════════════════════════╗\n\
         ║     Runtime Discovery Results            ║\n\
         ╚══════════════════════════════════════════╝\n\n"
    ));

    if runtimes.is_empty() {
        s.push_str("⚠️  No runtimes found.\n");
        return s;
    }

    for (i, rt) in runtimes.iter().enumerate() {
        s.push_str(&format!("[{}/{}] {}\n", i + 1, runtimes.len(), rt.display_name_short()));
        s.push_str(&format!("      Binary: {}\n", rt.binary_path));
        s.push_str(&format!("      Score:  {:.3}\n", rt.capability_score()));
        s.push_str(&format!("      Caps:   flash_attn={} cuda={} kv_quant={} embed={} vision={} grammar={} spec={} bonsai={}\n",
            rt.supports_flash_attention,
            rt.supports_cuda,
            rt.supports_kv_cache_quant,
            rt.supports_embeddings,
            rt.supports_vision,
            rt.supports_grammar,
            rt.supports_speculative_decoding,
            rt.supports_bonsai,
        ));
        if !rt.special_binaries.is_empty() {
            s.push_str(&format!("      Special: {:?}\n", rt.special_binaries));
        }
        if !rt.kv_cache_types.is_empty() {
            s.push_str(&format!("      KV Types: {:?}\n", rt.kv_cache_types));
        }
        s.push('\n');
    }
    s
}
