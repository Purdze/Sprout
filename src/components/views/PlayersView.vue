<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Server } from '../../types';

const props = defineProps<{
  server: Server;
}>();

const emit = defineEmits<{
  command: [cmd: string];
  viewInventory: [playerName: string];
}>();

const playerSearch = ref('');
const playerViewMode = ref<'grid' | 'list'>('grid');
const knownPlayers = ref<string[]>([]);

async function loadKnownPlayers() {
  try {
    knownPlayers.value = await invoke<string[]>('get_known_players', { id: props.server.id });
  } catch {
    knownPlayers.value = [];
  }
}

onMounted(loadKnownPlayers);
watch(() => props.server.id, loadKnownPlayers);

watch(
  () => props.server.playerList,
  async (list) => {
    if (list && list.length > 0) {
      try {
        await invoke('update_known_players', { id: props.server.id, online: list });
        await loadKnownPlayers();
      } catch {
        // ignore
      }
    }
  },
  { deep: true }
);

const onlinePlayers = computed(() => new Set(props.server.playerList || []));

const allPlayers = computed(() => {
  const online = props.server.playerList || [];
  const offlineOnly = knownPlayers.value.filter((p) => !onlinePlayers.value.has(p));
  return [...online, ...offlineOnly];
});

const filteredPlayers = computed(() => {
  const list = allPlayers.value;
  if (!playerSearch.value) return list;
  return list.filter((p) => p.toLowerCase().includes(playerSearch.value.toLowerCase()));
});
</script>

<template>
  <div class="players-view">
    <div class="players-toolbar">
      <div class="player-count-badge">
        <span class="count-online">{{ onlinePlayers.size }}</span>
        <span class="count-sep">/</span>
        <span class="count-total">{{ allPlayers.length }}</span>
      </div>
      <input v-model="playerSearch" class="player-search" placeholder="Search players..." />
      <div class="view-toggle">
        <button
          :class="['toggle-btn', { active: playerViewMode === 'grid' }]"
          title="Grid view"
          @click="playerViewMode = 'grid'"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <rect x="1" y="1" width="6" height="6" rx="1" />
            <rect x="9" y="1" width="6" height="6" rx="1" />
            <rect x="1" y="9" width="6" height="6" rx="1" />
            <rect x="9" y="9" width="6" height="6" rx="1" />
          </svg>
        </button>
        <button
          :class="['toggle-btn', { active: playerViewMode === 'list' }]"
          title="List view"
          @click="playerViewMode = 'list'"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
            <rect x="1" y="2" width="14" height="3" rx="1" />
            <rect x="1" y="7" width="14" height="3" rx="1" />
            <rect x="1" y="12" width="14" height="3" rx="1" />
          </svg>
        </button>
      </div>
    </div>

    <div v-if="allPlayers.length === 0" class="players-empty">
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
        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
        <circle cx="9" cy="7" r="4" />
        <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
        <path d="M16 3.13a4 4 0 0 1 0 7.75" />
      </svg>
      <span>No players have joined this server yet</span>
    </div>
    <div v-else-if="filteredPlayers.length === 0" class="players-empty">
      <span>No players match "{{ playerSearch }}"</span>
    </div>
    <div v-else :class="['player-list', playerViewMode]">
      <div v-for="player in filteredPlayers" :key="player" :class="['player-item', playerViewMode]">
        <div class="player-identity" @click="emit('viewInventory', player)">
          <img
            class="player-avatar"
            :src="`https://mc-heads.net/avatar/${player}/36`"
            :alt="player"
          />
          <span class="player-name">{{ player }}</span>
          <span :class="['status-dot', { online: onlinePlayers.has(player) }]" />
        </div>
        <div class="player-actions">
          <button
            class="player-btn kick"
            :disabled="server.status !== 'running' || !onlinePlayers.has(player)"
            @click="emit('command', `kick ${player}`)"
          >
            Kick
          </button>
          <button
            class="player-btn ban"
            :disabled="server.status !== 'running' || !onlinePlayers.has(player)"
            @click="emit('command', `ban ${player}`)"
          >
            Ban
          </button>
          <button
            class="player-btn banip"
            :disabled="server.status !== 'running' || !onlinePlayers.has(player)"
            @click="emit('command', `ban-ip ${player}`)"
          >
            Ban IP
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.players-view {
  flex: 1;
  overflow-y: auto;
  padding: 18px;
  background: var(--bg-base);
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.players-toolbar {
  display: flex;
  gap: 10px;
  align-items: center;
}

.player-count-badge {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 6px 12px;
  background: var(--bg-raised);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}

.count-online {
  color: var(--color-success);
}

.count-sep {
  color: var(--text-faint);
}

.count-total {
  color: var(--text-secondary);
}

.player-search {
  flex: 1;
  padding: 9px 14px;
  background: var(--bg-raised);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 13px;
  font-family: var(--font-ui);
  transition: border-color var(--transition-fast);
}

.player-search:focus {
  border-color: var(--accent);
}

.view-toggle {
  display: flex;
  background: var(--bg-raised);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.toggle-btn {
  padding: 8px 10px;
  background: none;
  border: none;
  color: var(--text-faint);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition:
    color var(--transition-fast),
    background var(--transition-fast);
}

.toggle-btn:hover {
  color: var(--text-secondary);
}

.toggle-btn.active {
  background: var(--bg-active);
  color: var(--text-primary);
}

.players-empty {
  color: var(--text-faint);
  font-style: italic;
  text-align: center;
  padding: 48px 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  opacity: 0.5;
}

.player-list.grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 8px;
}

.player-list.list {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}

.player-item {
  display: flex;
  align-items: center;
  background: var(--bg-raised);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  transition:
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.player-item:hover {
  border-color: var(--border-default);
  box-shadow: var(--shadow-sm);
}

.player-item.grid {
  flex-direction: column;
  gap: 10px;
  padding: 14px;
}

.player-item.list {
  flex-direction: row;
  gap: 12px;
  padding: 10px 14px;
}

.player-item.list .player-name {
  flex: 1;
  text-align: left;
}

.player-item.list .player-actions {
  flex-wrap: nowrap;
}

.player-avatar {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-sm);
  image-rendering: pixelated;
  border: 1px solid var(--border-subtle);
}

.player-name {
  color: var(--text-primary);
  font-size: 13px;
  font-weight: 500;
  text-align: center;
  transition: color var(--transition-fast);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-faint);
  flex-shrink: 0;
}

.status-dot.online {
  background: var(--color-success);
  box-shadow: 0 0 6px rgba(74, 222, 128, 0.35);
}

.player-actions {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
  justify-content: center;
}

.player-btn {
  padding: 4px 10px;
  border: none;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  font-family: var(--font-ui);
  cursor: pointer;
  transition:
    opacity var(--transition-fast),
    transform var(--transition-fast);
}

.player-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.player-btn.kick {
  background: var(--color-warning);
  color: #0a0a0f;
}

.player-btn.ban,
.player-btn.banip {
  background: var(--color-danger);
  color: #fff;
}

.player-btn:hover:not(:disabled) {
  opacity: 0.85;
  transform: translateY(-1px);
}

.player-identity {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  transition: opacity var(--transition-fast);
}

.player-identity:hover {
  opacity: 0.9;
}

.player-identity:hover .player-name {
  color: var(--accent);
}

.player-item.grid .player-identity {
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.player-item.list .player-identity {
  flex: 1;
}
</style>
