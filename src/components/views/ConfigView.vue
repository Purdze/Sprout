<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Server } from '../../types';
import {
  detectFormat,
  parseConfig,
  serializeConfig,
  type ConfigFormat,
} from '../../utils/configParser';
import VisualConfigEditor from './VisualConfigEditor.vue';

const props = defineProps<{
  server: Server;
}>();

const emit = defineEmits<{
  loadConfigFiles: [dir: string];
  loadConfigFile: [dir: string, file: string];
  saveConfigFile: [dir: string, file: string, content: string];
  togglePlugin: [file: string, enable: boolean];
}>();

const selectedConfigFile = ref('');
const configContent = ref('');
const configModified = ref(false);
const selectedDirectory = ref('config');
const copied = ref(false);
const directories = ['config', 'data', 'worlds', 'logs', 'plugins'];

const editorMode = ref<'raw' | 'visual'>('raw');
const configFormat = ref<ConfigFormat>('unknown');
const parsedConfig = ref<Record<string, unknown> | null>(null);
const parseError = ref(false);

const selectedPlugin = ref('');
const pluginSubFiles = ref<string[]>([]);
const pluginFolders = ref<Set<string>>(new Set());
const collapsedSections = ref(new Set<string>(['plugins', 'folders', 'files']));

function resetEditorState() {
  configContent.value = '';
  configModified.value = false;
  parsedConfig.value = null;
  parseError.value = false;
  editorMode.value = 'raw';
}

function confirmDiscard(): boolean {
  return !configModified.value || confirm('You have unsaved changes. Discard them?');
}

const isPluginsDir = computed(() => selectedDirectory.value === 'plugins');

function toggleSection(section: string) {
  if (collapsedSections.value.has(section)) {
    collapsedSections.value.delete(section);
  } else {
    collapsedSections.value.add(section);
  }
}

const pluginDlls = computed(() => {
  if (!isPluginsDir.value || !props.server.configFiles) return [];
  return props.server.configFiles
    .filter((f) => f.endsWith('.dll') || f.endsWith('.dll.disabled'))
    .map((f) => ({
      file: f,
      name: f.replace(/\.dll(\.disabled)?$/, ''),
      enabled: f.endsWith('.dll') && !f.endsWith('.dll.disabled'),
    }));
});

const pluginFolderList = computed(() => [...pluginFolders.value].sort());

const topLevelPluginConfigs = computed(() => {
  if (!isPluginsDir.value || !props.server.configFiles) return [];
  return props.server.configFiles.filter(
    (f) => !f.endsWith('.dll') && !f.endsWith('.dll.disabled')
  );
});

const fileList = computed(() => {
  if (isPluginsDir.value) return [];
  return props.server.configFiles || [];
});

watch(
  selectedDirectory,
  async (dir) => {
    selectedConfigFile.value = '';
    selectedPlugin.value = '';
    resetEditorState();
    pluginSubFiles.value = [];
    pluginFolders.value = new Set();
    emit('loadConfigFiles', dir);

    if (dir === 'plugins') {
      try {
        const dirs = await invoke<string[]>('list_subdirectories', {
          path: props.server.path,
          dir: 'plugins',
        });
        pluginFolders.value = new Set(dirs);
      } catch {
        pluginFolders.value = new Set();
      }
    }
  },
  { immediate: true }
);

watch(
  () => props.server.configFiles,
  (files) => {
    if (files && files.length > 0 && !selectedConfigFile.value && !isPluginsDir.value) {
      selectedConfigFile.value = files[0];
      emit('loadConfigFile', selectedDirectory.value, files[0]);
    }
  }
);

watch(
  () => props.server.configContent,
  (content) => {
    configContent.value = content || '';
    configModified.value = false;
    tryParseConfig();
  }
);

function tryParseConfig() {
  const fmt = detectFormat(selectedConfigFile.value);
  configFormat.value = fmt;
  if (fmt !== 'unknown') {
    const result = parseConfig(configContent.value, fmt);
    if (result) {
      parsedConfig.value = result;
      parseError.value = false;
      editorMode.value = 'visual';
    } else {
      parsedConfig.value = null;
      parseError.value = true;
      editorMode.value = 'raw';
    }
  } else {
    parsedConfig.value = null;
    parseError.value = false;
    editorMode.value = 'raw';
  }
}

function switchToRaw() {
  if (editorMode.value === 'visual' && parsedConfig.value) {
    configContent.value = serializeConfig(parsedConfig.value, configFormat.value);
  }
  editorMode.value = 'raw';
}

function switchToVisual() {
  const result = parseConfig(configContent.value, configFormat.value);
  if (result) {
    parsedConfig.value = result;
    parseError.value = false;
    editorMode.value = 'visual';
  } else {
    parseError.value = true;
  }
}

function onVisualUpdate(data: Record<string, unknown>) {
  parsedConfig.value = data;
  configModified.value = true;
}

async function selectPlugin(name: string) {
  if (!confirmDiscard()) return;
  selectedPlugin.value = name;
  selectedConfigFile.value = '';
  resetEditorState();

  try {
    const files = await invoke<string[]>('list_config_files', {
      path: props.server.path,
      dir: `plugins/${name}`,
    });
    pluginSubFiles.value = files;
  } catch {
    pluginSubFiles.value = [];
  }
}

function selectConfigFile(file: string) {
  if (!confirmDiscard()) return;
  selectedConfigFile.value = file;
  resetEditorState();
  emit('loadConfigFile', selectedDirectory.value, file);
}

function selectPluginFile(fileName: string) {
  if (!confirmDiscard()) return;
  const fullPath = `${selectedPlugin.value}/${fileName}`;
  selectedConfigFile.value = fullPath;
  resetEditorState();
  emit('loadConfigFile', 'plugins', fullPath);
}

function backToPluginFiles() {
  if (!confirmDiscard()) return;
  selectedConfigFile.value = '';
  resetEditorState();
}

function saveConfig() {
  let content = configContent.value;
  if (editorMode.value === 'visual' && parsedConfig.value) {
    content = serializeConfig(parsedConfig.value, configFormat.value);
    configContent.value = content;
  }
  emit('saveConfigFile', selectedDirectory.value, selectedConfigFile.value, content);
  configModified.value = false;
}

function copyToClipboard() {
  navigator.clipboard.writeText(configContent.value);
  copied.value = true;
  setTimeout(() => {
    copied.value = false;
  }, 2000);
}

const isReadOnly = () =>
  selectedConfigFile.value.endsWith('.gz') || selectedDirectory.value === 'logs';

const formatLabel = computed(() => {
  if (configFormat.value === 'toml') return 'TOML';
  if (configFormat.value === 'json') return 'JSON';
  return '';
});

const showModeToggle = computed(
  () => configFormat.value !== 'unknown' && selectedConfigFile.value && !isReadOnly()
);
</script>

<template>
  <div class="config-view">
    <div class="config-sidebar">
      <div class="config-sidebar-header">
        <select v-model="selectedDirectory" class="directory-select">
          <option v-for="dir in directories" :key="dir" :value="dir">{{ dir }}</option>
        </select>
        <span v-if="server.configFiles && server.configFiles.length > 0" class="file-count">
          {{ isPluginsDir ? pluginDlls.length + ' plugins' : server.configFiles.length + ' files' }}
        </span>
      </div>

      <template v-if="isPluginsDir">
        <div class="config-file-list">
          <template v-if="pluginDlls.length > 0">
            <div class="sidebar-section-header" @click="toggleSection('plugins')">
              <svg
                :class="['disclosure', { open: !collapsedSections.has('plugins') }]"
                width="10"
                height="10"
                viewBox="0 0 24 24"
                fill="currentColor"
              >
                <path d="M8 5l8 7-8 7z" />
              </svg>
              <span>Plugins</span>
              <span class="section-count">{{ pluginDlls.length }}</span>
            </div>
            <template v-if="!collapsedSections.has('plugins')">
              <div v-for="plugin in pluginDlls" :key="plugin.file" class="plugin-row">
                <span class="plugin-name" :class="{ disabled: !plugin.enabled }">{{
                  plugin.name
                }}</span>
                <button
                  :class="['toggle-switch', { on: plugin.enabled }]"
                  :disabled="server.status !== 'stopped'"
                  :title="
                    server.status !== 'stopped'
                      ? 'Stop server to toggle'
                      : plugin.enabled
                        ? 'Disable'
                        : 'Enable'
                  "
                  @click.stop="emit('togglePlugin', plugin.file, !plugin.enabled)"
                >
                  <span class="toggle-knob" />
                </button>
              </div>
            </template>
          </template>

          <template v-if="pluginFolderList.length > 0">
            <div class="sidebar-section-header" @click="toggleSection('folders')">
              <svg
                :class="['disclosure', { open: !collapsedSections.has('folders') }]"
                width="10"
                height="10"
                viewBox="0 0 24 24"
                fill="currentColor"
              >
                <path d="M8 5l8 7-8 7z" />
              </svg>
              <span>Folders</span>
              <span class="section-count">{{ pluginFolderList.length }}</span>
            </div>
            <template v-if="!collapsedSections.has('folders')">
              <div
                v-for="folder in pluginFolderList"
                :key="folder"
                :class="['config-file-item', { active: selectedPlugin === folder }]"
                @click="selectPlugin(folder)"
              >
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <path
                    d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
                  />
                </svg>
                <span>{{ folder }}</span>
              </div>
            </template>
          </template>

          <template v-if="topLevelPluginConfigs.length > 0">
            <div class="sidebar-section-header" @click="toggleSection('files')">
              <svg
                :class="['disclosure', { open: !collapsedSections.has('files') }]"
                width="10"
                height="10"
                viewBox="0 0 24 24"
                fill="currentColor"
              >
                <path d="M8 5l8 7-8 7z" />
              </svg>
              <span>Files</span>
              <span class="section-count">{{ topLevelPluginConfigs.length }}</span>
            </div>
            <template v-if="!collapsedSections.has('files')">
              <div
                v-for="file in topLevelPluginConfigs"
                :key="file"
                :class="[
                  'config-file-item',
                  { active: selectedConfigFile === file && !selectedPlugin },
                ]"
                @click="
                  selectedPlugin = '';
                  selectConfigFile(file);
                "
              >
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                  <polyline points="14 2 14 8 20 8" />
                </svg>
                <span>{{ file }}</span>
              </div>
            </template>
          </template>

          <div
            v-if="
              pluginDlls.length === 0 &&
              pluginFolderList.length === 0 &&
              topLevelPluginConfigs.length === 0
            "
            class="config-empty"
          >
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z" />
              <polyline points="13 2 13 9 20 9" />
            </svg>
            <span>No plugins found</span>
          </div>
        </div>
      </template>

      <template v-else>
        <div class="config-file-list">
          <div
            v-for="file in fileList"
            :key="file"
            :class="['config-file-item', { active: selectedConfigFile === file }]"
            @click="selectConfigFile(file)"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
            </svg>
            <span>{{ file }}</span>
          </div>
          <div v-if="!server.configFiles || server.configFiles.length === 0" class="config-empty">
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z" />
              <polyline points="13 2 13 9 20 9" />
            </svg>
            <span>No files found</span>
          </div>
        </div>
      </template>

      <div v-if="server.status !== 'stopped'" class="config-warning">Stop server to edit</div>
    </div>

    <div class="config-editor">
      <div class="config-editor-header">
        <span class="config-editor-title">
          <template v-if="isPluginsDir && selectedPlugin">
            <span class="breadcrumb-link" @click="backToPluginFiles">{{ selectedPlugin }}</span>
            <template v-if="selectedConfigFile">
              <span class="breadcrumb-sep">/</span>
              <span>{{ selectedConfigFile.split('/').slice(1).join('/') }}</span>
            </template>
          </template>
          <template v-else>
            {{ selectedConfigFile || (isPluginsDir ? 'Select a plugin' : 'Select a file') }}
          </template>
          <span v-if="formatLabel && selectedConfigFile" class="format-badge">{{
            formatLabel
          }}</span>
          <span v-if="selectedConfigFile && isReadOnly()" class="readonly-badge">Read Only</span>
        </span>
        <div class="config-editor-actions">
          <div v-if="showModeToggle" class="mode-toggle">
            <button
              :class="['mode-btn', { active: editorMode === 'visual' }]"
              :disabled="editorMode === 'visual'"
              @click="switchToVisual"
            >
              Visual
            </button>
            <button
              :class="['mode-btn', { active: editorMode === 'raw' }]"
              :disabled="editorMode === 'raw'"
              @click="switchToRaw"
            >
              Raw
            </button>
          </div>
          <span
            v-if="parseError && editorMode === 'raw' && configFormat !== 'unknown'"
            class="parse-error"
          >
            Parse error
          </span>
          <template v-if="selectedDirectory === 'logs'">
            <button :class="['config-copy-btn', { copied }]" @click="copyToClipboard">
              {{ copied ? 'Copied!' : 'Copy' }}
            </button>
          </template>
          <template v-else-if="selectedConfigFile">
            <span v-if="configModified" class="config-modified">Modified</span>
            <button
              class="config-save-btn"
              :disabled="
                !configModified || server.status !== 'stopped' || selectedConfigFile.endsWith('.gz')
              "
              @click="saveConfig"
            >
              Save
            </button>
          </template>
        </div>
      </div>

      <div v-if="isPluginsDir && selectedPlugin && !selectedConfigFile" class="plugin-file-browser">
        <div
          v-for="file in pluginSubFiles"
          :key="file"
          class="plugin-file-row"
          @click="selectPluginFile(file)"
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
          <span class="plugin-file-name">{{ file }}</span>
          <span v-if="file.includes('.')" class="plugin-file-ext">{{
            file.split('.').pop()?.toUpperCase()
          }}</span>
        </div>
        <div v-if="pluginSubFiles.length === 0" class="plugin-no-files">
          <svg
            width="32"
            height="32"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
          <span>No config files for this plugin</span>
        </div>
      </div>

      <div
        v-else-if="isPluginsDir && !selectedPlugin && !selectedConfigFile"
        class="plugin-landing"
      >
        <svg
          width="40"
          height="40"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" />
          <polyline points="14 2 14 8 20 8" />
        </svg>
        <span>Select a config folder to browse</span>
      </div>

      <div
        v-else-if="editorMode === 'visual' && parsedConfig && selectedConfigFile"
        class="visual-editor-wrapper"
      >
        <VisualConfigEditor
          :data="parsedConfig"
          :disabled="server.status !== 'stopped'"
          :path="[]"
          @update:data="onVisualUpdate"
        />
      </div>

      <textarea
        v-else
        v-model="configContent"
        class="config-textarea"
        :placeholder="
          server.status !== 'stopped'
            ? 'Stop the server to edit config'
            : selectedConfigFile
              ? 'Loading...'
              : 'Select a file to edit'
        "
        :disabled="!selectedConfigFile || server.status !== 'stopped' || isReadOnly()"
        spellcheck="false"
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        @input="configModified = true"
      ></textarea>
    </div>
  </div>
</template>

<style scoped>
.config-view {
  flex: 1;
  display: flex;
  overflow: hidden;
  background: var(--bg-base);
}

.config-sidebar {
  width: 210px;
  background: var(--bg-raised);
  border-right: 1px solid var(--border-default);
  display: flex;
  flex-direction: column;
}

.config-sidebar-header {
  padding: 10px;
  border-bottom: 1px solid var(--border-default);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.file-count {
  font-size: 10px;
  color: var(--text-faint);
  text-align: center;
  font-variant-numeric: tabular-nums;
}

.directory-select {
  width: 100%;
  padding: 7px 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 13px;
  font-family: var(--font-ui);
  cursor: pointer;
  transition: border-color var(--transition-fast);
}

.directory-select:focus {
  outline: none;
  border-color: var(--accent);
}

.plugin-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px 6px 14px;
  gap: 6px;
  transition: background var(--transition-fast);
}

.plugin-name {
  font-size: 12.5px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.plugin-name.disabled {
  color: var(--text-faint);
}

.sidebar-section-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-faint);
  border-top: 1px solid var(--border-subtle);
  cursor: pointer;
  user-select: none;
  transition: color var(--transition-fast);
}

.sidebar-section-header:first-child {
  border-top: none;
}

.sidebar-section-header:hover {
  color: var(--text-tertiary);
}

.disclosure {
  transition: transform var(--transition-fast);
  flex-shrink: 0;
}

.disclosure.open {
  transform: rotate(90deg);
}

.section-count {
  margin-left: auto;
  font-variant-numeric: tabular-nums;
}

.toggle-switch {
  position: relative;
  width: 32px;
  height: 18px;
  background: var(--bg-active);
  border: 1px solid var(--border-default);
  border-radius: 9px;
  cursor: pointer;
  transition: all var(--transition-fast);
  flex-shrink: 0;
  padding: 0;
}

.toggle-switch.on {
  background: var(--color-success);
  border-color: var(--color-success);
}

.toggle-switch:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 12px;
  height: 12px;
  background: var(--text-primary);
  border-radius: 50%;
  transition: transform var(--transition-fast);
}

.toggle-switch.on .toggle-knob {
  transform: translateX(14px);
}

.config-file-list {
  flex: 1;
  overflow-y: auto;
}

.config-file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 13px;
  transition:
    background var(--transition-fast),
    color var(--transition-fast);
}

.config-file-item:hover {
  background: var(--bg-surface);
  color: var(--text-secondary);
}

.config-file-item.active {
  background: var(--bg-active);
  color: var(--text-primary);
}

.config-empty {
  padding: 28px 14px;
  color: var(--text-faint);
  font-size: 12px;
  font-style: italic;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  opacity: 0.5;
}

.config-warning {
  padding: 10px 14px;
  background: var(--color-warning-muted);
  color: var(--color-warning);
  font-size: 11px;
  font-weight: 500;
  text-align: center;
  border-top: 1px solid var(--border-default);
}

.config-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.config-editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--bg-raised);
  border-bottom: 1px solid var(--border-default);
}

.config-editor-title {
  color: var(--text-primary);
  font-size: 13px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 6px;
}

.breadcrumb-link {
  color: var(--accent);
  cursor: pointer;
  transition: color var(--transition-fast);
}

.breadcrumb-link:hover {
  color: var(--accent-hover);
}

.breadcrumb-sep {
  color: var(--text-faint);
}

.format-badge {
  font-size: 9px;
  padding: 2px 7px;
  background: var(--accent-muted);
  color: var(--accent);
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 600;
}

.readonly-badge {
  font-size: 9px;
  padding: 2px 7px;
  background: var(--bg-active);
  border-radius: 4px;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 600;
}

.config-editor-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.mode-toggle {
  display: flex;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.mode-btn {
  padding: 4px 12px;
  background: none;
  border: none;
  color: var(--text-tertiary);
  font-size: 11px;
  font-weight: 500;
  font-family: var(--font-ui);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.mode-btn:hover:not(:disabled) {
  color: var(--text-secondary);
}

.mode-btn.active {
  background: var(--bg-active);
  color: var(--text-primary);
}

.mode-btn:disabled {
  cursor: default;
}

.parse-error {
  font-size: 11px;
  color: var(--color-danger);
  font-weight: 500;
}

.config-modified {
  color: var(--accent);
  font-size: 12px;
  font-weight: 500;
}

.config-save-btn,
.config-copy-btn {
  padding: 6px 16px;
  border: none;
  border-radius: var(--radius-sm);
  color: #0a0a0f;
  font-size: 12px;
  font-weight: 600;
  font-family: var(--font-ui);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.config-save-btn {
  background: var(--color-success);
}

.config-save-btn:hover:not(:disabled) {
  box-shadow: 0 0 12px rgba(74, 222, 128, 0.25);
}

.config-save-btn:disabled {
  background: var(--bg-surface);
  color: var(--text-faint);
  cursor: not-allowed;
}

.config-copy-btn {
  background: var(--color-info);
}

.config-copy-btn:hover {
  box-shadow: 0 0 12px rgba(96, 165, 250, 0.25);
}

.config-copy-btn.copied {
  background: var(--color-success);
}

.plugin-file-browser {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.plugin-file-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 13px;
  transition:
    background var(--transition-fast),
    color var(--transition-fast);
}

.plugin-file-row:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.plugin-file-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.plugin-file-ext {
  font-size: 9px;
  padding: 2px 6px;
  background: var(--bg-surface);
  border-radius: 4px;
  color: var(--text-faint);
  font-weight: 600;
  letter-spacing: 0.04em;
  flex-shrink: 0;
}

.plugin-no-files {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 60px 20px;
  color: var(--text-faint);
  font-size: 13px;
  opacity: 0.5;
}

.plugin-landing {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--text-faint);
  font-size: 13px;
  opacity: 0.5;
}

.visual-editor-wrapper {
  flex: 1;
  overflow-y: auto;
  background: var(--bg-base);
}

.config-textarea {
  flex: 1;
  padding: 14px 16px;
  background: var(--bg-base);
  border: none;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 12.5px;
  resize: none;
  line-height: 1.65;
}

.config-textarea:focus {
  outline: none;
}

.config-textarea:disabled {
  color: var(--text-faint);
}
</style>
