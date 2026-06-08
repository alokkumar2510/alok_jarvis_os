// ALOK OS v3 - Frontend Controller

// Window label is detected by the synchronous inline script in index.html
// (before CSS renders) and stored in window._detectedWindowLabel.
// Use that as the ground truth — no race conditions, no IPC needed.
let windowLabel = window._detectedWindowLabel || 'main';

// Sanity check: also try URL param and Tauri synchronous API
if (windowLabel === 'main') {
  const _urlParams = new URLSearchParams(window.location.search);
  const paramLabel = _urlParams.get('window');
  if (paramLabel) windowLabel = paramLabel;
}
if (window.__TAURI__) {
  console.log("TAURI GLOBAL KEYS:", Object.keys(window.__TAURI__));
  if (window.__TAURI__.webviewWindow) {
    console.log("TAURI webviewWindow KEYS:", Object.keys(window.__TAURI__.webviewWindow));
  }
}
if (windowLabel === 'main' && window.__TAURI__) {
  try {
    if (window.__TAURI__.webviewWindow) {
      const lbl = window.__TAURI__.webviewWindow.getCurrentWebviewWindow().label;
      if (lbl) windowLabel = lbl;
    } else if (window.__TAURI__.window) {
      const lbl = window.__TAURI__.window.getCurrentWindow().label;
      if (lbl) windowLabel = lbl;
    }
  } catch (err) {
    console.error("Error detecting label via Tauri API:", err);
  }
}

// Apply class synchronously before parsing other scripts so background is set immediately
if (windowLabel === 'command_center') {
  document.documentElement.classList.add('window-command-center');
  if (document.body) {
    document.body.className = 'window-command-center';
  } else {
    document.addEventListener('DOMContentLoaded', () => {
      document.body.className = 'window-command-center';
    });
  }
} else {
  document.documentElement.classList.add('window-main');
  if (document.body) {
    document.body.className = 'window-main';
  } else {
    document.addEventListener('DOMContentLoaded', () => {
      document.body.className = 'window-main';
    });
  }
}

// Dynamic wrappers for Tauri APIs to avoid race-condition crashes on startup and support multiple windows/browsers
const getTauri = () => window.__TAURI__;

// Wait for Tauri injection (up to 1000ms) to prevent race condition window labeling errors
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
    if (cmd === 'get_window_label') return 'main';
    if (cmd === 'get_system_stats') return { ram_usage: 112.4, latency_ms: 280 };
    if (cmd === 'load_settings') return { groq_model: 'llama3-8b-8192', wakeword_sensitivity: 0.75, tts_speed: 1.0 };
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

// ==========================================
// STATE MANAGEMENT & CONFIGS
// ==========================================
let currentTab = 'control';
let isBubbleExpanded = false;
let voiceState = 'idle'; // 'idle', 'listening', 'processing', 'speaking'
let presenceState = 'Monitoring'; // 'Sleeping', 'Listening', 'Thinking', 'Speaking', 'Working', 'Monitoring'
let voiceStateMsg = '';
let voiceStateTranscript = '';
let wakeRingProgress = 1.0;
let breathePhase = 0;

// Force-Directed Graph Data
let graphNodes = [
  { id: 'user', label: 'User (Alok)', type: 'person', x: 250, y: 200, vx: 0, vy: 0 },
  { id: 'alok_os', label: 'ALOK OS', type: 'system', x: 400, y: 200, vx: 0, vy: 0 },
  { id: 'chrome', label: 'Chrome', type: 'app', x: 550, y: 120, vx: 0, vy: 0 },
  { id: 'vscode', label: 'VS Code', type: 'app', x: 550, y: 280, vx: 0, vy: 0 },
  { id: 'project_flutter', label: 'Flutter App', type: 'project', x: 700, y: 280, vx: 0, vy: 0 }
];
let graphEdges = [
  { from: 'user', to: 'alok_os', relation: 'Controls' },
  { from: 'alok_os', to: 'chrome', relation: 'Launches' },
  { from: 'alok_os', to: 'vscode', relation: 'Launches' },
  { from: 'vscode', to: 'project_flutter', relation: 'Edits' }
];

let selectedNode = null;
let graphSearchQuery = '';
let draggedNode = null;

// UI Settings Cache
let configSettings = {
  groq_api_key: '',
  groq_model: 'llama3-8b-8192',
  wakeword_sensitivity: 0.75,
  tts_speed: 1.0
};

// ==========================================
// DOM ELEMENTS REFERENCE
// ==========================================
let assistantContainer;
let statusBubble;
let miniCard;
let dashboard;
let statusText;
let transcriptPreview;
let actionFeedback;
let chatLogs;
let manualTextInput;
let sendCommandBtn;
let activeGoalTitle;
let plannerStepsList;
let graphCanvas;
let startupToggle;
let ocrToggle;
let groqApiKey;
let groqModelSelect;
let wakewordSensitivity;
let ttsSpeed;
let subtitlesContainer;
let subtitlesSpeaker;
let subtitlesText;
let closeSubtitlesBtn;
let closeDashboardBtn;
let sensitivityVal;
let speedVal;
let progressPercentageText;

// Proactive Notification DOM Elements
let proactiveToast;
let toastTitle;
let toastMessage;
let toastIcon;
let toastActions;
let toastAcceptBtn;
let toastDeclineBtn;
let closeToastBtn;
let toastTimeout = null;

// Waveform Animation Interval
let waveformCanvas;
let waveformInterval = null;

// ==========================================
// RENDER LOOP & CANVAS CONFIG
// ==========================================
let bgCanvas;
let bgCtx;
let orbCanvas;
let orbCtx;
let waveformCtx = null;
let graphCtx = null;
let controlCtx = null;

// Particle arrays
let bgParticles = [];
const BG_PARTICLE_COUNT = 75;
let orbParticles = [];
const ORB_PARTICLE_COUNT = 30;

let globalTime = 0;
let mouseX = window.innerWidth / 2;
let mouseY = window.innerHeight / 2;

async function checkAndSetupDependencies() {
  try {
    const status = await invoke('check_dependencies');
    if (!status.whisper_cli || !status.whisper_model || !status.piper_cli || !status.piper_model) {
      console.log('Dependencies missing. Showing setup overlay...');
      const overlay = document.getElementById('setup-overlay');
      if (overlay) {
        overlay.classList.remove('hidden');
        overlay.style.opacity = '1';
      }

      // Listen for progress updates
      let setupListener = null;
      setupListener = await listen('dependency-setup-status', (event) => {
        const payload = event.payload;
        const statusText = document.getElementById('setup-status-text');
        const progressPct = document.getElementById('setup-progress-pct');
        const progressFill = document.getElementById('setup-progress-fill');
        
        if (statusText) statusText.textContent = payload.message;
        if (progressPct) progressPct.textContent = `${payload.percent}%`;
        if (progressFill) progressFill.style.width = `${payload.percent}%`;
      });

      // Call download
      await invoke('download_dependencies');
      
      // Setup complete! Clean up overlay
      console.log('Setup finished successfully!');
      if (overlay) {
        gsap.to(overlay, {
          opacity: 0,
          duration: 0.8,
          ease: 'power2.inOut',
          onComplete: () => {
            overlay.classList.add('hidden');
          }
        });
      }
      if (setupListener && typeof setupListener === 'function') {
        setupListener();
      }
    } else {
      console.log('All dependencies present.');
    }
  } catch (err) {
    console.error('Error in dependency setup:', err);
    // Hide overlay in case of error so user isn't stuck
    const overlay = document.getElementById('setup-overlay');
    if (overlay) overlay.classList.add('hidden');
  }
}

// ==========================================
// INITIALIZATION
// ==========================================
window.addEventListener('DOMContentLoaded', () => {
  initializeDOMElements();
  setupEventListeners();
  setupTauriListeners();
  
  // Initialize Canvases
  setupCanvases();
  initBackgroundParticles();
  initOrbParticles();
  
  // Start diagnostics & Lucide Icons
  startSystemDiagnostics();
  
  if (window.lucide) {
    window.lucide.createIcons();
  }
  
  // Run Main Animation Loop
  requestAnimationFrame(mainRenderLoop);
  
  // Initial load
  loadSettings();
  
  // Check window label & apply modes
  checkWindowLabel();

  // Run dependency setup checks
  checkAndSetupDependencies();
});


function initializeDOMElements() {
  assistantContainer = document.getElementById('assistant-container');
  statusBubble = document.getElementById('status-bubble');
  miniCard = document.getElementById('mini-card');
  dashboard = document.getElementById('dashboard');
  
  statusText = document.getElementById('status-text');
  transcriptPreview = document.getElementById('transcript-preview');
  actionFeedback = document.getElementById('action-feedback');
  chatLogs = document.getElementById('chat-logs');
  
  manualTextInput = document.getElementById('manual-text-input');
  sendCommandBtn = document.getElementById('send-command-btn');
  
  activeGoalTitle = document.getElementById('active-goal-title');
  plannerStepsList = document.getElementById('planner-steps-list');
  progressPercentageText = document.getElementById('progress-percentage-text');
  
  graphCanvas = document.getElementById('graph-canvas');
  
  startupToggle = document.getElementById('startup-toggle');
  ocrToggle = document.getElementById('ocr-toggle');
  groqApiKey = document.getElementById('groq-api-key');
  groqModelSelect = document.getElementById('groq-model-select');
  wakewordSensitivity = document.getElementById('wakeword-sensitivity');
  ttsSpeed = document.getElementById('tts-speed');
  
  subtitlesContainer = document.getElementById('subtitles-container');
  subtitlesSpeaker = document.getElementById('subtitles-speaker');
  subtitlesText = document.getElementById('subtitles-text');
  closeSubtitlesBtn = document.getElementById('close-subtitles-btn');
  closeDashboardBtn = document.getElementById('close-dashboard-btn');
  
  sensitivityVal = document.getElementById('sensitivity-val');
  speedVal = document.getElementById('speed-val');
  
  waveformCanvas = document.getElementById('waveform-canvas');
  bgCanvas = document.getElementById('bg-canvas');
  orbCanvas = document.getElementById('orb-canvas');

  // Initialize proactive toast elements
  proactiveToast = document.getElementById('proactive-toast');
  toastTitle = document.getElementById('toast-title');
  toastMessage = document.getElementById('toast-message');
  toastIcon = document.getElementById('toast-icon');
  toastActions = document.getElementById('toast-actions');
  toastAcceptBtn = document.getElementById('toast-accept-btn');
  toastDeclineBtn = document.getElementById('toast-decline-btn');
  closeToastBtn = document.getElementById('close-toast-btn');
}

async function checkWindowLabel() {
  try {
    // Primary: URL param (set by Rust, race-condition-free)
    let resolvedLabel = new URLSearchParams(window.location.search).get('window') || windowLabel;

    // Secondary: Tauri synchronous API
    if (resolvedLabel === 'main' && window.__TAURI__) {
      try {
        if (window.__TAURI__.webviewWindow) {
          const currentWindow = window.__TAURI__.webviewWindow.getCurrentWebviewWindow();
          if (currentWindow && currentWindow.label) {
            resolvedLabel = currentWindow.label;
          }
        } else if (window.__TAURI__.window) {
          const currentWindow = window.__TAURI__.window.getCurrentWindow();
          if (currentWindow && currentWindow.label) {
            resolvedLabel = currentWindow.label;
          }
        }
      } catch (err) {
        console.warn('Sync window label failed:', err);
      }
    }

    // Tertiary: Tauri IPC invoke — ONLY if we still think we're 'main'
    // (don't call this from a command_center window or it routes wrong)
    if (resolvedLabel === 'main') {
      try {
        const invokeLabel = await invoke('get_window_label');
        if (invokeLabel && invokeLabel !== 'main') {
          resolvedLabel = invokeLabel;
        }
      } catch (err) {
        console.warn('invoke get_window_label failed:', err);
      }
    }

    windowLabel = resolvedLabel;
    console.log('checkWindowLabel resolved label to:', windowLabel);

    if (windowLabel === 'command_center') {
      document.body.className = 'window-command-center';
      document.documentElement.className = 'window-command-center';
      currentTab = 'control';

      // Explicitly reveal dashboard for the Command Center window
      if (dashboard) {
        dashboard.classList.remove('hidden');
        dashboard.style.opacity = '1';
        dashboard.style.transform = 'none';
      }

      resizeGraphCanvas();
      resizeControlGraphCanvas();
    } else {
      document.body.className = 'window-main';
      document.documentElement.className = 'window-main';
      await invoke('show_bubble');
    }
  } catch (err) {
    console.error('Failed to check window label:', err);
    document.body.className = 'window-main';
    document.documentElement.className = 'window-main';
  }
}

function setupEventListeners() {
  let isLaunchingCommandCenter = false;
  // Toggle full HUD dashboard on status bubble click
  statusBubble.addEventListener('click', async () => {
    if (document.body.classList.contains('window-main')) {
      if (isLaunchingCommandCenter) return;
      isLaunchingCommandCenter = true;

      // Visual feedback: click animation (shrink & release) using GSAP
      gsap.to(statusBubble, {
        scale: 0.88,
        duration: 0.1,
        yoyo: true,
        repeat: 1,
        ease: 'power2.inOut'
      });

      try {
        await invoke('show_command_center');
      } catch (err) {
        console.error('Failed to show command center:', err);
      } finally {
        // Debounce for 2.5 seconds to prevent spamming window creation requests
        setTimeout(() => {
          isLaunchingCommandCenter = false;
        }, 2500);
      }
    } else {
      toggleDashboard();
    }
  });

  // Close dashboard btn click
  if (closeDashboardBtn) {
    closeDashboardBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      if (document.body.classList.contains('window-command-center')) {
        invoke('close_window');
      } else {
        if (isBubbleExpanded) toggleDashboard();
      }
    });
  }

  // Close Mini Card
  document.getElementById('close-card-btn').addEventListener('click', (e) => {
    e.stopPropagation();
    hideMiniCard();
  });

  // Close Subtitles / stop speech
  if (closeSubtitlesBtn) {
    closeSubtitlesBtn.addEventListener('click', async (e) => {
      e.stopPropagation();
      subtitlesContainer.classList.add('hidden');
      try {
        await invoke('stop_speech');
      } catch (err) {
        console.error('Failed to stop speech:', err);
      }
    });
  }

  // Close Proactive Toast click handler
  if (closeToastBtn) {
    closeToastBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      hideProactiveToast();
    });
  }

  // Toast decline action button
  if (toastDeclineBtn) {
    toastDeclineBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      hideProactiveToast();
    });
  }

  // Toast accept action button
  if (toastAcceptBtn) {
    toastAcceptBtn.addEventListener('click', async (e) => {
      e.stopPropagation();
      const action = toastAcceptBtn.getAttribute('data-action');
      const target = toastAcceptBtn.getAttribute('data-target');
      hideProactiveToast();

      if (action === 'launch_app') {
        try {
          await invoke('process_intent', { text: `open ${target}` });
        } catch (err) {
          console.error('Failed to process launch app intent:', err);
        }
      } else if (action === 'browser_visit') {
        try {
          if (target.startsWith('search:')) {
            const query = target.substring(7);
            await invoke('process_intent', { text: `search for ${query}` });
          } else {
            await invoke('process_intent', { text: `open website ${target}` });
          }
        } catch (err) {
          console.error('Failed to process browser visit intent:', err);
        }
      } else if (action === 'run_intent') {
        try {
          await invoke('process_intent', { text: target });
        } catch (err) {
          console.error('Failed to process run intent:', err);
        }
      }
    });
  }

  // Tab switching using GSAP spring fades
  document.querySelectorAll('.nav-tab').forEach(button => {
    button.addEventListener('click', () => {
      const targetTab = button.getAttribute('data-tab');
      switchTab(targetTab);
    });
  });

  // Manual command execution
  sendCommandBtn.addEventListener('click', submitCommand);
  manualTextInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') submitCommand();
  });

  // Purge dialogue logs
  document.getElementById('clear-chat-btn').addEventListener('click', () => {
    chatLogs.innerHTML = '';
  });

  // Settings modification
  startupToggle.addEventListener('change', async () => {
    await invoke('toggle_startup', { enabled: startupToggle.checked });
  });
  
  ocrToggle.addEventListener('change', async () => {
    await invoke('toggle_ocr', { enabled: ocrToggle.checked });
  });

  wakewordSensitivity.addEventListener('input', () => {
    sensitivityVal.textContent = parseFloat(wakewordSensitivity.value).toFixed(2);
    saveSettings();
  });
  
  ttsSpeed.addEventListener('input', () => {
    speedVal.textContent = parseFloat(ttsSpeed.value).toFixed(1) + 'x';
    saveSettings();
  });

  groqApiKey.addEventListener('input', () => {
    saveSettings();
  });
  
  groqModelSelect.addEventListener('change', () => {
    saveSettings();
  });

  // Window track mouse position
  window.addEventListener('mousemove', (e) => {
    mouseX = e.clientX;
    mouseY = e.clientY;
  });

  // Graph actions: Search filtering, click inspect, and drag physics
  if (graphCanvas) {
    graphCanvas.addEventListener('mousedown', handleGraphMouseDown);
    graphCanvas.addEventListener('mousemove', handleGraphMouseMove);
    window.addEventListener('mouseup', handleGraphMouseUp);
  }
  
  const searchInput = document.getElementById('graph-search');
  if (searchInput) {
    searchInput.addEventListener('input', (e) => {
      graphSearchQuery = e.target.value.toLowerCase().trim();
    });
  }

  // Developer Console Shortcut: CTRL + SHIFT + A
  window.addEventListener('keydown', async (e) => {
    if (e.ctrlKey && e.shiftKey && (e.key === 'A' || e.key === 'a')) {
      e.preventDefault();
      try {
        await invoke('toggle_developer_console');
      } catch (err) {
        console.error('Failed to dev console:', err);
      }
    }
  });

  const mcClearLogBtn = document.getElementById('mc-clear-log-btn');
  if (mcClearLogBtn) {
    mcClearLogBtn.addEventListener('click', () => {
      const mcTerminalLogs = document.getElementById('mc-terminal-logs');
      if (mcTerminalLogs) {
        mcTerminalLogs.innerHTML = '<div class="log-line system">Logs cleared. Monitoring online.</div>';
      }
    });
  }
}

// ==========================================
// LAYOUT ENGINE - TAB SWITCHING & DASHBOARD
// ==========================================
function toggleDashboard() {
  if (isBubbleExpanded) {
    // Animate Dashboard Out
    gsap.to(dashboard, {
      opacity: 0,
      scale: 0.97,
      duration: 0.4,
      ease: 'power3.inOut',
      onComplete: () => {
        dashboard.classList.add('hidden');
        assistantContainer.classList.add('collapsed');
      }
    });
    isBubbleExpanded = false;
  } else {
    // Reveal & Animate Dashboard In
    dashboard.classList.remove('hidden');
    assistantContainer.classList.remove('collapsed');
    
    gsap.fromTo(dashboard, 
      { opacity: 0, scale: 0.97 }, 
      { opacity: 1, scale: 1, duration: 0.5, ease: 'elastic.out(1, 0.85)' }
    );
    
    isBubbleExpanded = true;
    hideMiniCard();
    
    if (currentTab === 'graph') {
      resizeGraphCanvas();
    }
  }
}

function switchTab(tabId) {
  if (tabId === currentTab) return;

  const currentPane = document.getElementById(`pane-${currentTab}`);
  const nextPane = document.getElementById(`pane-${tabId}`);
  const currentPill = document.querySelector(`.nav-tab[data-tab="${currentTab}"]`);
  const nextPill = document.querySelector(`.nav-tab[data-tab="${tabId}"]`);

  if (!nextPane) return;

  if (currentPill) currentPill.classList.remove('active');
  if (nextPill) nextPill.classList.add('active');

  gsap.to(currentPane, {
    opacity: 0,
    y: -10,
    duration: 0.2,
    ease: 'power2.inOut',
    onComplete: () => {
      currentPane.classList.remove('active');
      nextPane.classList.add('active');
      
      gsap.fromTo(nextPane, 
        { opacity: 0, y: 10 }, 
        { opacity: 1, y: 0, duration: 0.35, ease: 'power3.out' }
      );
      
      currentTab = tabId;
      if (tabId === 'graph') {
        loadMemoryGraph();
      } else if (tabId === 'control') {
        resizeControlGraphCanvas();
        updateMissionControlHUD();
      }
    }
  });
}

// ==========================================
// VOICE HUD & FOCUS CARD MANAGEMENT
// ==========================================
function showMiniCard(status, text, feedback) {
  statusText.textContent = status;
  transcriptPreview.textContent = text ? `"${text}"` : '...';
  actionFeedback.textContent = feedback || '';

  if (miniCard.classList.contains('hidden')) {
    miniCard.classList.remove('hidden');
    gsap.fromTo(miniCard, 
      { opacity: 0, scale: 0.9, y: 20 },
      { opacity: 1, scale: 1, y: 0, duration: 0.45, ease: 'elastic.out(1, 0.75)' }
    );
  }
}

function hideMiniCard() {
  if (!miniCard.classList.contains('hidden')) {
    gsap.to(miniCard, {
      opacity: 0,
      scale: 0.9,
      y: 15,
      duration: 0.3,
      ease: 'power3.inOut',
      onComplete: () => {
        miniCard.classList.add('hidden');
      }
    });
  }
}

// Subtitles Bubble popup
let subtitleTimeout = null;
function showSubtitles(speaker, text) {
  if (!subtitlesContainer || !subtitlesSpeaker || !subtitlesText) return;
  
  if (subtitleTimeout) {
    clearTimeout(subtitleTimeout);
  }
  
  subtitlesSpeaker.textContent = speaker.toUpperCase() + ':';
  subtitlesText.textContent = text;
  
  subtitlesSpeaker.className = (speaker.toLowerCase() === 'user') ? 'user' : '';
  
  if (subtitlesContainer.classList.contains('hidden')) {
    subtitlesContainer.classList.remove('hidden');
    
    const isOverlay = document.body.classList.contains('window-main');
    const startY = isOverlay ? -20 : 20; // Slide down from top when in overlay mode
    
    gsap.fromTo(subtitlesContainer, 
      { opacity: 0, y: startY, scale: 0.95, x: isOverlay ? '0%' : '-50%' },
      { opacity: 1, y: 0, scale: 1, x: isOverlay ? '0%' : '-50%', duration: 0.4, ease: 'power3.out' }
    );
  }
  
  subtitleTimeout = setTimeout(() => {
    const isOverlay = document.body.classList.contains('window-main');
    const endY = isOverlay ? -15 : 15;
    
    gsap.to(subtitlesContainer, {
      opacity: 0,
      y: endY,
      scale: 0.95,
      x: isOverlay ? '0%' : '-50%',
      duration: 0.3,
      onComplete: () => subtitlesContainer.classList.add('hidden')
    });
  }, 5000);
}

function addDialogueEntry(sender, message) {
  const bubble = document.createElement('div');
  bubble.classList.add('chat-bubble', sender.toLowerCase());
  
  const content = document.createElement('span');
  content.textContent = message;
  
  const meta = document.createElement('span');
  meta.classList.add('bubble-meta');
  meta.textContent = new Date().toLocaleTimeString();
  
  bubble.appendChild(content);
  bubble.appendChild(meta);
  
  chatLogs.appendChild(bubble);
  
  // Use direct scrollTop instead of GSAP scrollTo (ScrollToPlugin not registered)
  chatLogs.scrollTop = chatLogs.scrollHeight;
}

async function submitCommand() {
  const text = manualTextInput.value.trim();
  if (!text) return;
  
  addDialogueEntry('user', text);
  addMcTerminalLog(`Command input: "${text}"`, 'system');
  manualTextInput.value = '';
  
  try {
    const response = await invoke('process_intent', { text: text });
    addDialogueEntry('alok', response);
    const isCorrection = response.toLowerCase().includes("sorry") || response.toLowerCase().includes("apologize") || response.toLowerCase().includes("corrected");
    addMcTerminalLog(`Execution output: "${response.substring(0, 60)}..."`, isCorrection ? 'learning' : 'system');
  } catch (err) {
    addDialogueEntry('alok', `Execution failed: ${err}`);
    addMcTerminalLog(`Execution error: ${err}`, 'error');
  }
}

// ==========================================
// PRESENCE AUDIO ENGINE (WEB AUDIO API SYNTHESIZER)
// ==========================================
class PresenceAudioEngine {
  constructor() {
    this.ctx = null;
    this.masterGain = null;
    this.volume = 0.04; // Very subtle, professional default volume limit
  }

  init() {
    if (this.ctx) return;
    const AudioContextClass = window.AudioContext || window.webkitAudioContext;
    if (!AudioContextClass) return;
    
    try {
      this.ctx = new AudioContextClass();
      this.masterGain = this.ctx.createGain();
      this.masterGain.gain.setValueAtTime(this.volume, this.ctx.currentTime);
      this.masterGain.connect(this.ctx.destination);
    } catch (e) {
      console.warn("PresenceAudioEngine: Failed to initialize AudioContext:", e);
    }
  }

  playWakeUp() {
    this.init();
    if (!this.ctx) return;
    if (this.ctx.state === 'suspended') this.ctx.resume();

    const now = this.ctx.currentTime;
    
    // Play an elegant, ascending chime (dual sine wave)
    const osc1 = this.ctx.createOscillator();
    const osc2 = this.ctx.createOscillator();
    const gainNode = this.ctx.createGain();
    
    osc1.type = 'sine';
    osc1.frequency.setValueAtTime(520, now); // C5
    osc1.frequency.exponentialRampToValueAtTime(784, now + 0.12); // G5
    
    osc2.type = 'sine';
    osc2.frequency.setValueAtTime(349, now); // F4
    osc2.frequency.exponentialRampToValueAtTime(520, now + 0.12); // C5

    gainNode.gain.setValueAtTime(0, now);
    gainNode.gain.linearRampToValueAtTime(0.5, now + 0.04);
    gainNode.gain.exponentialRampToValueAtTime(0.001, now + 0.3);

    // Warm filter to keep it professional
    const filter = this.ctx.createBiquadFilter();
    filter.type = 'lowpass';
    filter.frequency.setValueAtTime(1000, now);

    osc1.connect(gainNode);
    osc2.connect(gainNode);
    gainNode.connect(filter);
    filter.connect(this.masterGain);

    osc1.start(now);
    osc2.start(now);
    
    osc1.stop(now + 0.35);
    osc2.stop(now + 0.35);
  }

  playThinking() {
    this.init();
    if (!this.ctx) return;
    if (this.ctx.state === 'suspended') this.ctx.resume();

    const now = this.ctx.currentTime;
    
    // A soft, low-frequency double pulse
    const osc = this.ctx.createOscillator();
    const gainNode = this.ctx.createGain();
    
    osc.type = 'sine';
    osc.frequency.setValueAtTime(200, now); // G3

    gainNode.gain.setValueAtTime(0, now);
    gainNode.gain.linearRampToValueAtTime(0.7, now + 0.06);
    gainNode.gain.exponentialRampToValueAtTime(0.001, now + 0.22);

    const filter = this.ctx.createBiquadFilter();
    filter.type = 'lowpass';
    filter.frequency.setValueAtTime(500, now);

    osc.connect(gainNode);
    gainNode.connect(filter);
    filter.connect(this.masterGain);

    osc.start(now);
    osc.stop(now + 0.25);
  }

  playSpeakingStart() {
    this.init();
    if (!this.ctx) return;
    if (this.ctx.state === 'suspended') this.ctx.resume();

    const now = this.ctx.currentTime;
    
    // Play a gentle, crisp double-tap tone
    const osc1 = this.ctx.createOscillator();
    const osc2 = this.ctx.createOscillator();
    const gainNode = this.ctx.createGain();
    
    osc1.type = 'sine';
    osc1.frequency.setValueAtTime(659.25, now); // E5
    osc1.frequency.setValueAtTime(880, now + 0.05); // A5

    osc2.type = 'sine';
    osc2.frequency.setValueAtTime(440, now); // A4
    osc2.frequency.setValueAtTime(659.25, now + 0.05); // E5

    gainNode.gain.setValueAtTime(0, now);
    gainNode.gain.linearRampToValueAtTime(0.4, now + 0.02);
    gainNode.gain.setValueAtTime(0.3, now + 0.05);
    gainNode.gain.exponentialRampToValueAtTime(0.001, now + 0.2);

    const filter = this.ctx.createBiquadFilter();
    filter.type = 'lowpass';
    filter.frequency.setValueAtTime(1500, now);

    osc1.connect(gainNode);
    osc2.connect(gainNode);
    gainNode.connect(filter);
    filter.connect(this.masterGain);

    osc1.start(now);
    osc2.start(now);
    
    osc1.stop(now + 0.25);
    osc2.stop(now + 0.25);
  }

  playSleeping() {
    this.init();
    if (!this.ctx) return;
    if (this.ctx.state === 'suspended') this.ctx.resume();
    const now = this.ctx.currentTime;
    const osc = this.ctx.createOscillator();
    const gainNode = this.ctx.createGain();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(90, now);
    osc.frequency.exponentialRampToValueAtTime(70, now + 0.5);
    gainNode.gain.setValueAtTime(0, now);
    gainNode.gain.linearRampToValueAtTime(0.3, now + 0.1);
    gainNode.gain.exponentialRampToValueAtTime(0.001, now + 0.6);
    osc.connect(gainNode);
    gainNode.connect(this.masterGain);
    osc.start(now);
    osc.stop(now + 0.65);
  }

  playWorking() {
    this.init();
    if (!this.ctx) return;
    if (this.ctx.state === 'suspended') this.ctx.resume();
    const now = this.ctx.currentTime;
    const osc = this.ctx.createOscillator();
    const gainNode = this.ctx.createGain();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(580, now);
    osc.frequency.exponentialRampToValueAtTime(880, now + 0.08);
    gainNode.gain.setValueAtTime(0, now);
    gainNode.gain.linearRampToValueAtTime(0.4, now + 0.02);
    gainNode.gain.exponentialRampToValueAtTime(0.001, now + 0.15);
    osc.connect(gainNode);
    gainNode.connect(this.masterGain);
    osc.start(now);
    osc.stop(now + 0.20);
  }

  playMonitoring() {
    this.init();
    if (!this.ctx) return;
    if (this.ctx.state === 'suspended') this.ctx.resume();
    const now = this.ctx.currentTime;
    const osc = this.ctx.createOscillator();
    const gainNode = this.ctx.createGain();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(440, now);
    osc.frequency.exponentialRampToValueAtTime(360, now + 0.2);
    gainNode.gain.setValueAtTime(0, now);
    gainNode.gain.linearRampToValueAtTime(0.2, now + 0.05);
    gainNode.gain.exponentialRampToValueAtTime(0.001, now + 0.25);
    osc.connect(gainNode);
    gainNode.connect(this.masterGain);
    osc.start(now);
    osc.stop(now + 0.30);
  }
}

const presenceAudio = new PresenceAudioEngine();

// ==========================================
// BACKGROUND & ASSISTANT ORB ANIMATION ENGINE
// ==========================================
function resizeControlGraphCanvas() {
  const ctrlCanvas = document.getElementById('control-graph-canvas');
  if (ctrlCanvas && ctrlCanvas.parentElement) {
    ctrlCanvas.width = ctrlCanvas.parentElement.offsetWidth;
    ctrlCanvas.height = ctrlCanvas.parentElement.offsetHeight;
    initControlGraph();
  }
}

function setupCanvases() {
  window.addEventListener('resize', () => {
    resizeBackgroundCanvas();
    resizeControlGraphCanvas();
  });
  resizeBackgroundCanvas();
  orbCanvas.width = 160;
  orbCanvas.height = 160;
  
  resizeControlGraphCanvas();
}

function resizeBackgroundCanvas() {
  bgCanvas.width = window.innerWidth;
  bgCanvas.height = window.innerHeight;
}

function initBackgroundParticles() {
  bgParticles = [];
  for (let i = 0; i < BG_PARTICLE_COUNT; i++) {
    bgParticles.push({
      x: Math.random() * window.innerWidth,
      y: Math.random() * window.innerHeight,
      z: Math.random() * 0.8 + 0.2,
      size: Math.random() * 1.5 + 0.5,
      angle: Math.random() * Math.PI * 2,
      speed: Math.random() * 0.15 + 0.05
    });
  }
}

function initOrbParticles() {
  orbParticles = [];
  for (let i = 0; i < ORB_PARTICLE_COUNT; i++) {
    orbParticles.push({
      angle: Math.random() * Math.PI * 2,
      radius: Math.random() * 32 + 20,
      speed: (Math.random() * 0.02 + 0.005) * (Math.random() > 0.5 ? 1 : -1),
      size: Math.random() * 2 + 1,
      phase: Math.random() * 100,
      opacity: Math.random() * 0.6 + 0.3
    });
  }
}

function mainRenderLoop() {
  globalTime += 1;
  
  if (document.body.classList.contains('window-command-center')) {
    drawBackground();
  }
  
  drawOrb();
  
  if (document.body.classList.contains('window-command-center') && currentTab === 'graph') {
    updateAndDrawGraph();
  }
  
  if (document.body.classList.contains('window-command-center') && currentTab === 'control') {
    drawControlGraph();
  }
  
  requestAnimationFrame(mainRenderLoop);
}

function drawBackground() {
  if (!bgCtx) {
    bgCtx = bgCanvas.getContext('2d');
  }
  bgCtx.clearRect(0, 0, bgCanvas.width, bgCanvas.height);
  
  const mxOffset = (mouseX - bgCanvas.width / 2) * 0.02;
  const myOffset = (mouseY - bgCanvas.height / 2) * 0.02;
  
  bgCtx.fillStyle = 'rgba(0, 245, 255, 0.05)';
  
  bgParticles.forEach(p => {
    p.angle += 0.0005;
    p.x += Math.cos(p.angle) * p.speed;
    p.y += Math.sin(p.angle) * p.speed;
    
    if (p.x < 0) p.x = bgCanvas.width;
    if (p.x > bgCanvas.width) p.x = 0;
    if (p.y < 0) p.y = bgCanvas.height;
    if (p.y > bgCanvas.height) p.y = 0;
    
    const renderX = p.x + mxOffset * p.z;
    const renderY = p.y + myOffset * p.z;
    
    bgCtx.beginPath();
    bgCtx.fillStyle = `rgba(0, 245, 255, ${p.z * 0.15})`;
    bgCtx.arc(renderX, renderY, p.size * p.z, 0, Math.PI * 2);
    bgCtx.fill();
  });
}

function drawOrb() {
  if (!orbCtx) {
    orbCtx = orbCanvas.getContext('2d');
  }
  const cx = orbCanvas.width / 2;
  const cy = orbCanvas.height / 2;
  
  orbCtx.clearRect(0, 0, orbCanvas.width, orbCanvas.height);
  
  let baseRadius = 24;
  let coreColor = 'rgba(138, 43, 226, 0.32)';
  let glowColor = 'rgba(0, 245, 255, 0.16)';
  
  // 1. Set parameters for each presence state
  if (presenceState === 'Sleeping') {
    // Midnight slow breathing
    const breathVal = Math.sin(globalTime * 0.008);
    const breathFactor = (breathVal + 1.0) / 2.0;
    baseRadius = 18.0 + breathFactor * 3.0;
    coreColor = 'rgba(20, 20, 80, 0.12)';
    glowColor = 'rgba(10, 10, 40, 0.08)';
  } else if (presenceState === 'Listening') {
    baseRadius = 28;
    coreColor = 'rgba(0, 245, 255, 0.4)';
    glowColor = 'rgba(0, 245, 255, 0.45)';
  } else if (presenceState === 'Thinking') {
    baseRadius = 22;
    coreColor = 'rgba(0, 229, 255, 0.45)';
    glowColor = 'rgba(138, 43, 226, 0.55)';
  } else if (presenceState === 'Speaking') {
    baseRadius = 28;
    coreColor = 'rgba(138, 43, 226, 0.5)';
    glowColor = 'rgba(0, 245, 255, 0.35)';
  } else if (presenceState === 'Working') {
    baseRadius = 26;
    coreColor = 'rgba(0, 255, 100, 0.25)';
    glowColor = 'rgba(0, 255, 100, 0.15)';
  } else { // Monitoring (Default / Idle)
    const breathVal = Math.sin(globalTime * 0.022);
    const breathFactor = (breathVal + 1.0) / 2.0;
    baseRadius = 23.0 + breathFactor * 3.5;
    coreColor = 'rgba(0, 150, 255, 0.25)';
    glowColor = 'rgba(0, 245, 255, 0.12)';
  }
  
  const pulseSpeed = presenceState === 'Listening' ? 0.025 : (presenceState === 'Thinking' ? 0.065 : 0.015);
  const pulse = (presenceState === 'Sleeping' || presenceState === 'Monitoring') ? 0.0 : Math.sin(globalTime * pulseSpeed) * 2.5;
  const orbRadius = baseRadius + pulse;
  
  // Radial glow gradient background
  const glowGrad = orbCtx.createRadialGradient(cx, cy, 3, cx, cy, orbRadius * 2);
  glowGrad.addColorStop(0, coreColor);
  glowGrad.addColorStop(0.5, glowColor);
  glowGrad.addColorStop(1, 'rgba(11, 15, 25, 0)');
  
  orbCtx.beginPath();
  orbCtx.fillStyle = glowGrad;
  orbCtx.arc(cx, cy, orbRadius * 2, 0, Math.PI * 2);
  orbCtx.fill();
  
  // Draw current state visuals
  if (presenceState === 'Listening') {
    // Elegant listening bars
    const bars = 24;
    for (let i = 0; i < bars; i++) {
      const angle = (i / bars) * Math.PI * 2 + (globalTime * 0.012);
      const val = Math.sin(globalTime * 0.12 + i) * 6 + 10;
      const x1 = cx + Math.cos(angle) * orbRadius;
      const y1 = cy + Math.sin(angle) * orbRadius;
      const x2 = cx + Math.cos(angle) * (orbRadius + val);
      const y2 = cy + Math.sin(angle) * (orbRadius + val);
      
      orbCtx.beginPath();
      orbCtx.strokeStyle = 'rgba(0, 245, 255, 0.8)';
      orbCtx.lineWidth = 2.0;
      orbCtx.moveTo(x1, y1);
      orbCtx.lineTo(x2, y2);
      orbCtx.stroke();
    }
  } else if (presenceState === 'Thinking') {
    // Cogitation swirl (multi-lobed rose curve)
    orbCtx.save();
    orbCtx.translate(cx, cy);
    
    // Outer thinking rose curve
    orbCtx.beginPath();
    orbCtx.strokeStyle = 'rgba(0, 245, 255, 0.65)';
    orbCtx.lineWidth = 1.5;
    const lobesOuter = 5;
    const rotOuter = globalTime * 0.025;
    for (let theta = 0; theta <= Math.PI * 2; theta += 0.05) {
      const r = orbRadius + Math.cos(lobesOuter * theta + rotOuter) * 7;
      const x = Math.cos(theta) * r;
      const y = Math.sin(theta) * r;
      if (theta === 0) orbCtx.moveTo(x, y);
      else orbCtx.lineTo(x, y);
    }
    orbCtx.closePath();
    orbCtx.stroke();
    
    // Inner thinking counter-rotating swirl
    orbCtx.beginPath();
    orbCtx.strokeStyle = 'rgba(138, 43, 226, 0.45)';
    orbCtx.lineWidth = 1.0;
    const lobesInner = 3;
    const rotInner = -globalTime * 0.04;
    for (let theta = 0; theta <= Math.PI * 2; theta += 0.05) {
      const r = (orbRadius - 6) + Math.sin(lobesInner * theta + rotInner) * 4;
      const x = Math.cos(theta) * r;
      const y = Math.sin(theta) * r;
      if (theta === 0) orbCtx.moveTo(x, y);
      else orbCtx.lineTo(x, y);
    }
    orbCtx.closePath();
    orbCtx.stroke();
    
    orbCtx.restore();
  } else if (presenceState === 'Speaking') {
    // Speech wave envelope
    orbCtx.beginPath();
    orbCtx.strokeStyle = 'rgba(255, 255, 255, 0.85)';
    orbCtx.lineWidth = 2.0;
    
    const segments = 60;
    for (let i = 0; i <= segments; i++) {
      const angle = (i / segments) * Math.PI * 2;
      const waveVal = Math.sin(i * 0.25 + globalTime * 0.15) * Math.cos(globalTime * 0.04) * 7 +
                      Math.cos(i * 0.7 - globalTime * 0.18) * 3;
      const r = orbRadius + waveVal;
      const x = cx + Math.cos(angle) * r;
      const y = cy + Math.sin(angle) * r;
      
      if (i === 0) orbCtx.moveTo(x, y);
      else orbCtx.lineTo(x, y);
    }
    orbCtx.closePath();
    orbCtx.stroke();

    // Outer faint secondary outline
    orbCtx.beginPath();
    orbCtx.strokeStyle = 'rgba(0, 245, 255, 0.3)';
    orbCtx.lineWidth = 1.0;
    for (let i = 0; i <= segments; i++) {
      const angle = (i / segments) * Math.PI * 2;
      const waveVal = Math.sin(i * 0.4 - globalTime * 0.08) * Math.cos(globalTime * 0.05) * 11;
      const r = orbRadius + 4 + waveVal;
      const x = cx + Math.cos(angle) * r;
      const y = cy + Math.sin(angle) * r;
      
      if (i === 0) orbCtx.moveTo(x, y);
      else orbCtx.lineTo(x, y);
    }
    orbCtx.closePath();
    orbCtx.stroke();
  } else if (presenceState === 'Working') {
    // Pulsing green matrix orbits
    orbCtx.save();
    orbCtx.translate(cx, cy);
    const orbits = 3;
    const dotColor = 'rgba(0, 255, 100, 0.85)';
    const lineColor = 'rgba(0, 255, 100, 0.25)';
    
    let points = [];
    for (let o = 0; o < orbits; o++) {
      const radius = orbRadius - 10 + o * 10;
      const dots = 4;
      const rotation = globalTime * (0.01 + o * 0.005) * (o % 2 === 0 ? 1 : -1);
      
      for (let d = 0; d < dots; d++) {
        const angle = (d / dots) * Math.PI * 2 + rotation;
        const px = Math.cos(angle) * radius;
        const py = Math.sin(angle) * radius;
        points.push({ x: px, y: py });
        
        orbCtx.beginPath();
        orbCtx.fillStyle = dotColor;
        orbCtx.arc(px, py, 2.5, 0, Math.PI * 2);
        orbCtx.fill();
      }
    }
    
    // Draw interconnective lines between close nodes
    orbCtx.strokeStyle = lineColor;
    orbCtx.lineWidth = 0.8;
    for (let i = 0; i < points.length; i++) {
      for (let j = i + 1; j < points.length; j++) {
        const dx = points[i].x - points[j].x;
        const dy = points[i].y - points[j].y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist < 28) {
          orbCtx.beginPath();
          orbCtx.moveTo(points[i].x, points[i].y);
          orbCtx.lineTo(points[j].x, points[j].y);
          orbCtx.stroke();
        }
      }
    }
    orbCtx.restore();
  } else if (presenceState === 'Monitoring') {
    // Slow radial sweep (radar style)
    orbCtx.save();
    orbCtx.translate(cx, cy);
    
    const angle = (globalTime * 0.015) % (Math.PI * 2);
    
    // Radar sweep sector gradient
    orbCtx.beginPath();
    orbCtx.moveTo(0, 0);
    const sweepSegments = 30;
    for (let s = 0; s <= sweepSegments; s++) {
      const a = angle - (s / sweepSegments) * 0.8; // sweep angle fading backwards
      const r = orbRadius * 1.8;
      const px = Math.cos(a) * r;
      const py = Math.sin(a) * r;
      orbCtx.lineTo(px, py);
    }
    orbCtx.closePath();
    
    const radarGrad = orbCtx.createRadialGradient(0, 0, 2, 0, 0, orbRadius * 1.8);
    radarGrad.addColorStop(0, 'rgba(0, 245, 255, 0.15)');
    radarGrad.addColorStop(0.5, 'rgba(0, 150, 255, 0.08)');
    radarGrad.addColorStop(1, 'rgba(0, 245, 255, 0)');
    orbCtx.fillStyle = radarGrad;
    orbCtx.fill();
    
    // Main sweep line
    const lx = Math.cos(angle) * (orbRadius * 1.8);
    const ly = Math.sin(angle) * (orbRadius * 1.8);
    orbCtx.beginPath();
    orbCtx.strokeStyle = 'rgba(0, 245, 255, 0.7)';
    orbCtx.lineWidth = 1.5;
    orbCtx.moveTo(0, 0);
    orbCtx.lineTo(lx, ly);
    orbCtx.stroke();
    
    orbCtx.restore();
    
    // Ambient Core Glow for Idle/Monitoring
    orbCtx.beginPath();
    const coreGrad = orbCtx.createRadialGradient(cx, cy, 2, cx, cy, orbRadius);
    coreGrad.addColorStop(0, '#FFFFFF');
    coreGrad.addColorStop(0.35, 'rgba(0, 245, 255, 0.75)');
    coreGrad.addColorStop(0.75, 'rgba(138, 43, 226, 0.75)');
    coreGrad.addColorStop(1, 'rgba(11, 15, 25, 0.9)');
    
    orbCtx.fillStyle = coreGrad;
    orbCtx.arc(cx, cy, orbRadius, 0, Math.PI * 2);
    orbCtx.fill();
  } else {
    // Generic backup (Ambient Core Glow for Idle)
    orbCtx.beginPath();
    const coreGrad = orbCtx.createRadialGradient(cx, cy, 2, cx, cy, orbRadius);
    coreGrad.addColorStop(0, '#FFFFFF');
    coreGrad.addColorStop(0.35, 'rgba(0, 245, 255, 0.75)');
    coreGrad.addColorStop(0.75, 'rgba(138, 43, 226, 0.75)');
    coreGrad.addColorStop(1, 'rgba(11, 15, 25, 0.9)');
    
    orbCtx.fillStyle = coreGrad;
    orbCtx.arc(cx, cy, orbRadius, 0, Math.PI * 2);
    orbCtx.fill();
  }
  
  // 2. Draw Wake-Up Shockwave ring if active
  if (wakeRingProgress < 1.0) {
    wakeRingProgress += 0.038;
    const ringRadius = orbRadius * (1.0 + wakeRingProgress * 1.4);
    const alpha = 1.0 - wakeRingProgress;
    
    orbCtx.beginPath();
    orbCtx.strokeStyle = `rgba(0, 245, 255, ${alpha * 0.7})`;
    orbCtx.lineWidth = 2.5 * (1.0 - wakeRingProgress);
    orbCtx.arc(cx, cy, ringRadius, 0, Math.PI * 2);
    orbCtx.stroke();
    
    orbCtx.beginPath();
    orbCtx.strokeStyle = `rgba(255, 255, 255, ${alpha * 0.4})`;
    orbCtx.lineWidth = 1.0;
    orbCtx.arc(cx, cy, ringRadius - 4, 0, Math.PI * 2);
    orbCtx.stroke();
  }
  
  // 3. Draw surrounding orbiting particles
  orbParticles.forEach(p => {
    const targetRadius = (presenceState === 'Thinking') ? p.radius * 0.55 : p.radius;
    const speedMultiplier = (presenceState === 'Thinking') ? 2.5 : (presenceState === 'Listening' ? 1.4 : 1.0);
    
    p.angle += p.speed * speedMultiplier;
    
    const px = cx + Math.cos(p.angle) * targetRadius;
    const py = cy + Math.sin(p.angle) * targetRadius + Math.sin(p.phase + globalTime * 0.025) * 4.5;
    
    let partColor = `rgba(138, 43, 226, ${p.opacity})`;
    if (presenceState === 'Listening') partColor = `rgba(0, 245, 255, ${p.opacity})`;
    else if (presenceState === 'Working') partColor = `rgba(0, 255, 100, ${p.opacity})`;
    else if (presenceState === 'Thinking') partColor = `rgba(138, 43, 226, ${p.opacity})`;
    
    orbCtx.beginPath();
    orbCtx.fillStyle = partColor;
    orbCtx.arc(px, py, p.size, 0, Math.PI * 2);
    orbCtx.fill();
    
    if (presenceState === 'Thinking' && Math.random() < 0.08) {
      orbCtx.beginPath();
      orbCtx.strokeStyle = 'rgba(0, 245, 255, 0.15)';
      orbCtx.lineWidth = 0.5;
      orbCtx.moveTo(cx, cy);
      orbCtx.lineTo(px, py);
      orbCtx.stroke();
    }
  });
}

let waveTime = 0;
function drawFocusCardWaveform() {
  if (!waveformCanvas) return;
  if (!waveformCtx) {
    waveformCtx = waveformCanvas.getContext('2d');
  }
  const ctx = waveformCtx;
  
  if (waveformCanvas.width !== waveformCanvas.parentElement.offsetWidth) {
    waveformCanvas.width = waveformCanvas.parentElement.offsetWidth;
    waveformCanvas.height = waveformCanvas.parentElement.offsetHeight;
  }
  
  ctx.clearRect(0, 0, waveformCanvas.width, waveformCanvas.height);
  waveTime += 0.15;
  
  const midY = waveformCanvas.height / 2;
  const w = waveformCanvas.width;
  ctx.lineWidth = 2;
  
  for (let layer = 0; layer < 3; layer++) {
    ctx.beginPath();
    ctx.strokeStyle = layer === 0 ? 'rgba(0, 245, 255, 0.8)' : layer === 1 ? 'rgba(138, 43, 226, 0.5)' : 'rgba(255,255,255,0.2)';
    ctx.lineWidth = layer === 0 ? 2.5 : 1.5;
    
    for (let x = 0; x < w; x++) {
      const frequency = 0.025 + (layer * 0.01);
      const amplitude = layer === 0 ? 10 : layer === 1 ? 6 : 3;
      const speed = waveTime * (1 + layer * 0.5);
      
      const dampening = Math.sin((x / w) * Math.PI);
      const y = midY + Math.sin(x * frequency - speed) * amplitude * dampening * (Math.sin(waveTime * 0.02) * 0.5 + 0.8);
      
      if (x === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.stroke();
  }
}

// ==========================================
// FORCE-DIRECTED MEMORY GRAPH CANVAS
// ==========================================
function resizeGraphCanvas() {
  const container = graphCanvas.parentElement;
  graphCanvas.width = container.offsetWidth;
  graphCanvas.height = container.offsetHeight;
  
  const cx = graphCanvas.width / 2;
  const cy = graphCanvas.height / 2;
  graphNodes.forEach((node, i) => {
    if (node.x === 0 || node.x === undefined || Math.abs(node.x - 250) < 5) {
      const ang = (i / graphNodes.length) * Math.PI * 2;
      node.x = cx + Math.cos(ang) * 150;
      node.y = cy + Math.sin(ang) * 150;
    }
  });
}

function updateAndDrawGraph() {
  const canvas = graphCanvas;
  if (!graphCtx) {
    graphCtx = canvas.getContext('2d');
  }
  const ctx = graphCtx;
  const width = canvas.width;
  const height = canvas.height;
  const cx = width / 2;
  const cy = height / 2;

  const kAttract = 0.003;
  const kRepel = 1200;
  const restLength = 140;
  const damping = 0.82;
  const gravity = 0.05;

  for (let i = 0; i < graphNodes.length; i++) {
    const u = graphNodes[i];
    for (let j = i + 1; j < graphNodes.length; j++) {
      const v = graphNodes[j];
      const dx = v.x - u.x;
      const dy = v.y - u.y;
      const distSq = dx * dx + dy * dy + 0.01;
      const dist = Math.sqrt(distSq);
      
      if (dist < 350) {
        const force = kRepel / distSq;
        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;
        
        if (u !== draggedNode) {
          u.vx -= fx;
          u.vy -= fy;
        }
        if (v !== draggedNode) {
          v.vx += fx;
          v.vy += fy;
        }
      }
    }
  }

  graphEdges.forEach(edge => {
    const u = graphNodes.find(n => n.id === edge.from);
    const v = graphNodes.find(n => n.id === edge.to);
    
    if (u && v) {
      const dx = v.x - u.x;
      const dy = v.y - u.y;
      const dist = Math.sqrt(dx * dx + dy * dy) + 0.01;
      const force = (dist - restLength) * kAttract;
      const fx = (dx / dist) * force;
      const fy = (dy / dist) * force;
      
      if (u !== draggedNode) {
        u.vx += fx;
        u.vy += fy;
      }
      if (v !== draggedNode) {
        v.vx -= fx;
        v.vy -= fy;
      }
    }
  });

  graphNodes.forEach(node => {
    if (node === draggedNode) return;
    node.vx += (cx - node.x) * gravity;
    node.vy += (cy - node.y) * gravity;
    
    const mdx = node.x - mouseX;
    const mdy = node.y - mouseY;
    const mdistSq = mdx * mdx + mdy * mdy + 0.01;
    const mdist = Math.sqrt(mdistSq);
    if (mdist < 100) {
      const force = 150 / mdist;
      node.vx += (mdx / mdist) * force;
      node.vy += (mdy / mdist) * force;
    }

    node.vx *= damping;
    node.vy *= damping;
    node.x += node.vx;
    node.y += node.vy;
    
    node.x = Math.max(40, Math.min(width - 40, node.x));
    node.y = Math.max(40, Math.min(height - 40, node.y));
  });

  ctx.clearRect(0, 0, width, height);

  graphEdges.forEach(edge => {
    const u = graphNodes.find(n => n.id === edge.from);
    const v = graphNodes.find(n => n.id === edge.to);
    
    if (u && v) {
      const isHighlighted = (selectedNode && (selectedNode.id === u.id || selectedNode.id === v.id));
      ctx.beginPath();
      ctx.strokeStyle = isHighlighted ? 'rgba(0, 245, 255, 0.4)' : 'rgba(255, 255, 255, 0.06)';
      ctx.lineWidth = isHighlighted ? 2.5 : 1.25;
      ctx.moveTo(u.x, u.y);
      ctx.lineTo(v.x, v.y);
      ctx.stroke();

      const midX = (u.x + v.x) / 2;
      const midY = (u.y + v.y) / 2;
      ctx.fillStyle = isHighlighted ? 'rgba(0, 245, 255, 0.85)' : 'var(--text-muted)';
      ctx.font = '8.5px var(--font-mono)';
      ctx.textAlign = 'center';
      ctx.fillText(edge.relation, midX, midY - 6);

      const speed = 0.0015;
      const progress = (globalTime * speed + (graphEdges.indexOf(edge) * 0.25)) % 1;
      const pulseX = u.x + (v.x - u.x) * progress;
      const pulseY = u.y + (v.y - u.y) * progress;
      
      ctx.beginPath();
      ctx.fillStyle = isHighlighted ? '#FFFFFF' : 'var(--accent-electric)';
      ctx.shadowColor = 'var(--accent-electric)';
      ctx.shadowBlur = 8;
      ctx.arc(pulseX, pulseY, 3, 0, Math.PI * 2);
      ctx.fill();
      ctx.shadowBlur = 0;
    }
  });

  graphNodes.forEach(node => {
    const isSelected = selectedNode && selectedNode.id === node.id;
    const isMatched = graphSearchQuery !== '' && node.label.toLowerCase().includes(graphSearchQuery);
    
    if (isMatched) {
      ctx.beginPath();
      ctx.strokeStyle = 'rgba(0, 245, 255, 0.5)';
      ctx.lineWidth = 2.0;
      ctx.arc(node.x, node.y, 42 + Math.sin(globalTime * 0.1) * 3, 0, Math.PI * 2);
      ctx.stroke();
    }
    
    if (isSelected) {
      ctx.beginPath();
      const haloGrad = ctx.createRadialGradient(node.x, node.y, 20, node.x, node.y, 45);
      haloGrad.addColorStop(0, 'rgba(0, 245, 255, 0.25)');
      haloGrad.addColorStop(1, 'rgba(0, 245, 255, 0)');
      ctx.fillStyle = haloGrad;
      ctx.arc(node.x, node.y, 45, 0, Math.PI * 2);
      ctx.fill();
    }

    ctx.beginPath();
    let nodeColor = 'rgba(255, 255, 255, 0.03)';
    let strokeColor = 'rgba(255, 255, 255, 0.1)';
    let accentColor = 'rgba(255,255,255,0.4)';

    if (node.type === 'person') {
      nodeColor = 'rgba(0, 245, 255, 0.06)';
      strokeColor = 'rgba(0, 245, 255, 0.3)';
      accentColor = 'var(--accent-cyan)';
    } else if (node.type === 'system') {
      nodeColor = 'rgba(138, 43, 226, 0.06)';
      strokeColor = 'rgba(138, 43, 226, 0.3)';
      accentColor = 'var(--accent-violet)';
    } else if (node.type === 'app') {
      nodeColor = 'rgba(16, 185, 129, 0.06)';
      strokeColor = 'rgba(16, 185, 129, 0.3)';
      accentColor = 'var(--accent-emerald)';
    } else if (node.type === 'project') {
      nodeColor = 'rgba(244, 63, 94, 0.06)';
      strokeColor = 'rgba(244, 63, 94, 0.3)';
      accentColor = 'var(--accent-rose)';
    }

    ctx.fillStyle = isSelected ? 'rgba(0, 245, 255, 0.12)' : nodeColor;
    ctx.strokeStyle = isSelected ? 'var(--accent-electric)' : strokeColor;
    ctx.lineWidth = isSelected ? 2.5 : 1.25;
    
    const radius = 34;
    ctx.arc(node.x, node.y, radius, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();

    ctx.fillStyle = '#FFFFFF';
    ctx.font = '10px var(--font-primary)';
    ctx.fontWeight = '500';
    ctx.textAlign = 'center';
    ctx.fillText(node.label, node.x, node.y + 3);

    ctx.fillStyle = accentColor;
    ctx.font = '6.5px var(--font-mono)';
    ctx.fillText(node.type.toUpperCase(), node.x, node.y + 15);
  });
}

function handleGraphMouseDown(event) {
  const rect = graphCanvas.getBoundingClientRect();
  const clickX = event.clientX - rect.left;
  const clickY = event.clientY - rect.top;

  const node = graphNodes.find(n => {
    const dist = Math.hypot(n.x - clickX, n.y - clickY);
    return dist <= 34;
  });

  if (node) {
    draggedNode = node;
    selectedNode = node;
    displayInspectorDetails(node);
  } else {
    selectedNode = null;
    displayInspectorDetails(null);
  }
}

function handleGraphMouseMove(event) {
  if (draggedNode) {
    const rect = graphCanvas.getBoundingClientRect();
    draggedNode.x = event.clientX - rect.left;
    draggedNode.y = event.clientY - rect.top;
    draggedNode.vx = 0;
    draggedNode.vy = 0;
  }
}

function handleGraphMouseUp() {
  draggedNode = null;
}

function displayInspectorDetails(node) {
  const inspector = document.getElementById('inspector-details');
  if (!inspector) return;

  if (node) {
    const incoming = graphEdges.filter(e => e.to === node.id).map(e => {
      const srcNode = graphNodes.find(n => n.id === e.from);
      return `${srcNode ? srcNode.label : e.from} ➔ [${e.relation}] ➔ Self`;
    });
    const outgoing = graphEdges.filter(e => e.from === node.id).map(e => {
      const destNode = graphNodes.find(n => n.id === e.to);
      return `Self ➔ [${e.relation}] ➔ ${destNode ? destNode.label : e.to}`;
    });
    const relations = [...incoming, ...outgoing];

    inspector.innerHTML = `
      <div class="inspector-card">
        <h4>${node.label} <span class="tag">${node.type}</span></h4>
        <p><strong>System ID:</strong> <code style="font-family: var(--font-mono); font-size: 0.75rem; color: var(--accent-electric);">${node.id}</code></p>
        <p><strong>Network Connections:</strong></p>
        <ul>
          ${relations.length > 0 
            ? relations.map(r => `<li>${r}</li>`).join('') 
            : '<li style="color: var(--text-muted); font-style: italic;">No active relations</li>'}
        </ul>
      </div>
    `;
  } else {
    inspector.innerHTML = `
      <div class="inspector-empty">
        Select a node in the Memory Graph to inspect metadata, history, and neural links.
      </div>
    `;
  }
}

// ==========================================
// TAURI COMMAND & EVENT INTEGRATIONS
// ==========================================
async function loadSettings() {
  try {
    const config = await invoke('load_settings');
    configSettings = config;
    
    groqApiKey.value = config.groq_api_key || '';
    groqModelSelect.value = config.groq_model || 'llama3-8b-8192';
    
    wakewordSensitivity.value = config.wakeword_sensitivity || 0.75;
    sensitivityVal.textContent = parseFloat(wakewordSensitivity.value).toFixed(2);
    
    ttsSpeed.value = config.tts_speed || 1.0;
    speedVal.textContent = parseFloat(ttsSpeed.value).toFixed(1) + 'x';
  } catch (err) {
    console.error('Failed to load system configs:', err);
  }
}

async function saveSettings() {
  const config = {
    groq_api_key: groqApiKey.value,
    groq_model: groqModelSelect.value,
    wakeword_sensitivity: parseFloat(wakewordSensitivity.value),
    tts_speed: parseFloat(ttsSpeed.value)
  };
  configSettings = config;
  await invoke('save_settings', { settings: config });
}

function setupTauriListeners() {
  listen('voice-state-change', async (event) => {
    const { state, message, transcript } = event.payload;
    
    if (state !== voiceState) {
      if (state === 'listening') {
        presenceAudio.playWakeUp();
        wakeRingProgress = 0.0;
      } else if (state === 'processing') {
        presenceAudio.playThinking();
      } else if (state === 'speaking') {
        presenceAudio.playSpeakingStart();
      }
    }
    
    voiceState = state;
    voiceStateMsg = message;
    voiceStateTranscript = transcript;
    
    const statusTextEl = document.getElementById('diag-voice-status');
    if (statusTextEl) {
      statusTextEl.textContent = state.toUpperCase();
      statusTextEl.className = 'value glow-text';
      if (state === 'idle') statusTextEl.className = 'value';
    }
    
    const isOverlay = document.body.classList.contains('window-main');
    
    if (state === 'listening') {
      if (isOverlay) {
        await invoke('show_card');
      }
      showMiniCard('Listening...', transcript, message || 'Awaiting voice input');
      showSubtitles('System', 'Listening...');
      
      clearInterval(waveformInterval);
      waveformInterval = setInterval(drawFocusCardWaveform, 40);
    } else if (state === 'processing') {
      if (isOverlay) {
        await invoke('show_card');
      }
      showMiniCard('Processing...', transcript, message || 'Thinking...');
      showSubtitles('System', message || 'Processing...');
      
      clearInterval(waveformInterval);
    } else if (state === 'speaking') {
      if (isOverlay) {
        await invoke('show_card');
      }
      showMiniCard('Speaking...', transcript, message || 'Speaking reply');
      
      clearInterval(waveformInterval);
      waveformInterval = setInterval(drawFocusCardWaveform, 40);
    } else {
      hideMiniCard();
      clearInterval(waveformInterval);
      
      if (subtitlesContainer) {
        subtitlesContainer.classList.add('hidden');
      }
      
      if (isOverlay) {
        setTimeout(async () => {
          if (voiceState === 'idle') {
            await invoke('show_bubble');
          }
        }, 400);
      }
    }
  });

  listen('presence-state-change', async (event) => {
    const { state } = event.payload;
    if (state !== presenceState) {
      if (state === 'Sleeping') {
        presenceAudio.playSleeping();
      } else if (state === 'Listening') {
        presenceAudio.playWakeUp();
        wakeRingProgress = 0.0;
      } else if (state === 'Thinking') {
        presenceAudio.playThinking();
      } else if (state === 'Speaking') {
        presenceAudio.playSpeakingStart();
      } else if (state === 'Working') {
        presenceAudio.playWorking();
      } else if (state === 'Monitoring') {
        presenceAudio.playMonitoring();
      }
      presenceState = state;
    }
  });

  listen('dialogue-event', (event) => {
    const { sender, message } = event.payload;
    addDialogueEntry(sender, message);
    
    let speaker = sender;
    if (sender === 'Alok') speaker = 'Jarvis';
    showSubtitles(speaker, message);
    addMcTerminalLog(`${sender}: ${message.substring(0, 50)}${message.length > 50 ? '...' : ''}`, 'system');
  });

  listen('planner-update', (event) => {
    const { goal, steps } = event.payload;
    addMcTerminalLog(`Goal planner update: ${goal || 'None'}`, 'swarm');
    activeGoalTitle.textContent = goal || 'No Active Goal';
    
    plannerStepsList.innerHTML = '';
    if (!steps || steps.length === 0) {
      plannerStepsList.innerHTML = `<li class="step-empty">No active goal execution checklist.</li>`;
      progressPercentageText.textContent = '0%';
      const fill = document.querySelector('.progress-fill');
      if (fill) fill.style.width = '0%';
      return;
    }
    
    let completedCount = 0;
    steps.forEach(step => {
      const li = document.createElement('li');
      li.className = step.status.toLowerCase();
      
      let statusIcon = '⏳';
      if (step.status === 'Running') statusIcon = '🌀';
      else if (step.status === 'Completed') { statusIcon = '✅'; completedCount++; }
      else if (step.status === 'Failed') statusIcon = '❌';
      
      li.innerHTML = `
        <span style="font-size:1.15rem; margin-right:6px;">${statusIcon}</span> 
        <div>
          <strong>${step.name}</strong>
          <small>${step.description}</small>
        </div>
      `;
      plannerStepsList.appendChild(li);
    });

    const percent = Math.round((completedCount / steps.length) * 100);
    progressPercentageText.textContent = `${percent}%`;
    const fill = document.querySelector('.progress-fill');
    if (fill) fill.style.width = `${percent}%`;
  });

  listen('graph-update', (event) => {
    const { nodes, edges } = event.payload;
    nodes.forEach(n => {
      const existing = graphNodes.find(ex => ex.id === n.id);
      if (existing) {
        n.x = existing.x;
        n.y = existing.y;
        n.vx = existing.vx;
        n.vy = existing.vy;
      } else {
        n.x = 0;
        n.y = 0;
        n.vx = 0;
        n.vy = 0;
      }
    });
    graphNodes = nodes;
    graphEdges = edges;
    
    if (document.body.classList.contains('window-command-center') && currentTab === 'graph') {
      resizeGraphCanvas();
    }
  });

  listen('proactive-suggestion', (event) => {
    addMcTerminalLog(`Proactive suggestion: ${event.payload.title}`, 'learning');
    showProactiveToast(event.payload);
  });
}

function startSystemDiagnostics() {
  setTimeout(updateMissionControlHUD, 200);
  
  setInterval(async () => {
    try {
      const stats = await invoke('get_system_stats');
      const ramEl = document.getElementById('diag-ram');
      const latEl = document.getElementById('diag-wakeword-latency');
      if (ramEl) ramEl.textContent = `${stats.ram_usage.toFixed(1)} MB`;
      if (latEl) latEl.textContent = `${stats.latency_ms} ms`;
      
      // Query self-diagnostics
      const diag = await invoke('get_self_diagnostics');
      const healthEl = document.getElementById('diag-health');
      if (healthEl && diag) {
        healthEl.textContent = `${diag.health_score}%`;
        if (diag.health_score < 70) {
          healthEl.className = 'value red';
        } else if (diag.health_score < 90) {
          healthEl.className = 'value yellow';
        } else {
          healthEl.className = 'value green';
        }
      }
      
      if (currentTab === 'control') {
        updateMissionControlHUD();
      }
    } catch (err) {
      // Fail silently
    }
  }, 2000);
}

async function loadMemoryGraph() {
  try {
    const graphData = await invoke('get_memory_graph');
    if (graphData && graphData.nodes && graphData.edges) {
      graphData.nodes.forEach(n => {
        const existing = graphNodes.find(ex => ex.id === n.id);
        if (existing) {
          n.x = existing.x;
          n.y = existing.y;
          n.vx = existing.vx;
          n.vy = existing.vy;
        } else {
          n.x = 0;
          n.y = 0;
          n.vx = 0;
          n.vy = 0;
        }
      });
      graphNodes = graphData.nodes;
      graphEdges = graphData.edges;
      resizeGraphCanvas();
    }
  } catch (err) {
    console.error('Failed to load memory graph:', err);
  }
}

async function showProactiveToast(payload) {
  if (!proactiveToast) return;

  const { type, title, message, action, target } = payload;

  if (toastTimeout) {
    clearTimeout(toastTimeout);
  }

  // Set appropriate class based on suggestion type for styles
  proactiveToast.className = 'hidden'; // Reset classes
  // Trigger layout recalculation to ensure the class transition animates
  void proactiveToast.offsetWidth;
  proactiveToast.classList.add(type);

  // Set icons
  let iconName = 'bell';
  if (type === 'battery') iconName = 'battery';
  else if (type === 'meeting') iconName = 'calendar';
  else if (type === 'habit') iconName = 'sparkles';
  else if (type === 'warning') iconName = 'alert-triangle';
  else if (type === 'opportunity') iconName = 'lightbulb';
  else if (type === 'suggestion') iconName = 'sparkles';

  if (toastIcon) {
    toastIcon.setAttribute('data-lucide', iconName);
  }

  if (toastTitle) toastTitle.textContent = title;
  if (toastMessage) toastMessage.textContent = message;

  // Render actions
  if (action === 'none' || !action) {
    if (toastAcceptBtn) toastAcceptBtn.style.display = 'none';
    if (toastDeclineBtn) toastDeclineBtn.textContent = 'Dismiss';
  } else {
    if (toastAcceptBtn) {
      toastAcceptBtn.style.display = 'block';
      toastAcceptBtn.setAttribute('data-action', action);
      toastAcceptBtn.setAttribute('data-target', target);
      if (action === 'launch_app') {
        toastAcceptBtn.textContent = 'Launch';
      } else if (action === 'browser_visit') {
        toastAcceptBtn.textContent = 'Open';
      } else if (action === 'run_intent') {
        toastAcceptBtn.textContent = 'Execute';
      }
    }
    if (toastDeclineBtn) toastDeclineBtn.textContent = 'Dismiss';
  }

  // Refresh lucide icons
  if (window.lucide) {
    window.lucide.createIcons();
  }

  // Expand the overlay window if needed to fit the notification toast
  const isOverlay = document.body.classList.contains('window-main');
  if (isOverlay) {
    try {
      await invoke('show_toast_window');
    } catch (err) {
      console.error('Failed to resize window for toast:', err);
    }
  }

  proactiveToast.classList.remove('hidden');

  // Auto-hide after 8 seconds
  toastTimeout = setTimeout(() => {
    hideProactiveToast();
  }, 8000);
}

async function hideProactiveToast() {
  if (!proactiveToast) return;

  if (toastTimeout) {
    clearTimeout(toastTimeout);
    toastTimeout = null;
  }

  proactiveToast.classList.add('hidden');

  // Restore overlay window size if we are in main overlay window
  const isOverlay = document.body.classList.contains('window-main');
  if (isOverlay) {
    setTimeout(async () => {
      // Ensure the toast is still hidden before resizing window down
      if (proactiveToast.classList.contains('hidden')) {
        try {
          if (voiceState !== 'idle') {
            await invoke('show_card');
          } else {
            await invoke('show_bubble');
          }
        } catch (err) {
          console.error('Failed to restore window size after toast:', err);
        }
      }
    }, 450); // Wait for transition out animation to complete
  }
}

// ==========================================
// MISSION CONTROL HUD LOGIC & ANIMATIONS
// ==========================================

function addMcTerminalLog(message, type = 'system') {
  const terminal = document.getElementById('mc-terminal-logs');
  if (!terminal) return;
  const line = document.createElement('div');
  line.className = `log-line ${type}`;
  line.textContent = `${new Date().toLocaleTimeString()} - ${message}`;
  terminal.appendChild(line);
  
  while (terminal.children.length > 50) {
    terminal.removeChild(terminal.firstChild);
  }
  terminal.scrollTop = terminal.scrollHeight;
}

let mcNodes = [];
function initControlGraph() {
  const canvas = document.getElementById('control-graph-canvas');
  if (!canvas) return;
  mcNodes = [];
  const count = 18;
  for (let i = 0; i < count; i++) {
    mcNodes.push({
      x: Math.random() * (canvas.width - 40) + 20,
      y: Math.random() * (canvas.height - 40) + 20,
      vx: (Math.random() - 0.5) * 0.5,
      vy: (Math.random() - 0.5) * 0.5,
      radius: Math.random() * 3.5 + 1.5,
      color: Math.random() > 0.45 ? 'rgba(0, 245, 255, ' : 'rgba(138, 43, 226, '
    });
  }
}

function drawControlGraph() {
  const canvas = document.getElementById('control-graph-canvas');
  if (!canvas) return;
  if (!controlCtx) {
    controlCtx = canvas.getContext('2d');
  }
  const ctx = controlCtx;
  
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  
  const maxDist = 90;
  for (let i = 0; i < mcNodes.length; i++) {
    const n1 = mcNodes[i];
    n1.x += n1.vx;
    n1.y += n1.vy;
    
    if (n1.x < 10 || n1.x > canvas.width - 10) n1.vx *= -1;
    if (n1.y < 10 || n1.y > canvas.height - 10) n1.vy *= -1;
    
    for (let j = i + 1; j < mcNodes.length; j++) {
      const n2 = mcNodes[j];
      const dx = n2.x - n1.x;
      const dy = n2.y - n1.y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      
      if (dist < maxDist) {
        const alpha = (1 - dist / maxDist) * 0.22;
        ctx.beginPath();
        ctx.strokeStyle = `rgba(0, 245, 255, ${alpha})`;
        ctx.lineWidth = 1;
        ctx.moveTo(n1.x, n1.y);
        ctx.lineTo(n2.x, n2.y);
        ctx.stroke();
      }
    }
  }
  
  mcNodes.forEach(node => {
    ctx.beginPath();
    ctx.fillStyle = node.color + '0.8)';
    ctx.shadowColor = 'rgba(0, 245, 255, 0.4)';
    ctx.shadowBlur = 6;
    ctx.arc(node.x, node.y, node.radius, 0, Math.PI * 2);
    ctx.fill();
    ctx.shadowBlur = 0;
  });
}

async function updateMissionControlHUD() {
  if (currentTab !== 'control') return;
  try {
    const data = await invoke('get_mission_control_data');
    if (!data) return;

    const mcMemoryStats = document.getElementById('mc-memory-stats');
    if (mcMemoryStats) {
      mcMemoryStats.textContent = `${data.nodes_count} Nodes / ${data.edges_count} Links`;
    }

    const cpuVal = document.getElementById('mc-cpu-val');
    const cpuFill = document.getElementById('mc-cpu-fill');
    if (cpuVal && cpuFill) {
      cpuVal.textContent = `${data.system_cpu.toFixed(1)}%`;
      cpuFill.style.width = `${data.system_cpu}%`;
    }

    const ramVal = document.getElementById('mc-ram-val');
    const ramFill = document.getElementById('mc-ram-fill');
    if (ramVal && ramFill) {
      ramVal.textContent = `${data.system_memory.toFixed(1)} MB`;
      const percent = Math.min(100, (data.system_memory / 200) * 100);
      ramFill.style.width = `${percent}%`;
    }

    const diskVal = document.getElementById('mc-disk-val');
    const diskFill = document.getElementById('mc-disk-fill');
    if (diskVal && diskFill) {
      diskVal.textContent = `${data.disk_free.toFixed(1)} GB`;
      const percent = Math.min(100, (data.disk_free / 500) * 100);
      diskFill.style.width = `${percent}%`;
    }

    const batteryVal = document.getElementById('mc-battery-val');
    const batteryFill = document.getElementById('mc-battery-fill');
    const batteryStatus = document.getElementById('mc-battery-status');
    if (batteryVal && batteryFill) {
      batteryVal.textContent = `${data.battery_health.toFixed(1)}%`;
      batteryFill.style.width = `${data.battery_health}%`;
    }
    if (batteryStatus) {
      batteryStatus.textContent = `Health: ${data.battery_health.toFixed(1)}%`;
    }

    const latencyStt = document.getElementById('mc-latency-stt');
    if (latencyStt) latencyStt.textContent = `${data.stt_latency} ms`;

    const latencyAction = document.getElementById('mc-latency-action');
    if (latencyAction) latencyAction.textContent = `${data.action_latency} ms`;

    const latencyGroq = document.getElementById('mc-latency-groq');
    if (latencyGroq) latencyGroq.textContent = `${data.groq_latency} ms`;

    const mcSuccessRate = document.getElementById('mc-success-rate');
    if (mcSuccessRate) {
      mcSuccessRate.textContent = `Success Rate: ${data.success_rate.toFixed(1)}%`;
    }

    const mcCorrectionsCount = document.getElementById('mc-corrections-count');
    if (mcCorrectionsCount) {
      mcCorrectionsCount.textContent = data.corrections_count;
    }

    const mcHabitsCount = document.getElementById('mc-habits-count');
    const mcHabitsEntries = document.getElementById('mc-habits-entries');
    if (data.habits && data.habits.length > 0) {
      if (mcHabitsCount) mcHabitsCount.textContent = data.habits.length;
      if (mcHabitsEntries) {
        mcHabitsEntries.innerHTML = data.habits.map(h => `
          <div class="habit-entry-item">
            <span class="habit-pattern">${h.key} ➔ ${h.value}</span>
            <span class="habit-confidence">Conf: ${(h.confidence * 100).toFixed(0)}%</span>
          </div>
        `).join('');
      }
    } else {
      if (mcHabitsCount) mcHabitsCount.textContent = '0';
      if (mcHabitsEntries) {
        mcHabitsEntries.innerHTML = '<div class="no-habits-mc">No habits recorded yet.</div>';
      }
    }

    if (data.agent_activity) {
      let activeAgents = 0;
      for (const [agentName, statusText] of Object.entries(data.agent_activity)) {
        const row = document.querySelector(`.agent-row[data-agent="${agentName}"]`);
        if (row) {
          const dot = row.querySelector('.pulse-dot');
          const statusSpan = row.querySelector('.agent-status');
          
          if (statusText && statusText.toLowerCase() !== 'idle' && statusText !== '') {
            dot.className = 'pulse-dot working';
            statusSpan.textContent = statusText;
            activeAgents++;
          } else {
            dot.className = 'pulse-dot idle';
            statusSpan.textContent = 'Idle';
          }
        }
      }
      
      const swarmStatus = document.getElementById('mc-swarm-status');
      if (swarmStatus) {
        if (activeAgents > 0) {
          swarmStatus.textContent = `${activeAgents} Agents Active`;
          swarmStatus.className = 'status-indicator-mc active';
        } else {
          swarmStatus.textContent = 'Swarm Idle';
          swarmStatus.className = 'status-indicator-mc idle';
        }
      }
    }

  } catch (err) {
    console.error('Failed to update Mission Control HUD:', err);
  }
}


