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
    <div
      v-if="server.status === 'stopped' && (!server.cpuHistory || server.cpuHistory.length === 0)"
      class="graphs-empty"
    >
      <svg
        width="36"
        height="36"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
      </svg>
      <span>Start the server to see performance metrics</span>
    </div>
    <template v-else>
      <div class="graph-container">
        <div class="graph-header">
          <span class="graph-title">CPU Usage</span>
          <span class="graph-value cpu">{{ server.cpu.toFixed(1) }}%</span>
        </div>
        <div class="graph">
          <span class="graph-scale">{{ maxCpu.toFixed(0) }}%</span>
          <svg viewBox="0 0 100 40" preserveAspectRatio="none">
            <line x1="0" y1="10" x2="100" y2="10" stroke="#22222e" stroke-width="0.1" />
            <line x1="0" y1="20" x2="100" y2="20" stroke="#22222e" stroke-width="0.1" />
            <line x1="0" y1="30" x2="100" y2="30" stroke="#22222e" stroke-width="0.1" />
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
          <span class="graph-value mem">{{ server.memory.toFixed(0) }} MB</span>
        </div>
        <div class="graph">
          <span class="graph-scale">{{ maxMemory.toFixed(0) }} MB</span>
          <svg viewBox="0 0 100 40" preserveAspectRatio="none">
            <line x1="0" y1="10" x2="100" y2="10" stroke="#22222e" stroke-width="0.1" />
            <line x1="0" y1="20" x2="100" y2="20" stroke="#22222e" stroke-width="0.1" />
            <line x1="0" y1="30" x2="100" y2="30" stroke="#22222e" stroke-width="0.1" />
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
          <span class="graph-value tps">{{ server.tps.toFixed(1) }}</span>
        </div>
        <div class="graph">
          <span class="graph-scale">20</span>
          <svg viewBox="0 0 100 40" preserveAspectRatio="none">
            <line x1="0" y1="10" x2="100" y2="10" stroke="#22222e" stroke-width="0.1" />
            <line x1="0" y1="20" x2="100" y2="20" stroke="#22222e" stroke-width="0.1" />
            <line x1="0" y1="30" x2="100" y2="30" stroke="#22222e" stroke-width="0.1" />
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
    </template>
  </div>
</template>

<style scoped>
.graphs-view {
  flex: 1;
  overflow-y: auto;
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  background: var(--bg-base);
}

.graph-container {
  background: var(--bg-raised);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: 16px;
  transition: border-color var(--transition-normal);
}

.graph-container:hover {
  border-color: var(--border-default);
}

.graph-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 10px;
}

.graph-title {
  color: var(--text-tertiary);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-weight: 600;
}

.graphs-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--text-faint);
  font-style: italic;
  opacity: 0.5;
}

.graph-value {
  font-size: 15px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.graph-value.cpu {
  color: var(--accent);
}

.graph-value.mem {
  color: var(--color-success);
}

.graph-value.tps {
  color: var(--color-info);
}

.graph {
  height: 88px;
  background: var(--bg-base);
  border-radius: var(--radius-sm);
  overflow: hidden;
  position: relative;
  border: 1px solid var(--border-subtle);
}

.graph svg {
  width: 100%;
  height: 100%;
}

.graph-scale {
  position: absolute;
  top: 6px;
  right: 10px;
  font-size: 10px;
  color: var(--text-faint);
  font-family: var(--font-mono);
  font-weight: 500;
}
</style>
