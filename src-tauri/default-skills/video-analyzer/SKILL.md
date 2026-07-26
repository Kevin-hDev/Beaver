---
name: video-analyzer
description: Use when analyzing, watching, or transcribing videos. Triggers on: video analysis, YouTube, transcribe video, extract frames, watch video, analyze video, video summary.
---

# Video Analyzer — Download, transcribe, and extract visual content from videos

Workflow to make video content accessible to Claude: download subtitles or audio,
transcribe, extract key frames, then read and analyze everything.

## Configuration

| Variable | Purpose | Location |
|----------|---------|----------|
| `$VOXTRAL_API_KEY` | Mistral API key for transcription | `~/.zshenv` |

**Output directory:** `~/.local/share/cl-go-dash/assets/video-analyzer/`
(Beaver keeps this historical storage identifier for compatibility)

**Required tools:**
- `yt-dlp` (brew install yt-dlp) — video/audio download
- `deno` (brew install deno) — required by yt-dlp for YouTube since Nov 2025
- `ffmpeg` (brew install ffmpeg) — frame extraction and audio conversion
- `whisper` (uv tool install openai-whisper) — local transcription fallback

---

## Quick Start

```bash
# 1. Check tools are installed
yt-dlp --version && ffmpeg -version | head -1 && deno --version | head -1

# 2. Try subtitles first (instant, no download needed)
yt-dlp --skip-download --write-auto-subs --sub-lang fr,en --convert-subs srt \
  -o "~/.local/share/cl-go-dash/assets/video-analyzer/DD-MM-YYYY/video-title/transcript" "URL"

# 3. If no subs, download audio and transcribe
yt-dlp -x --audio-format wav -o "{output-dir}/audio.wav" "URL"

# 4. Download video (720p max) and extract frames
yt-dlp -f "bv[height<=720]+ba" --merge-output-format mp4 -o "{output-dir}/video.mp4" "URL"
ffmpeg -i video.mp4 -vf fps=1/60 {output-dir}/frames/frame_%03d.jpg
```

---

## Directory Structure

One subfolder per day (DD-MM-YYYY), one subfolder per video (cleaned title).

```
~/.local/share/cl-go-dash/assets/video-analyzer/
  30-03-2026/
    my-video-title-here/
      transcript.txt          # Subtitles or transcription
      frames/                 # Extracted key frames
        frame_001.jpg
        frame_010.jpg
      summary.md              # Claude's structured summary
      source.txt              # URL + title + duration + analysis date
```

---

## Phase 0 — Setup output directory

```bash
# Get today's date (DD-MM-YYYY format)
DATE=$(date +%d-%m-%Y)

# Clean video title: lowercase, spaces to hyphens, remove special chars
TITLE="my-video-title"  # sanitize from yt-dlp metadata

# Create directory structure
OUTPUT_DIR="~/.local/share/cl-go-dash/assets/video-analyzer/${DATE}/${TITLE}"
mkdir -p "${OUTPUT_DIR}/frames"
```

Before starting, verify all required tools are installed:
```bash
which yt-dlp ffmpeg deno
```
If any is missing, install via `brew install <tool>`.
For whisper: `uv tool install openai-whisper`.

---

## Phase 1 — Try subtitles first (fastest path)

Always attempt subtitles before downloading audio — it's instant and uses no bandwidth.

```bash
yt-dlp --skip-download --write-auto-subs --sub-lang fr,en --convert-subs srt \
  -o "${OUTPUT_DIR}/transcript" "URL"
```

If subtitles are found, an `.srt` file appears in the output directory.
Convert SRT to plain text for easier reading:
```bash
sed 's/<[^>]*>//g' "${OUTPUT_DIR}/transcript.fr.srt" | grep -v '^[0-9]' | grep -v '^\s*$' | grep -v -- '-->' > "${OUTPUT_DIR}/transcript.txt"
```

If subtitles exist, **skip Phase 2 entirely** and go to Phase 3.

---

## Phase 2 — Download audio and transcribe (fallback)

Only if Phase 1 found no subtitles.

### Download audio
```bash
yt-dlp -x --audio-format wav -o "${OUTPUT_DIR}/audio.wav" "URL"
```

### Transcribe with Voxtral API (preferred)
```bash
curl -s -X POST https://api.mistral.ai/v1/audio/transcriptions \
  -H "Authorization: Bearer $VOXTRAL_API_KEY" \
  -F "file=@${OUTPUT_DIR}/audio.wav" \
  -F "model=voxtral-mini-transcribe-2-2602" \
  -F "language=fr" \
  | jq -r '.text' > "${OUTPUT_DIR}/transcript.txt"
```

### Transcribe with Whisper (local fallback)
If Voxtral fails or for offline use:
```bash
whisper "${OUTPUT_DIR}/audio.wav" --model turbo --language fr \
  --output_format txt --output_dir "${OUTPUT_DIR}"
```

---

## Phase 3 — Download video for frame extraction

Download at 720p max to limit file size:
```bash
yt-dlp -f "bv[height<=720]+ba" --merge-output-format mp4 \
  -o "${OUTPUT_DIR}/video.mp4" "URL"
```

---

## Phase 4 — Extract key frames

Choose frequency based on content type and duration.
Get video duration first:
```bash
DURATION=$(ffmpeg -i "${OUTPUT_DIR}/video.mp4" 2>&1 | grep Duration | awk '{print $2}' | tr -d ,)
```

### Talks, vlogs, interviews — every 60 seconds
```bash
ffmpeg -i "${OUTPUT_DIR}/video.mp4" -vf fps=1/60 \
  "${OUTPUT_DIR}/frames/frame_%03d.jpg"
```

### Presentations, slides, tutorials — adapt interval to duration

**Under 5 minutes** — every 5 seconds (~60 frames max):
```bash
ffmpeg -i "${OUTPUT_DIR}/video.mp4" -vf fps=1/5 \
  "${OUTPUT_DIR}/frames/frame_%03d.jpg"
```

**5 to 20 minutes** — every 10 seconds (~120 frames max):
```bash
ffmpeg -i "${OUTPUT_DIR}/video.mp4" -vf fps=1/10 \
  "${OUTPUT_DIR}/frames/frame_%04d.jpg"
```

**Over 20 minutes** — every 30 seconds (~120 frames for 1h):
```bash
ffmpeg -i "${OUTPUT_DIR}/video.mp4" -vf fps=1/30 \
  "${OUTPUT_DIR}/frames/frame_%04d.jpg"
```

| Content type | Duration | Interval | Frames (approx) |
|-------------|----------|----------|-----------------|
| Vlogs, talks, interviews | any | 60s | ~60/h |
| Presentations, slides | < 5 min | 5s | ~60 max |
| Presentations, slides | 5-20 min | 10s | ~120 max |
| Presentations, slides | > 20 min | 30s | ~120/h |

---

## Phase 5 — Analyze content

After extraction, Claude can now "see" and "hear" the video:

1. **Read the transcript** — Read `{output-dir}/transcript.txt` (full text)
2. **View the frames** — Read each `{output-dir}/frames/frame_*.jpg` (multimodal)
3. **Write summary** — Produce a structured summary in `{output-dir}/summary.md`
4. **Save metadata** — Write `{output-dir}/source.txt` with:
   - Source URL
   - Video title
   - Duration
   - Analysis date

### Summary template

```markdown
# [Video Title]

## Key points
- ...

## Detailed notes
### [Topic 1]
...

## Visual observations
- Frame 001: [description]
- Frame 010: [description]

## Quotes
> "Notable quote from transcript"

## Links and references mentioned
- ...
```

---

## Phase 6 — Cleanup

Delete heavy files after extraction — keep only transcript, frames, and summary:

```bash
rm -f "${OUTPUT_DIR}/video.mp4" "${OUTPUT_DIR}/audio.wav"
rm -f "${OUTPUT_DIR}"/transcript.*.srt  # remove raw SRT, keep transcript.txt
```

**Keep:** `transcript.txt`, `frames/`, `summary.md`, `source.txt`
**Delete:** `video.mp4`, `audio.wav`, raw `.srt` files

---

## Common issues

| Problem | Cause | Solution |
|---------|-------|----------|
| yt-dlp hangs on YouTube | Missing deno runtime | `brew install deno` |
| No subtitles found | Video has no auto-captions | Fall back to Phase 2 (audio + transcribe) |
| Voxtral transcription fails | File too large (>25MB) | Split audio: `ffmpeg -i audio.wav -f segment -segment_time 600 chunk_%03d.wav` |
| Too many frames extracted | Wrong fps setting | Use `fps=1/60` for talks, `fps=1/10` only for slides |
| Frames are black | Video hasn't started yet | Add `-ss 5` to skip first 5 seconds |
| yt-dlp format error | Format not available at 720p | Try `-f "bv*[height<=720]+ba"` or remove format filter |
