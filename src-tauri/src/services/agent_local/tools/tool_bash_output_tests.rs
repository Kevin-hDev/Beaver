use super::{ShellOutputBuffer, ShellStream, LIVE_PREVIEW_BYTES, PENDING_BYTES, RETAINED_BYTES};

#[test]
fn small_output_is_kept_verbatim() {
    let mut output = ShellOutputBuffer::default();
    output.append(ShellStream::Stdout, b"hello\nworld");

    assert_eq!(output.rendered(), "hello\nworld");
    let pending = output.take_pending();
    assert_eq!(pending.stdout, "hello\nworld");
    assert_eq!(pending.stderr, "");
    assert!(!pending.truncated);
    assert_eq!(output.take_pending().stdout, "");
}

#[test]
fn retained_output_keeps_the_beginning_and_end() {
    let mut output = ShellOutputBuffer::default();
    output.append(ShellStream::Stdout, &vec![b'a'; RETAINED_BYTES]);
    output.append(ShellStream::Stdout, &vec![b'z'; RETAINED_BYTES]);

    let rendered = output.rendered();
    assert!(rendered.starts_with(&"a".repeat(RETAINED_BYTES / 2)));
    assert!(rendered.ends_with(&"z".repeat(RETAINED_BYTES / 2)));
    assert!(rendered.contains("octets omis"));
}

#[test]
fn pending_and_live_buffers_are_bounded() {
    let mut output = ShellOutputBuffer::default();
    output.append(ShellStream::Stdout, &vec![b'a'; PENDING_BYTES]);
    output.append(ShellStream::Stderr, &vec![b'z'; LIVE_PREVIEW_BYTES]);

    let pending = output.take_pending();
    assert!(pending.stdout.len() + pending.stderr.len() <= PENDING_BYTES + 128);
    assert!(pending.stdout.starts_with('a'));
    assert!(pending.stderr.ends_with('z'));
    assert!(pending.truncated);
    assert!(output.live_preview().len() <= LIVE_PREVIEW_BYTES);
    assert!(!output.take_pending().truncated);
}

#[test]
fn invalid_utf8_is_rendered_without_panicking() {
    let mut output = ShellOutputBuffer::default();
    output.append(ShellStream::Stdout, &[0xff, b'a']);

    assert!(output.rendered().ends_with('a'));
}

#[test]
fn streams_are_kept_separate_even_without_newlines() {
    let mut output = ShellOutputBuffer::default();
    output.append(ShellStream::Stdout, b"out");
    output.append(ShellStream::Stderr, b"error");
    output.append(ShellStream::Stdout, b"tail");

    let pending = output.take_pending();

    assert_eq!(pending.stdout, "outtail");
    assert_eq!(pending.stderr, "error");
}
