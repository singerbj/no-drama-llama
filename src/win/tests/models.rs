//! Model downloads (single and split files), verification, cancel + resume, and the probes.

use super::super::{install, models, probe, sys};
use super::hf::{self, file};
use super::{copy_exe, example_exe, long_tempdir};
use crate::catalog;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

fn bytes(n: usize, seed: u8) -> Vec<u8> {
    (0..n)
        .map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed))
        .collect()
}

#[test]
fn downloads_a_single_file_model() {
    let m = catalog::find("qwen3.8-27b:UD-IQ2_XXS").unwrap();
    let data = bytes(300_000, 1);
    let fake = hf::start(vec![file(m.repo, &m.files[0], data.clone())]);
    let (_t, dir) = long_tempdir();
    let mut last = (0, 0);
    models::download_model(
        &fake.base,
        &m,
        &dir,
        &dir.join(".dl"),
        |d, t| last = (d, t),
        None,
    )
    .unwrap();
    assert_eq!(std::fs::read(dir.join(m.primary_file())).unwrap(), data);
    assert_eq!(last, (300_000, 300_000));
    assert!(!dir
        .join(".dl")
        .join(format!("{}.part", m.primary_file()))
        .exists());
    // already there: not downloaded again
    let before = fake.downloads.lock().unwrap().len();
    models::download_model(&fake.base, &m, &dir, &dir.join(".dl"), |_, _| {}, None).unwrap();
    assert_eq!(fake.downloads.lock().unwrap().len(), before);
}

#[test]
fn downloads_every_part_of_a_split_model() {
    let m = catalog::find("qwen3.8-flash-next:UD-IQ1_S").unwrap();
    let files: Vec<_> = m
        .files
        .iter()
        .enumerate()
        .map(|(i, f)| file(m.repo, f, bytes(50_000 + i * 1000, i as u8)))
        .collect();
    let expected: Vec<Vec<u8>> = files.iter().map(|f| f.data.clone()).collect();
    let fake = hf::start(files);
    let (_t, dir) = long_tempdir();
    let mut progress = Vec::new();
    models::download_model(
        &fake.base,
        &m,
        &dir,
        &dir.join(".dl"),
        |d, t| progress.push((d, t)),
        None,
    )
    .unwrap();
    for (name, data) in m.local_files().iter().zip(&expected) {
        assert_eq!(&std::fs::read(dir.join(name)).unwrap(), data, "{name}");
    }
    let total: u64 = expected.iter().map(|d| d.len() as u64).sum();
    assert!(
        progress.iter().all(|(_, t)| *t == total),
        "progress covers all parts"
    );
    assert!(progress.windows(2).all(|w| w[0].0 <= w[1].0), "monotonic");
    let listed = crate::paths::group_models(
        std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| {
                (
                    e.file_name().to_string_lossy().into_owned(),
                    e.metadata().unwrap().len(),
                )
            })
            .collect(),
    );
    assert_eq!(
        listed,
        vec![(m.primary_file(), total)],
        "shows as one model"
    );
    assert!(crate::paths::missing_parts(&dir, &m.primary_file()).is_empty());
}

#[test]
fn bad_checksum_deletes_the_file() {
    let m = catalog::find("qwen3.8-27b:UD-IQ2_XXS").unwrap();
    let mut f = file(m.repo, &m.files[0], bytes(10_000, 2));
    f.sha = Some("0".repeat(64));
    let fake = hf::start(vec![f]);
    let (_t, dir) = long_tempdir();
    let e = models::download_model(&fake.base, &m, &dir, &dir.join(".dl"), |_, _| {}, None)
        .unwrap_err();
    assert!(format!("{e:#}").contains("checksum"), "{e:#}");
    assert!(!dir.join(m.primary_file()).exists());
    assert!(!dir
        .join(".dl")
        .join(format!("{}.part", m.primary_file()))
        .exists());
}

#[test]
fn cancel_keeps_the_partial_file_and_resume_finishes_it() {
    let m = catalog::find("qwen3.8-27b:UD-IQ2_XXS").unwrap();
    let data = bytes(8_000_000, 3);
    let fake = hf::start(vec![file(m.repo, &m.files[0], data.clone())]);
    let (_t, dir) = long_tempdir();
    let cancel = AtomicBool::new(true);
    let e = models::download_model(
        &fake.base,
        &m,
        &dir,
        &dir.join(".dl"),
        |_, _| {},
        Some(&cancel),
    )
    .unwrap_err();
    assert_eq!(e.to_string(), super::super::net::CANCELLED);
    assert!(!dir.join(m.primary_file()).exists());
    // simulate a partial download, then resume
    std::fs::write(
        dir.join(".dl").join(format!("{}.part", m.primary_file())),
        &data[..3_000_000],
    )
    .unwrap();
    models::download_model(&fake.base, &m, &dir, &dir.join(".dl"), |_, _| {}, None).unwrap();
    assert_eq!(std::fs::read(dir.join(m.primary_file())).unwrap(), data);
}

#[test]
fn missing_file_on_the_server_fails_cleanly() {
    let m = catalog::find("qwen3.8-27b:UD-IQ2_XXS").unwrap();
    let fake = hf::start(vec![]);
    let (_t, dir) = long_tempdir();
    let e = models::download_model(&fake.base, &m, &dir, &dir.join(".dl"), |_, _| {}, None)
        .unwrap_err();
    assert!(format!("{e:#}").contains("not found"), "{e:#}");
}

#[test]
fn probes_read_the_llama_cpp_build() {
    let (_t, dir) = long_tempdir();
    let exe = dir.join("llama").join("llama-server.exe");
    copy_exe(&example_exe("fake_llama_server"), &exe);
    let devs = probe::devices(&exe);
    assert_eq!(devs.len(), 1);
    assert_eq!(devs[0].description, "Fake GPU 9000");
    assert!(probe::server_caps(&exe).fit);
    let (pc, name) = probe::machine(&exe);
    assert_eq!(pc.vram, 16384 * 1024 * 1024);
    assert!(pc.ram > 1 << 30, "RAM detected");
    assert_eq!(name.as_deref(), Some("Fake GPU 9000 (16 GB)"));
    assert!(probe::devices(&dir.join("missing.exe")).is_empty());
    let _ = probe::nvidia_gpus(); // no NVIDIA on CI: must just be empty
}

#[test]
fn run_with_timeout_kills_hung_tools() {
    let (_t, dir) = long_tempdir();
    let sleeper = dir.join("hung-tool.exe");
    copy_exe(&example_exe("fake_llama_server"), &sleeper);
    let t = std::time::Instant::now();
    assert!(sys::run_with_timeout(&sleeper, &[], Duration::from_millis(500)).is_err());
    assert!(t.elapsed() < Duration::from_secs(5));
    let out = sys::run_with_timeout(
        std::path::Path::new("cmd.exe"),
        &["/c", "echo", "hi", "&", "exit", "3"],
        Duration::from_secs(10),
    )
    .unwrap();
    assert!(out.contains("hi"), "output kept even on a non-zero exit");
}

#[test]
fn cuda_runtime_zip_lands_next_to_the_server() {
    use std::io::Write;
    let (_t, dir) = long_tempdir();
    let zip_path = dir.join("cudart.zip");
    let mut w = zip::ZipWriter::new(std::fs::File::create(&zip_path).unwrap());
    let o =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for n in ["cudart64_13.dll", "sub/cublas64_13.dll"] {
        w.start_file(n, o).unwrap();
        w.write_all(b"dll").unwrap();
    }
    w.finish().unwrap();
    let target = dir.join("llama");
    std::fs::create_dir_all(&target).unwrap();
    std::fs::write(target.join("llama-server.exe"), b"keep me").unwrap();
    install::extract_zip_into(&zip_path, &target).unwrap();
    assert!(target.join("cudart64_13.dll").exists());
    assert!(target.join("cublas64_13.dll").exists(), "flattened");
    assert_eq!(
        std::fs::read(target.join("llama-server.exe")).unwrap(),
        b"keep me",
        "existing files kept"
    );
    assert!(!target.join(".extract").exists());
}

#[test]
fn installed_backend_is_remembered() {
    let (_t, dir) = long_tempdir();
    let p = crate::paths::Paths::under(&dir);
    assert_eq!(install::installed_backend(&p), None);
    std::fs::create_dir_all(&p.llama_dir).unwrap();
    std::fs::write(p.llama_dir.join("backend.txt"), "cuda13").unwrap();
    assert_eq!(
        install::installed_backend(&p),
        Some(crate::hardware::Backend::Cuda13)
    );
}
