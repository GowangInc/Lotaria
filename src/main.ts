import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

// Types
interface Config {
  is_active: boolean;
  interval: string;
  vision_provider: string;
  vision_model: string;
  tts_provider: string;
  tts_model: string;
  tts_voice: string;
  api_keys: Record<string, string>;
  speech_bubble_enabled: boolean;
  audio_enabled: boolean;
  first_run: boolean;
  mood: string;
  pet_style: string;
  gemini_free_tier: boolean;
}

interface ProviderDef {
  key: string;
  name: string;
  env_var: string;
  docs_url: string;
  vision_models: string[];
  tts_models: string[];
  tts_voices: string[];
  live_voices: string[];
  recommended: boolean;
  cost_note: string;
  requires_tts_provider: boolean;
}

interface RoastResult {
  text: string;
  audio_base64: string | null;
  audio_duration: number;
  timestamp: number;
  error?: string;
}

interface HistoryEntry {
  roast: string;
  time: string;
  timestamp: number;
}

// State
let config: Config = {
  is_active: false,
  interval: 'frequent',
  vision_provider: 'gemini',
  vision_model: 'gemini-2.0-flash',
  tts_provider: 'gemini',
  tts_model: 'gemini-2.5-flash-live',
  tts_voice: 'Kore',
  api_keys: {},
  speech_bubble_enabled: true,
  audio_enabled: true,
  first_run: true,
  mood: 'roast',
  pet_style: 'default',
  gemini_free_tier: true,
};

let providers: ProviderDef[] = [];
let isProcessing = false;
let audioContext: AudioContext | null = null;

// DOM Elements
const app = document.getElementById('app') as HTMLElement;
const character = document.getElementById('character') as HTMLElement;
const eyes = document.querySelectorAll('.eye') as NodeListOf<HTMLElement>;
const speechBubble = document.getElementById('speech-bubble') as HTMLElement;
const speechText = document.getElementById('speech-text') as HTMLElement;
const thinkingIndicator = document.getElementById('thinking') as HTMLElement;
const contextMenu = document.getElementById('context-menu') as HTMLElement;
const settingsOverlay = document.getElementById('settings-overlay') as HTMLElement;
const welcomeOverlay = document.getElementById('welcome-overlay') as HTMLElement;

// Initialize
async function init() {
  // Load config
  config = await invoke('get_config');
  providers = await invoke('get_providers');

  // Listen for roast events from monitoring
  await listen<RoastResult>('roast', (event) => {
    deliverRoast(event.payload);
  });

  // Check first run
  if (config.first_run) {
    showWelcomeModal();
  }

  // Start eye tracking
  startEyeTracking();

  // Setup event listeners
  setupEventListeners();

  // Setup drag
  setupDrag();
}

// Eye tracking
function startEyeTracking() {
  document.addEventListener('mousemove', (e) => {
    eyes.forEach(eye => {
      const rect = eye.getBoundingClientRect();
      const eyeCenterX = rect.left + rect.width / 2;
      const eyeCenterY = rect.top + rect.height / 2;
      
      const angle = Math.atan2(e.clientY - eyeCenterY, e.clientX - eyeCenterX);
      const distance = Math.min(3, Math.hypot(e.clientX - eyeCenterX, e.clientY - eyeCenterY) / 20);
      
      const pupil = eye.querySelector('.pupil') as HTMLElement;
      if (pupil) {
        pupil.style.transform = `translate(${Math.cos(angle) * distance}px, ${Math.sin(angle) * distance}px)`;
      }
    });
  });
}

// Deliver a roast
function deliverRoast(result: RoastResult) {
  if (result.error) {
    showError(result.text);
    return;
  }

  if (config.speech_bubble_enabled) {
    speechText.textContent = result.text;
    speechBubble.classList.add('visible');
    speechBubble.classList.remove('hidden');
  }

  // Play audio if available
  if (result.audio_base64 && config.audio_enabled) {
    playAudio(result.audio_base64);
  }

  // Auto-hide after duration
  setTimeout(() => {
    speechBubble.classList.remove('visible');
    speechBubble.classList.add('hidden');
  }, (result.audio_duration * 1000) + 2000);
}

// Play base64 audio
async function playAudio(base64Audio: string) {
  try {
    if (!audioContext) {
      audioContext = new AudioContext();
    }

    const audioData = Uint8Array.from(atob(base64Audio), c => c.charCodeAt(0));
    const audioBuffer = await audioContext.decodeAudioData(audioData.buffer);
    
    const source = audioContext.createBufferSource();
    source.buffer = audioBuffer;
    source.connect(audioContext.destination);
    source.start();
  } catch (e) {
    console.error('Audio playback error:', e);
  }
}

// Show error
function showError(message: string) {
  speechText.textContent = message;
  speechBubble.classList.add('visible', 'error');
  speechBubble.classList.remove('hidden');
  
  setTimeout(() => {
    speechBubble.classList.remove('visible', 'error');
    speechBubble.classList.add('hidden');
  }, 5000);
}

// Trigger a roast
async function triggerRoast() {
  if (isProcessing) return;
  
  isProcessing = true;
  thinkingIndicator.classList.remove('hidden');
  character.classList.add('thinking');

  try {
    const result = await invoke<RoastResult>('roast_now');
    deliverRoast(result);
  } catch (e) {
    showError(`Error: ${e}`);
  } finally {
    isProcessing = false;
    thinkingIndicator.classList.add('hidden');
    character.classList.remove('thinking');
  }
}

// Context menu - positioned within app bounds
function showContextMenu(x: number, y: number) {
  const appRect = app.getBoundingClientRect();
  const menuWidth = 180;
  const menuHeight = 150; // approximate
  
  // Adjust position to stay within bounds
  let adjustedX = x;
  let adjustedY = y;
  
  if (x + menuWidth > appRect.width) {
    adjustedX = x - menuWidth;
  }
  if (y + menuHeight > appRect.height) {
    adjustedY = y - menuHeight;
  }
  
  contextMenu.style.left = `${Math.max(0, adjustedX)}px`;
  contextMenu.style.top = `${Math.max(0, adjustedY)}px`;
  contextMenu.classList.remove('hidden');
}

function hideContextMenu() {
  contextMenu.classList.add('hidden');
}

// Settings modal
async function showSettings() {
  const apiKeys = await invoke<Record<string, string>>('get_api_keys');
  
  // Populate API keys
  const apiKeysList = document.getElementById('api-keys-list') as HTMLElement;
  apiKeysList.innerHTML = '';
  
  providers.forEach(provider => {
    const row = document.createElement('div');
    row.className = 'key-row';
    const hasKey = apiKeys[provider.key] && apiKeys[provider.key].length > 0;
    
    row.innerHTML = `
      <label>${provider.name}</label>
      <input type="password" id="key-input-${provider.key}" 
             placeholder="${hasKey ? apiKeys[provider.key] : 'Enter API key...'}" />
      <span class="key-status ${hasKey ? 'set' : 'unset'}">${hasKey ? '✓' : '•'}</span>
      <button class="save-key-btn" data-provider="${provider.key}">Save</button>
    `;
    apiKeysList.appendChild(row);
  });

  // Add save listeners
  apiKeysList.querySelectorAll('.save-key-btn').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      const provider = (e.target as HTMLElement).dataset.provider!;
      const input = document.getElementById(`key-input-${provider}`) as HTMLInputElement;
      const key = input.value.trim();
      if (!key) return;

      await invoke('save_api_key', { provider, key });
      input.value = '';
      input.placeholder = '...' + key.slice(-4);
      
      const status = document.getElementById(`key-status-${provider}`) as HTMLElement;
      status.textContent = '✓';
      status.classList.add('set');
      status.classList.remove('unset');
    });
  });

  // Populate vision settings
  buildVisionUI();
  buildTtsUI();
  updateCostEstimator();

  settingsOverlay.classList.add('open');
  
  // Expand window for settings
  const window = getCurrentWebviewWindow();
  try {
    await window.setSize({ type: 'Physical', width: 500, height: 700 });
    
    // Center window
    const monitors = await window.availableMonitors();
    if (monitors.length > 0) {
      const monitor = monitors[0];
      const x = (monitor.size.width - 500) / 2;
      const y = (monitor.size.height - 700) / 2;
      await window.setPosition({ type: 'Physical', x, y });
    }
  } catch (err) {
    console.error('Window resize error:', err);
  }
}

async function closeSettings() {
  settingsOverlay.classList.remove('open');
  
  // Restore window size
  const window = getCurrentWebviewWindow();
  try {
    await window.setSize({ type: 'Physical', width: 420, height: 400 });
    
    // Position bottom-right
    const monitors = await window.availableMonitors();
    if (monitors.length > 0) {
      const monitor = monitors[0];
      const x = monitor.size.width - 440;
      const y = monitor.size.height - 420;
      await window.setPosition({ type: 'Physical', x, y });
    }
  } catch (err) {
    console.error('Window restore error:', err);
  }
}

function buildVisionUI() {
  const provSelect = document.getElementById('vision-provider-select') as HTMLSelectElement;
  const modelSelect = document.getElementById('vision-model-select') as HTMLSelectElement;
  
  provSelect.innerHTML = '';
  providers.forEach(prov => {
    if (prov.vision_models.length === 0) return;
    const opt = document.createElement('option');
    opt.value = prov.key;
    opt.textContent = prov.name;
    if (prov.key === config.vision_provider) opt.selected = true;
    provSelect.appendChild(opt);
  });

  function updateModels() {
    const prov = providers.find(p => p.key === provSelect.value);
    if (!prov) return;
    
    modelSelect.innerHTML = '';
    prov.vision_models.forEach(model => {
      const opt = document.createElement('option');
      opt.value = model;
      opt.textContent = model;
      if (model === config.vision_model) opt.selected = true;
      modelSelect.appendChild(opt);
    });
  }

  provSelect.addEventListener('change', updateModels);
  updateModels();
}

function buildTtsUI() {
  const provSelect = document.getElementById('tts-provider-select') as HTMLSelectElement;
  const modelSelect = document.getElementById('tts-model-select') as HTMLSelectElement;
  const voiceSelect = document.getElementById('tts-voice-select') as HTMLSelectElement;
  
  provSelect.innerHTML = '';
  providers.forEach(prov => {
    if (prov.tts_models.length === 0) return;
    const opt = document.createElement('option');
    opt.value = prov.key;
    opt.textContent = prov.name;
    if (prov.key === config.tts_provider) opt.selected = true;
    provSelect.appendChild(opt);
  });

  function updateModels() {
    const prov = providers.find(p => p.key === provSelect.value);
    if (!prov) return;
    
    modelSelect.innerHTML = '';
    prov.tts_models.forEach(model => {
      const opt = document.createElement('option');
      opt.value = model;
      opt.textContent = model;
      if (model === config.tts_model) opt.selected = true;
      modelSelect.appendChild(opt);
    });
    
    updateVoices();
  }

  function updateVoices() {
    const prov = providers.find(p => p.key === provSelect.value);
    if (!prov) return;
    
    voiceSelect.innerHTML = '';
    const voices = provSelect.value === 'gemini' && modelSelect.value?.includes('live')
      ? prov.live_voices
      : prov.tts_voices;
    
    voices.forEach(voice => {
      const opt = document.createElement('option');
      opt.value = voice;
      opt.textContent = voice;
      if (voice === config.tts_voice) opt.selected = true;
      voiceSelect.appendChild(opt);
    });
  }

  provSelect.addEventListener('change', updateModels);
  modelSelect.addEventListener('change', updateVoices);
  updateModels();
}

function updateCostEstimator() {
  // Simple cost estimation
  const visionCost = document.getElementById('cost-daily') as HTMLElement;
  visionCost.textContent = '~$0.02/day';
}

// Welcome modal
function showWelcomeModal() {
  const providerList = document.getElementById('welcome-provider-list') as HTMLElement;
  providerList.innerHTML = '';
  
  providers.forEach(prov => {
    if (prov.vision_models.length === 0) return;
    
    const div = document.createElement('div');
    div.className = `provider-option${prov.recommended ? ' recommended' : ''}`;
    div.dataset.provider = prov.key;
    div.innerHTML = `
      <div class="provider-name">${prov.name}</div>
      <div class="provider-cost">${prov.cost_note}</div>
    `;
    div.addEventListener('click', () => selectWelcomeProvider(prov.key));
    providerList.appendChild(div);
  });

  selectWelcomeProvider('gemini');
  welcomeOverlay.classList.add('open');
}

let selectedWelcomeProvider = 'gemini';

function selectWelcomeProvider(key: string) {
  selectedWelcomeProvider = key;
  document.querySelectorAll('.provider-option').forEach(el => {
    el.classList.toggle('selected', (el as HTMLElement).dataset.provider === key);
  });
  
  const prov = providers.find(p => p.key === key);
  if (prov) {
    const docsLink = document.getElementById('welcome-docs-link') as HTMLAnchorElement;
    docsLink.href = prov.docs_url;
    
    const costText = document.getElementById('welcome-cost-text') as HTMLElement;
    costText.textContent = prov.cost_note;
  }
}

async function startFromWelcome() {
  const apiKey = (document.getElementById('welcome-api-key') as HTMLInputElement).value.trim();
  if (!apiKey) return;

  // Save API key
  await invoke('save_api_key', { provider: selectedWelcomeProvider, key: apiKey });
  
  // Set config
  const prov = providers.find(p => p.key === selectedWelcomeProvider);
  if (prov) {
    await invoke('set_config', { key: 'vision_provider', value: selectedWelcomeProvider });
    await invoke('set_config', { key: 'vision_model', value: prov.vision_models[0] });
    if (prov.tts_models.length > 0) {
      await invoke('set_config', { key: 'tts_provider', value: selectedWelcomeProvider });
      await invoke('set_config', { key: 'tts_model', value: prov.tts_models[0] });
    }
  }

  // Mark first run complete
  await invoke('mark_first_run_complete');
  config.first_run = false;

  // Start monitoring
  await toggleMonitoring();

  welcomeOverlay.classList.remove('open');
}

// Toggle monitoring
async function toggleMonitoring() {
  const isActive = await invoke<boolean>('toggle_monitoring');
  config.is_active = isActive;
  
  const monitorItem = document.getElementById('menu-monitor') as HTMLElement;
  monitorItem.textContent = isActive ? '⏸ Pause Monitoring' : '▶ Start Monitoring';
}

// Drag support
function setupDrag() {
  const window = getCurrentWebviewWindow();
  let isDragging = false;
  let startX = 0;
  let startY = 0;

  character.addEventListener('mousedown', async (e) => {
    if (e.button !== 0) return;
    isDragging = true;
    startX = e.screenX;
    startY = e.screenY;
  });

  document.addEventListener('mousemove', async (e) => {
    if (!isDragging) return;
    
    const deltaX = e.screenX - startX;
    const deltaY = e.screenY - startY;
    
    try {
      const pos = await window.outerPosition();
      await window.setPosition({ type: 'Physical', x: pos.x + deltaX, y: pos.y + deltaY });
    } catch (err) {
      console.error('Drag error:', err);
    }
    
    startX = e.screenX;
    startY = e.screenY;
  });

  document.addEventListener('mouseup', () => {
    isDragging = false;
  });
}

// Event listeners
function setupEventListeners() {
  // Right-click context menu
  character.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    showContextMenu(e.clientX, e.clientY);
  });

  document.addEventListener('click', () => hideContextMenu());

  // Menu items
  document.getElementById('menu-roast')?.addEventListener('click', triggerRoast);
  document.getElementById('menu-monitor')?.addEventListener('click', toggleMonitoring);
  document.getElementById('menu-settings')?.addEventListener('click', showSettings);
  document.getElementById('menu-quit')?.addEventListener('click', () => invoke('quit'));

  // Settings
  document.getElementById('settings-close')?.addEventListener('click', closeSettings);
  document.getElementById('settings-save')?.addEventListener('click', async () => {
    // Save settings
    const visionProv = (document.getElementById('vision-provider-select') as HTMLSelectElement).value;
    const visionModel = (document.getElementById('vision-model-select') as HTMLSelectElement).value;
    const ttsProv = (document.getElementById('tts-provider-select') as HTMLSelectElement).value;
    const ttsModel = (document.getElementById('tts-model-select') as HTMLSelectElement).value;
    const ttsVoice = (document.getElementById('tts-voice-select') as HTMLSelectElement).value;

    await invoke('set_config', { key: 'vision_provider', value: visionProv });
    await invoke('set_config', { key: 'vision_model', value: visionModel });
    await invoke('set_config', { key: 'tts_provider', value: ttsProv });
    await invoke('set_config', { key: 'tts_model', value: ttsModel });
    await invoke('set_config', { key: 'tts_voice', value: ttsVoice });

    // Reload config
    config = await invoke('get_config');
    closeSettings();
  });

  document.getElementById('clear-history-btn')?.addEventListener('click', async () => {
    await invoke('clear_history');
  });

  // Welcome modal
  document.getElementById('welcome-start')?.addEventListener('click', startFromWelcome);
  document.getElementById('welcome-settings')?.addEventListener('click', () => {
    welcomeOverlay.classList.remove('open');
    showSettings();
  });

  // API key input validation
  document.getElementById('welcome-api-key')?.addEventListener('input', (e) => {
    const btn = document.getElementById('welcome-start') as HTMLButtonElement;
    btn.disabled = (e.target as HTMLInputElement).value.trim().length === 0;
  });
}

// Start
init();
