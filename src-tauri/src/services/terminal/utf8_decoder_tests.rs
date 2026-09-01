use super::utf8_decoder::Utf8StreamDecoder;
use super::{owned_session::spawn_reader_with_exit_for_test, PtyChannelEvent};
use std::io::Read;
use std::sync::{Arc, Mutex};

const SAMPLE: &str = "é🦫東京";

#[test]
fn every_split_reconstructs_the_exact_text() {
    let bytes = SAMPLE.as_bytes();

    for split in 1..bytes.len() {
        let mut decoder = Utf8StreamDecoder::new();
        let decoded =
            decoder.push(&bytes[..split]) + &decoder.push(&bytes[split..]) + &decoder.finish();

        assert_eq!(decoded, SAMPLE, "split at byte {split}");
    }
}

#[test]
fn byte_by_byte_reconstructs_the_exact_text() {
    let mut decoder = Utf8StreamDecoder::new();
    let mut decoded = String::new();

    for byte in SAMPLE.as_bytes() {
        decoded.push_str(&decoder.push(std::slice::from_ref(byte)));
    }
    decoded.push_str(&decoder.finish());

    assert_eq!(decoded, SAMPLE);
}

#[test]
fn incomplete_final_suffix_emits_one_replacement() {
    let beaver = "🦫".as_bytes();
    let mut decoder = Utf8StreamDecoder::new();

    assert_eq!(decoder.push(&beaver[..3]), "");
    assert_eq!(decoder.finish(), "\u{fffd}");
}

#[test]
fn invalid_bytes_do_not_drop_the_valid_suffix() {
    let mut decoder = Utf8StreamDecoder::new();

    assert_eq!(decoder.push(b"avant\xffapres"), "avant\u{fffd}apres");
    assert_eq!(decoder.finish(), "");
}

struct OneByteReader {
    bytes: Vec<u8>,
    offset: usize,
}

impl Read for OneByteReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let Some(byte) = self.bytes.get(self.offset) else {
            return Ok(0);
        };
        buffer[0] = *byte;
        self.offset += 1;
        Ok(1)
    }
}

fn collect_reader_events(bytes: &[u8], exit_code: Option<u32>) -> Vec<PtyChannelEvent> {
    let events = Arc::new(Mutex::new(Vec::new()));
    let observed = Arc::clone(&events);
    let (_, reader) = spawn_reader_with_exit_for_test(
        Box::new(OneByteReader {
            bytes: bytes.to_vec(),
            offset: 0,
        }),
        move |event| {
            observed.lock().expect("events lock").push(event);
            Ok(())
        },
        exit_code,
    );
    reader.join().expect("reader thread");
    let events = match Arc::try_unwrap(events) {
        Ok(events) => events,
        Err(_) => panic!("single events owner"),
    };
    events.into_inner().expect("events lock")
}

#[test]
fn reader_uses_one_decoder_across_all_reads() {
    let events = collect_reader_events(SAMPLE.as_bytes(), Some(0));
    let exit = events.last().expect("exit event");
    let data = events[..events.len() - 1]
        .iter()
        .map(|event| event.data.as_str())
        .collect::<String>();

    assert_eq!(data, SAMPLE);
    assert!(exit.is_exit);
    assert_eq!(exit.exit_code, Some(0));
}

#[test]
fn reader_flushes_incomplete_suffix_before_unknown_exit() {
    let events = collect_reader_events(b"ok\xf0\x9f\xa6", None);

    let exit = events.last().expect("exit event");
    let replacement = &events[events.len() - 2];
    let valid = events[..events.len() - 2]
        .iter()
        .map(|event| event.data.as_str())
        .collect::<String>();

    assert_eq!(valid, "ok");
    assert_eq!(replacement.data, "\u{fffd}");
    assert_eq!(replacement.exit_code, None);
    assert!(exit.is_exit);
    assert_eq!(exit.exit_code, None);
}
