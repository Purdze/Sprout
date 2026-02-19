<script setup lang="ts">
import { computed } from 'vue';
import GraphCard from './GraphCard.vue';
import type { Server } from '../../types';

const props = defineProps<{
  server: Server;
}>();

const maxCpu = computed(() => {
  const history = props.server.cpuHistory || [];
  if (history.length === 0) return 10;
  const max = Math.max(...history);
  return Math.max(max * 1.2, 5);
});

const maxMemory = computed(() => {
  const history = props.server.memoryHistory || [];
  if (history.length === 0) return 100;
  const max = Math.max(...history);
  return Math.max(max * 1.2, 50);
});

const maxTps = computed(() => 20);
</script>

<template>
  <div class="graphs-view">
    <GraphCard
      title="CPU Usage"
      :value="`${server.cpu.toFixed(1)}%`"
      :scale-label="`${maxCpu.toFixed(0)}%`"
      :history="server.cpuHistory"
      :max-value="maxCpu"
      color="#f97316"
      gradient-id="cpuGradient"
    />

    <GraphCard
      title="Memory"
      :value="`${server.memory.toFixed(0)} MB`"
      :scale-label="`${maxMemory.toFixed(0)} MB`"
      :history="server.memoryHistory"
      :max-value="maxMemory"
      color="#4ade80"
      gradient-id="memGradient"
    />

    <GraphCard
      title="TPS"
      :value="server.tps.toFixed(1)"
      scale-label="20"
      :history="server.tpsHistory"
      :max-value="maxTps"
      color="#60a5fa"
      gradient-id="tpsGradient"
    />
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
  background: var(--bg-darkest);
}
</style>
