// JARVIS OS - Developer Console Controller
const getTauri = () => window.__TAURI__;

// Wait for Tauri injection (up to 1000ms) to prevent race condition labeling/calling errors
const waitTauri = () => {
  return new Promise((resolve) => {
    let attempts = 0;
    const check = () => {
      if (window.__TAURI__) {
        resolve(window.__TAURI__);
      } else if (attempts > 20) { // 20 * 50ms = 1000ms
        resolve(null);
      } else {
        attempts++;
        setTimeout(check, 50);
      }
    };
    check();
  });
};

async function invoke(cmd, args) {
  const t = await waitTauri();
  if (t) {
    return await t.core.invoke(cmd, args);
  } else {
    console.warn(`Tauri invoke not available for: ${cmd}`);
    if (cmd === 'get_system_stats') return { cpu_usage: 0.5, ram_usage: 112.4 };
    return Promise.resolve();
  }
}

async function listen(event, callback) {
  const t = await waitTauri();
  if (t) {
    t.event.listen(event, callback);
  } else {
    console.warn(`Tauri listen not available for: ${event}`);
  }
}

// Tab management
let activeTab = 'voice';
const logsStore = {
  voice: [],
  wakeword: [],
  intent: [],
  actions: [],
  memory: [],
  planner: [],
  groq: []
};

// DOM Elements
const tabButtons = document.querySelectorAll('.tab-btn');
const logsList = document.getElementById('logs-list');
const metricsPanel = document.getElementById('metrics-panel');
const filterContainer = document.getElementById('filter-container');
const logSearch = document.getElementById('log-search');
const clearLogsBtn = document.getElementById('clear-logs-btn');

const topCpu = document.getElementById('top-cpu');
const topRam = document.getElementById('top-ram');
const topState = document.getElementById('top-state');

// Metrics
const metricTtsLatency = document.getElementById('metric-tts-latency');
const metricVadEnergy = document.getElementById('metric-vad-energy');
const metricWhisperLatency = document.getElementById('metric-whisper-latency');
const metricIntentScore = document.getElementById('metric-intent-score');
const metricActionLatency = document.getElementById('metric-action-latency');
const metricMemoriesCount = document.getElementById('metric-memories-count');

// Initialize Console
document.addEventListener('DOMContentLoaded', () => {
  setupTabListeners();
  setupTauriEventListeners();
  startDiagnosticsPolling();
});

// Setup tab buttons
function setupTabListeners() {
  tabButtons.forEach(btn => {
    btn.addEventListener('click', () => {
      tabButtons.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      activeTab = btn.getAttribute('data-tab');
      
      if (activeTab === 'metrics') {
        logsList.style.display = 'none';
        filterContainer.style.display = 'none';
        metricsPanel.style.display = 'grid';
      } else {
        metricsPanel.style.display = 'none';
        logsList.style.display = 'block';
        filterContainer.style.display = 'flex';
        renderLogs();
      }
    });
  });

  clearLogsBtn.addEventListener('click', () => {
    if (activeTab !== 'metrics') {
      logsStore[activeTab] = [];
      updateBadges();
      renderLogs();
    }
  });

  logSearch.addEventListener('input', renderLogs);
}

// Render filtered logs
function renderLogs() {
  logsList.innerHTML = '';
  const query = logSearch.value.trim().toLowerCase();
  const logs = logsStore[activeTab] || [];
  
  const filtered = logs.filter(log => {
    return log.message.toLowerCase().includes(query) || log.level.toLowerCase().includes(query);
  });

  filtered.forEach(log => {
    const line = document.createElement('div');
    line.className = `log-line ${log.level.toLowerCase()}`;
    
    const timeSpan = document.createElement('span');
    timeSpan.className = 'time';
    timeSpan.textContent = new Date(log.timestamp).toLocaleTimeString();

    const levelSpan = document.createElement('span');
    levelSpan.className = 'level';
    levelSpan.textContent = `[${log.level}]`;

    const msgSpan = document.createElement('span');
    msgSpan.className = 'msg';
    msgSpan.textContent = log.message;

    line.appendChild(timeSpan);
    line.appendChild(levelSpan);
    line.appendChild(msgSpan);
    
    logsList.appendChild(line);
  });
  
  logsList.scrollTop = logsList.scrollHeight;
}

// Update log badges counts
function updateBadges() {
  Object.keys(logsStore).forEach(tab => {
    const badge = document.getElementById(`badge-${tab}`);
    if (badge) {
      badge.textContent = logsStore[tab].length;
    }
  });
}

// Setup listeners for Rust events
function setupTauriEventListeners() {
  // Listen to dev-log events
  listen('dev-log', (event) => {
    const { tab, level, message, timestamp } = event.payload;
    const key = tab.toLowerCase().replace(' ', '');
    
    if (logsStore[key]) {
      logsStore[key].push({
        level,
        message,
        timestamp: timestamp || Date.now()
      });
      
      // Limit to last 150 entries per tab
      if (logsStore[key].length > 150) {
        logsStore[key].shift();
      }
      
      updateBadges();
      if (activeTab === key) {
        renderLogs();
      }
    }
  });

  // Listen to performance metrics
  listen('dev-perf', (event) => {
    const { 
      cpu, 
      ram, 
      state, 
      tts_latency, 
      vad_energy, 
      whisper_latency, 
      intent_score, 
      action_latency,
      memories_count
    } = event.payload;

    if (cpu !== undefined) topCpu.textContent = `${cpu.toFixed(1)} %`;
    if (ram !== undefined) topRam.textContent = `${ram.toFixed(1)} MB`;
    if (state !== undefined) {
      topState.textContent = state;
      if (state.toLowerCase() === 'idle') topState.style.color = 'var(--accent-green)';
      else if (state.toLowerCase() === 'listening') topState.style.color = 'var(--accent-cyan)';
      else topState.style.color = 'var(--accent-purple)';
    }

    if (tts_latency !== undefined) metricTtsLatency.textContent = `${tts_latency} ms`;
    if (vad_energy !== undefined) metricVadEnergy.textContent = vad_energy.toFixed(3);
    if (whisper_latency !== undefined) metricWhisperLatency.textContent = `${whisper_latency} ms`;
    if (intent_score !== undefined) metricIntentScore.textContent = intent_score.toFixed(2);
    if (action_latency !== undefined) metricActionLatency.textContent = `${action_latency} ms`;
    if (memories_count !== undefined) metricMemoriesCount.textContent = memories_count.toString();
  });

  // Listen to voice state change
  listen('voice-state-change', (event) => {
    const { state } = event.payload;
    topState.textContent = state;
    if (state.toLowerCase() === 'idle') topState.style.color = 'var(--accent-green)';
    else if (state.toLowerCase() === 'listening') topState.style.color = 'var(--accent-cyan)';
    else topState.style.color = 'var(--accent-purple)';
  });
}

// Diagnostics Polling
function startDiagnosticsPolling() {
  setInterval(async () => {
    try {
      const stats = await invoke('get_system_stats');
      
      // Emit metrics updates directly
      topCpu.textContent = `${stats.cpu_usage.toFixed(1)} %`;
      topRam.textContent = `${stats.ram_usage.toFixed(1)} MB`;
    } catch (err) {
      // Silent error
    }
  }, 2000);
}
