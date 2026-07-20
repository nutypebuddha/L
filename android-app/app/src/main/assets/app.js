// L.ai Assistant — Frontend Logic

const isAndroid = typeof LaiBridge !== 'undefined';
let isListening = false;
let conversationHistory = [];
let currentInput = '';
let recognition = null;

// ── Bridge to native ───────────────────────────────────────────
function nativeExec(cmd) {
  if (!isAndroid) return simulateResponse(cmd);
  return LaiBridge.executeCommand(cmd);
}

function nativeToast(msg) {
  if (isAndroid) LaiBridge.showToast(msg);
  else console.log('Toast:', msg);
}

function haptic(style) {
  if (!isAndroid) return;
  try {
    if (style === 'light') LaiBridge.vibrate(10);
    else if (style === 'medium') LaiBridge.vibrate(20);
    else if (style === 'heavy') LaiBridge.vibrate(40);
    else if (style === 'success') { LaiBridge.vibrate(10); setTimeout(() => LaiBridge.vibrate(10), 80); }
    else if (style === 'error') { LaiBridge.vibrate(50); setTimeout(() => LaiBridge.vibrate(50), 100); }
  } catch(e) {}
}

// ── Init ───────────────────────────────────────────────────────
function bootDeck() {
  const input = document.getElementById('user-input');

  // Cyberdeck boot log animation
  const bootLines = [
    'L.AI CYBERDECK // cold boot',
    'init kernel ......... <b class="ok">OK</b>',
    'mount /dev/lai ...... <b class="ok">OK</b>',
    'load gate(cid) ...... <b class="ok">OK</b>',
    'mesh proof->athena .. <b class="ok">OK</b>',
    'probe daemon:7878 ... <b id="boot-daemon">PROBE</b>',
    'ready.'
  ];
  const logEl = document.getElementById('boot-log');
  let bi = 0;
  const bootTimer = setInterval(() => {
    if (!logEl) { clearInterval(bootTimer); return; }
    if (bi < bootLines.length) {
      const line = document.createElement('div');
      line.innerHTML = bootLines[bi];
      logEl.appendChild(line);
      bi++;
      if (bootLines[bi - 1].includes('probe daemon')) {
        const d = document.getElementById('boot-daemon');
        if (d) d.innerHTML = isAndroid ? '<b class="ok">LINKED</b>' : '<b class="warn">DEMO</b>';
      }
    } else {
      clearInterval(bootTimer);
    }
  }, 230);

  // Also probe daemon status live so failures surface on-screen
  if (isAndroid) {
    try { document.getElementById('device-info').textContent = LaiBridge.getDeviceInfo(); } catch(e) {}
  }

  input.addEventListener('keydown', e => {
    if (e.key === 'Enter') { haptic('light'); sendText(); }
  });

  loadHistory();

  // Dismiss splash after boot animation completes
  setTimeout(dismissSplash, 2100);

  // Hard fallback: never get stuck on splash, even if timers are throttled
  setTimeout(dismissSplash, 6000);
}

function dismissSplash() {
  const splash = document.getElementById('splash');
  if (!splash || !splash.classList.contains('active')) return;
  splash.style.opacity = '0';
  splash.style.transition = 'opacity 0.4s ease-out';
  setTimeout(() => {
    splash.classList.remove('active');
    splash.style.opacity = '';
    const chat = document.getElementById('chat-screen');
    if (!chat.classList.contains('active')) {
      chat.classList.add('active');
      addSystemMessage(isAndroid ? 'L.AI deck online. speak or type.' : 'L.AI deck online [demo]. speak or type.');
    }
    setTimeout(() => document.getElementById('user-input').focus(), 120);
  }, 400);
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', bootDeck);
} else {
  bootDeck();
}

// ── Screen transitions ─────────────────────────────────────────
function showScreen(id) {
  haptic('light');
  const current = document.querySelector('.screen.active');
  const next = document.getElementById(id);
  if (current === next) return;

  // Fade out current
  current.style.opacity = '0';
  setTimeout(() => {
    current.classList.remove('active');
    current.style.opacity = '';
    next.classList.add('active');
    // Focus input if going to chat
    if (id === 'chat-screen') {
      setTimeout(() => document.getElementById('user-input').focus(), 100);
    }
  }, 150);
}

// ── Messages ───────────────────────────────────────────────────
function addUserMessage(text) {
  const el = document.createElement('div');
  el.className = 'msg user';
  el.innerHTML = `<div class="msg-bubble">${escHtml(text)}</div><div class="msg-meta">now</div>`;
  document.getElementById('messages-inner').appendChild(el);
  scrollDown();
  conversationHistory.push({ role: 'user', text, time: Date.now() });
}

function addAssistantMessage(text, source) {
  const el = document.createElement('div');
  el.className = 'msg assistant';
  let sourceHtml = source ? `<div class="msg-source">${escHtml(source)}</div>` : '';
  el.innerHTML = `<div class="msg-bubble">${formatResponse(text)}${sourceHtml}</div><div class="msg-meta">L &middot; now</div>`;
  document.getElementById('messages-inner').appendChild(el);
  scrollDown();
  conversationHistory.push({ role: 'assistant', text, source, time: Date.now() });
  saveHistory();
  haptic('success');
}

function addSystemMessage(text) {
  const el = document.createElement('div');
  el.className = 'msg system';
  el.innerHTML = `<div class="msg-bubble">${escHtml(text)}</div>`;
  document.getElementById('messages-inner').appendChild(el);
  scrollDown();
}

function addTyping() {
  const el = document.createElement('div');
  el.className = 'msg assistant';
  el.id = 'typing-indicator';
  el.innerHTML = '<div class="msg-bubble"><div class="typing"><span></span><span></span><span></span></div></div>';
  document.getElementById('messages-inner').appendChild(el);
  scrollDown();
}

function removeTyping() {
  const el = document.getElementById('typing-indicator');
  if (el) {
    el.style.opacity = '0';
    el.style.transform = 'translateY(-8px)';
    el.style.transition = 'all 0.2s ease-out';
    setTimeout(() => el.remove(), 200);
  }
}

function scrollDown() {
  const c = document.getElementById('messages');
  requestAnimationFrame(() => {
    c.scrollTo({ top: c.scrollHeight, behavior: 'smooth' });
  });
}

function formatResponse(text) {
  let s = escHtml(text);
  // Bold **text**
  s = s.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
  // Inline code
  s = s.replace(/`([^`]+)`/g, '<code style="background:rgba(124,106,255,0.12);color:#A78BFA;padding:2px 8px;border-radius:6px;font-size:13px;font-weight:500">$1</code>');
  // Line breaks
  s = s.replace(/\n/g, '<br>');
  return s;
}

function escHtml(s) {
  return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}

// ── Send ───────────────────────────────────────────────────────
function sendText() {
  const input = document.getElementById('user-input');
  const text = input.value.trim();
  if (!text) return;
  input.value = '';
  input.blur();
  processInput(text);
}

function sendQuick(action) {
  haptic('light');
  const prompts = {
    solve: 'solve my problem: ',
    validate: 'validate this claim: ',
    web: 'search the web for ',
    remember: 'remember that ',
    timer: 'set a timer for 5 minutes',
    remind: 'remind me to ',
    formula: 'show me the formula for ',
    convert: 'convert ',
    traverse: 'traverse the graph for ',
    score: 'score this: ',
    wheel: 'wheel analysis of '
  };
  const input = document.getElementById('user-input');
  input.value = prompts[action] || '';
  input.focus();
  // Move cursor to end
  input.setSelectionRange(input.value.length, input.value.length);
}

// Recall every remembered fact — sends the natural-language command the
// agent's `recall` tool understands (empty key = list all).
function doRecallAll() {
  haptic('light');
  processInput('recall everything you remember about me');
}

// Settings > Memory panel: query the daemon's recall tool and render the
// remembered facts inline (does not touch the chat transcript).
async function refreshMemory() {
  haptic('light');
  const list = document.getElementById('memory-list');
  if (!list) return;
  list.innerHTML = '<p class="dim">loading\u2026</p>';
  try {
    const result = await executeLai('list every fact you remember, one per line');
    const text = (result && result.response) ? result.response.trim() : '';
    if (!text || /nothing|no facts|don't remember|do not remember/i.test(text)) {
      list.innerHTML = '<p class="dim">no facts remembered yet</p>';
      return;
    }
    const lines = text.split('\n').map(s => s.replace(/^[-*\u2022\d.\)\s]+/, '').trim()).filter(Boolean);
    if (!lines.length) { list.innerHTML = '<p class="dim">no facts remembered yet</p>'; return; }
    list.innerHTML = '';
    for (const line of lines) {
      const row = document.createElement('div');
      row.className = 'memory-row';
      const span = document.createElement('span');
      span.className = 'memory-fact';
      span.textContent = line;
      const btn = document.createElement('button');
      btn.className = 'memory-forget';
      btn.textContent = 'forget';
      btn.onclick = () => forgetFact(line);
      row.appendChild(span);
      row.appendChild(btn);
      list.appendChild(row);
    }
  } catch (e) {
    list.innerHTML = '<p class="dim">error loading memory: ' + e.message + '</p>';
    haptic('error');
  }
}

// Ask the daemon to forget a specific fact, then reload the panel.
async function forgetFact(fact) {
  haptic('medium');
  try {
    await executeLai('forget this: ' + fact);
    nativeToast('Forgotten');
  } catch (e) {
    nativeToast('Failed to forget');
    haptic('error');
  }
  refreshMemory();
}

function processInput(text) {
  addUserMessage(text);
  addTyping();
  document.getElementById('status-text').textContent = 'thinking...';

  setTimeout(async () => {
    try {
      const result = await executeLai(text);
      removeTyping();
      const responseText = result.response || 'No response from L.ai engine.';
      const source = result.source || detectSource(text);
      if (responseText && !responseText.startsWith('Error:')) {
        addAssistantMessage(responseText, source);
      } else {
        addAssistantMessage(responseText, '');
      }
    } catch (e) {
      removeTyping();
      addAssistantMessage('Error: ' + e.message, '');
      haptic('error');
    }
    document.getElementById('status-text').textContent = 'ready';
  }, 400);
}

async function executeLai(text) {
  let cmd = `assistant --text "${text.replace(/"/g, '\\"')}" --format json`;

  return new Promise((resolve) => {
    if (isAndroid) {
      // Use setTimeout to avoid blocking UI — @JavascriptInterface is synchronous
      setTimeout(() => {
        try {
          const raw = LaiBridge.executeCommand(cmd);
          try {
            const parsed = JSON.parse(raw);
            resolve(parsed);
          } catch {
            resolve({ response: raw, source: 'l.ai', intent: 'unknown', toast: raw });
          }
        } catch (err) {
          // Surface daemon diagnostics so failures are actionable
          let diag = '';
          try { diag = '\n\n[daemon] ' + LaiBridge.getDaemonStatus().replace(/\n/g, ' '); } catch (e) {}
          resolve({ response: 'Error: ' + err.message + diag, source: '', intent: 'error', toast: 'Error' });
        }
      }, 50);
    } else {
      resolve({ response: simulateResponse(text), source: 'demo', intent: detectSource(text), toast: 'Demo mode' });
    }
  });
}

function detectSource(text) {
  const lower = text.toLowerCase();
  if (/\b(search the web|web search|look up online|google)\b/.test(lower)) return 'web';
  if (/\b(remember|recall|forget|remembered)\b/.test(lower)) return 'memory';
  if (/\b(timer|remind|reminder|alarm)\b/.test(lower)) return 'schedule';
  if (/\b(solve|fix|debug)\b/.test(lower)) return 'proof';
  if (/\b(validate|verify|check|score)\b/.test(lower)) return 'gate';
  if (/\b(search|find|traverse|graph)\b/.test(lower)) return 'athena';
  if (/\b(convert|transform)\b/.test(lower)) return 'tanto';
  if (/\b(formula|equation|math)\b/.test(lower)) return 'tanto';
  if (/\b(wheel|vedic|planet)\b/.test(lower)) return 'athena';
  return 'l.ai';
}

function simulateResponse(text) {
  const lower = text.toLowerCase();
  if (/\b(hello|hi|hey|greetings)\b/.test(lower)) return "Hey. I'm L \u2014 your verification assistant. What do you need?";
  if (/\b(solve|fix)\b/.test(lower)) return "I can help. Describe the problem specifically \u2014 what's the error, what did you expect, what actually happened?";
  if (/\b(validate|verify)\b/.test(lower)) return "To validate a claim, I need the specific statement and context. Give me something concrete to check against the 5-gate system.";
  if (/\b(search|find)\b/.test(lower)) return "Searching the 528-formula corpus. Be more specific about what domain you're looking in \u2014 proof, gate, athena, or tanto?";
  if (/\b(formula|math|calculate)\b/.test(lower)) return "Which formula? I have 528 across 9 Vedic domains. Name the domain or the specific relationship you want to compute.";
  if (/\b(wheel|vedic|chart)\b/.test(lower)) return "Wheel analysis maps planetary positions to your project's domains. Mangala = action, Brihaspati = wisdom, Budha = logic, Shukra = value. What should I analyze?";
  if (/\b(help|what can you)\b/.test(lower)) return "I'm L.ai \u2014 voice-first verification assistant.\n\n\u2022 solve \u2014 describe a problem, get a solution approach\n\u2022 validate \u2014 verify claims against 5 gates\n\u2022 search \u2014 find formulas in the 528-entry corpus\n\u2022 formula \u2014 compute specific relationships\n\u2022 score \u2014 rate quality of reasoning\n\u2022 wheel \u2014 Vedic analysis of any topic\n\u2022 convert \u2014 transform between formats\n\u2022 traverse \u2014 explore the entity graph";
  return "Understood. I'm processing through the proof\u2192gate\u2192athena pipeline. Give me more detail on what you want verified.";
}

// ── Voice ──────────────────────────────────────────────────────
function toggleVoice() {
  haptic('medium');

  if (!('webkitSpeechRecognition' in window) && !('SpeechRecognition' in window)) {
    nativeToast('Speech recognition not available');
    haptic('error');
    return;
  }

  if (isListening) {
    stopListening();
    return;
  }

  const SR = window.SpeechRecognition || window.webkitSpeechRecognition;
  recognition = new SR();
  recognition.continuous = false;
  recognition.interimResults = false;
  recognition.lang = 'en-US';

  recognition.onstart = () => {
    isListening = true;
    document.getElementById('mic-btn').classList.add('listening');
    document.getElementById('status-text').textContent = 'listening...';
  };

  recognition.onresult = (e) => {
    const text = e.results[0][0].transcript;
    document.getElementById('user-input').value = text;
    haptic('success');
    processInput(text);
  };

  recognition.onend = () => stopListening();
  recognition.onerror = () => { stopListening(); haptic('error'); };
  recognition.start();
}

function stopListening() {
  isListening = false;
  document.getElementById('mic-btn').classList.remove('listening');
  document.getElementById('status-text').textContent = 'ready';
  if (recognition) { try { recognition.stop(); } catch(e) {} }
}

// ── History persistence ────────────────────────────────────────
function loadHistory() {
  try {
    if (isAndroid) {
      const data = LaiBridge.getHistory();
      if (data && data !== '[]') {
        conversationHistory = JSON.parse(data);
        const recent = conversationHistory.slice(-20);
        recent.forEach(m => {
          const el = document.createElement('div');
          el.className = m.role === 'user' ? 'msg user' : 'msg assistant';
          const bubble = m.role === 'user'
            ? `<div class="msg-bubble">${escHtml(m.text)}</div>`
            : `<div class="msg-bubble">${formatResponse(m.text)}${m.source ? `<div class="msg-source">${escHtml(m.source)}</div>` : ''}</div>`;
          el.innerHTML = bubble;
          document.getElementById('messages-inner').appendChild(el);
        });
        scrollDown();
      }
    }
  } catch(e) {}
}

function saveHistory() {
  try {
    if (isAndroid && conversationHistory.length > 0) {
      const trimmed = conversationHistory.slice(-100);
      LaiBridge.saveHistory(JSON.stringify(trimmed));
    }
  } catch(e) {}
}

function clearChat() {
  haptic('medium');
  const container = document.getElementById('messages-inner');
  container.style.opacity = '0';
  container.style.transform = 'translateY(12px)';
  container.style.transition = 'all 0.3s ease-out';
  setTimeout(() => {
    container.innerHTML = '';
    container.style.opacity = '1';
    container.style.transform = '';
    conversationHistory = [];
    saveHistory();
    addSystemMessage('Chat cleared.');
  }, 300);
}

function clearHistory() {
  clearChat();
  nativeToast('History cleared');
}

function showDeviceInfo() {
  haptic('light');
  if (isAndroid) {
    nativeToast(LaiBridge.getDeviceInfo());
  } else {
    nativeToast('Desktop mode — no native bridge');
  }
}

function showDaemonStatus() {
  haptic('light');
  if (!isAndroid) { nativeToast('Desktop mode — no daemon'); return; }
  try {
    const status = LaiBridge.getDaemonStatus();
    document.getElementById('daemon-status').textContent = status.includes('health=200') ? 'running' : 'DOWN';
    addSystemMessage('Backend status:\n' + status);
  } catch (e) {
    addSystemMessage('Backend status error: ' + e.message);
  }
}

function restartDaemon() {
  haptic('medium');
  if (!isAndroid) { nativeToast('Desktop mode — no daemon'); return; }
  try {
    const r = LaiBridge.restartDaemon();
    addSystemMessage('Restart requested: ' + r);
    setTimeout(() => showDaemonStatus(), 2500);
  } catch (e) {
    addSystemMessage('Restart error: ' + e.message);
  }
}
