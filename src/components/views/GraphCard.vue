<script setup lang="ts">
defineProps<{
  title: string;
  value: string;
  scaleLabel: string;
  history: number[];
  maxValue: number;
  color: string;
  gradientId: string;
}>();

function getPoints(history: number[], maxValue: number): string {
  if (!history || history.length === 0) return '';
  return history
    .map((v, i, arr) => `${(i / (arr.length - 1 || 1)) * 100},${40 - (v / maxValue) * 40}`)
    .join(' ');
}

function getFillPoints(history: number[], maxValue: number): string {
  if (!history || history.length <= 1) return '';
  return `0,40 ${getPoints(history, maxValue)} 100,40`;
}
</script>

<template>
  <div class="graph-container">
    <div class="graph-header">
      <span class="graph-title">{{ title }}</span>
      <span class="graph-value">{{ value }}</span>
    </div>
    <div class="graph">
      <span class="graph-scale">{{ scaleLabel }}</span>
      <svg viewBox="0 0 100 40" preserveAspectRatio="none">
        <line x1="0" y1="10" x2="100" y2="10" stroke="#333" stroke-width="0.1" />
        <line x1="0" y1="20" x2="100" y2="20" stroke="#333" stroke-width="0.1" />
        <line x1="0" y1="30" x2="100" y2="30" stroke="#333" stroke-width="0.1" />
        <polyline
          :points="getPoints(history, maxValue)"
          fill="none"
          :stroke="color"
          stroke-width="0.1"
        />
        <polygon
          v-if="(history || []).length > 1"
          :points="getFillPoints(history, maxValue)"
          :fill="`url(#${gradientId})`"
        />
        <defs>
          <linearGradient :id="gradientId" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" :stop-color="color" stop-opacity="0.3" />
            <stop offset="100%" :stop-color="color" stop-opacity="0" />
          </linearGradient>
        </defs>
      </svg>
    </div>
  </div>
</template>

<style scoped>
.graph-container {
  background: var(--bg-dark);
  border-radius: 8px;
  padding: 12px;
}

.graph-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.graph-title {
  color: var(--text-tertiary);
  font-size: 12px;
  text-transform: uppercase;
}

.graph-value {
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 500;
}

.graph {
  height: 80px;
  background: var(--bg-darkest);
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
  color: var(--text-dim);
}
</style>
