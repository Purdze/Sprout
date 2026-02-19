<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import InventoryCell from './InventoryCell.vue';
import { MINECRAFT_ASSET_BASE, getPlayerAvatarUrl, formatDimension } from '../../utils/minecraft';
import type { Server, PlayerDetails } from '../../types';

const props = defineProps<{
  server: Server;
  playerName: string;
}>();

const emit = defineEmits<{
  back: [];
  command: [cmd: string];
}>();

const serverId = computed(() => props.server.id);
const serverPath = computed(() => props.server.path);
const serverStatus = computed(() => props.server.status);
const isOnline = computed(() => props.server.playerList?.includes(props.playerName) ?? false);

const loading = ref(false);
const error = ref('');
const player = ref<PlayerDetails | null>(null);
const activeTab = ref<'inventory' | 'enderchest'>('inventory');
const lastRefreshed = ref('');

function updateLastRefreshed() {
  lastRefreshed.value = new Date().toLocaleTimeString('en-US', {
    hour: 'numeric',
    minute: '2-digit',
    second: '2-digit',
    hour12: true,
  });
}

async function fetchInventory() {
  const result = await invoke<PlayerDetails>('get_player_inventory', {
    id: serverId.value,
    path: serverPath.value,
    playerName: props.playerName,
  });
  player.value = result;
  updateLastRefreshed();
}

async function refresh() {
  try {
    await fetchInventory();
  } catch (e) {
    error.value = String(e);
  }
}

function delayedRefresh() {
  window.setTimeout(refresh, 500);
}

async function loadPlayer() {
  if (!props.playerName) return;
  loading.value = true;
  error.value = '';
  try {
    await fetchInventory();
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(loadPlayer);

watch(
  () => props.playerName,
  async (newName, oldName) => {
    if (newName && newName !== oldName) {
      await loadPlayer();
    }
  }
);

function getSlotItem(slotNum: number) {
  if (!player.value) return null;
  return player.value.inventory.find((s) => s.slot === slotNum) || null;
}

const armorSlots = [103, 102, 101, 100];
const mainSlots = computed(() => Array.from({ length: 27 }, (_, i) => i + 9));
const hotbar = computed(() => Array.from({ length: 9 }, (_, i) => i));
const enderSlots = computed(() => Array.from({ length: 27 }, (_, i) => i));

function getEnderSlotItem(slotNum: number) {
  if (!player.value) return null;
  return player.value.enderChest.find((s) => s.slot === slotNum) || null;
}

function formatPosition(x: number, y: number, z: number): string {
  return `${Math.floor(x)}, ${Math.floor(y)}, ${Math.floor(z)}`;
}
</script>

<template>
  <div class="inventory-view">
    <div class="panel-header">
      <button class="back-btn" title="Back to players" @click="emit('back')">
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M19 12H5" />
          <path d="M12 19l-7-7 7-7" />
        </svg>
      </button>
      <img class="player-head" :src="getPlayerAvatarUrl(playerName, 56)" :alt="playerName" />
      <div class="header-info">
        <h2 class="player-title">
          {{ playerName }}
          <span :class="['online-badge', { online: isOnline }]">{{
            isOnline ? 'Online' : 'Offline'
          }}</span>
        </h2>
        <div v-if="player" class="player-stats">
          <span class="pstat health"
            ><img
              class="pstat-mc-icon"
              :src="`${MINECRAFT_ASSET_BASE}/gui/sprites/hud/heart/full.png`"
              alt="Health"
            />
            {{ player.health.toFixed(0) }}/{{ player.maxHealth.toFixed(0) }}</span
          >
          <span class="pstat food"
            ><img
              class="pstat-mc-icon"
              :src="`${MINECRAFT_ASSET_BASE}/gui/sprites/hud/food_full.png`"
              alt="Food"
            />
            {{ player.food }}/20</span
          >
          <span class="pstat xp"
            ><img
              class="pstat-mc-icon"
              :src="`${MINECRAFT_ASSET_BASE}/item/experience_bottle.png`"
              alt="XP"
            />
            Lvl {{ player.xpLevel }}</span
          >
        </div>
        <div v-if="player" class="header-actions">
          <button
            class="act-btn neutral"
            :disabled="serverStatus !== 'running'"
            @click="emit('command', `whitelist add ${playerName}`)"
          >
            Whitelist
          </button>
          <button
            v-if="!player.isOp"
            class="act-btn neutral"
            :disabled="serverStatus !== 'running'"
            @click="
              emit('command', `op ${playerName}`);
              delayedRefresh();
            "
          >
            Op
          </button>
          <button
            v-else
            class="act-btn neutral"
            :disabled="serverStatus !== 'running'"
            @click="
              emit('command', `deop ${playerName}`);
              delayedRefresh();
            "
          >
            Deop
          </button>
          <button
            class="act-btn warn"
            :disabled="serverStatus !== 'running' || !isOnline"
            @click="
              emit('command', `kick ${playerName}`);
              emit('back');
            "
          >
            Kick
          </button>
          <button
            class="act-btn danger"
            :disabled="serverStatus !== 'running'"
            @click="emit('command', `ban ${playerName}`)"
          >
            Ban
          </button>
          <button
            class="act-btn danger"
            :disabled="serverStatus !== 'running'"
            @click="emit('command', `ban-ip ${playerName}`)"
          >
            Ban IP
          </button>
        </div>
      </div>
      <span v-if="lastRefreshed" class="last-refreshed">{{ lastRefreshed }}</span>
      <button class="refresh-btn" :disabled="loading" title="Refresh inventory" @click="refresh">
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M1 4v6h6" />
          <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
        </svg>
      </button>
    </div>

    <div v-if="loading" class="panel-body state-msg">Loading inventory...</div>
    <div v-else-if="error" class="panel-body state-msg err">{{ error }}</div>

    <div v-else-if="player" class="panel-body">
      <div class="tab-bar">
        <button
          :class="['tab-btn', { active: activeTab === 'inventory' }]"
          @click="activeTab = 'inventory'"
        >
          Inventory
        </button>
        <button
          :class="['tab-btn', { active: activeTab === 'enderchest' }]"
          @click="activeTab = 'enderchest'"
        >
          Ender Chest
        </button>
      </div>

      <div class="stats-grid">
        <div class="stat-item disabled" title="Not yet implemented by Pumpkin">
          <span class="stat-label">Playtime</span>
          <span class="stat-value muted">Not implemented</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">World</span>
          <span class="stat-value">{{ formatDimension(player.dimension) }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">Position</span>
          <span class="stat-value">{{
            formatPosition(player.posX, player.posY, player.posZ)
          }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">Gamemode</span>
          <span class="stat-value">{{ player.gameMode }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">Last Slept</span>
          <span class="stat-value">{{ player.lastSlept ?? 'None' }}</span>
        </div>
        <div class="stat-item disabled" title="Not yet implemented by Pumpkin">
          <span class="stat-label">Last Death</span>
          <span class="stat-value muted">Not implemented</span>
        </div>
        <div class="stat-item disabled" title="Not yet implemented by Pumpkin">
          <span class="stat-label">KDR</span>
          <span class="stat-value muted">Not implemented</span>
        </div>
        <div class="stat-item disabled" title="Not yet implemented by Pumpkin">
          <span class="stat-label">Deaths</span>
          <span class="stat-value muted">Not implemented</span>
        </div>
        <div class="stat-item disabled" title="Not yet implemented by Pumpkin">
          <span class="stat-label">Items Picked Up</span>
          <span class="stat-value muted">Not implemented</span>
        </div>
        <div class="stat-item disabled" title="Not yet implemented by Pumpkin">
          <span class="stat-label">Items Used</span>
          <span class="stat-value muted">Not implemented</span>
        </div>
        <div class="stat-item disabled" title="Not yet implemented by Pumpkin">
          <span class="stat-label">Mobs Killed</span>
          <span class="stat-value muted">Not implemented</span>
        </div>
        <div class="stat-item disabled" title="Not yet implemented by Pumpkin">
          <span class="stat-label">Distance</span>
          <span class="stat-value muted">Not implemented</span>
        </div>
      </div>

      <template v-if="activeTab === 'inventory'">
        <div class="inv-row">
          <div class="inv-main">
            <div class="section-label">Inventory</div>
            <div class="inv-grid">
              <InventoryCell v-for="s in mainSlots" :key="s" :item="getSlotItem(s)" />
            </div>
          </div>
          <div class="inv-equip">
            <div class="section-label">Armor</div>
            <div class="equip-cells">
              <InventoryCell
                v-for="slot in armorSlots"
                :key="slot"
                :item="getSlotItem(slot)"
                variant="equip"
              />
            </div>
          </div>
        </div>

        <div class="inv-row">
          <div class="inv-main">
            <div class="section-label">Hotbar</div>
            <div class="inv-grid">
              <InventoryCell v-for="s in hotbar" :key="s" :item="getSlotItem(s)" />
            </div>
          </div>
          <div class="inv-equip">
            <div class="section-label">Offhand</div>
            <div class="equip-cells">
              <InventoryCell :item="getSlotItem(-106)" variant="equip" />
            </div>
          </div>
        </div>
      </template>

      <template v-if="activeTab === 'enderchest'">
        <div class="section-label">Ender Chest</div>
        <div class="inv-grid">
          <InventoryCell v-for="s in enderSlots" :key="s" :item="getEnderSlotItem(s)" />
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.inventory-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--bg-darkest);
  overflow-y: auto;
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 18px 20px;
  background: linear-gradient(135deg, var(--bg-dark) 0%, #222 100%);
  border-bottom: 1px solid #2a2a2a;
}

.back-btn {
  width: 32px;
  height: 32px;
  background: none;
  border: 1px solid var(--bg-light);
  border-radius: 6px;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.15s;
}

.back-btn:hover {
  color: var(--text-primary);
  border-color: var(--text-dim);
  background: #2a2a2a;
}

.player-head {
  width: 56px;
  height: 56px;
  border-radius: 6px;
  image-rendering: pixelated;
  border: 2px solid var(--bg-light);
  flex-shrink: 0;
}

.header-info {
  flex: 1;
  min-width: 0;
}

.player-title {
  margin: 0;
  font-size: 20px;
  color: var(--text-primary);
  font-weight: 600;
  letter-spacing: -0.02em;
  display: flex;
  align-items: center;
  gap: 10px;
}

.online-badge {
  font-size: 11px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: 4px;
  background: var(--bg-light);
  color: var(--text-muted);
}

.online-badge.online {
  background: rgba(74, 222, 128, 0.15);
  color: var(--color-success);
}

.player-stats {
  display: flex;
  gap: 14px;
  margin-top: 6px;
  flex-wrap: wrap;
}

.pstat {
  font-size: 13px;
  color: #777;
  display: flex;
  align-items: center;
  gap: 4px;
}

.pstat-mc-icon {
  width: 14px;
  height: 14px;
  image-rendering: pixelated;
  object-fit: contain;
}

.pstat.mode {
  color: var(--text-dim);
  font-style: italic;
}

.last-refreshed {
  font-size: 11px;
  color: var(--text-faint);
  white-space: nowrap;
}

.refresh-btn {
  width: 32px;
  height: 32px;
  background: none;
  border: 1px solid var(--bg-light);
  border-radius: 6px;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.15s;
}

.refresh-btn:hover:not(:disabled) {
  color: var(--color-primary);
  border-color: var(--color-primary);
  background: rgba(249, 115, 22, 0.08);
}

.refresh-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.panel-body {
  padding: 18px 20px;
}

.state-msg {
  color: var(--text-dim);
  font-style: italic;
  text-align: center;
  padding: 40px 20px;
}

.state-msg.err {
  color: var(--color-danger);
}

.tab-bar {
  display: flex;
  gap: 4px;
  margin-bottom: 14px;
  border-bottom: 1px solid #2a2a2a;
  padding-bottom: 10px;
}

.tab-btn {
  padding: 6px 16px;
  background: none;
  border: 1px solid #2a2a2a;
  border-radius: 6px;
  color: var(--text-muted);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.tab-btn:hover {
  color: #aaa;
  border-color: var(--bg-hover);
}

.tab-btn.active {
  background: var(--bg-dark);
  color: var(--color-primary);
  border-color: var(--color-primary);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 6px;
  margin-bottom: 14px;
}

.stat-item {
  background: #141414;
  border: 1px solid #222;
  border-radius: 6px;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.stat-label {
  font-size: 10px;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.stat-value {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

.stat-item.disabled {
  opacity: 0.4;
}

.stat-value.muted {
  font-size: 11px;
  font-style: italic;
  color: var(--text-dim);
}

.inv-row {
  display: grid;
  grid-template-columns: repeat(10, 1fr);
  gap: 3px;
  margin-bottom: 8px;
}

.inv-main {
  grid-column: span 9;
  display: flex;
  flex-direction: column;
}

.section-label {
  font-size: 11px;
  color: var(--text-dim);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 6px;
}

.inv-grid {
  display: grid;
  grid-template-columns: repeat(9, 1fr);
  gap: 3px;
}

.inv-equip {
  display: flex;
  flex-direction: column;
  margin-left: 6px;
}

.equip-cells {
  display: flex;
  flex-direction: column;
  gap: 3px;
  flex: 1;
}

.header-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 6px;
}

.act-btn {
  padding: 4px 10px;
  border: 1px solid var(--bg-light);
  border-radius: 4px;
  font-size: 11px;
  cursor: pointer;
  font-weight: 500;
  background: none;
  transition: all 0.15s;
}

.act-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.act-btn:hover:not(:disabled) {
  opacity: 0.85;
}

.act-btn.neutral {
  color: #999;
  border-color: var(--bg-light);
}

.act-btn.neutral:hover:not(:disabled) {
  color: var(--text-primary);
  border-color: var(--text-dim);
  background: #2a2a2a;
}

.act-btn.warn {
  color: var(--color-warning);
  border-color: #f59e0b44;
}

.act-btn.warn:hover:not(:disabled) {
  background: rgba(245, 158, 11, 0.1);
}

.act-btn.danger {
  color: var(--color-danger);
  border-color: #ef444444;
}

.act-btn.danger:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.1);
}
</style>
