use super::stdio_line::read_bounded_line;
use tokio::io::{AsyncWriteExt, BufReader};

#[tokio::test]
async fn rejects_a_line_before_it_can_exceed_the_limit() {
    let (mut writer, reader) = tokio::io::duplex(64);
    let producer = tokio::spawn(async move {
        writer.write_all(&[b'x'; 65]).await.unwrap();
        writer.shutdown().await.unwrap();
    });
    let mut reader = BufReader::with_capacity(16, reader);
    assert!(read_bounded_line(&mut reader, 64).await.is_err());
    producer.await.unwrap();
}

#[tokio::test]
async fn accepts_a_complete_line_at_the_limit() {
    let data = std::io::Cursor::new(b"1234567\n".to_vec());
    let mut reader = BufReader::new(data);
    let line = read_bounded_line(&mut reader, 8).await.unwrap().unwrap();
    assert_eq!(line, b"1234567\n");
}
