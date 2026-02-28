<script setup lang="ts">
import { ref, watch } from 'vue';
import type { Server } from '../../types';

const props = defineProps<{
  server: Server;
}>();

const emit = defineEmits<{
  loadConfigFiles: [dir: string];
  loadConfigFile: [dir: string, file: string];
  saveConfigFile: [dir: string, file: string, content: string];
}>();

const selectedConfigFile = ref('');
const configContent = ref('');
const configModified = ref(false);
const selectedDirectory = ref('config');
const copied = ref(false);
const directories = ['config', 'data', 'worlds', 'logs', 'plugins'];

watch(
  selectedDirectory,
  (dir) => {
    selectedConfigFile.value = '';
    configContent.value = '';
    configModified.value = false;
    emit('loadConfigFiles', dir);
  },
  { immediate: true }
);

watch(
  () => props.server.configFiles,
  (files) => {
    if (files && files.length > 0 && !selectedConfigFile.value) {
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
  }
);

function selectConfigFile(file: string) {
  if (configModified.value) {
    if (!confirm('You have unsaved changes. Discard them?')) return;
  }
  selectedConfigFile.value = file;
  emit('loadConfigFile', selectedDirectory.value, file);
}

function saveConfig() {
  emit('saveConfigFile', selectedDirectory.value, selectedConfigFile.value, configContent.value);
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
</script>

<template>
  <div class="config-view">
    <div class="config-sidebar">
      <div class="config-sidebar-header">
        <select v-model="selectedDirectory" class="directory-select">
          <option v-for="dir in directories" :key="dir" :value="dir">{{ dir }}</option>
        </select>
        <span v-if="server.configFiles && server.configFiles.length > 0" class="file-count"
          >{{ server.configFiles.length }} files</span
        >
      </div>
      <div class="config-file-list">
        <div
          v-for="file in server.configFiles"
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
      <div v-if="server.status !== 'stopped'" class="config-warning">Stop server to edit</div>
    </div>
    <div class="config-editor">
      <div class="config-editor-header">
        <span class="config-editor-title">
          {{ selectedConfigFile || 'Select a file' }}
          <span v-if="isReadOnly()" class="readonly-badge">Read Only</span>
        </span>
        <div class="config-editor-actions">
          <button
            v-if="selectedDirectory === 'logs'"
            :class="['config-copy-btn', { copied }]"
            @click="copyToClipboard"
          >
            {{ copied ? 'Copied!' : 'Copy' }}
          </button>
          <template v-else>
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
      <textarea
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
  gap: 10px;
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
