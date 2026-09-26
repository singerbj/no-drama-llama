//! Models you can download from the tray or installer, and which one suits this PC.
//!
//! Sizes are from the Hugging Face repos; downloads are still verified against Hugging Face's
//! own size and SHA-256 at download time.

const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

/// VRAM kept free beside the weights for context (KV cache), compute buffers and the 1 GiB
/// margin llama.cpp's --fit leaves. Qwen 3.8 is mostly linear attention, so this still buys a
/// long context.
pub const HEADROOM_GIB: f64 = 5.0;
/// RAM kept for Windows and your other apps when a model spills into system memory.
pub const RAM_RESERVE_GIB: f64 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Every weight is used for every token: must be on the GPU to be fast.
    Dense,
    /// Mixture of experts: only a few GB active per token, so most of it can live in RAM.
    Moe,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogModel {
    /// Stable id, e.g. `qwen3.8-27b:UD-Q4_K_XL` (installer `--model`)
    pub id: String,
    pub family: &'static str,
    pub quant: &'static str,
    pub repo: &'static str,
    /// Paths inside the repo; more than one = split GGUF (llama.cpp loads the first).
    pub files: Vec<String>,
    pub size: u64,
    pub kind: Kind,
}

impl CatalogModel {
    /// File name in the models folder that llama.cpp loads (the first shard).
    pub fn primary_file(&self) -> String {
        file_name(&self.files[0]).to_string()
    }

    pub fn local_files(&self) -> Vec<String> {
        self.files
            .iter()
            .map(|f| file_name(f).to_string())
            .collect()
    }

    pub fn size_gib(&self) -> f64 {
        self.size as f64 / GIB
    }

    pub fn label(&self) -> String {
        format!(
            "{} {}  ({:.1} GB)",
            self.family,
            self.quant,
            self.size as f64 / 1e9
        )
    }
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

const DENSE_27B: [(&str, u64); 9] = [
    ("Q8_0", 29_047_086_048),
    ("UD-Q6_K_XL", 25_299_061_664),
    ("UD-Q5_K_XL", 20_876_938_144),
    ("UD-Q4_K_XL", 17_559_178_144),
    ("UD-IQ4_XS", 14_252_845_984),
    ("UD-Q3_K_XL", 13_146_393_504),
    ("UD-IQ3_XXS", 10_934_860_704),
    ("UD-Q2_K_XL", 9_828_981_664),
    ("UD-IQ2_XXS", 7_266_070_528),
];

/// (quant, total bytes, shards)
const FLASH_NEXT: [(&str, u64, u32); 7] = [
    ("UD-Q4_K_XL", 111_300_000_000, 4),
    ("UD-IQ4_XS", 93_700_000_000, 3),
    ("UD-Q3_K_XL", 90_000_000_000, 3),
    ("UD-IQ3_XXS", 82_000_000_000, 3),
    ("UD-Q2_K_XL", 78_900_000_000, 3),
    ("UD-IQ1_M", 74_500_000_000, 3),
    ("UD-IQ1_S", 72_500_000_000, 3),
];

/// Lowest dense quant still worth recommending when a better model is possible.
const DENSE_MIN_GOOD: &str = "UD-IQ3_XXS";

/// Every model, best quality first within each family.
pub fn catalog() -> Vec<CatalogModel> {
    let mut v: Vec<CatalogModel> = DENSE_27B
        .iter()
        .map(|(q, size)| CatalogModel {
            id: format!("qwen3.8-27b:{q}"),
            family: "Qwen 3.8 27B",
            quant: q,
            repo: "unsloth/Qwen3.8-27B-GGUF",
            files: vec![format!("Qwen3.8-27B-{q}.gguf")],
            size: *size,
            kind: Kind::Dense,
        })
        .collect();
    v.extend(FLASH_NEXT.iter().map(|(q, size, shards)| {
        CatalogModel {
            id: format!("qwen3.8-flash-next:{q}"),
            family: "Qwen 3.8 Flash-Next (125B MoE)",
            quant: q,
            repo: "unsloth/Qwen3.8-Flash-Next-GGUF",
            files: (1..=*shards)
                .map(|i| format!("{q}/Qwen3.8-Flash-Next-{q}-{i:05}-of-{shards:05}.gguf"))
                .collect(),
            size: *size,
            kind: Kind::Moe,
        }
    }));
    v
}

pub fn find(id: &str) -> Option<CatalogModel> {
    catalog()
        .into_iter()
        .find(|m| m.id.eq_ignore_ascii_case(id))
}

/// The catalog entry a models-folder file belongs to, if any.
pub fn by_primary_file(file: &str) -> Option<CatalogModel> {
    catalog()
        .into_iter()
        .find(|m| m.primary_file().eq_ignore_ascii_case(file))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    /// Whole model on the GPU with room for a long context: fast.
    Gpu,
    /// Part of it in system RAM. Fine for MoE models, slow for dense ones.
    GpuAndRam,
    /// Needs more memory than the PC has.
    TooBig,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Machine {
    /// Primary GPU's memory (0 = no usable GPU)
    pub vram: u64,
    pub ram: u64,
}

impl Machine {
    fn vram_gib(&self) -> f64 {
        self.vram as f64 / GIB
    }
    fn ram_gib(&self) -> f64 {
        self.ram as f64 / GIB
    }
}

pub fn fit(m: &CatalogModel, pc: &Machine) -> Fit {
    if m.size_gib() + HEADROOM_GIB <= pc.vram_gib() {
        Fit::Gpu
    } else if m.size_gib() + HEADROOM_GIB
        <= pc.vram_gib() + (pc.ram_gib() - RAM_RESERVE_GIB).max(0.0)
    {
        Fit::GpuAndRam
    } else {
        Fit::TooBig
    }
}

/// Human-readable note for the menu.
pub fn fit_note(m: &CatalogModel, pc: &Machine) -> &'static str {
    match (fit(m, pc), m.kind) {
        (Fit::Gpu, _) => "fits your GPU",
        (Fit::GpuAndRam, Kind::Moe) => "GPU + RAM",
        (Fit::GpuAndRam, Kind::Dense) => "partly in RAM - slow",
        (Fit::TooBig, _) => "too big for this PC",
    }
}

/// The best model for this PC:
/// 1. the best Qwen 3.8 27B quant (IQ3_XXS or better) that fits on the GPU;
/// 2. else Flash-Next, if the PC has the RAM for it;
/// 3. else the best 27B quant that fits on the GPU at all;
/// 4. else the smallest 27B that fits in GPU + RAM (slow, but it runs).
pub fn recommend(pc: &Machine) -> Option<CatalogModel> {
    let all = catalog();
    let dense: Vec<&CatalogModel> = all.iter().filter(|m| m.kind == Kind::Dense).collect();
    let min_good = dense
        .iter()
        .position(|m| m.quant == DENSE_MIN_GOOD)
        .unwrap();
    if let Some(m) = dense[..=min_good].iter().find(|m| fit(m, pc) == Fit::Gpu) {
        return Some((*m).clone());
    }
    if let Some(m) = all
        .iter()
        .filter(|m| m.kind == Kind::Moe)
        .find(|m| fit(m, pc) != Fit::TooBig)
    {
        return Some(m.clone());
    }
    if let Some(m) = dense.iter().find(|m| fit(m, pc) == Fit::Gpu) {
        return Some((*m).clone());
    }
    dense
        .iter()
        .rev()
        .find(|m| fit(m, pc) != Fit::TooBig)
        .map(|m| (*m).clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pc(vram_gib: f64, ram_gib: f64) -> Machine {
        Machine {
            vram: (vram_gib * GIB) as u64,
            ram: (ram_gib * GIB) as u64,
        }
    }

    fn rec(vram: f64, ram: f64) -> String {
        recommend(&pc(vram, ram))
            .map(|m| m.id)
            .unwrap_or_else(|| "none".into())
    }

    #[test]
    fn recommendations_by_gpu() {
        // (VRAM as llama.cpp reports it, RAM) -> model
        let cases = [
            (47.98, 64.0, "qwen3.8-27b:Q8_0"), // RTX 6000 Ada / W7900 48 GB
            (31.84, 64.0, "qwen3.8-27b:UD-Q6_K_XL"), // RTX 5090 32 GB
            (23.98, 32.0, "qwen3.8-27b:UD-Q4_K_XL"), // RX 7900 XTX / RTX 4090 24 GB (the original default)
            (19.98, 32.0, "qwen3.8-27b:UD-IQ4_XS"),  // RX 7900 XT 20 GB
            (15.99, 32.0, "qwen3.8-27b:UD-IQ3_XXS"), // RTX 4080 / RX 7800 XT 16 GB
            (11.99, 128.0, "qwen3.8-flash-next:UD-Q4_K_XL"), // 12 GB card + 128 GB RAM: the MoE model
            (11.99, 32.0, "qwen3.8-27b:UD-IQ2_XXS"),         // 12 GB, normal RAM
            (7.99, 32.0, "qwen3.8-27b:UD-IQ2_XXS"),          // 8 GB: partly in RAM, slow
            (0.0, 16.0, "none"),                             // no GPU, little RAM
        ];
        for (vram, ram, want) in cases {
            assert_eq!(rec(vram, ram), want, "{vram} GiB VRAM, {ram} GiB RAM");
        }
    }

    #[test]
    fn original_24gb_default_is_preserved() {
        assert_eq!(
            recommend(&pc(23.98, 32.0)).unwrap().primary_file(),
            crate::settings::DEFAULT_MODEL
        );
    }

    #[test]
    fn fit_ratings() {
        let q4 = find("qwen3.8-27b:UD-Q4_K_XL").unwrap();
        assert_eq!(fit(&q4, &pc(24.0, 32.0)), Fit::Gpu);
        assert_eq!(fit(&q4, &pc(12.0, 32.0)), Fit::GpuAndRam);
        assert_eq!(fit(&q4, &pc(0.0, 8.0)), Fit::TooBig);
        assert_eq!(fit_note(&q4, &pc(12.0, 32.0)), "partly in RAM - slow");
        let moe = find("qwen3.8-flash-next:UD-IQ1_S").unwrap();
        assert_eq!(fit_note(&moe, &pc(24.0, 96.0)), "GPU + RAM");
        assert_eq!(fit_note(&moe, &pc(24.0, 32.0)), "too big for this PC");
    }

    #[test]
    fn more_memory_never_gives_a_worse_model() {
        let rank = |id: &str| catalog().iter().position(|m| m.id == id);
        let mut last_dense_size = 0;
        for tenth in 0..=500 {
            let v = tenth as f64 / 10.0;
            if let Some(m) = recommend(&pc(v, 32.0)) {
                if m.kind == Kind::Dense && fit(&m, &pc(v, 32.0)) == Fit::Gpu {
                    assert!(
                        m.size >= last_dense_size,
                        "{v} GiB: {} after a bigger one",
                        m.id
                    );
                    last_dense_size = m.size;
                }
                assert!(rank(&m.id).is_some());
            }
        }
    }

    #[test]
    fn catalog_is_consistent() {
        let all = catalog();
        let ids: std::collections::HashSet<_> = all.iter().map(|m| m.id.clone()).collect();
        assert_eq!(ids.len(), all.len(), "unique ids");
        for m in &all {
            assert!(m.files.iter().all(|f| f.ends_with(".gguf")));
            assert!(
                crate::settings::is_valid_model_name(&m.primary_file()),
                "{}",
                m.primary_file()
            );
            assert!(m.size > 1_000_000_000);
            if m.files.len() > 1 {
                assert!(
                    m.primary_file().contains("-00001-of-"),
                    "{}",
                    m.primary_file()
                );
                let n = m.files.len();
                assert!(m
                    .files
                    .last()
                    .unwrap()
                    .ends_with(&format!("-{n:05}-of-{n:05}.gguf")));
            }
        }
        // sizes are best-first within a family
        for fam in ["Qwen 3.8 27B", "Qwen 3.8 Flash-Next (125B MoE)"] {
            let sizes: Vec<u64> = all
                .iter()
                .filter(|m| m.family == fam)
                .map(|m| m.size)
                .collect();
            assert!(sizes.windows(2).all(|w| w[0] >= w[1]), "{fam}");
        }
    }

    #[test]
    fn lookup() {
        assert_eq!(find("QWEN3.8-27B:ud-q4_k_xl").unwrap().quant, "UD-Q4_K_XL");
        assert!(find("gpt-5").is_none());
        let m = by_primary_file("Qwen3.8-Flash-Next-UD-IQ1_S-00001-of-00003.gguf").unwrap();
        assert_eq!(m.local_files().len(), 3);
        assert_eq!(
            m.files[1],
            "UD-IQ1_S/Qwen3.8-Flash-Next-UD-IQ1_S-00002-of-00003.gguf"
        );
        assert!(by_primary_file("my-own-model.gguf").is_none());
        assert!(find("qwen3.8-27b:UD-Q4_K_XL")
            .unwrap()
            .label()
            .contains("17.6 GB"));
    }
}
