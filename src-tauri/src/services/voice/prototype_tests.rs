use std::{
    path::{Component, Path, PathBuf},
    time::Instant,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use super::{
    audio_buffer::AudioBuffer,
    capture::{activity::SpeechClock, session::CaptureSession, stream::open_input_stream},
    download::{
        download_archive, ensure_disk_available, install_archive, installed_receipt, load_catalog,
        verify_file, VoiceCatalogEntry, VoiceEngine, VoiceModelRole,
    },
    model::{
        benchmarks::{Benchmark, BenchmarkStore},
        lifecycle::ModelLifecycle,
        recognizer::{recognize_probe, EffectiveLanguage, ExecutionProfile},
    },
    transcription::transcribe,
    types::{VoiceInputDevice, VoiceUnloadDelay},
};

const DATA_ENV: &str = "VOICE_PROTOTYPE_DATA_DIR";
const CORPUS_ENV: &str = "VOICE_TEST_CORPUS_DIR";
const REPORT_ENV: &str = "VOICE_PROTOTYPE_REPORT";
const PROBE_PCM_ENV: &str = "VOICE_PROBE_PCM";
const MAX_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
const MAX_FIXTURES: usize = 60;
const MAX_CORPUS_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_WAV_BYTES: u64 = 20 * 1024 * 1024;
const MAX_REFERENCE_CHARS: usize = 4_096;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusManifest {
    version: u8,
    dataset: String,
    dataset_revision: String,
    dataset_url: String,
    license: String,
    attribution: String,
    modifications: String,
    fixtures: Vec<Fixture>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    id: String,
    language: String,
    source_id: Option<String>,
    source_ids: Option<Vec<String>>,
    audio: String,
    sha256: String,
    duration_ms: u64,
    reference: String,
    purpose: String,
}

#[derive(Serialize)]
struct PrototypeReport {
    platform: String,
    corpus_revision: String,
    models: Vec<ModelReport>,
}

#[derive(Serialize)]
struct ModelReport {
    model_id: String,
    revision: String,
    profile: ExecutionProfile,
    cold_load_ms: u64,
    hot_load_ms: u64,
    compute_ms: u64,
    audio_ms: u64,
    peak_rss_bytes: u64,
    english_error_rate: f64,
    chinese_error_rate: f64,
    punctuation_rate: f64,
    timestamps_complete: bool,
    assembly_strategy: String,
    long_sliced_supported: bool,
    long_single_error_rate: f64,
    long_sliced_error_rate: Option<f64>,
}

fn prototype_data_dir() -> PathBuf {
    let path =
        PathBuf::from(std::env::var(DATA_ENV).expect("VOICE_PROTOTYPE_DATA_DIR is required"));
    assert!(path.is_absolute());
    std::fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

async fn ensure_installed(entry: &VoiceCatalogEntry, data_dir: &Path) {
    if installed_receipt(entry, data_dir).unwrap().is_some() {
        return;
    }
    ensure_disk_available(data_dir, entry).unwrap();
    let archive = download_archive(
        entry,
        &entry.id,
        data_dir,
        &tokio_util::sync::CancellationToken::new(),
        |_, _| {},
    )
    .await
    .unwrap();
    let owned_entry = entry.clone();
    let owned_data = data_dir.to_path_buf();
    tokio::task::spawn_blocking(move || install_archive(&owned_entry, &archive, &owned_data))
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
#[ignore = "downloads and installs the complete immutable voice catalogue"]
async fn install_complete_catalogue_for_native_probe() {
    let data_dir = prototype_data_dir();
    let catalog = load_catalog(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    for entry in &catalog.entries {
        ensure_installed(entry, &data_dir).await;
        assert!(installed_receipt(entry, &data_dir).unwrap().is_some());
    }
}

#[test]
fn corpus_manifest_is_bounded_licensed_and_complete() {
    let root = corpus_root();
    let manifest = read_manifest(&root);
    assert_eq!(manifest.version, 1);
    assert_eq!(manifest.dataset, "google/fleurs");
    assert_eq!(manifest.license, "CC-BY-4.0");
    assert_eq!(manifest.dataset_revision.len(), 40);
    assert!(manifest.dataset_url.starts_with("https://huggingface.co/"));
    assert!(!manifest.attribution.is_empty());
    assert!(!manifest.modifications.is_empty());
    assert_eq!(language_fixture_count(&manifest, "en"), 10);
    assert_eq!(language_fixture_count(&manifest, "zh"), 10);
    let long = manifest
        .fixtures
        .iter()
        .find(|fixture| fixture.purpose == "assembly")
        .unwrap();
    assert!(long.duration_ms >= 30_000);
    assert_eq!(long.source_ids.as_ref().map(Vec::len), Some(10));
    assert!(long.source_id.is_none());
    for fixture in &manifest.fixtures {
        let pcm = read_wav(&root, fixture);
        assert_eq!(pcm.len() as u64 * 1_000 / 16_000, fixture.duration_ms);
    }
}

#[tokio::test]
#[ignore = "loads all native models and evaluates the licensed external corpus"]
async fn native_model_matrix() {
    let data_dir = prototype_data_dir();
    let corpus_root = corpus_root();
    let manifest = read_manifest(&corpus_root);
    let catalog = load_catalog(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let vad = catalog
        .entries
        .iter()
        .find(|entry| entry.role == VoiceModelRole::Vad)
        .unwrap();
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let lifecycle = ModelLifecycle::new(coordinator.work_supervisor());
    let profile = ExecutionProfile::cpu(4).unwrap();
    let mut reports = Vec::new();

    for asr in catalog
        .entries
        .iter()
        .filter(|entry| entry.role == VoiceModelRole::Asr)
    {
        assert!(installed_receipt(asr, &data_dir).unwrap().is_some());
        assert!(installed_receipt(vad, &data_dir).unwrap().is_some());
        let cold_started = Instant::now();
        let mut lease = lifecycle
            .acquire(
                &data_dir,
                asr,
                vad,
                profile.clone(),
                VoiceUnloadDelay::OnExit,
            )
            .unwrap();
        let cold_load_ms = elapsed_ms(cold_started);
        let evaluation = evaluate_model(&mut lease, asr, &manifest, &corpus_root);
        drop(lease);
        let hot_started = Instant::now();
        let hot = lifecycle
            .acquire(
                &data_dir,
                asr,
                vad,
                profile.clone(),
                VoiceUnloadDelay::OnExit,
            )
            .unwrap();
        let hot_load_ms = elapsed_ms(hot_started);
        drop(hot);
        let benchmark = Benchmark {
            model_id: asr.id.clone(),
            revision: asr.revision.clone(),
            audio_ms: evaluation.audio_ms,
            compute_ms: evaluation.compute_ms.max(1),
            profile: profile.clone(),
        };
        assert!(BenchmarkStore.record(&data_dir, benchmark).unwrap());
        reports.push(ModelReport {
            model_id: asr.id.clone(),
            revision: asr.revision.clone(),
            profile: profile.clone(),
            cold_load_ms,
            hot_load_ms,
            compute_ms: evaluation.compute_ms,
            audio_ms: evaluation.audio_ms,
            peak_rss_bytes: peak_rss_bytes(),
            english_error_rate: evaluation.english.mean(),
            chinese_error_rate: evaluation.chinese.mean(),
            punctuation_rate: evaluation.punctuated as f64 / evaluation.short_count as f64,
            timestamps_complete: evaluation.timestamps_complete,
            assembly_strategy: if evaluation.timestamps_complete {
                "timestamp-overlap".into()
            } else {
                "quiet-no-overlap".into()
            },
            long_sliced_supported: evaluation.long_sliced_error_rate.is_some(),
            long_single_error_rate: evaluation.long_single_error_rate,
            long_sliced_error_rate: evaluation.long_sliced_error_rate,
        });
    }
    lifecycle.begin_closing();
    assert!(
        lifecycle
            .stop_and_wait(Instant::now() + std::time::Duration::from_secs(5))
            .await
    );
    write_report(
        &corpus_root,
        &PrototypeReport {
            platform: std::env::consts::OS.into(),
            corpus_revision: manifest.dataset_revision,
            models: reports,
        },
    );
}

#[tokio::test]
#[ignore = "loads native Parakeet and opens the real microphone"]
async fn native_parakeet_opens_microphone_under_two_seconds() {
    let data_dir = prototype_data_dir();
    let catalog = load_catalog(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let asr = catalog
        .entries
        .iter()
        .find(|entry| entry.id == "parakeet-tdt-v3-int8")
        .unwrap();
    let vad = catalog
        .entries
        .iter()
        .find(|entry| entry.role == VoiceModelRole::Vad)
        .unwrap();
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let lifecycle = ModelLifecycle::new(coordinator.work_supervisor());
    let started = Instant::now();
    let preparation = lifecycle
        .acquire_for_capture(
            &data_dir,
            asr,
            vad,
            ExecutionProfile::cpu(8).unwrap(),
            VoiceUnloadDelay::OnExit,
        )
        .unwrap();
    let vad_ready_ms = elapsed_ms(started);
    let stream = open_input_stream(&VoiceInputDevice::SystemDefault).unwrap();
    stream.start().unwrap();
    let stream_started_ms = elapsed_ms(started);
    let first_audio_deadline = Instant::now() + std::time::Duration::from_secs(2);
    loop {
        if !stream.ring().drain().samples.is_empty() {
            break;
        }
        assert!(
            Instant::now() < first_audio_deadline,
            "microphone stayed empty"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let listening_ready_ms = elapsed_ms(started);
    assert!(
        listening_ready_ms < 2_000,
        "microphone ready: {listening_ready_ms} ms"
    );
    let lease = preparation.into_ready().unwrap();
    let asr_ready_ms = elapsed_ms(started);
    eprintln!(
        "vad_ready_ms={vad_ready_ms} stream_started_ms={stream_started_ms} listening_ready_ms={listening_ready_ms} asr_ready_ms={asr_ready_ms}"
    );
    drop(stream);
    drop(lease);
    lifecycle.begin_closing();
    assert!(
        lifecycle
            .stop_and_wait(Instant::now() + std::time::Duration::from_secs(5))
            .await
    );
}

#[tokio::test]
#[ignore = "loads native Parakeet and opens the real microphone"]
async fn native_capture_finish_keeps_pending_microphone_samples() {
    let data_dir = prototype_data_dir();
    let catalog = load_catalog(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let asr = catalog
        .entries
        .iter()
        .find(|entry| entry.id == "parakeet-tdt-v3-int8")
        .unwrap();
    let vad = catalog
        .entries
        .iter()
        .find(|entry| entry.role == VoiceModelRole::Vad)
        .unwrap();
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let lifecycle = ModelLifecycle::new(coordinator.work_supervisor());
    let lease = lifecycle
        .acquire(
            &data_dir,
            asr,
            vad,
            ExecutionProfile::cpu(8).unwrap(),
            VoiceUnloadDelay::OnExit,
        )
        .unwrap();
    let capture = CaptureSession::open(&VoiceInputDevice::SystemDefault, false, true, 1).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let (audio, _) = capture.finish(lease.prepared().vad()).unwrap();
    assert!(
        !audio.samples().is_empty(),
        "pending microphone samples were discarded"
    );
    drop(lease);
    lifecycle.begin_closing();
    assert!(
        lifecycle
            .stop_and_wait(Instant::now() + std::time::Duration::from_secs(5))
            .await
    );
}

#[test]
#[ignore = "loads native Parakeet and a caller-provided 16 kHz signed PCM probe"]
fn native_parakeet_transcribes_after_leading_silence() {
    let data_dir = prototype_data_dir();
    let catalog = load_catalog(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
    let asr = catalog
        .entries
        .iter()
        .find(|entry| entry.id == "parakeet-tdt-v3-int8")
        .unwrap();
    let vad = catalog
        .entries
        .iter()
        .find(|entry| entry.role == VoiceModelRole::Vad)
        .unwrap();
    let coordinator = crate::app_exit::AppExitCoordinator::initialize().unwrap();
    let lifecycle = ModelLifecycle::new(coordinator.work_supervisor());
    let mut lease = lifecycle
        .acquire(
            &data_dir,
            asr,
            vad,
            ExecutionProfile::cpu(8).unwrap(),
            VoiceUnloadDelay::Immediately,
        )
        .unwrap();
    let bytes = std::fs::read(std::env::var(PROBE_PCM_ENV).unwrap()).unwrap();
    let (pairs, remainder) = bytes.as_chunks::<2>();
    assert!(remainder.is_empty());
    let pcm: Vec<i16> = pairs.iter().map(|pair| i16::from_le_bytes(*pair)).collect();
    let waveform = as_waveform(&pcm);
    let detector = lease.prepared().vad();
    detector.reset();
    let mut clock = SpeechClock::default();
    for chunk in waveform.chunks(800) {
        clock.observe_vad(detector, chunk, false);
    }
    clock.observe_vad(detector, &[], true);
    let mut audio = AudioBuffer::default();
    audio.append(&pcm, 0).unwrap();
    audio
        .trim_to(
            clock
                .inference_range(audio.samples().len())
                .expect("speech range"),
        )
        .unwrap();
    let mut input = as_waveform(audio.samples());
    let result =
        recognize_probe(lease.prepared_mut(), &input, &EffectiveLanguage::Automatic).unwrap();
    input.zeroize();
    eprintln!(
        "speech_ms={} kept_ms={} text={:?}",
        clock.spoken_ms(),
        audio.samples().len() * 1_000 / 16_000,
        result.text
    );
    assert!(!result.text.trim().is_empty());
}

struct Evaluation {
    english: Rates,
    chinese: Rates,
    punctuated: usize,
    short_count: usize,
    timestamps_complete: bool,
    long_single_error_rate: f64,
    long_sliced_error_rate: Option<f64>,
    audio_ms: u64,
    compute_ms: u64,
}

#[derive(Default)]
struct Rates(Vec<f64>);

impl Rates {
    fn mean(&self) -> f64 {
        self.0.iter().sum::<f64>() / self.0.len() as f64
    }
}

fn evaluate_model(
    lease: &mut super::model::lifecycle::ModelLease,
    entry: &VoiceCatalogEntry,
    manifest: &CorpusManifest,
    root: &Path,
) -> Evaluation {
    let mut english = Rates::default();
    let mut chinese = Rates::default();
    let mut punctuated = 0;
    let mut short_count = 0;
    let mut timestamps_complete = true;
    let mut audio_ms = 0_u64;
    let compute_started = Instant::now();
    for fixture in manifest
        .fixtures
        .iter()
        .filter(|fixture| fixture.purpose == "language")
    {
        let pcm = read_wav(root, fixture);
        let mut waveform = as_waveform(&pcm);
        let result = recognize_probe(
            lease.prepared_mut(),
            &waveform,
            &effective_language(entry.engine, &fixture.language),
        )
        .unwrap();
        waveform.zeroize();
        let rate = error_rate(&fixture.reference, &result.text, &fixture.language);
        if fixture.language == "zh" {
            chinese.0.push(rate);
        } else {
            english.0.push(rate);
        }
        punctuated += usize::from(result.text.chars().any(is_punctuation));
        timestamps_complete &= result.timestamps_ms.is_some();
        short_count += 1;
        audio_ms = audio_ms.saturating_add(fixture.duration_ms);
    }
    let long = manifest
        .fixtures
        .iter()
        .find(|fixture| fixture.purpose == "assembly")
        .unwrap();
    let pcm = read_wav(root, long);
    let mut waveform = as_waveform(&pcm);
    let single = recognize_probe(
        lease.prepared_mut(),
        &waveform,
        &effective_language(entry.engine, &long.language),
    )
    .unwrap();
    waveform.zeroize();
    let long_single_error_rate = error_rate(&long.reference, &single.text, &long.language);
    let mut audio = AudioBuffer::default();
    audio.append(&pcm, 0).unwrap();
    let mut sliced = transcribe(
        lease,
        &mut audio,
        &effective_language(entry.engine, &long.language),
    )
    .ok();
    let long_sliced_error_rate = sliced
        .as_ref()
        .map(|result| error_rate(&long.reference, &result.text, &long.language));
    if let Some(result) = &mut sliced {
        result.text.zeroize();
    }
    audio_ms = audio_ms.saturating_add(long.duration_ms.saturating_mul(2));
    if !timestamps_complete {
        audio_ms = audio_ms.saturating_add(long.duration_ms.min(31_000));
    }
    let compute_ms = elapsed_ms(compute_started);
    Evaluation {
        english,
        chinese,
        punctuated,
        short_count,
        timestamps_complete,
        long_single_error_rate,
        long_sliced_error_rate,
        audio_ms,
        compute_ms,
    }
}

fn corpus_root() -> PathBuf {
    let path = PathBuf::from(std::env::var(CORPUS_ENV).expect("VOICE_TEST_CORPUS_DIR is required"));
    assert!(path.is_absolute());
    path.canonicalize().unwrap()
}

fn read_manifest(root: &Path) -> CorpusManifest {
    use crate::services::private_store::BoundedFile;

    let path = root.join("manifest.json");
    let bytes =
        match crate::services::private_store::read_bounded_regular(&path, MAX_MANIFEST_BYTES)
            .unwrap()
        {
            BoundedFile::Content(bytes) => bytes,
            BoundedFile::Missing => panic!("missing voice corpus manifest"),
        };
    let manifest: CorpusManifest = serde_json::from_slice(&bytes).unwrap();
    assert!((1..=MAX_FIXTURES).contains(&manifest.fixtures.len()));
    let mut total_bytes = 0_u64;
    for fixture in &manifest.fixtures {
        assert!(valid_label(&fixture.id, 64));
        assert!(matches!(fixture.language.as_str(), "en" | "zh"));
        assert!(fixture.source_id.is_some() ^ fixture.source_ids.is_some());
        assert!(fixture.source_ids.as_ref().is_none_or(|ids| {
            !ids.is_empty() && ids.len() <= 20 && ids.iter().all(|id| valid_label(id, 64))
        }));
        assert!(fixture
            .source_id
            .as_ref()
            .is_none_or(|id| valid_label(id, 64)));
        assert!((1..=10 * 60 * 1_000).contains(&fixture.duration_ms));
        assert!((1..=MAX_REFERENCE_CHARS).contains(&fixture.reference.chars().count()));
        assert!(matches!(fixture.purpose.as_str(), "language" | "assembly"));
        assert_eq!(fixture.sha256.len(), 64);
        assert!(fixture.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()));
        let relative = Path::new(&fixture.audio);
        assert!(!relative.is_absolute());
        assert!(relative
            .components()
            .all(|component| matches!(component, Component::Normal(_))));
        let path = root.join(relative);
        let canonical = path.canonicalize().unwrap();
        assert!(canonical.starts_with(root));
        let bytes = canonical.metadata().unwrap().len();
        total_bytes = total_bytes.checked_add(bytes).unwrap();
        assert!(total_bytes <= MAX_CORPUS_BYTES);
        verify_file(&canonical, bytes, &fixture.sha256).unwrap();
    }
    manifest
}

fn valid_label(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn language_fixture_count(manifest: &CorpusManifest, language: &str) -> usize {
    manifest
        .fixtures
        .iter()
        .filter(|fixture| fixture.purpose == "language" && fixture.language == language)
        .count()
}

fn read_wav(root: &Path, fixture: &Fixture) -> Vec<i16> {
    use crate::services::private_store::BoundedFile;

    let path = root.join(&fixture.audio);
    let mut bytes =
        match crate::services::private_store::read_bounded_regular(&path, MAX_WAV_BYTES).unwrap() {
            BoundedFile::Content(bytes) => bytes,
            BoundedFile::Missing => panic!("missing voice corpus fixture"),
        };
    assert_eq!(hex::encode(Sha256::digest(&bytes)), fixture.sha256);
    assert!(bytes.len() >= 44 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE");
    let mut cursor = 12_usize;
    let mut valid_format = false;
    let mut pcm = None;
    while cursor.checked_add(8).is_some_and(|end| end <= bytes.len()) {
        let id = &bytes[cursor..cursor + 4];
        let size = u32::from_le_bytes(bytes[cursor + 4..cursor + 8].try_into().unwrap()) as usize;
        let start = cursor + 8;
        let end = start
            .checked_add(size)
            .filter(|end| *end <= bytes.len())
            .unwrap();
        if id == b"fmt " {
            valid_format = size >= 16
                && u16::from_le_bytes(bytes[start..start + 2].try_into().unwrap()) == 1
                && u16::from_le_bytes(bytes[start + 2..start + 4].try_into().unwrap()) == 1
                && u32::from_le_bytes(bytes[start + 4..start + 8].try_into().unwrap()) == 16_000
                && u32::from_le_bytes(bytes[start + 8..start + 12].try_into().unwrap()) == 32_000
                && u16::from_le_bytes(bytes[start + 12..start + 14].try_into().unwrap()) == 2
                && u16::from_le_bytes(bytes[start + 14..start + 16].try_into().unwrap()) == 16;
        } else if id == b"data" {
            assert!(size > 0 && size.is_multiple_of(2));
            pcm = Some(
                bytes[start..end]
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|sample| i16::from_le_bytes([sample[0], sample[1]]))
                    .collect(),
            );
        }
        cursor = end.checked_add(size % 2).unwrap();
    }
    bytes.zeroize();
    assert!(valid_format);
    pcm.unwrap()
}

fn as_waveform(pcm: &[i16]) -> Vec<f32> {
    pcm.iter()
        .map(|sample| f32::from(*sample) / 32_768.0)
        .collect()
}

fn effective_language(engine: VoiceEngine, language: &str) -> EffectiveLanguage {
    if engine == VoiceEngine::CohereTranscribe {
        EffectiveLanguage::Language(language.into())
    } else {
        EffectiveLanguage::Automatic
    }
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

#[cfg(unix)]
fn peak_rss_bytes() -> u64 {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) } != 0 {
        return 0;
    }
    let usage = unsafe { usage.assume_init() };
    #[cfg(target_os = "macos")]
    return u64::try_from(usage.ru_maxrss).unwrap_or(0);
    #[cfg(not(target_os = "macos"))]
    u64::try_from(usage.ru_maxrss)
        .unwrap_or(0)
        .saturating_mul(1_024)
}

#[cfg(windows)]
fn peak_rss_bytes() -> u64 {
    0
}

fn write_report(corpus_root: &Path, report: &PrototypeReport) {
    let path =
        PathBuf::from(std::env::var(REPORT_ENV).expect("VOICE_PROTOTYPE_REPORT is required"));
    assert!(path.is_absolute());
    let parent = path.parent().unwrap().canonicalize().unwrap();
    assert_eq!(parent, corpus_root);
    let bytes = serde_json::to_vec_pretty(report).unwrap();
    assert!(bytes.len() <= MAX_MANIFEST_BYTES as usize);
    crate::services::private_store::atomic_write(&path, &bytes).unwrap();
}

fn is_punctuation(character: char) -> bool {
    matches!(
        character,
        '.' | ',' | '!' | '?' | ';' | ':' | '。' | '！' | '？' | '；' | '：'
    )
}

fn error_rate(reference: &str, transcript: &str, language: &str) -> f64 {
    let reference = units(reference, language);
    let transcript = units(transcript, language);
    if reference.is_empty() {
        return f64::from(!transcript.is_empty());
    }
    let mut previous: Vec<usize> = (0..=transcript.len()).collect();
    let mut current = vec![0; transcript.len() + 1];
    for (row, expected) in reference.iter().enumerate() {
        current[0] = row + 1;
        for (column, actual) in transcript.iter().enumerate() {
            current[column + 1] = (previous[column + 1] + 1)
                .min(current[column] + 1)
                .min(previous[column] + usize::from(expected != actual));
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[transcript.len()] as f64 / reference.len() as f64
}

fn units(text: &str, language: &str) -> Vec<String> {
    if language == "zh" {
        text.chars()
            .filter(|character| !character.is_whitespace() && !is_punctuation(*character))
            .take(MAX_REFERENCE_CHARS)
            .map(|character| character.to_lowercase().collect())
            .collect()
    } else {
        text.split_whitespace()
            .take(MAX_REFERENCE_CHARS)
            .filter_map(|word| {
                let normalized: String = word
                    .chars()
                    .filter(|character| character.is_alphanumeric())
                    .flat_map(char::to_lowercase)
                    .collect();
                (!normalized.is_empty()).then_some(normalized)
            })
            .collect()
    }
}

#[cfg(test)]
mod unit_tests {
    #[test]
    fn error_rate_handles_words_and_han_characters() {
        assert_eq!(super::error_rate("one two", "one two", "en"), 0.0);
        assert_eq!(super::error_rate("one two", "one", "en"), 0.5);
        assert_eq!(super::error_rate("你好。", "你好", "zh"), 0.0);
        assert_eq!(super::error_rate("你好", "你", "zh"), 0.5);
    }
}
