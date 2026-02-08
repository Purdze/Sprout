<script setup lang="ts">
import { ref } from 'vue';
import ConsoleView from './views/ConsoleView.vue';
import GraphsView from './views/GraphsView.vue';
import PlayersView from './views/PlayersView.vue';
import ConfigView from './views/ConfigView.vue';
import DomainsView from './views/DomainsView.vue';
import type { Server } from '../types';

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
}>();

const activeView = ref<'console' | 'graphs' | 'players' | 'config' | 'domains'>('console');
</script>

<template>
  <div class="server-tab">
    <!-- Stats Bar -->
    <div class="stats-bar">
      <div class="stat">
        <span class="stat-label">Status</span>
        <span :class="['stat-value', 'status', server.status]">
          {{ server.status.charAt(0).toUpperCase() + server.status.slice(1) }}
        </span>
      </div>
      <div class="stat">
        <span class="stat-label">CPU</span>
        <span class="stat-value">{{ server.cpu.toFixed(1) }}%</span>
      </div>
      <div class="stat">
        <span class="stat-label">Memory</span>
        <span class="stat-value">{{ server.memory.toFixed(0) }} MB</span>
      </div>
      <div class="stat">
        <span class="stat-label">TPS</span>
        <span class="stat-value">{{ server.tps.toFixed(1) }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">Players</span>
        <span class="stat-value">{{ server.players }}/{{ server.maxPlayers }}</span>
      </div>
      <div class="controls">
        <button class="btn clear" @click="emit('clear')">Clear</button>
        <button v-if="server.status === 'stopped'" class="btn start" @click="emit('start')">
          Start
        </button>
        <button
          v-else
          class="btn stop"
          :disabled="server.status === 'starting'"
          @click="emit('stop')"
        >
          Stop
        </button>
      </div>
    </div>

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
        :class="['view-tab', { active: activeView === 'players' }]"
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
        :class="['view-tab', { active: activeView === 'domains' }]"
        @click="activeView = 'domains'"
      >
        Domains
      </button>
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
    />

    <ConfigView
      v-show="activeView === 'config'"
      :server="server"
      @load-config-files="(dir) => emit('loadConfigFiles', dir)"
      @load-config-file="(dir, file) => emit('loadConfigFile', dir, file)"
      @save-config-file="(dir, file, content) => emit('saveConfigFile', dir, file, content)"
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

.stats-bar {
  display: flex;
  align-items: center;
  gap: 24px;
  padding: 12px 16px;
  background: #252525;
  border-bottom: 1px solid #333;
}

.stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.stat-label {
  font-size: 11px;
  color: #666;
  text-transform: uppercase;
}

.stat-value {
  font-size: 14px;
  color: #fff;
  font-weight: 500;
}

.stat-value.status.running {
  color: #4ade80;
}

.stat-value.status.starting {
  color: #fbbf24;
}

.stat-value.status.stopped {
  color: #666;
}

.controls {
  margin-left: auto;
  display: flex;
  gap: 8px;
}

.btn {
  padding: 8px 20px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.btn.start {
  background: #4ade80;
  color: #000;
}

.btn.stop {
  background: #ef4444;
  color: #fff;
}

.btn.clear {
  background: #333;
  color: #fff;
}

.btn.clear:hover {
  background: #444;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.view-tabs {
  display: flex;
  background: #252525;
  border-bottom: 1px solid #333;
  padding: 0 12px;
}

.view-tab {
  padding: 10px 16px;
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  font-size: 13px;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}

.view-tab:hover {
  color: #ccc;
}

.view-tab.active {
  color: #f97316;
  border-bottom-color: #f97316;
}

.server-tab > :deep(.console-view),
.server-tab > :deep(.graphs-view),
.server-tab > :deep(.players-view),
.server-tab > :deep(.config-view),
.server-tab > :deep(.domains-view) {
  flex: 1;
  min-height: 0;
}
</style>
