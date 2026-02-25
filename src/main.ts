import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { register } from '@tauri-apps/plugin-global-shortcut';

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
  roast_intensity: number;
  mood_rotation: string;
  blacklist: string[];
  break_reminder_minutes: number;
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
  roast_intensity: 5,
  mood_rotation: '',
  blacklist: [],
  break_reminder_minutes: 0,
};

let providers: ProviderDef[] = [];
let isProcessing = false;
let lastActiveTab = 'general';
let tabsInitialized = false;

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

  // Apply Windows theme and accent color
  applyWindowsTheme();

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

  // Listen for tray events
  await listen('tray-toggle-monitoring', async () => {
    await toggleMonitoring();
  });
  await listen('tray-open-settings', async () => {
    await showSettings();
  });

  // Listen for break reminders
  await listen('break-reminder', async () => {
    console.log('Break reminder triggered');
    await setClickThrough(false);
    speechText.textContent = 'Hey, you\'ve been at it a while. Time to stretch, hydrate, and look away from the screen for a bit!';
    speechBubble.classList.add('visible');
    speechBubble.classList.remove('hidden');
    setTimeout(async () => {
      speechBubble.classList.remove('visible');
      speechBubble.classList.add('hidden');
      if (!contextMenu.classList.contains('hidden') ||
        settingsOverlay.classList.contains('open') ||
        welcomeOverlay.classList.contains('open')) {
        return;
      }
      await setClickThrough(true);
    }, 10000);
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

  // Register global hotkey (Ctrl+Shift+R) for instant roast
  try {
    await register('CommandOrControl+Shift+R', () => {
      console.log('Global hotkey triggered - roasting!');
      triggerRoast();
    });
    console.log('Global hotkey registered: Ctrl+Shift+R');
  } catch (e) {
    console.warn('Failed to register global hotkey:', e);
  }
}

// Apply Windows theme and accent color
function applyWindowsTheme() {
  // Detect dark/light mode
  const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

  // Listen for theme changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    applyWindowsTheme();
  });

  // Try to get Windows accent color from CSS
  const accentColor = getComputedStyle(document.documentElement).getPropertyValue('accent-color') || '#e94560';

  // Apply accent color to CSS variables
  document.documentElement.style.setProperty('--color-primary', accentColor);
  document.documentElement.style.setProperty('--color-primary-hover', adjustBrightness(accentColor, 20));

  console.log(`Theme applied: ${isDark ? 'dark' : 'light'}, accent: ${accentColor}`);
}

// Adjust color brightness
function adjustBrightness(hex: string, percent: number): string {
  const num = parseInt(hex.replace('#', ''), 16);
  const r = Math.min(255, Math.max(0, (num >> 16) + percent));
  const g = Math.min(255, Math.max(0, ((num >> 8) & 0x00FF) + percent));
  const b = Math.min(255, Math.max(0, (num & 0x0000FF) + percent));
  return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, '0')}`;
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

        // Calculate direction from THIS eye's center to cursor (independent tracking)
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

// Pet click reactions
const PET_QUIPS = [
  'Hey! Watch it!',
  'Do I look like a button to you?',
  'Poke me again, I dare you.',
  '*yawns* Oh, it\'s you.',
  'I was napping, thanks.',
  'That tickles!',
  'Stop that. I\'m working here.',
  'You have nothing better to do?',
  'Boop!',
  'I see you, I judge you.',
  'Yes? Can I help you?',
  'Go back to work.',
  'I\'m not a stress ball.',
  '*stares judgmentally*',
  'Touch grass instead.',
];

let lastPokeTime = 0;

async function pokePet() {
  const now = Date.now();
  if (now - lastPokeTime < 1500) return; // Debounce
  if (isProcessing) return;
  lastPokeTime = now;

  // Poke animation
  character.classList.remove('poked');
  void character.offsetWidth; // Force reflow to restart animation
  character.classList.add('poked');
  setTimeout(() => character.classList.remove('poked'), 400);

  // Show random quip
  const quip = PET_QUIPS[Math.floor(Math.random() * PET_QUIPS.length)];
  await setClickThrough(false);
  speechText.textContent = quip;
  speechBubble.classList.add('visible');
  speechBubble.classList.remove('hidden');

  setTimeout(async () => {
    speechBubble.classList.remove('visible');
    speechBubble.classList.add('hidden');
    if (!contextMenu.classList.contains('hidden') ||
      settingsOverlay.classList.contains('open') ||
      welcomeOverlay.classList.contains('open')) {
      return;
    }
    await setClickThrough(true);
  }, 3000);
}

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

  // Update dynamic labels
  updateMuteLabel();
  const monitorItem = document.getElementById('menu-monitor') as HTMLElement;
  monitorItem.textContent = config.is_active ? '⏸ Pause Monitoring' : '▶ Start Monitoring';

  // Populate mood submenu
  populateMoodSubmenu();

  // Close any open submenu
  document.getElementById('mood-submenu')?.classList.remove('open');

  const appRect = app.getBoundingClientRect();
  const menuWidth = 180;
  const menuHeight = 250; // increased for new items

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

function updateMuteLabel() {
  const muteItem = document.getElementById('menu-mute') as HTMLElement;
  if (muteItem) {
    muteItem.textContent = config.audio_enabled ? '🔇 Mute Audio' : '🔊 Unmute Audio';
  }
}

function populateMoodSubmenu() {
  const submenu = document.getElementById('mood-submenu') as HTMLElement;
  if (!submenu) return;
  submenu.innerHTML = '';

  const moods = ['roast', 'helpful', 'encouraging', 'sarcastic', 'zen', 'anime', 'gordon', 'therapist', 'detective', 'hype'];
  moods.forEach(mood => {
    const item = document.createElement('div');
    item.className = `menu-item${config.mood === mood ? ' active-mood' : ''}`;
    item.textContent = mood.charAt(0).toUpperCase() + mood.slice(1);
    item.addEventListener('click', async (e) => {
      e.stopPropagation();
      config.mood = mood;
      await invoke('set_config', { key: 'mood', value: mood });
      hideContextMenu();
    });
    submenu.appendChild(item);
  });
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

    // Resize window to fit settings content and keep it on screen
    const window = getCurrentWebviewWindow();
    const scaleFactor = await window.scaleFactor();
    const currentPos = await window.outerPosition();

    // Get primary monitor size
    const { availableMonitors } = await import('@tauri-apps/api/window');
    const monitors = await availableMonitors();
    const primaryMonitor = monitors[0];

    const settingsWidth = 420;
    const settingsHeight = 700;

    // Calculate new position to keep window on screen
    let newX = currentPos.x;
    let newY = currentPos.y;

    // Check right edge
    if (newX + settingsWidth * scaleFactor > primaryMonitor.size.width) {
      newX = primaryMonitor.size.width - settingsWidth * scaleFactor;
    }

    // Check bottom edge
    if (newY + settingsHeight * scaleFactor > primaryMonitor.size.height) {
      newY = primaryMonitor.size.height - settingsHeight * scaleFactor;
    }

    // Check left edge
    if (newX < 0) newX = 0;

    // Check top edge
    if (newY < 0) newY = 0;

    // Reposition if needed
    if (newX !== currentPos.x || newY !== currentPos.y) {
      await window.setPosition({ type: 'Physical', x: newX, y: newY });
    }

    await window.setSize({ type: 'Physical', width: settingsWidth, height: settingsHeight });

    const apiKeys = await invoke<Record<string, string>>('get_api_keys');

    // Populate API keys (skip local providers that don't need keys)
    const apiKeysList = document.getElementById('api-keys-list') as HTMLElement;
    apiKeysList.innerHTML = '';

    providers.forEach(provider => {
      // Skip local providers that don't need API keys
      if (provider.key === 'ollama' || provider.key === 'piper') {
        return;
      }

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
    buildIntervalUI();
    buildMoodUI();
    buildPetStyleUI();
    updateCostEstimator();

    // Populate blacklist
    const blacklistInput = document.getElementById('blacklist-input') as HTMLTextAreaElement;
    if (blacklistInput) {
      blacklistInput.value = (config.blacklist || []).join('\n');
    }

    // Populate break reminder
    const breakSelect = document.getElementById('break-reminder-select') as HTMLSelectElement;
    if (breakSelect) {
      breakSelect.value = String(config.break_reminder_minutes || 0);
    }

    // Tab switching
    if (!tabsInitialized) {
      const tabStrip = document.getElementById('settings-tabs') as HTMLElement;
      tabStrip.addEventListener('click', (e) => {
        const tab = (e.target as HTMLElement).closest('.settings-tab') as HTMLElement;
        if (!tab) return;
        const tabName = tab.dataset.tab!;

        // Update active tab button
        tabStrip.querySelectorAll('.settings-tab').forEach(t => t.classList.remove('active'));
        tab.classList.add('active');

        // Update active panel
        document.querySelectorAll('.settings-tab-panel').forEach(p => p.classList.remove('active'));
        const panel = document.querySelector(`.settings-tab-panel[data-panel="${tabName}"]`) as HTMLElement;
        if (panel) panel.classList.add('active');

        lastActiveTab = tabName;
      });
      tabsInitialized = true;
    }

    // Restore last active tab
    const tabStrip = document.getElementById('settings-tabs') as HTMLElement;
    tabStrip.querySelectorAll('.settings-tab').forEach(t => {
      t.classList.toggle('active', (t as HTMLElement).dataset.tab === lastActiveTab);
    });
    document.querySelectorAll('.settings-tab-panel').forEach(p => {
      p.classList.toggle('active', (p as HTMLElement).dataset.panel === lastActiveTab);
    });

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
    if (prov.vision_models.length === 0 && prov.key !== 'ollama') return;
    const opt = document.createElement('option');
    opt.value = prov.key;
    opt.textContent = prov.name;
    if (prov.key === config.vision_provider) opt.selected = true;
    provSelect.appendChild(opt);
  });

  async function updateModels() {
    const prov = providers.find(p => p.key === provSelect.value);
    if (!prov) return;

    modelSelect.innerHTML = '';

    // For Ollama, fetch models dynamically
    if (prov.key === 'ollama') {
      try {
        const ollamaModels = await invoke<string[]>('get_ollama_models');
        if (ollamaModels.length === 0) {
          const opt = document.createElement('option');
          opt.value = '';
          opt.textContent = 'No vision models found - install llama3.2-vision';
          modelSelect.appendChild(opt);
        } else {
          ollamaModels.forEach(model => {
            const opt = document.createElement('option');
            opt.value = model;
            opt.textContent = model;
            if (model === config.vision_model) opt.selected = true;
            modelSelect.appendChild(opt);
          });
        }
      } catch (e) {
        const opt = document.createElement('option');
        opt.value = '';
        opt.textContent = 'Ollama not running - install from ollama.com';
        modelSelect.appendChild(opt);
      }
    } else {
      // Static models for other providers
      prov.vision_models.forEach(model => {
        const opt = document.createElement('option');
        opt.value = model;
        opt.textContent = model;
        if (model === config.vision_model) opt.selected = true;
        modelSelect.appendChild(opt);
      });
    }
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

async function buildIntervalUI() {
  const intervalSelect = document.getElementById('interval-select') as HTMLSelectElement;
  if (!intervalSelect) return;

  try {
    const intervals = await invoke<[string, string][]>('get_intervals');
    intervalSelect.innerHTML = '';
    intervals.forEach(([key, label]) => {
      const opt = document.createElement('option');
      opt.value = key;
      opt.textContent = label;
      if (key === config.interval) opt.selected = true;
      intervalSelect.appendChild(opt);
    });
  } catch (e) {
    console.error('Failed to load intervals:', e);
  }
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

    // Intensity slider
    const intensitySlider = document.getElementById('intensity-slider') as HTMLInputElement;
    const intensityValue = document.getElementById('intensity-value') as HTMLElement;
    if (intensitySlider && intensityValue) {
      intensitySlider.value = String(config.roast_intensity || 5);
      intensityValue.textContent = intensitySlider.value;
      intensitySlider.addEventListener('input', () => {
        intensityValue.textContent = intensitySlider.value;
      });
    }

    // Mood rotation
    const rotationSelect = document.getElementById('mood-rotation-select') as HTMLSelectElement;
    if (rotationSelect) {
      rotationSelect.value = config.mood_rotation || '';
    }
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
  let isPolling = false;

  setInterval(async () => {
    if (isPolling) return;
    isPolling = true;

    try {
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
    } finally {
      isPolling = false;
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

  // Pet click reaction (left-click, not drag)
  let clickStartX = 0;
  let clickStartY = 0;
  character.addEventListener('mousedown', (e) => {
    if (e.button === 0) { clickStartX = e.screenX; clickStartY = e.screenY; }
  });
  character.addEventListener('mouseup', (e) => {
    if (e.button !== 0) return;
    const dist = Math.hypot(e.screenX - clickStartX, e.screenY - clickStartY);
    if (dist < 5) pokePet(); // Only trigger if not dragging
  });

  document.addEventListener('click', (e) => {
    if (!contextMenu.contains(e.target as Node)) hideContextMenu();
  });

  // Menu items
  document.getElementById('menu-roast')?.addEventListener('click', () => { hideContextMenu(); triggerRoast(); });
  document.getElementById('menu-monitor')?.addEventListener('click', () => { hideContextMenu(); toggleMonitoring(); });
  document.getElementById('menu-settings')?.addEventListener('click', () => { hideContextMenu(); showSettings(); });
  document.getElementById('menu-quit')?.addEventListener('click', () => { hideContextMenu(); invoke('quit'); });

  // Mute toggle
  document.getElementById('menu-mute')?.addEventListener('click', async () => {
    config.audio_enabled = !config.audio_enabled;
    await invoke('set_config', { key: 'audio_enabled', value: config.audio_enabled });
    updateMuteLabel();
  });

  // Mood submenu toggle
  document.getElementById('menu-mood-toggle')?.addEventListener('click', (e) => {
    e.stopPropagation();
    const submenu = document.getElementById('mood-submenu') as HTMLElement;
    submenu.classList.toggle('open');
  });

  // Settings
  document.getElementById('settings-close')?.addEventListener('click', closeSettings);
  document.getElementById('settings-save')?.addEventListener('click', async () => {
    // Save settings
    const interval = (document.getElementById('interval-select') as HTMLSelectElement)?.value;
    const visionProv = (document.getElementById('vision-provider-select') as HTMLSelectElement).value;
    const visionModel = (document.getElementById('vision-model-select') as HTMLSelectElement).value;
    const ttsProv = (document.getElementById('tts-provider-select') as HTMLSelectElement).value;
    const ttsModel = (document.getElementById('tts-model-select') as HTMLSelectElement).value;
    const ttsVoice = (document.getElementById('tts-voice-select') as HTMLSelectElement).value;
    const mood = (document.getElementById('mood-select') as HTMLSelectElement)?.value;
    const customMood = (document.getElementById('custom-mood-input') as HTMLTextAreaElement)?.value || '';
    const intensity = parseInt((document.getElementById('intensity-slider') as HTMLInputElement)?.value || '5');

    if (interval) await invoke('set_config', { key: 'interval', value: interval });
    await invoke('set_config', { key: 'vision_provider', value: visionProv });
    await invoke('set_config', { key: 'vision_model', value: visionModel });
    await invoke('set_config', { key: 'tts_provider', value: ttsProv });
    await invoke('set_config', { key: 'tts_model', value: ttsModel });
    await invoke('set_config', { key: 'tts_voice', value: ttsVoice });
    if (mood) await invoke('set_config', { key: 'mood', value: mood });
    await invoke('set_config', { key: 'custom_mood', value: customMood });
    await invoke('set_config', { key: 'roast_intensity', value: intensity });
    const moodRotation = (document.getElementById('mood-rotation-select') as HTMLSelectElement)?.value || '';
    await invoke('set_config', { key: 'mood_rotation', value: moodRotation });
    const blacklistText = (document.getElementById('blacklist-input') as HTMLTextAreaElement)?.value || '';
    await invoke('set_config', { key: 'blacklist', value: blacklistText });
    const breakMins = parseInt((document.getElementById('break-reminder-select') as HTMLSelectElement)?.value || '0');
    await invoke('set_config', { key: 'break_reminder_minutes', value: breakMins });

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
