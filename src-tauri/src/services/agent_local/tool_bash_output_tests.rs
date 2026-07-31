use super::{ShellOutputBuffer, LIVE_PREVIEW_BYTES, PENDING_BYTES, RETAINED_BYTES};

#[test]
fn small_output_is_kept_verbatim() {
    let mut output = ShellOutputBuffer::default();
    output.append(b"hello\nworld");

    assert_eq!(output.rendered(), "hello\nworld");
    assert_eq!(output.take_pending(), "hello\nworld");
    assert_eq!(output.take_pending(), "");
}

#[test]
fn retained_output_keeps_the_beginning_and_end() {
    let mut output = ShellOutputBuffer::default();
    output.append(&vec![b'a'; RETAINED_BYTES]);
    output.append(&vec![b'z'; RETAINED_BYTES]);

    let rendered = output.rendered();
    assert!(rendered.starts_with(&"a".repeat(RETAINED_BYTES / 2)));
    assert!(rendered.ends_with(&"z".repeat(RETAINED_BYTES / 2)));
    assert!(rendered.contains("octets omis"));
}

#[test]
fn pending_and_live_buffers_are_bounded() {
    let mut output = ShellOutputBuffer::default();
    output.append(&vec![b'a'; PENDING_BYTES]);
    output.append(&vec![b'z'; LIVE_PREVIEW_BYTES]);

    let pending = output.take_pending();
    assert!(pending.len() <= PENDING_BYTES + 64);
    assert!(pending.starts_with('a'));
    assert!(pending.ends_with('z'));
    assert!(output.live_preview().len() <= LIVE_PREVIEW_BYTES);
    assert!(output.is_truncated());
}

#[test]
fn invalid_utf8_is_rendered_without_panicking() {
    let mut output = ShellOutputBuffer::default();
    output.append(&[0xff, b'a']);

    assert!(output.rendered().ends_with('a'));
}
