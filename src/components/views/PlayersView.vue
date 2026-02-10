<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getPlayerAvatarUrl } from '../../utils/minecraft';
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
      No players have joined this server yet
    </div>
    <div v-else-if="filteredPlayers.length === 0" class="players-empty">
      No players match "{{ playerSearch }}"
    </div>
    <div v-else :class="['player-list', playerViewMode]">
      <div v-for="player in filteredPlayers" :key="player" :class="['player-item', playerViewMode]">
        <div class="player-identity" @click="emit('viewInventory', player)">
          <img
            class="player-avatar"
            :src="getPlayerAvatarUrl(player, 36)"
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
  padding: 16px;
  background: var(--bg-darkest);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.players-toolbar {
  display: flex;
  gap: 12px;
  align-items: center;
}

.player-search {
  flex: 1;
  padding: 8px 12px;
  background: var(--bg-dark);
  border: 1px solid var(--bg-light);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 14px;
}

.view-toggle {
  display: flex;
  background: var(--bg-dark);
  border-radius: 6px;
  overflow: hidden;
}

.toggle-btn {
  padding: 8px 10px;
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.toggle-btn:hover {
  color: var(--text-tertiary);
}

.toggle-btn.active {
  background: var(--bg-light);
  color: var(--text-primary);
}

.players-empty {
  color: var(--text-faint);
  font-style: italic;
  text-align: center;
  padding: 40px;
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

.player-item.grid {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: var(--bg-dark);
  border-radius: 8px;
}

.player-item.list {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: var(--bg-dark);
  border-radius: 8px;
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
  border-radius: 6px;
  image-rendering: pixelated;
}

.player-name {
  color: var(--text-primary);
  font-size: 14px;
  text-align: center;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-dim);
  flex-shrink: 0;
}

.status-dot.online {
  background: var(--color-success);
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
  font-size: 12px;
  cursor: pointer;
  transition: opacity 0.2s;
}

.player-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.player-btn.kick {
  background: var(--color-warning);
  color: #000;
}

.player-btn.ban {
  background: var(--color-danger);
  color: var(--text-primary);
}

.player-btn.banip {
  background: var(--color-danger);
  color: var(--text-primary);
}

.player-btn:hover:not(:disabled) {
  opacity: 0.8;
}

.player-identity {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  transition: opacity 0.15s;
}

.player-identity:hover {
  opacity: 0.85;
}

.player-identity:hover .player-name {
  color: var(--color-primary);
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
