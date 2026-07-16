process.on('uncaughtException', (err) => {
  console.error('⚠ uncaught:', err.message);
});
process.on('unhandledRejection', (err) => {
  console.error('⚠ rejection:', err.message);
});

const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
const rateLimit = require('express-rate-limit');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const { detectPlatform } = require('./middleware/platform');
const { auth } = require('./middleware/auth');
const { log } = require('./middleware/log');
const { getInstructions } = require('./cid_instructions');
const { getRandomFact, getFactCount } = require('./funny_facts');

const app = express();
const PORT = process.env.PORT || 3000;
const startTime = Date.now();
const URL_FILE = path.resolve(__dirname, '..', 'current-url.txt');
const URL_FILE_ALT = path.resolve(__dirname, '..', 'current-url-alt.txt');
const CID_BINARY = process.env.CID_BINARY || path.resolve(__dirname, '../../target/release/lai-gate');

// --- helpers ---

function getCurrentUrl() {
  try {
    return fs.readFileSync(URL_FILE, 'utf-8').trim();
  } catch {
    return null;
  }
}

function getCurrentUrlAlt() {
  try {
    return fs.readFileSync(URL_FILE_ALT, 'utf-8').trim();
  } catch {
    return null;
  }
}

function getUptime() {
  const sec = Math.floor((Date.now() - startTime) / 1000);
  const d = Math.floor(sec / 86400);
  const h = Math.floor((sec % 86400) / 3600);
  const m = Math.floor((sec % 3600) / 60);
  const s = sec % 60;
  const parts = [];
  if (d) parts.push(`${d}d`);
  if (h) parts.push(`${h}h`);
  if (m) parts.push(`${m}m`);
  parts.push(`${s}s`);
  return parts.join(' ');
}

function getCidVersion() {
  try {
    const out = execSync(`"${CID_BINARY}" tanto eval "1+1"`, { encoding: 'utf-8', timeout: 5000 });
    return 'v0.3.0 (Tanto OK)';
  } catch {
    return 'v0.3.0 (binary NOT found)';
  }
}
const CID_VERSION = getCidVersion(); // cache at startup

// --- middleware ---

app.use(helmet({ contentSecurityPolicy: false }));
app.use(cors());
app.use(express.json({ limit: '1mb' }));
app.use(rateLimit({ windowMs: 60000, max: 120 }));

// --- static assets (inline SVG logo) ---

const LOGO_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 120" width="80" height="80">
  <path d="M60 5 L115 30 L115 90 L60 115 L5 90 L5 30 Z" fill="none" stroke="currentColor" stroke-width="2"/>
  <path d="M60 20 L95 35 L95 85 L60 100 L25 85 L25 35 Z" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.5"/>
  <circle cx="60" cy="60" r="8" fill="currentColor" opacity="0.3"/>
  <path d="M60 5 L60 115 M5 30 L115 30 M5 90 L115 90" stroke="currentColor" stroke-width="0.5" opacity="0.2"/>
  <text x="60" y="108" text-anchor="middle" font-size="8" fill="currentColor" font-family="monospace" opacity="0.5">cid</text>
</svg>`;

// --- routes ---

app.get('/health', (_, res) => res.json({
  status: 'ok',
  service: 'cid-bridge',
  version: '1.0.0',
  uptime: getUptime(),
  started: new Date(startTime).toISOString()
}));

app.get('/status', (_, res) => {
  const url = getCurrentUrl();
  const urlAlt = getCurrentUrlAlt();
  res.json({
    service: 'cid-bridge',
    version: '1.0.0',
    public_url: url,
    public_url_alt: urlAlt,
    uptime: getUptime(),
    started: new Date(startTime).toISOString(),
    cid: CID_VERSION,
    tunnels: {
      cloudflare: { status: url ? 'connected' : 'offline', url: url },
      localhost_run: { status: urlAlt ? 'connected' : 'offline', url: urlAlt }
    }
  });
});

app.get('/fact', (req, res) => {
  const fact = getRandomFact();
  const fmt = req.query.format || 'json';
  if (fmt === 'text') {
    res.type('text/plain').send(`${fact.text}\n\n— CID-verified funny fact (confidence: ${(fact.confidence * 100).toFixed(0)}%, passed: ${fact.passed ? 'yes' : 'no'}, CID says: ${fact.note})`);
  } else {
    res.json({
      fact: fact.text,
      cid_verified: fact.validated,
      confidence: fact.confidence,
      passed: fact.passed,
      cid_says: fact.note,
      total_facts: getFactCount()
    });
  }
});

app.get('/cid.txt', (req, res) => {
  // Prefer the ChatGPT-friendly localhost.run URL for AI chatbot access
  const url = getCurrentUrlAlt() || getCurrentUrl() || 'https://your-bridge.lhr.life';
  res.type('text/plain').send(getInstructions(url));
});

app.get('/integrate', (req, res) => {
  const url = getCurrentUrl() || 'https://your-bridge.trycloudflare.com';
  const urlAlt = getCurrentUrlAlt() || 'https://your-bridge.lhr.life';
  res.send(`<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>CID Bridge — Integrate</title>
  <style>
    :root { --bg: #0a0a0f; --surface: #12121a; --border: #2a2a4a; --text: #e2e2f0; --text-dim: #8888aa; --accent: #818cf8; --accent2: #6366f1; --green: #34d399; --gold: #f59e0b; }
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { background: var(--bg); color: var(--text); font-family: -apple-system, BlinkMacSystemFont, system-ui, sans-serif; padding: 2rem 1rem; display: flex; flex-direction: column; align-items: center; }
    .container { max-width: 800px; width: 100%; }
    h1 { font-size: 2rem; font-weight: 700; margin-bottom: 0.5rem; background: linear-gradient(135deg, var(--accent), var(--accent2)); -webkit-background-clip: text; -webkit-text-fill-color: transparent; }
    .subtitle { color: var(--text-dim); margin-bottom: 2rem; }
    .card { background: var(--surface); border: 1px solid var(--border); border-radius: 12px; padding: 1.5rem; margin-bottom: 1rem; }
    .card h2 { font-size: 1.2rem; margin-bottom: 0.75rem; color: var(--accent); }
    .card h3 { font-size: 1rem; margin-bottom: 0.5rem; color: var(--text); }
    .card p { color: var(--text-dim); margin-bottom: 0.75rem; font-size: 0.9rem; }
    table { width: 100%; border-collapse: collapse; margin: 0.5rem 0; }
    td, th { padding: 0.4rem 0.75rem; text-align: left; border-bottom: 1px solid var(--border); font-size: 0.85rem; }
    th { color: var(--text-dim); font-weight: 600; }
    td { color: var(--text); font-family: monospace; }
    code { background: var(--bg); padding: 0.15rem 0.4rem; border-radius: 4px; font-family: 'SF Mono', monospace; font-size: 0.8rem; color: var(--accent); }
    pre { background: var(--bg); border: 1px solid var(--border); border-radius: 8px; padding: 1rem; overflow-x: auto; font-family: 'SF Mono', monospace; font-size: 0.8rem; margin: 0.5rem 0; }
    .badge { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 0.7rem; font-weight: 600; }
    .badge.easy { background: #1a3a2e; color: var(--green); }
    .badge.med { background: #3a3a1a; color: #facc15; }
    .badge.hard { background: #3a1a1a; color: #f87171; }
    .badge.gpt { background: #1a2e3a; color: var(--gold); }
    a { color: var(--accent); text-decoration: none; }
    a:hover { text-decoration: underline; }
    .copy-btn { background: var(--accent2); border: none; color: white; padding: 0.3rem 0.6rem; border-radius: 4px; cursor: pointer; font-size: 0.75rem; }
    .footer { margin-top: 2rem; text-align: center; font-size: 0.8rem; color: var(--text-dim); }
    .url-box { background: var(--bg); border: 1px solid var(--accent); border-radius: 8px; padding: 1rem; text-align: center; font-family: monospace; font-size: 1.1rem; color: var(--accent); margin: 1rem 0; word-break: break-all; }
    .url-box-alt { background: var(--bg); border: 1px solid var(--gold); border-radius: 8px; padding: 1rem; text-align: center; font-family: monospace; font-size: 1.1rem; color: var(--gold); margin: 0.5rem 0; word-break: break-all; }
  </style>
</head>
<body>
  <div class="container">
    <h1>🔌 CID Bridge — Integrate</h1>
    <p class="subtitle">Connect any AI chatbot to CID validation</p>

    <div class="card">
      <h2>🌐 Your Bridge URLs</h2>
      <div class="url-box-alt">🚀 ${urlAlt}</div>
      <p style="text-align:center;font-size:0.85rem;color:var(--gold);">⭐ ChatGPT-friendly — use this one for Custom GPT Actions</p>
      <div class="url-box">🌩️ ${url}</div>
      <p style="text-align:center;font-size:0.8rem;color:var(--text-dim);">Cloudflare Quick Tunnel (may be blocked by ChatGPT)</p>
    </div>

    <div class="card">
      <h2>🤖 Grok (xAI) <span class="badge easy">30s</span></h2>
      <p>Grok supports MCP natively. Just add a server:</p>
      <table>
        <tr><th>Setting</th><th>Value</th></tr>
        <tr><td>Name</td><td><code>CID Bridge</code></td></tr>
        <tr><td>URL</td><td><code>${urlAlt}/mcp</code></td></tr>
      </table>
      <p>Settings → MCP Servers → Add Server. Done.</p>
    </div>

    <div class="card">
      <h2>🤖 ChatGPT <span class="badge med">2 min</span></h2>
      <p>Create a <strong>Custom GPT</strong> → <strong>Configure</strong> → <strong>Add Action</strong>:</p>
      <pre>openapi: 3.1.0
info:
  title: CID Bridge
  version: 1.0.0
servers:
  - url: ${urlAlt}
paths:
  /validate:
    post:
      summary: Validate text through CID
      operationId: cidValidate
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [text]
              properties:
                text: { type: string }
                context: { type: string, enum: [math, logic, facts, general], default: general }
      responses:
        '200':
          description: Validation result</pre>
      <p>Then add this instruction: <em>"Use cidValidate() when asked to check math, logic, or facts."</em></p>
    </div>

    <div class="card">
      <h2>🤖 Claude Desktop <span class="badge easy">30s</span></h2>
      <p>Edit <code>claude_desktop_config.json</code>:</p>
      <pre>{
  "mcpServers": {
    "cid-bridge": {
      "type": "http",
      "url": "${urlAlt}/mcp"
    }
  }
}</pre>
    </div>

    <div class="card">
      <h2>🤖 Claude Code (CLI) <span class="badge easy">10s</span></h2>
      <pre>claude mcp add cid-bridge --type=http --url=${urlAlt}/mcp</pre>
    </div>

    <div class="card">
      <h2>🤖 Open WebUI <span class="badge easy">30s</span></h2>
      <p>Settings → Connections → MCP Servers → Add:</p>
      <table>
        <tr><th>Setting</th><th>Value</th></tr>
        <tr><td>Name</td><td><code>CID Bridge</code></td></tr>
        <tr><td>URL</td><td><code>${urlAlt}/mcp</code></td></tr>
      </table>
    </div>

    <div class="card">
      <h2>🤖 Direct API (any platform)</h2>
      <p>For function-calling models, use this function definition:</p>
      <pre>{
  "name": "cid_validate",
  "description": "Validate text through CID",
  "parameters": {
    "type": "object",
    "properties": {
      "text": { "type": "string" },
      "context": { "type": "string", "enum": ["math","logic","facts","general"] }
    },
    "required": ["text"]
  }
}</pre>
      <pre>curl -X POST ${urlAlt}/validate \\
  -H 'Content-Type: application/json' \\
  -d '{"text":"2+2=4","context":"math"}'</pre>
    </div>

    <div class="footer">
      <a href="/">Dashboard</a> &middot;
      <a href="https://codeberg.org/NutypeBuddha/cid-bridge">cid-bridge</a> &middot;
      Wintermore Housekeeping
    </div>
  </div>
</body>
</html>`);
});

app.get('/', (req, res) => {
  const url = getCurrentUrl();
  const urlAlt = getCurrentUrlAlt();
  const uptime = getUptime();
  const cidVer = CID_VERSION;

  const primaryUrl = urlAlt || url || '⏳ waiting for a tunnel...';
  const online = !!(urlAlt || url);

  res.send(`<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>CID Bridge</title>
  <style>
    *{margin:0;padding:0;box-sizing:border-box}
    body{
      background:#0f0f1a;
      color:#e0e0f0;
      font-family:'Courier New',monospace;
      min-height:100vh;
      display:flex;
      flex-direction:column;
      align-items:center;
      justify-content:center;
      padding:1.5rem;
      text-align:center
    }
    .infographic{
      max-width:620px;
      width:100%;
      border:1px solid #333366;
      border-radius:0;
      padding:2.5rem 2rem;
      position:relative
    }
    .infographic:before{
      content:'';
      position:absolute;
      top:-1px;left:-1px;right:-1px;bottom:-1px;
      z-index:-1
    }
    h1{
      font-size:1rem;
      text-transform:uppercase;
      letter-spacing:0.3em;
      color:#6666aa;
      margin-bottom:1.5rem;
      font-weight:400
    }
    .url-box{
      border:2px dashed ${online ? '#44dd88' : '#664444'};
      padding:1.25rem;
      margin:1.5rem 0;
      word-break:break-all;
      position:relative;
      transition:all 0.3s
    }
    .url-box .status-dot{
      display:inline-block;
      width:10px;height:10px;
      border-radius:50%;
      margin-right:8px;
      vertical-align:middle
    }
    .url-box .status-dot.online{background:#44dd88;box-shadow:0 0 12px #44dd8844}
    .url-box .status-dot.offline{background:#664444}
    .url-box .url-text{
      font-size:1.1rem;
      color:${online ? '#44dd88' : '#887777'};
      transition:color 0.3s
    }
    .url-box .url-label{
      display:block;
      font-size:0.65rem;
      text-transform:uppercase;
      letter-spacing:0.15em;
      color:#666688;
      margin-bottom:0.5rem
    }
    .copy-btn{
      display:inline-block;
      margin-top:0.75rem;
      background:none;
      border:1px solid #444477;
      color:#aaaacc;
      padding:0.4rem 1rem;
      cursor:pointer;
      font-family:inherit;
      font-size:0.7rem;
      text-transform:uppercase;
      letter-spacing:0.1em;
      transition:all 0.2s
    }
    .copy-btn:hover{background:#44447766;color:#fff;border-color:#6666aa}
    .tagline{
      font-size:0.85rem;
      color:#8888aa;
      margin:1rem 0 2rem;
      line-height:1.6
    }
    .tagline span{color:#ff8888}
    .stats{
      display:flex;
      justify-content:center;
      gap:2rem;
      margin:1.5rem 0;
      font-size:0.75rem;
      color:#666688
    }
    .stats .stat span{display:block;color:#aaaacc;font-size:0.85rem;margin-top:0.2rem}
    .footer{
      margin-top:2rem;
      font-size:0.65rem;
      color:#555577;
      line-height:2
    }
    .footer a{color:#7777aa;text-decoration:none}
    .footer a:hover{color:#aaaacc;text-decoration:underline}
    .blink{animation:blink 1.5s ease-in-out infinite}
    @keyframes blink{0%,100%{opacity:1}50%{opacity:0.3}}
    @media(max-width:480px){
      .infographic{padding:1.5rem 1rem}
      .stats{flex-direction:column;gap:0.5rem}
      .url-box .url-text{font-size:0.9rem}
    }
  </style>
</head>
<body>
  <div class="infographic">
    <h1>❖ CID Bridge ❖</h1>

    <div class="url-box" id="urlBox">
      <span class="url-label">Universal Link <span style="color:#555577">(your AI backend is here)</span></span>
      <span class="status-dot ${online ? 'online' : 'offline'}" id="statusDot"></span>
      <span class="url-text ${online ? '' : 'blink'}" id="urlText">${primaryUrl}</span>
      <button class="copy-btn" id="copyBtn" onclick="copyUrl()">[ copy ]</button>
    </div>

    <div class="tagline">
      ${online
        ? 'Tunnels are up. CID is ready. <span>You can validate things now.</span>'
        : '<span class="blink">⚠ No tunnel yet.</span> Run <span style="color:#66aacc">./cid-share.sh</span> or check the server logs.'}
    </div>

    <div class="stats">
      <div class="stat">uptime <span id="uptimeStat">${uptime}</span></div>
      <div class="stat">cid engine <span id="cidStat">${cidVer}</span></div>
      <div class="stat">tunnel <span id="tunnelStat">${online ? 'connected' : 'offline'}</span></div>
    </div>

    <div style="font-size:0.7rem;color:#555577;margin-top:1rem">
      /health · /status · /validate · /fact · /integrate · /cid.txt · /mcp
    </div>

    <div class="footer">
      <a href="https://codeberg.org/NutypeBuddha/cid-bridge">source</a>
      · built by wintermore housekeeping ·
      <a href="/integrate">hook up an ai →</a>
    </div>
  </div>

  <script>
    function copyUrl(){
      const btn=document.getElementById('copyBtn');
      const text=document.getElementById('urlText').textContent;
      navigator.clipboard.writeText(text).then(()=>{
        btn.textContent='[ copied! ]';
        setTimeout(()=>{btn.textContent='[ copy ]'},2000);
      });
    }
    async function refreshStatus(){
      try{
        const r=await fetch('/status');
        const d=await r.json();
        const u=d.public_url_alt||d.public_url;
        const dot=document.getElementById('statusDot');
        const txt=document.getElementById('urlText');
        const st=document.getElementById('tunnelStat');
        if(u){
          txt.textContent=u;
          dot.className='status-dot online';
          st.textContent='connected';
          txt.className='url-text';
          document.querySelector('.tagline span').textContent='You can validate things now.';
        }else{
          dot.className='status-dot offline';
          st.textContent='offline';
          txt.className='url-text blink';
        }
        document.getElementById('uptimeStat').textContent=d.uptime;
      }catch(e){}
    }
    setInterval(refreshStatus,10000);
  </script>
</body>
</html>`);
});

app.post('/mcp', auth, detectPlatform, async (req, res) => {
  const { platform, action, data, metadata } = req.body;
  log(`MCP ${action} from ${platform}`);

  try {
    const Adapter = require(`./adapters/${platform}`);
    const adapter = new Adapter();
    const result = await adapter.execute(action, data, metadata);
    res.json({ success: true, data: result });
  } catch (err) {
    if (err.code === 'MODULE_NOT_FOUND') {
      return res.status(400).json({ success: false, error: `unsupported platform: ${platform}` });
    }
    res.status(500).json({ success: false, error: err.message });
  }
});

app.post('/validate', async (req, res) => {
  const { text, context } = req.body;
  const result = await require('./adapters/cid').validate(text, context || 'general');
  res.json({ success: true, data: result });
});

app.listen(PORT, () => {
  console.log(`CID Bridge listening on :${PORT}`);
  console.log(`GET  /          — status dashboard`);
  console.log(`GET  /health    — health check`);
  console.log(`GET  /status    — JSON status`);
  console.log(`POST /validate  — direct CID validation`);
  console.log(`POST /mcp       — MCP protocol (any AI platform)`);
  console.log(`GET  /fact      — random CID-verified funny fact`);
  console.log(`GET  /integrate — AI integration guide`);
});
