<script setup lang="ts">
import { computed } from 'vue';
import type { Server } from '../../types';

const props = defineProps<{
  server: Server;
}>();

// Dynamic scaling - use actual max with 20% headroom, minimum floor for visibility
const maxCpu = computed(() => {
  const history = props.server.cpuHistory || [];
  if (history.length === 0) return 10;
  const max = Math.max(...history);
  return Math.max(max * 1.2, 5); // At least 5% scale for visibility
});

const maxMemory = computed(() => {
  const history = props.server.memoryHistory || [];
  if (history.length === 0) return 100;
  const max = Math.max(...history);
  return Math.max(max * 1.2, 50); // At least 50MB scale
});

const maxTps = computed(() => 20); // TPS is always 0-20

function getPoints(history: number[], maxValue: number): string {
  if (!history || history.length === 0) return '';
  return history
    .map((v, i, arr) => `${(i / (arr.length - 1 || 1)) * 100},${40 - (v / maxValue) * 40}`)
    .join(' ');
}

function getFillPoints(history: number[], maxValue: number): string {
  if (!history || history.length <= 1) return '';
  const linePoints = history
    .map((v, i, arr) => `${(i / (arr.length - 1 || 1)) * 100},${40 - (v / maxValue) * 40}`)
    .join(' ');
  return `0,40 ${linePoints} 100,40`;
}
</script>

<template>
  <div class="graphs-view">
    <div class="graph-container">
      <div class="graph-header">
        <span class="graph-title">CPU Usage</span>
        <span class="graph-value">{{ server.cpu.toFixed(1) }}%</span>
      </div>
      <div class="graph">
        <span class="graph-scale">{{ maxCpu.toFixed(0) }}%</span>
        <svg viewBox="0 0 100 40" preserveAspectRatio="none">
          <line x1="0" y1="10" x2="100" y2="10" stroke="#333" stroke-width="0.1" />
          <line x1="0" y1="20" x2="100" y2="20" stroke="#333" stroke-width="0.1" />
          <line x1="0" y1="30" x2="100" y2="30" stroke="#333" stroke-width="0.1" />
          <polyline
            :points="getPoints(server.cpuHistory, maxCpu)"
            fill="none"
            stroke="#f97316"
            stroke-width="0.1"
          />
          <polygon
            v-if="(server.cpuHistory || []).length > 1"
            :points="getFillPoints(server.cpuHistory, maxCpu)"
            fill="url(#cpuGradient)"
          />
          <defs>
            <linearGradient id="cpuGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#f97316" stop-opacity="0.3" />
              <stop offset="100%" stop-color="#f97316" stop-opacity="0" />
            </linearGradient>
          </defs>
        </svg>
      </div>
    </div>

    <div class="graph-container">
      <div class="graph-header">
        <span class="graph-title">Memory</span>
        <span class="graph-value">{{ server.memory.toFixed(0) }} MB</span>
      </div>
      <div class="graph">
        <span class="graph-scale">{{ maxMemory.toFixed(0) }} MB</span>
        <svg viewBox="0 0 100 40" preserveAspectRatio="none">
          <line x1="0" y1="10" x2="100" y2="10" stroke="#333" stroke-width="0.1" />
          <line x1="0" y1="20" x2="100" y2="20" stroke="#333" stroke-width="0.1" />
          <line x1="0" y1="30" x2="100" y2="30" stroke="#333" stroke-width="0.1" />
          <polyline
            :points="getPoints(server.memoryHistory, maxMemory)"
            fill="none"
            stroke="#4ade80"
            stroke-width="0.1"
          />
          <polygon
            v-if="(server.memoryHistory || []).length > 1"
            :points="getFillPoints(server.memoryHistory, maxMemory)"
            fill="url(#memGradient)"
          />
          <defs>
            <linearGradient id="memGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#4ade80" stop-opacity="0.3" />
              <stop offset="100%" stop-color="#4ade80" stop-opacity="0" />
            </linearGradient>
          </defs>
        </svg>
      </div>
    </div>

    <div class="graph-container">
      <div class="graph-header">
        <span class="graph-title">TPS</span>
        <span class="graph-value">{{ server.tps.toFixed(1) }}</span>
      </div>
      <div class="graph">
        <span class="graph-scale">20</span>
        <svg viewBox="0 0 100 40" preserveAspectRatio="none">
          <line x1="0" y1="10" x2="100" y2="10" stroke="#333" stroke-width="0.1" />
          <line x1="0" y1="20" x2="100" y2="20" stroke="#333" stroke-width="0.1" />
          <line x1="0" y1="30" x2="100" y2="30" stroke="#333" stroke-width="0.1" />
          <polyline
            :points="getPoints(server.tpsHistory, maxTps)"
            fill="none"
            stroke="#60a5fa"
            stroke-width="0.1"
          />
          <polygon
            v-if="(server.tpsHistory || []).length > 1"
            :points="getFillPoints(server.tpsHistory, maxTps)"
            fill="url(#tpsGradient)"
          />
          <defs>
            <linearGradient id="tpsGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#60a5fa" stop-opacity="0.3" />
              <stop offset="100%" stop-color="#60a5fa" stop-opacity="0" />
            </linearGradient>
          </defs>
        </svg>
      </div>
    </div>
  </div>
</template>

<style scoped>
.graphs-view {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  background: #0d0d0d;
}

.graph-container {
  background: #1a1a1a;
  border-radius: 8px;
  padding: 12px;
}

.graph-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.graph-title {
  color: #888;
  font-size: 12px;
  text-transform: uppercase;
}

.graph-value {
  color: #fff;
  font-size: 14px;
  font-weight: 500;
}

.graph {
  height: 80px;
  background: #0d0d0d;
  border-radius: 4px;
  overflow: hidden;
  position: relative;
}

.graph svg {
  width: 100%;
  height: 100%;
}

.graph-scale {
  position: absolute;
  top: 4px;
  right: 8px;
  font-size: 10px;
  color: #555;
}
</style>
