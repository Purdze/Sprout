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

// Load config files when directory changes
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

// Select first file when list changes
watch(
  () => props.server.configFiles,
  (files) => {
    if (files && files.length > 0 && !selectedConfigFile.value) {
      selectedConfigFile.value = files[0];
      emit('loadConfigFile', selectedDirectory.value, files[0]);
    }
  }
);

// Update content when server content changes
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
          No config files found
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
  background: #0d0d0d;
}

.config-sidebar {
  width: 200px;
  background: #1a1a1a;
  border-right: 1px solid #333;
  display: flex;
  flex-direction: column;
}

.config-sidebar-header {
  padding: 8px;
  border-bottom: 1px solid #333;
}

.directory-select {
  width: 100%;
  padding: 6px 8px;
  background: #252525;
  border: 1px solid #333;
  border-radius: 4px;
  color: #fff;
  font-size: 13px;
  cursor: pointer;
}

.directory-select:focus {
  outline: none;
  border-color: #f97316;
}

.config-file-list {
  flex: 1;
  overflow-y: auto;
}

.config-file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  color: #888;
  cursor: pointer;
  font-size: 13px;
}

.config-file-item:hover {
  background: #252525;
  color: #ccc;
}

.config-file-item.active {
  background: #333;
  color: #fff;
}

.config-empty {
  padding: 12px;
  color: #444;
  font-size: 12px;
  font-style: italic;
}

.config-warning {
  padding: 10px 12px;
  background: #f59e0b20;
  color: #f59e0b;
  font-size: 11px;
  text-align: center;
  border-top: 1px solid #333;
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
  padding: 12px;
  background: #1a1a1a;
  border-bottom: 1px solid #333;
}

.config-editor-title {
  color: #fff;
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.readonly-badge {
  font-size: 10px;
  padding: 2px 6px;
  background: #666;
  border-radius: 3px;
  color: #fff;
  text-transform: uppercase;
}

.config-editor-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.config-modified {
  color: #f97316;
  font-size: 12px;
}

.config-save-btn {
  padding: 6px 14px;
  background: #4ade80;
  border: none;
  border-radius: 4px;
  color: #000;
  font-size: 12px;
  cursor: pointer;
}

.config-save-btn:disabled {
  background: #333;
  color: #666;
  cursor: not-allowed;
}

.config-copy-btn {
  padding: 6px 14px;
  background: #60a5fa;
  border: none;
  border-radius: 4px;
  color: #000;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.config-copy-btn:hover {
  background: #93c5fd;
}

.config-copy-btn.copied {
  background: #4ade80;
}

.config-textarea {
  flex: 1;
  padding: 12px;
  background: #0d0d0d;
  border: none;
  color: #ccc;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  resize: none;
  line-height: 1.5;
}

.config-textarea:focus {
  outline: none;
}

.config-textarea:disabled {
  color: #444;
}
</style>
