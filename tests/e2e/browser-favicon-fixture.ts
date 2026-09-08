import { createServer } from "node:http";
import { deflateSync } from "node:zlib";

function crc32(bytes: Buffer) {
  let crc = 0xffffffff;
  for (const byte of bytes) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit++) crc = (crc >>> 1) ^ (0xedb88320 & -(crc & 1));
  }
  return (crc ^ 0xffffffff) >>> 0;
}
function chunk(type: string, bytes: Buffer) {
  const body = Buffer.concat([Buffer.from(type), bytes]);
  const length = Buffer.alloc(4); length.writeUInt32BE(bytes.length);
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(body));
  return Buffer.concat([length, body, crc]);
}
function png(red: number, blue: number) {
  const header = Buffer.alloc(13);
  header.writeUInt32BE(32, 0); header.writeUInt32BE(32, 4); header[8] = 8; header[9] = 6;
  const pixels = Buffer.alloc(32 * (1 + 32 * 4));
  for (let y = 0; y < 32; y++) for (let x = 0; x < 32; x++) {
    const i = y * 129 + 1 + x * 4;
    pixels[i] = red; pixels[i + 1] = 80; pixels[i + 2] = blue; pixels[i + 3] = 255;
  }
  return Buffer.concat([Buffer.from("89504e470d0a1a0a", "hex"), chunk("IHDR", header),
    chunk("IDAT", deflateSync(pixels)), chunk("IEND", Buffer.alloc(0))]);
}

export async function startFaviconFixture() {
  const requests: { path: string; cookie: string; time: number }[] = [];
  let active = 0, peak = 0;
  const red = png(230, 30), blue = png(30, 230);
  const server = createServer((req, res) => {
    const url = new URL(req.url ?? "/", "http://127.0.0.1");
    if (requests.length === 128) requests.shift();
    requests.push({ path: url.pathname, cookie: req.headers.cookie ?? "", time: Date.now() });
    if (url.pathname.startsWith("/icon/")) {
      active++; peak = Math.max(peak, active);
      const timer = setTimeout(() => {
        res.writeHead(url.pathname.includes("missing") ? 404 : 200, {
          "Content-Type": "image/png", "Cache-Control": "no-store",
          "Set-Cookie": "favicon_response=1; Path=/; SameSite=Lax",
        });
        res.end(url.pathname.includes("blue") ? blue : red);
      }, url.pathname.includes("timeout") ? 7000 : url.pathname.includes("slow") ? 1800 : 0);
      res.once("close", () => { active--; clearTimeout(timer); });
      return;
    }
    if (url.pathname === "/cookie-proof") { res.end("ok"); return; }
    const name = url.pathname.slice(1);
    if (!["a", "b", "dynamic", "slow", "timeout", "missing", "many"].includes(name)) {
      res.writeHead(404); res.end(); return;
    }
    const icon = name === "b" ? "blue" : name === "a" || name === "dynamic" ? "red" : name;
    const links = name === "many"
      ? Array.from({ length: 100 }, (_, i) => `<link rel="icon" href="/icon/missing-${i}">`).join("")
      : `<link id="favicon" rel="icon" href="/icon/${icon}">`;
    res.writeHead(200, { "Content-Type": "text/html", "Set-Cookie": "fixture_page=1; Path=/; SameSite=Lax" });
    res.end(`<!doctype html><title>Favicon ${name}</title>${links}<body style="background:#eee;color:#222;font:24px sans-serif">Fixture ${name}<button>Focus page</button><script>
      setTimeout(() => fetch('/cookie-proof'), 2500);
      ${name === "dynamic" ? "setTimeout(() => document.querySelector('#favicon').href='/icon/blue', 1500);" : ""}
    </script></body>`);
  });
  await new Promise<void>((resolve, reject) => { server.once("error", reject); server.listen(0, "127.0.0.1", () => resolve()); });
  const address = server.address();
  if (!address || typeof address === "string") throw new Error("Invalid fixture listener");
  return { base: `http://127.0.0.1:${address.port}`, requests, peak: () => peak,
    close: () => new Promise<void>((resolve, reject) => { server.closeAllConnections(); server.close((error) => error ? reject(error) : resolve()); }) };
}
