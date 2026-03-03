<script setup lang="ts">
import { ref } from 'vue';
import type { Server, SavedCommand } from '../../types';

const props = defineProps<{
  server: Server;
}>();

const emit = defineEmits<{
  command: [cmd: string];
  'update:savedCommands': [commands: SavedCommand[]];
}>();

const newName = ref('');
const newCommand = ref('');
const editingIndex = ref<number | null>(null);
const editName = ref('');
const editCommand = ref('');

function addCommand() {
  const name = newName.value.trim();
  const command = newCommand.value.trim();
  if (!name || !command) return;

  const updated = [...props.server.savedCommands, { name, command }];
  emit('update:savedCommands', updated);
  newName.value = '';
  newCommand.value = '';
}

function removeCommand(index: number) {
  const updated = props.server.savedCommands.filter((_, i) => i !== index);
  emit('update:savedCommands', updated);
  if (editingIndex.value === index) editingIndex.value = null;
}

function startEdit(index: number) {
  editingIndex.value = index;
  editName.value = props.server.savedCommands[index].name;
  editCommand.value = props.server.savedCommands[index].command;
}

function cancelEdit() {
  editingIndex.value = null;
}

function saveEdit(index: number) {
  const name = editName.value.trim();
  const command = editCommand.value.trim();
  if (!name || !command) return;

  const updated = props.server.savedCommands.map((cmd, i) =>
    i === index ? { name, command } : cmd
  );
  emit('update:savedCommands', updated);
  editingIndex.value = null;
}
</script>

<template>
  <div class="commands-view">
    <div class="add-form">
      <input
        v-model="newName"
        class="form-input name-input"
        placeholder="Command name"
        @keydown.enter="addCommand"
      />
      <input
        v-model="newCommand"
        class="form-input command-input"
        placeholder="/give @p diamond 64"
        @keydown.enter="addCommand"
      />
      <button
        class="btn btn-add"
        :disabled="!newName.trim() || !newCommand.trim()"
        @click="addCommand"
      >
        Add
      </button>
    </div>

    <div class="command-list">
      <div v-for="(cmd, index) in server.savedCommands" :key="index" class="command-row">
        <template v-if="editingIndex !== index">
          <div class="command-info">
            <span class="command-name">{{ cmd.name }}</span>
            <code class="command-text">{{ cmd.command }}</code>
          </div>
          <div class="command-actions">
            <button
              class="btn btn-icon btn-run"
              title="Run command"
              @click="emit('command', cmd.command)"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" stroke="none">
                <path d="M8 5v14l11-7z" />
              </svg>
            </button>
            <button class="btn btn-icon btn-edit" title="Edit" @click="startEdit(index)">
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
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
              </svg>
            </button>
            <button class="btn btn-icon btn-delete" title="Delete" @click="removeCommand(index)">
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
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
          </div>
        </template>

        <template v-else>
          <div class="edit-form">
            <input
              v-model="editName"
              class="form-input name-input"
              @keydown.enter="saveEdit(index)"
              @keydown.escape="cancelEdit"
            />
            <input
              v-model="editCommand"
              class="form-input command-input"
              @keydown.enter="saveEdit(index)"
              @keydown.escape="cancelEdit"
            />
          </div>
          <div class="command-actions">
            <button class="btn btn-save" @click="saveEdit(index)">Save</button>
            <button class="btn btn-cancel" @click="cancelEdit">Cancel</button>
          </div>
        </template>
      </div>

      <div v-if="server.savedCommands.length === 0" class="empty-state">
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
          <polyline points="4 17 10 11 4 5" />
          <line x1="12" y1="19" x2="20" y2="19" />
        </svg>
        <p>No saved commands</p>
        <p class="empty-hint">Add frequently used commands above for quick access</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.commands-view {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.add-form {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  background: var(--bg-base);
  border-bottom: 1px solid var(--border-subtle);
}

.form-input {
  padding: 7px 10px;
  background: var(--bg-darkest);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 13px;
  font-family: var(--font-mono);
  outline: none;
  transition: border-color var(--transition-fast);
}

.form-input:focus {
  border-color: var(--color-primary);
}

.name-input {
  width: 160px;
  flex-shrink: 0;
}

.command-input {
  flex: 1;
  min-width: 0;
}

.btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6px 12px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  background: none;
  color: var(--text-secondary);
  transition:
    color var(--transition-fast),
    background var(--transition-fast),
    border-color var(--transition-fast);
}

.btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-add {
  color: var(--color-primary);
  border-color: rgba(96, 165, 250, 0.25);
}

.btn-add:hover:not(:disabled) {
  background: rgba(96, 165, 250, 0.1);
}

.btn-icon {
  padding: 6px 8px;
}

.btn-run,
.btn-save {
  color: var(--color-success);
  border-color: rgba(74, 222, 128, 0.2);
}

.btn-run:hover,
.btn-save:hover {
  background: var(--color-success-muted);
  border-color: rgba(74, 222, 128, 0.35);
}

.btn-delete {
  color: var(--color-danger);
  border-color: rgba(239, 68, 68, 0.2);
}

.btn-delete:hover {
  background: var(--color-danger-muted);
  border-color: rgba(239, 68, 68, 0.35);
}

.btn-cancel {
  color: var(--text-tertiary);
}

.command-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 16px;
}

.command-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  transition: background var(--transition-fast);
}

.command-row:hover {
  background: var(--bg-hover);
}

.command-info {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 12px;
}

.command-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  white-space: nowrap;
  flex-shrink: 0;
}

.command-text {
  font-size: 12px;
  font-family: var(--font-mono);
  color: var(--text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.command-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.edit-form {
  flex: 1;
  display: flex;
  gap: 8px;
  min-width: 0;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-tertiary);
  gap: 8px;
  opacity: 0.6;
}

.empty-state p {
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: 500;
}

.empty-state .empty-hint {
  font-size: 13px;
  color: var(--text-tertiary);
  font-weight: 400;
}
</style>
