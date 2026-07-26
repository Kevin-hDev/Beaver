---
name: voxtral-cli
description: Use when generating speech from text (TTS), transcribing audio to text, or sending voice messages via Telegram. Triggers on: voxtral, text-to-speech, audio generation, transcription, speech-to-text, voice message, vocal, vocale.
allowed-tools: Bash(curl:*), Bash(ffmpeg:*), Bash(jq:*), Bash(base64:*)
argument-hint: "[tts 'text'] [transcribe file.mp3] [telegram 'text']"
---

# Voxtral CLI — TTS + Transcription via Mistral API

## Configuration

| Variable | Purpose | Location |
|----------|---------|----------|
| `$VOXTRAL_API_KEY` | Mistral API key | `~/.zshenv` |
| `$TELEGRAM_BOT_TOKEN` | Telegram bot token | `~/.zshenv` |
| `$TELEGRAM_CHAT_ID` | Telegram chat ID | `~/.zshenv` |

Voice reference file: `~/.local/share/cl-go-dash/voice-reference.mp3`
(Beaver keeps this historical storage identifier for compatibility)

---

## Quick Start

### 1. TTS — Generate audio from text

```bash
# Compress voice reference
ffmpeg -y -i ~/.local/share/cl-go-dash/voice-reference.mp3 -ar 16000 -ac 1 -b:a 64k /tmp/voxtral-ref.mp3

# Encode to base64
base64 -i /tmp/voxtral-ref.mp3 > /tmp/voxtral-ref-b64.txt

# Build JSON payload (replace text in --arg text)
jq -n --arg text "Hello, this is a test." \
  --rawfile ref /tmp/voxtral-ref-b64.txt \
  '{model: "voxtral-mini-tts-2603", input: $text, ref_audio: ($ref | gsub("\n"; "")), response_format: "mp3"}' \
  > /tmp/voxtral-tts-payload.json

# Call API and decode audio
curl -s -X POST https://api.mistral.ai/v1/audio/speech \
  -H "Authorization: Bearer $VOXTRAL_API_KEY" \
  -H "Content-Type: application/json" \
  -d @/tmp/voxtral-tts-payload.json \
  | jq -r '.audio_data' | base64 -D > /tmp/voxtral-output.mp3
```

### 2. Transcription — Transcribe audio to text

```bash
curl -s -X POST https://api.mistral.ai/v1/audio/transcriptions \
  -H "Authorization: Bearer $VOXTRAL_API_KEY" \
  -F "file=@/path/to/audio.mp3" \
  -F "model=voxtral-mini-transcribe-2-2602" \
  | jq -r '.text'
```

### 3. Telegram — Send a voice message

```bash
# After TTS (step 1 above), convert to OGG/Opus
ffmpeg -y -i /tmp/voxtral-output.mp3 -c:a libopus -b:a 64k /tmp/voxtral-voice.ogg

# Send via Telegram
curl -s -X POST "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/sendVoice" \
  -F "chat_id=${TELEGRAM_CHAT_ID}" \
  -F "voice=@/tmp/voxtral-voice.ogg"
```

---

## TTS — Details

### Endpoint

`POST https://api.mistral.ai/v1/audio/speech`

### Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `model` | yes | `voxtral-mini-tts-2603` |
| `input` | yes | Text to synthesize |
| `ref_audio` | **yes** | Base64 of voice reference audio — 500 error without it |
| `response_format` | no | `mp3` (default), `wav`, `pcm`, `flac`, `opus` |

### ref_audio constraints

1. **Mandatory** — without `ref_audio`, the API returns a 500 error
2. **Compress before sending** — raw file is too large for base64 inline
3. **Pass JSON via `@file`** — base64 exceeds shell argument length limits
4. Recommended compression: `ffmpeg -ar 16000 -ac 1 -b:a 64k`

### TTS response

```json
{
  "audio_data": "base64_encoded_audio...",
  "usage": {"tts_characters": 42}
}
```

Decode: `jq -r '.audio_data' response.json | base64 -D > output.mp3`

### Error checking

Save response to verify before decoding:

```bash
curl -s -X POST https://api.mistral.ai/v1/audio/speech \
  -H "Authorization: Bearer $VOXTRAL_API_KEY" \
  -H "Content-Type: application/json" \
  -d @/tmp/voxtral-tts-payload.json > /tmp/voxtral-response.json

# Check for errors
jq -e '.audio_data' /tmp/voxtral-response.json > /dev/null 2>&1 \
  && jq -r '.audio_data' /tmp/voxtral-response.json | base64 -D > /tmp/voxtral-output.mp3 \
  || jq '.' /tmp/voxtral-response.json
```

---

## Transcription — Details

### Endpoint

`POST https://api.mistral.ai/v1/audio/transcriptions`

### Model

`voxtral-mini-transcribe-2-2602`

### Optional parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `language` | string | Language code: `fr`, `en`, `es`, `de`, `ja`, etc. |
| `diarize` | bool | `true` to identify speakers |
| `timestamp_granularities` | string | `word` or `segment` |

### With language and diarization

```bash
curl -s -X POST https://api.mistral.ai/v1/audio/transcriptions \
  -H "Authorization: Bearer $VOXTRAL_API_KEY" \
  -F "file=@audio.mp3" \
  -F "model=voxtral-mini-transcribe-2-2602" \
  -F "language=fr" \
  -F "diarize=true" \
  | jq '.'
```

### Transcription response

```json
{
  "text": "Full transcription here",
  "language": "fr",
  "segments": [{"text": "...", "start": 0.0, "end": 2.5}],
  "usage": {"prompt_tokens": 100, "total_seconds": 15.2}
}
```

---

## Telegram — Details

### Required format

Telegram only accepts voice messages in **OGG/Opus** format. Convert:

```bash
ffmpeg -y -i input.mp3 -c:a libopus -b:a 64k output.ogg
```

### Send an existing audio file as voice

```bash
ffmpeg -y -i /path/to/audio.mp3 -c:a libopus -b:a 64k /tmp/voxtral-voice.ogg
curl -s -X POST "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/sendVoice" \
  -F "chat_id=${TELEGRAM_CHAT_ID}" \
  -F "voice=@/tmp/voxtral-voice.ogg"
```

### With caption

```bash
curl -s -X POST "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/sendVoice" \
  -F "chat_id=${TELEGRAM_CHAT_ID}" \
  -F "voice=@/tmp/voxtral-voice.ogg" \
  -F "caption=Voice message generated by Voxtral"
```

---

## Full workflow: text to Telegram voice message

```bash
# 1. Prepare voice reference (reusable across calls)
ffmpeg -y -i ~/.local/share/cl-go-dash/voice-reference.mp3 \
  -ar 16000 -ac 1 -b:a 64k /tmp/voxtral-ref.mp3
base64 -i /tmp/voxtral-ref.mp3 > /tmp/voxtral-ref-b64.txt

# 2. Build TTS payload
jq -n --arg text "Here is the voice message to send on Telegram." \
  --rawfile ref /tmp/voxtral-ref-b64.txt \
  '{model: "voxtral-mini-tts-2603", input: $text, ref_audio: ($ref | gsub("\n"; "")), response_format: "mp3"}' \
  > /tmp/voxtral-tts-payload.json

# 3. Generate audio
curl -s -X POST https://api.mistral.ai/v1/audio/speech \
  -H "Authorization: Bearer $VOXTRAL_API_KEY" \
  -H "Content-Type: application/json" \
  -d @/tmp/voxtral-tts-payload.json \
  | jq -r '.audio_data' | base64 -D > /tmp/voxtral-output.mp3

# 4. Convert to OGG/Opus
ffmpeg -y -i /tmp/voxtral-output.mp3 -c:a libopus -b:a 64k /tmp/voxtral-voice.ogg

# 5. Send via Telegram
curl -s -X POST "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/sendVoice" \
  -F "chat_id=${TELEGRAM_CHAT_ID}" \
  -F "voice=@/tmp/voxtral-voice.ogg"
```

---

## Common pitfalls

| Problem | Cause | Solution |
|---------|-------|----------|
| 500 error on TTS | `ref_audio` missing | Always include base64 of voice reference |
| `Argument list too long` | JSON passed as `-d inline` | Use `-d @/tmp/voxtral-tts-payload.json` |
| Silent/corrupted audio | ref_audio not compressed | `ffmpeg -ar 16000 -ac 1 -b:a 64k` |
| `.audio_data` empty | Undetected API error | Check `.message` in response before decoding |
| Telegram voice unplayable | Wrong audio format | Convert to OGG/Opus: `ffmpeg -c:a libopus` |
| `--rawfile: Unknown option` | jq < 1.6 | Update: `brew install jq` |
