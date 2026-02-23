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
  custom_mood: string;
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
  vision_model: 'gemini-2.5-flash',
  tts_provider: 'gemini',
  tts_model: 'gemini-2.5-flash-preview-tts',
  tts_voice: 'Kore',
  api_keys: {},
  speech_bubble_enabled: true,
  audio_enabled: true,
  first_run: true,
  mood: 'roast',
  custom_mood: '',
  pet_style: 'default',
  gemini_free_tier: true,
};

let providers: ProviderDef[] = [];
let isProcessing = false;

// Pet style definitions
const PET_STYLES: { key: string; name: string }[] = [
  { key: 'default', name: 'Box' },
  { key: 'cat', name: 'Cat' },
  { key: 'ghost', name: 'Ghost' },
  { key: 'robot', name: 'Robot' },
  { key: 'blob', name: 'Blob' },
  { key: 'owl', name: 'Owl' },
  { key: 'alien', name: 'Alien' },
  { key: 'pumpkin', name: 'Pumpkin' },
  { key: 'cloud', name: 'Cloud' },
  { key: 'octopus', name: 'Octopus' },
];

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

  // Start click-through poller (allows clicking pet while passing through empty areas)
  startClickThroughPoller();

  // Listen for roast events from monitoring
  await listen<RoastResult>('roast', (event) => {
    deliverRoast(event.payload);
  });

  // Listen for monitoring tick events to trigger roasts
  await listen('monitoring-tick', async () => {
    console.log('Monitoring tick - triggering roast');
    await triggerRoast();
  });

  // Apply pet style
  character.className = `pet-${config.pet_style || 'default'}`;

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

// Eye tracking - follows cursor globally across the entire screen
function startEyeTracking() {
  const appWindow = getCurrentWebviewWindow();
  let polling = false;

  async function updateEyes() {
    if (polling) return; // Skip if previous poll hasn't finished
    polling = true;
    try {
      const [cursorX, cursorY] = await invoke<[number, number]>('get_cursor_position');
      const pos = await appWindow.outerPosition();
      const scaleFactor = await appWindow.scaleFactor();

      // Both GetCursorPos and outerPosition return physical pixels.
      // Subtract in physical space, then convert to CSS pixels to match getBoundingClientRect.
      const localX = (cursorX - pos.x) / scaleFactor;
      const localY = (cursorY - pos.y) / scaleFactor;

      eyes.forEach(eye => {
        const rect = eye.getBoundingClientRect();
        const eyeCenterX = rect.left + rect.width / 2;
        const eyeCenterY = rect.top + rect.height / 2;

        const dx = localX - eyeCenterX;
        const dy = localY - eyeCenterY;

        const angle = Math.atan2(dy, dx);
        const dist = Math.hypot(dx, dy);
        const maxMove = 5;
        const clampedDist = Math.min(maxMove, dist / 30);

        const pupil = eye.querySelector('.pupil') as HTMLElement;
        if (pupil) {
          const offsetX = Math.cos(angle) * clampedDist;
          const offsetY = Math.sin(angle) * clampedDist;
          pupil.style.transform = `translate(calc(-50% + ${offsetX}px), calc(-50% + ${offsetY}px))`;
        }
      });
    } catch (e) {
      console.error('Eye tracking error:', e);
    } finally {
      polling = false;
    }
  }

  setInterval(updateEyes, 60); // Poll at ~16fps
}

// Deliver a roast
async function deliverRoast(result: RoastResult) {
  if (result.error) {
    showError(result.text);
    return;
  }

  if (config.speech_bubble_enabled) {
    // Disable click-through so speech bubble can be read/interacted with
    await setClickThrough(false);
    speechText.textContent = result.text;
    speechBubble.classList.add('visible');
    speechBubble.classList.remove('hidden');
  }

  // Audio is already played by the Rust backend via rodio
  // No need to play again in the frontend

  // Auto-hide after duration
  setTimeout(async () => {
    speechBubble.classList.remove('visible');
    speechBubble.classList.add('hidden');
    // Re-enable click-through if no other overlays are open
    if (!contextMenu.classList.contains('hidden') ||
        settingsOverlay.classList.contains('open') ||
        welcomeOverlay.classList.contains('open')) {
      return;
    }
    await setClickThrough(true);
  }, (result.audio_duration * 1000) + 2000);
}

// Audio playback removed - handled by Rust backend via rodio
// This prevents double-playback and audio format issues

// Show error
async function showError(message: string) {
  await setClickThrough(false);
  speechText.textContent = message;
  speechBubble.classList.add('visible', 'error');
  speechBubble.classList.remove('hidden');

  setTimeout(async () => {
    speechBubble.classList.remove('visible', 'error');
    speechBubble.classList.add('hidden');
    // Re-enable click-through if no other overlays are open
    if (!contextMenu.classList.contains('hidden') ||
        settingsOverlay.classList.contains('open') ||
        welcomeOverlay.classList.contains('open')) {
      return;
    }
    await setClickThrough(true);
  }, 5000);
}

// Trigger a roast
async function triggerRoast() {
  if (isProcessing) return;

  isProcessing = true;
  thinkingIndicator.classList.remove('hidden');
  character.classList.add('thinking');

  // Collapse animation before capture
  character.classList.add('hiding');
  await new Promise(resolve => setTimeout(resolve, 300)); // Wait for collapse animation

  // Start the backend call (which includes screenshot, vision, and TTS)
  const roastPromise = invoke<RoastResult>('roast_now');

  // Expand back after screenshot is done (backend takes ~100-200ms to capture)
  // Start expanding while vision analysis is running in the background
  setTimeout(() => {
    character.classList.remove('hiding');
    character.classList.add('showing');
    setTimeout(() => character.classList.remove('showing'), 300);
  }, 400);

  try {
    const result = await roastPromise;
    deliverRoast(result);
  } catch (e) {
    // Make sure pet is visible on error
    character.classList.remove('hiding');
    character.classList.add('showing');
    setTimeout(() => character.classList.remove('showing'), 300);

    showError(`Error: ${e}`);
  } finally {
    isProcessing = false;
    thinkingIndicator.classList.add('hidden');
    character.classList.remove('thinking');
  }
}

// Context menu - positioned within app bounds
async function showContextMenu(x: number, y: number) {
  // Disable click-through so menu is interactive
  await setClickThrough(false);

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

async function hideContextMenu() {
  contextMenu.classList.add('hidden');
  // Re-enable click-through if no other overlays are open
  if (!settingsOverlay.classList.contains('open') &&
      !welcomeOverlay.classList.contains('open') &&
      !speechBubble.classList.contains('visible')) {
    await setClickThrough(true);
  }
}

// Settings modal
async function showSettings() {
  try {
    // Disable click-through so settings are interactive
    await setClickThrough(false);

    // Resize window to fit settings content
    const window = getCurrentWebviewWindow();
    await window.setSize({ type: 'Physical', width: 420, height: 700 });

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
      <div class="key-input-group">
        <input type="password" id="key-input-${provider.key}"
               placeholder="${hasKey ? '...' + apiKeys[provider.key].slice(-4) : 'Enter API key...'}" />
        <span class="key-status ${hasKey ? 'set' : 'unset'}" id="key-status-${provider.key}">${hasKey ? '✓' : '•'}</span>
        <button class="save-key-btn" data-provider="${provider.key}">Save</button>
      </div>
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

        try {
          await invoke('save_api_key', { provider, key });
          input.value = '';
          input.placeholder = '...' + key.slice(-4);

          const status = document.getElementById(`key-status-${provider}`) as HTMLElement;
          if (status) {
            status.textContent = '✓';
            status.classList.add('set');
            status.classList.remove('unset');
          }
        } catch (err) {
          console.error('Failed to save API key:', err);
          showError(`Failed to save API key: ${err}`);
        }
      });
    });

    // Populate vision settings
    buildVisionUI();
    buildTtsUI();
    buildMoodUI();
    buildPetStyleUI();
    updateCostEstimator();

    settingsOverlay.classList.add('open');
    console.log('Settings opened');
  } catch (err) {
    console.error('Settings error:', err);
    showError(`Failed to open settings: ${err}`);
  }
}

async function closeSettings() {
  settingsOverlay.classList.remove('open');

  // Re-enable click-through
  await setClickThrough(true);

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

async function buildMoodUI() {
  const moodSelect = document.getElementById('mood-select') as HTMLSelectElement;
  const customSection = document.getElementById('custom-mood-section') as HTMLElement;
  const customInput = document.getElementById('custom-mood-input') as HTMLTextAreaElement;
  if (!moodSelect) return;

  try {
    const moods = await invoke<[string, string][]>('get_moods');
    moodSelect.innerHTML = '';
    moods.forEach(([key, label]) => {
      const opt = document.createElement('option');
      opt.value = key;
      opt.textContent = label;
      if (key === config.mood) opt.selected = true;
      moodSelect.appendChild(opt);
    });

    // Add "Custom" option
    const customOpt = document.createElement('option');
    customOpt.value = 'custom';
    customOpt.textContent = 'Custom';
    if (config.mood === 'custom') customOpt.selected = true;
    moodSelect.appendChild(customOpt);

    // Load custom mood text
    if (customInput && config.custom_mood) {
      customInput.value = config.custom_mood;
    }

    // Show/hide custom section
    function toggleCustomSection() {
      if (moodSelect.value === 'custom') {
        customSection.style.display = 'block';
      } else {
        customSection.style.display = 'none';
      }
    }
    toggleCustomSection();
    moodSelect.addEventListener('change', toggleCustomSection);
  } catch (e) {
    console.error('Failed to load moods:', e);
  }
}

function buildPetStyleUI() {
  const petStyleSelect = document.getElementById('pet-style-select') as HTMLSelectElement;
  if (!petStyleSelect) return;

  petStyleSelect.innerHTML = '';
  PET_STYLES.forEach(style => {
    const opt = document.createElement('option');
    opt.value = style.key;
    opt.textContent = style.name;
    if (style.key === config.pet_style) opt.selected = true;
    petStyleSelect.appendChild(opt);
  });

  petStyleSelect.addEventListener('change', async () => {
    const newStyle = petStyleSelect.value;
    await invoke('set_config', { key: 'pet_style', value: newStyle });
    config.pet_style = newStyle;

    // Update character class
    character.className = `pet-${newStyle}`;
  });
}

function updateCostEstimator() {
  // Simple cost estimation - can be enhanced later
}

// Welcome modal
async function showWelcomeModal() {
  // Disable click-through so welcome modal is interactive
  await setClickThrough(false);

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
      if (prov.tts_voices.length > 0) {
        await invoke('set_config', { key: 'tts_voice', value: prov.tts_voices[0] });
      }
    }
  }

  // Mark first run complete
  await invoke('mark_first_run_complete');
  config.first_run = false;

  // Start monitoring
  await toggleMonitoring();

  welcomeOverlay.classList.remove('open');

  // Enable click-through now that welcome is closed
  await setClickThrough(true);
}

// Toggle monitoring
async function toggleMonitoring() {
  const isActive = await invoke<boolean>('toggle_monitoring');
  config.is_active = isActive;

  const monitorItem = document.getElementById('menu-monitor') as HTMLElement;
  monitorItem.textContent = isActive ? '⏸ Pause Monitoring' : '▶ Start Monitoring';
}

// Drag support — cache position at mousedown, no async during mousemove
function setupDrag() {
  const window = getCurrentWebviewWindow();
  let isDragging = false;
  let startScreenX = 0;
  let startScreenY = 0;
  let windowStartX = 0;
  let windowStartY = 0;

  character.addEventListener('mousedown', async (e) => {
    if (e.button !== 0) return;
    e.preventDefault();
    try {
      const pos = await window.outerPosition();
      windowStartX = pos.x;
      windowStartY = pos.y;
      startScreenX = e.screenX;
      startScreenY = e.screenY;
      isDragging = true;
    } catch (_) { }
  });

  document.addEventListener('mousemove', (e) => {
    if (!isDragging) return;

    const scale = globalThis.devicePixelRatio || 1;
    const newX = windowStartX + Math.round((e.screenX - startScreenX) * scale);
    const newY = windowStartY + Math.round((e.screenY - startScreenY) * scale);

    window.setPosition({ type: 'Physical', x: newX, y: newY }).catch(() => { });
  });

  document.addEventListener('mouseup', () => {
    isDragging = false;
  });
}

// Toggle window click-through
async function setClickThrough(enable: boolean) {
  try {
    await invoke('set_ignore_cursor_events', { ignore: enable });
  } catch (e) {
    console.error('Failed to set click-through:', e);
  }
}

// Check if cursor is over the pet area
async function checkCursorOverPet(): Promise<boolean> {
  try {
    const [cursorX, cursorY] = await invoke<[number, number]>('get_cursor_position');
    const window = getCurrentWebviewWindow();
    const pos = await window.outerPosition();
    const scaleFactor = await window.scaleFactor();
    
    // Pet is at bottom-right: 100x100px, positioned with 50px margin
    // Window is 420x400
    const margin = 50 * scaleFactor;
    const petSize = 100 * scaleFactor;
    const petX = pos.x + (420 * scaleFactor) - petSize - margin;
    const petY = pos.y + (400 * scaleFactor) - petSize - margin;
    
    return cursorX >= petX && cursorX <= petX + petSize &&
           cursorY >= petY && cursorY <= petY + petSize;
  } catch (e) {
    return false;
  }
}

// Poll cursor position to toggle click-through
function startClickThroughPoller() {
  let lastState: boolean | null = null;
  
  setInterval(async () => {
    // Don't change if overlays are open
    if (!contextMenu.classList.contains('hidden') || 
        settingsOverlay.classList.contains('open') ||
        welcomeOverlay.classList.contains('open') ||
        speechBubble.classList.contains('visible')) {
      if (lastState !== false) {
        await setClickThrough(false);
        lastState = false;
      }
      return;
    }
    
    const isOverPet = await checkCursorOverPet();
    if (isOverPet !== lastState) {
      await setClickThrough(!isOverPet);
      lastState = isOverPet;
    }
  }, 100); // Check 10 times per second
}

// Event listeners
function setupEventListeners() {

  // Right-click context menu
  character.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    showContextMenu(e.clientX, e.clientY);
  });

  document.addEventListener('click', (e) => {
    if (!contextMenu.contains(e.target as Node)) hideContextMenu();
  });

  // Menu items
  document.getElementById('menu-roast')?.addEventListener('click', () => { hideContextMenu(); triggerRoast(); });
  document.getElementById('menu-monitor')?.addEventListener('click', () => { hideContextMenu(); toggleMonitoring(); });
  document.getElementById('menu-settings')?.addEventListener('click', () => { hideContextMenu(); showSettings(); });
  document.getElementById('menu-quit')?.addEventListener('click', () => { hideContextMenu(); invoke('quit'); });

  // Settings
  document.getElementById('settings-close')?.addEventListener('click', closeSettings);
  document.getElementById('settings-save')?.addEventListener('click', async () => {
    // Save settings
    const visionProv = (document.getElementById('vision-provider-select') as HTMLSelectElement).value;
    const visionModel = (document.getElementById('vision-model-select') as HTMLSelectElement).value;
    const ttsProv = (document.getElementById('tts-provider-select') as HTMLSelectElement).value;
    const ttsModel = (document.getElementById('tts-model-select') as HTMLSelectElement).value;
    const ttsVoice = (document.getElementById('tts-voice-select') as HTMLSelectElement).value;
    const mood = (document.getElementById('mood-select') as HTMLSelectElement)?.value;
    const customMood = (document.getElementById('custom-mood-input') as HTMLTextAreaElement)?.value || '';

    await invoke('set_config', { key: 'vision_provider', value: visionProv });
    await invoke('set_config', { key: 'vision_model', value: visionModel });
    await invoke('set_config', { key: 'tts_provider', value: ttsProv });
    await invoke('set_config', { key: 'tts_model', value: ttsModel });
    await invoke('set_config', { key: 'tts_voice', value: ttsVoice });
    if (mood) await invoke('set_config', { key: 'mood', value: mood });
    await invoke('set_config', { key: 'custom_mood', value: customMood });

    // Reload config
    config = await invoke('get_config');
    closeSettings();
  });

  // Improve mood with AI
  document.getElementById('improve-mood-btn')?.addEventListener('click', async () => {
    const customMoodInput = document.getElementById('custom-mood-input') as HTMLTextAreaElement;
    const improveBtn = document.getElementById('improve-mood-btn') as HTMLButtonElement;
    const originalText = customMoodInput.value.trim();

    if (!originalText) {
      showError('Please enter a custom mood prompt first');
      return;
    }

    try {
      improveBtn.disabled = true;
      improveBtn.textContent = '✨ Improving...';

      // Call the improve_mood command
      const improved = await invoke<string>('improve_mood', { moodText: originalText });
      customMoodInput.value = improved;
    } catch (e) {
      showError(`Failed to improve mood: ${e}`);
    } finally {
      improveBtn.disabled = false;
      improveBtn.textContent = '✨ Improve with AI';
    }
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
