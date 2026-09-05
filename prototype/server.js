// Donate Me — backend ไม่ต้องใช้ dependency (Node.js เพียว ๆ)
// วิธีรัน: node server.js  แล้วเปิด http://localhost:3000
const http = require('http');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const PORT = 3000;
const PUBLIC = path.join(__dirname, 'public');
const STREAMER = { name: 'Streamer', promptpay: '0812345678' }; // TODO: เปลี่ยนเบอร์ PromptPay จริง
const GOAL = 5000;

// ---- ข้อมูลในหน่วยความจำ (จริง ๆ ควรต่อ database) ----
const donations = new Map(); // id -> donation
let total = 0;

// ---- SSE: push แจ้งเตือนไปหน้า overlay และ dashboard ----
const sseClients = new Set();
function broadcast(event, data) {
  const payload = `event: ${event}\ndata: ${JSON.stringify(data)}\n\n`;
  for (const res of sseClients) res.write(payload);
}

// ---- utils ----
const mime = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.css': 'text/css', '.png': 'image/png', '.mp3': 'audio/mpeg' };
function sendFile(res, file) {
  fs.readFile(path.join(PUBLIC, file), (err, buf) => {
    if (err) { res.writeHead(404); return res.end('Not found'); }
    res.writeHead(200, { 'Content-Type': mime[path.extname(file)] || 'application/octet-stream' });
    res.end(buf);
  });
}
function json(res, code, data) { res.writeHead(code, { 'Content-Type': 'application/json; charset=utf-8' }); res.end(JSON.stringify(data)); }
function readBody(req) {
  return new Promise(resolve => {
    let body = ''; req.on('data', c => body += c); req.on('end', () => resolve(body ? JSON.parse(body) : {}));
  });
}

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, `http://${req.headers.host}`);

  // SSE endpoint
  if (url.pathname === '/events') {
    res.writeHead(200, {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      'Connection': 'keep-alive',
      'Access-Control-Allow-Origin': '*',
    });
    res.write('retry: 2000\n\n');
    sseClients.add(res);
    req.on('close', () => sseClients.delete(res));
    return;
  }

  // ---- API ----
  if (req.method === 'POST' && url.pathname === '/api/donate') {
    const { name, amount, message, sound } = await readBody(req);
    if (!name || !amount || amount < 1) return json(res, 400, { error: 'กรอกข้อมูลไม่ครบ' });
    const id = crypto.randomBytes(8).toString('hex');
    const donation = { id, name: String(name).slice(0, 50), amount: Number(amount),
      message: String(message || '').slice(0, 200), sound: sound || 'chime', status: 'pending', ts: Date.now() };
    donations.set(id, donation);
    return json(res, 200, { payUrl: `/pay/${id}` });
  }

  // ในระบบจริง ตรงนี้คือ webhook จาก Omise/2C2P ที่ยืนยันว่าได้เงินจริง
  if (req.method === 'POST' && url.pathname.startsWith('/api/donate/') && url.pathname.endsWith('/confirm')) {
    const id = url.pathname.split('/')[3];
    const d = donations.get(id);
    if (!d) return json(res, 404, { error: 'not found' });
    if (d.status === 'pending') {
      d.status = 'paid';
      total += d.amount;
      broadcast('donation', d);          // ให้ overlay เด้งป็อบอัพ
      broadcast('stats', { total, goal: GOAL, recent: recentList() }); // ให้ dashboard อัปเดต
    }
    return json(res, 200, { ok: true });
  }

  if (url.pathname === '/api/stats') {
    return json(res, 200, { total, goal: GOAL, streamer: STREAMER.name, recent: recentList() });
  }

  if (url.pathname === '/api/test-alert') {
    const d = { id: 'test', name: 'ทดสอบระบบ', amount: 100, message: 'นี่คือการทดสอบแจ้งเตือน', sound: 'chime', ts: Date.now() };
    broadcast('donation', d);
    return json(res, 200, { ok: true });
  }

  // ---- หน้าเว็บ ----
  if (url.pathname === '/')            return sendFile(res, 'index.html');
  if (url.pathname === '/alert')       return sendFile(res, 'alert.html');
  if (url.pathname === '/dashboard')   return sendFile(res, 'dashboard.html');
  if (url.pathname.startsWith('/pay/')) return sendFile(res, 'pay.html');
  sendFile(res, url.pathname.slice(1));
});

function recentList() {
  return [...donations.values()].filter(d => d.status === 'paid')
    .sort((a, b) => b.ts - a.ts).slice(0, 10);
}

server.listen(PORT, () =>
  console.log(`✅ Donate Me รันแล้วที่ http://localhost:${PORT}
   • หน้าโดเนต:      http://localhost:${PORT}/
   • Overlay (OBS):  http://localhost:${PORT}/alert
   • Dashboard:      http://localhost:${PORT}/dashboard`));
