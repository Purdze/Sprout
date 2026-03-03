<script setup lang="ts">
import { ref } from 'vue';
import ConsoleView from './views/ConsoleView.vue';
import GraphsView from './views/GraphsView.vue';
import PlayersView from './views/PlayersView.vue';
import PlayerInventoryView from './views/PlayerInventoryView.vue';
import ConfigView from './views/ConfigView.vue';
import DomainsView from './views/DomainsView.vue';
import CommandsView from './views/CommandsView.vue';
import type { Server, SavedCommand } from '../types';

defineProps<{
  server: Server;
}>();

const emit = defineEmits<{
  start: [];
  stop: [];
  command: [cmd: string];
  clear: [];
  loadConfigFiles: [dir: string];
  loadConfigFile: [dir: string, file: string];
  saveConfigFile: [dir: string, file: string, content: string];
  togglePlugin: [file: string, enable: boolean];
  'update:savedCommands': [commands: SavedCommand[]];
}>();

const activeView = ref<
  'console' | 'graphs' | 'players' | 'config' | 'domains' | 'commands' | 'inventory'
>('console');
const selectedPlayer = ref('');
</script>

<template>
  <div class="server-tab">
    <!-- View Tabs -->
    <div class="view-tabs">
      <button
        :class="['view-tab', { active: activeView === 'console' }]"
        @click="activeView = 'console'"
      >
        Console
      </button>
      <button
        :class="['view-tab', { active: activeView === 'graphs' }]"
        @click="activeView = 'graphs'"
      >
        Graphs
      </button>
      <button
        :class="['view-tab', { active: activeView === 'players' || activeView === 'inventory' }]"
        @click="activeView = 'players'"
      >
        Players
      </button>
      <button
        :class="['view-tab', { active: activeView === 'config' }]"
        @click="activeView = 'config'"
      >
        Config
      </button>
      <button
        :class="['view-tab', { active: activeView === 'commands' }]"
        @click="activeView = 'commands'"
      >
        Commands
      </button>
      <button
        :class="['view-tab', { active: activeView === 'domains' }]"
        @click="activeView = 'domains'"
      >
        Domains
      </button>
    </div>

    <!-- Stats Strip -->
    <div class="stats-strip">
      <span :class="['status-dot', server.status]"></span>
      <span :class="['strip-status', server.status]">{{
        server.status.charAt(0).toUpperCase() + server.status.slice(1)
      }}</span>
      <span class="strip-sep"></span>
      <span class="strip-stat"
        ><span class="strip-label">CPU</span> {{ server.cpu.toFixed(1) }}%</span
      >
      <span class="strip-stat"
        ><span class="strip-label">Mem</span> {{ server.memory.toFixed(0) }} MB</span
      >
      <span class="strip-stat"
        ><span class="strip-label">TPS</span> {{ server.tps.toFixed(1) }}</span
      >
      <span class="strip-stat"
        ><span class="strip-label">Players</span> {{ server.players }}/{{ server.maxPlayers }}</span
      >
      <div class="controls">
        <button class="strip-btn" title="Clear logs" @click="emit('clear')">
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
            <path d="M3 6h18M8 6V4h8v2M5 6l1 14h12l1-14M10 11v6M14 11v6" />
          </svg>
        </button>
        <button
          v-if="server.status === 'stopped'"
          class="strip-btn power"
          title="Start server"
          @click="emit('start')"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" stroke="none">
            <path d="M8 5v14l11-7z" />
          </svg>
        </button>
        <button
          v-else
          class="strip-btn power active"
          title="Stop server"
          :disabled="server.status === 'starting'"
          @click="emit('stop')"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" stroke="none">
            <rect x="4" y="4" width="16" height="16" rx="2" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Views -->
    <ConsoleView
      v-show="activeView === 'console'"
      :server="server"
      @command="(cmd) => emit('command', cmd)"
    />

    <GraphsView v-show="activeView === 'graphs'" :server="server" />

    <PlayersView
      v-show="activeView === 'players'"
      :server="server"
      @command="(cmd) => emit('command', cmd)"
      @view-inventory="
        (name) => {
          selectedPlayer = name;
          activeView = 'inventory';
        }
      "
    />

    <PlayerInventoryView
      v-show="activeView === 'inventory'"
      :server="server"
      :player-name="selectedPlayer"
      @back="activeView = 'players'"
      @command="(cmd) => emit('command', cmd)"
    />

    <ConfigView
      v-show="activeView === 'config'"
      :server="server"
      @load-config-files="(dir) => emit('loadConfigFiles', dir)"
      @load-config-file="(dir, file) => emit('loadConfigFile', dir, file)"
      @save-config-file="(dir, file, content) => emit('saveConfigFile', dir, file, content)"
      @toggle-plugin="(file, enable) => emit('togglePlugin', file, enable)"
    />

    <CommandsView
      v-show="activeView === 'commands'"
      :server="server"
      @command="(cmd) => emit('command', cmd)"
      @update:saved-commands="(cmds) => emit('update:savedCommands', cmds)"
    />

    <DomainsView v-show="activeView === 'domains'" :server="server" />
  </div>
</template>

<style scoped>
.server-tab {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.stats-strip {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 7px 16px;
  background: var(--bg-base);
  border-bottom: 1px solid var(--border-subtle);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--text-faint);
  flex-shrink: 0;
}

.status-dot.running {
  background: var(--color-success);
  box-shadow: 0 0 6px rgba(74, 222, 128, 0.4);
  animation: pulse-glow 2s ease-in-out infinite;
}

.status-dot.starting {
  background: var(--color-warning);
  box-shadow: 0 0 6px rgba(251, 191, 36, 0.3);
  animation: pulse-glow 1s ease-in-out infinite;
}

.strip-status {
  font-weight: 600;
  color: var(--text-secondary);
}

.strip-status.running {
  color: var(--color-success);
}

.strip-status.starting {
  color: var(--color-warning);
}

.strip-status.stopped {
  color: var(--text-faint);
}

.strip-sep {
  width: 1px;
  height: 14px;
  background: var(--border-default);
}

.strip-stat {
  color: var(--text-secondary);
  font-weight: 500;
}

.strip-label {
  color: var(--text-faint);
  font-weight: 500;
  margin-right: 4px;
}

.controls {
  margin-left: auto;
  display: flex;
  gap: 4px;
}

.strip-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: none;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-tertiary);
  cursor: pointer;
  transition:
    color var(--transition-fast),
    border-color var(--transition-fast),
    background var(--transition-fast);
}

.strip-btn:hover {
  color: var(--text-primary);
  border-color: var(--border-strong);
  background: var(--bg-hover);
}

.strip-btn.power {
  color: var(--color-success);
  border-color: rgba(74, 222, 128, 0.2);
}

.strip-btn.power:hover {
  background: var(--color-success-muted);
  border-color: rgba(74, 222, 128, 0.35);
}

.strip-btn.power.active {
  color: var(--color-danger);
  border-color: rgba(239, 68, 68, 0.2);
}

.strip-btn.power.active:hover {
  background: var(--color-danger-muted);
  border-color: rgba(239, 68, 68, 0.35);
}

.strip-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.view-tabs {
  display: flex;
  background: var(--bg-overlay);
  border-bottom: 1px solid var(--border-default);
  padding: 6px 14px;
  gap: 4px;
  align-items: center;
}

.view-tab {
  padding: 7px 16px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  font-family: var(--font-ui);
  transition:
    color var(--transition-fast),
    background var(--transition-fast),
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
  position: relative;
}

.view-tab:hover {
  color: var(--text-secondary);
  background: rgba(255, 255, 255, 0.04);
}

.view-tab.active {
  color: var(--text-primary);
  background: var(--bg-surface);
  border-color: var(--border-default);
  box-shadow: var(--shadow-sm);
}

.server-tab > :deep(.console-view),
.server-tab > :deep(.graphs-view),
.server-tab > :deep(.players-view),
.server-tab > :deep(.inventory-view),
.server-tab > :deep(.config-view),
.server-tab > :deep(.commands-view),
.server-tab > :deep(.domains-view) {
  flex: 1;
  min-height: 0;
}
</style>
