<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { getVersion, getTauriVersion } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-shell';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { ACCENT_COLORS, applyAccent, type AccentColor } from '../utils/accent';

const props = defineProps<{
  show: boolean;
  updateAvailable: boolean;
  checkingUpdate: boolean;
  upToDate: boolean;
}>();

const emit = defineEmits<{
  'update:show': [value: boolean];
  checkUpdate: [];
  openUpdate: [];
}>();

const appVersion = ref('');
const tauriVersion = ref('');
const commitHash = __COMMIT_HASH__;

const launchOnStartup = ref(false);
const minimizeToTray = ref(true);
const selectedAccent = ref<string | null>(null);

onMounted(async () => {
  appVersion.value = await getVersion();
  tauriVersion.value = await getTauriVersion();
});

watch(
  () => props.show,
  async (visible) => {
    if (visible) {
      try {
        const settings = await invoke<{ minimize_to_tray: boolean; accent_color: string | null }>(
          'load_app_settings'
        );
        minimizeToTray.value = settings.minimize_to_tray;
        selectedAccent.value = settings.accent_color;
      } catch {
        // ignore
      }

      try {
        launchOnStartup.value = await isEnabled();
      } catch {
        // ignore
      }
    }
  }
);

async function toggleStartup() {
  launchOnStartup.value = !launchOnStartup.value;
  try {
    if (launchOnStartup.value) {
      await enable();
    } else {
      await disable();
    }
  } catch {
    launchOnStartup.value = !launchOnStartup.value;
  }
}

async function toggleMinimizeToTray() {
  minimizeToTray.value = !minimizeToTray.value;
  await saveSettings();
}

async function selectAccent(color: AccentColor) {
  selectedAccent.value = color.value;
  applyAccent(color.value);
  await saveSettings();
}

async function saveSettings() {
  try {
    await invoke('save_app_settings', {
      settings: {
        minimize_to_tray: minimizeToTray.value,
        accent_color: selectedAccent.value,
      },
    });
  } catch (e) {
    console.error('Failed to save settings:', e);
  }
}

</script>

<template>
  <div v-if="show" class="dialog-overlay" @click="emit('update:show', false)">
    <div class="settings-dialog" @click.stop>
      <div class="settings-header">
        <div class="settings-title">
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="12" cy="12" r="3" />
            <path
              d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
            />
          </svg>
          Settings
        </div>
        <button class="close-btn" @click="emit('update:show', false)">
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
          >
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="settings-body">
        <div class="section">
          <div class="section-label">General</div>
          <div class="section-card">
            <div class="setting-row">
              <span class="setting-label">Launch on startup</span>
              <button
                :class="['toggle', { on: launchOnStartup }]"
                @click="toggleStartup"
              >
                <span class="toggle-thumb" />
              </button>
            </div>
            <div class="setting-row">
              <span class="setting-label">Minimize to tray on close</span>
              <button
                :class="['toggle', { on: minimizeToTray }]"
                @click="toggleMinimizeToTray"
              >
                <span class="toggle-thumb" />
              </button>
            </div>
          </div>
        </div>

        <div class="section">
          <div class="section-label">Appearance</div>
          <div class="section-card">
            <div class="setting-row">
              <span class="setting-label">Accent color</span>
              <div class="color-swatches">
                <button
                  v-for="color in ACCENT_COLORS"
                  :key="color.name"
                  :class="['swatch', { selected: selectedAccent === color.value }]"
                  :style="{ background: color.value }"
                  :title="color.name"
                  @click="selectAccent(color)"
                />
              </div>
            </div>
          </div>
        </div>

        <div class="section">
          <div class="section-label">About</div>
          <div class="section-card about-card">
            <div class="about-info">
              <span class="about-name">Sprout</span>
              <span class="about-meta">
                {{ appVersion }} &middot; tauri {{ tauriVersion }} &middot;
                <span class="mono">{{ commitHash }}</span>
              </span>
            </div>
            <div class="about-actions">
              <button class="link-btn" @click="open('https://github.com/Purdze/Sprout')">
                GitHub
              </button>
              <button class="link-btn" @click="open('https://discord.gg/qsRhJUP4q5')">
                Discord
              </button>
              <span class="spacer" />
              <button
                v-if="updateAvailable"
                class="update-btn available"
                @click="emit('openUpdate')"
              >
                Update available
              </button>
              <span v-else-if="upToDate" class="up-to-date">Up to date</span>
              <button
                v-else
                class="update-btn"
                :disabled="checkingUpdate"
                @click="emit('checkUpdate')"
              >
                <svg
                  v-if="checkingUpdate"
                  class="spin"
                  width="12"
                  height="12"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                >
                  <path d="M21 12a9 9 0 1 1-6.219-8.56" />
                </svg>
                {{ checkingUpdate ? 'Checking' : 'Check for updates' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-dialog {
  background: var(--bg-overlay);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  width: 420px;
  max-height: 80vh;
  overflow-y: auto;
  box-shadow: var(--shadow-lg);
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 20px 0;
}

.settings-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: none;
  border: none;
  border-radius: var(--radius-sm);
  color: var(--text-faint);
  cursor: pointer;
  transition: color var(--transition-fast), background var(--transition-fast);
}

.close-btn:hover {
  color: var(--text-secondary);
  background: rgba(255, 255, 255, 0.06);
}

.settings-body {
  padding: 16px 20px 20px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.section-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-tertiary);
  margin-bottom: 8px;
}

.section-card {
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 4px 0;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
}

.setting-label {
  font-size: 13px;
  color: var(--text-secondary);
}

/* Toggle switch */
.toggle {
  position: relative;
  width: 36px;
  height: 20px;
  background: var(--bg-active);
  border: none;
  border-radius: 10px;
  cursor: pointer;
  transition: background var(--transition-fast);
  padding: 0;
  flex-shrink: 0;
}

.toggle.on {
  background: var(--accent);
}

.toggle-thumb {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 14px;
  height: 14px;
  background: #fff;
  border-radius: 50%;
  transition: transform var(--transition-fast);
  pointer-events: none;
}

.toggle.on .toggle-thumb {
  transform: translateX(16px);
}

/* Color swatches */
.color-swatches {
  display: flex;
  gap: 6px;
}

.swatch {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: border-color var(--transition-fast), transform var(--transition-fast);
  padding: 0;
}

.swatch:hover {
  transform: scale(1.15);
}

.swatch.selected {
  border-color: var(--text-primary);
  box-shadow: 0 0 0 2px var(--bg-overlay);
}

/* About section */
.about-card {
  padding: 12px 14px;
}

.about-info {
  margin-bottom: 10px;
}

.about-name {
  display: block;
  font-weight: 700;
  font-size: 13px;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.about-meta {
  font-size: 12px;
  color: var(--text-faint);
}

.mono {
  font-family: var(--font-mono);
  font-size: 11px;
}

.about-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 12px;
}

.spacer {
  flex: 1;
}

.link-btn {
  background: none;
  border: none;
  padding: 0;
  font-size: 12px;
  font-family: var(--font-ui);
  color: var(--text-tertiary);
  cursor: pointer;
  transition: color var(--transition-fast);
}

.link-btn:hover {
  color: var(--text-primary);
}

.update-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  background: none;
  border: none;
  padding: 0;
  font-size: 12px;
  font-family: var(--font-ui);
  cursor: pointer;
  transition: color var(--transition-fast);
  color: var(--text-faint);
}

.update-btn:not(.available):hover:not(:disabled) {
  color: var(--text-secondary);
}

.update-btn:disabled {
  cursor: default;
  opacity: 0.5;
}

.update-btn.available {
  color: var(--color-info);
  font-weight: 600;
}

.update-btn.available:hover {
  filter: brightness(1.2);
}

.up-to-date {
  color: var(--color-success);
  font-size: 12px;
}

.spin {
  animation: spin 0.8s linear infinite;
}
</style>
