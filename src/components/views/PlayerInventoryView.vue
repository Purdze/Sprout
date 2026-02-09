<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
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

function formatItemName(id: string): string {
  return id
    .replace('minecraft:', '')
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

const ASSET_BASE =
  'https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures';

function itemIconUrl(id: string): string {
  const name = id.replace('minecraft:', '');
  return `${ASSET_BASE}/item/${name}.png`;
}

function onIconError(e: Event) {
  const img = e.target as HTMLImageElement;
  const src = img.src;
  if (src.includes('/item/')) {
    // try block texture
    img.src = src.replace('/item/', '/block/');
  } else {
    // both failed, hide image
    img.style.display = 'none';
    const fallback = img.nextElementSibling as HTMLElement | null;
    if (fallback) fallback.style.display = '';
  }
}

const armorSlots = [103, 102, 101, 100];
const mainSlots = computed(() => Array.from({ length: 27 }, (_, i) => i + 9));
const hotbar = computed(() => Array.from({ length: 9 }, (_, i) => i));
const enderSlots = computed(() => Array.from({ length: 27 }, (_, i) => i));

function getEnderSlotItem(slotNum: number) {
  if (!player.value) return null;
  return player.value.enderChest.find((s) => s.slot === slotNum) || null;
}

function formatDimension(dim: string): string {
  const name = dim.replace('minecraft:', '');
  switch (name) {
    case 'overworld':
      return 'Overworld';
    case 'the_nether':
      return 'The Nether';
    case 'the_end':
      return 'The End';
    default:
      return formatItemName(dim);
  }
}

function formatPosition(x: number, y: number, z: number): string {
  return `${Math.floor(x)}, ${Math.floor(y)}, ${Math.floor(z)}`;
}
</script>

<template>
  <div class="inventory-view">
    <!-- Header -->
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
      <img
        class="player-head"
        :src="`https://mc-heads.net/avatar/${playerName}/56`"
        :alt="playerName"
      />
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
              src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/gui/sprites/hud/heart/full.png"
              alt="Health"
            />
            {{ player.health.toFixed(0) }}/{{ player.maxHealth.toFixed(0) }}</span
          >
          <span class="pstat food"
            ><img
              class="pstat-mc-icon"
              src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/gui/sprites/hud/food_full.png"
              alt="Food"
            />
            {{ player.food }}/20</span
          >
          <span class="pstat xp"
            ><img
              class="pstat-mc-icon"
              src="https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures/item/experience_bottle.png"
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

    <!-- Loading / Error -->
    <div v-if="loading" class="panel-body state-msg">Loading inventory...</div>
    <div v-else-if="error" class="panel-body state-msg err">{{ error }}</div>

    <!-- Inventory -->
    <div v-else-if="player" class="panel-body">
      <!-- Tab bar -->
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

      <!-- Stats grid -->
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

      <!-- Inventory tab -->
      <template v-if="activeTab === 'inventory'">
        <div class="inv-row">
          <div class="inv-main">
            <div class="section-label">Inventory</div>
            <div class="inv-grid">
              <div
                v-for="s in mainSlots"
                :key="s"
                :class="['inv-cell', { empty: !getSlotItem(s) }]"
                :title="getSlotItem(s) ? formatItemName(getSlotItem(s)!.id) : ''"
              >
                <template v-if="getSlotItem(s)">
                  <img
                    class="item-icon"
                    :src="itemIconUrl(getSlotItem(s)!.id)"
                    :alt="getSlotItem(s)!.name"
                    @error="onIconError"
                  />
                  <span class="cell-fallback" style="display: none">{{
                    formatItemName(getSlotItem(s)!.id)
                  }}</span>
                  <span v-if="getSlotItem(s)!.count > 1" class="cell-count">{{
                    getSlotItem(s)!.count
                  }}</span>
                </template>
              </div>
            </div>
          </div>
          <div class="inv-equip">
            <div class="section-label">Armor</div>
            <div class="equip-cells">
              <div
                v-for="slot in armorSlots"
                :key="slot"
                :class="['equip-cell', { empty: !getSlotItem(slot) }]"
                :title="getSlotItem(slot) ? formatItemName(getSlotItem(slot)!.id) : ''"
              >
                <template v-if="getSlotItem(slot)">
                  <img
                    class="item-icon"
                    :src="itemIconUrl(getSlotItem(slot)!.id)"
                    :alt="getSlotItem(slot)!.name"
                    @error="onIconError"
                  />
                  <span class="cell-fallback" style="display: none"></span>
                  <span v-if="getSlotItem(slot)!.count > 1" class="cell-count">{{
                    getSlotItem(slot)!.count
                  }}</span>
                </template>
              </div>
            </div>
          </div>
        </div>

        <div class="inv-row">
          <div class="inv-main">
            <div class="section-label">Hotbar</div>
            <div class="inv-grid">
              <div
                v-for="s in hotbar"
                :key="s"
                :class="['inv-cell', { empty: !getSlotItem(s) }]"
                :title="getSlotItem(s) ? formatItemName(getSlotItem(s)!.id) : ''"
              >
                <template v-if="getSlotItem(s)">
                  <img
                    class="item-icon"
                    :src="itemIconUrl(getSlotItem(s)!.id)"
                    :alt="getSlotItem(s)!.name"
                    @error="onIconError"
                  />
                  <span class="cell-fallback" style="display: none">{{
                    formatItemName(getSlotItem(s)!.id)
                  }}</span>
                  <span v-if="getSlotItem(s)!.count > 1" class="cell-count">{{
                    getSlotItem(s)!.count
                  }}</span>
                </template>
              </div>
            </div>
          </div>
          <div class="inv-equip">
            <div class="section-label">Offhand</div>
            <div class="equip-cells">
              <div
                :class="['equip-cell', { empty: !getSlotItem(-106) }]"
                :title="getSlotItem(-106) ? formatItemName(getSlotItem(-106)!.id) : ''"
              >
                <template v-if="getSlotItem(-106)">
                  <img
                    class="item-icon"
                    :src="itemIconUrl(getSlotItem(-106)!.id)"
                    :alt="getSlotItem(-106)!.name"
                    @error="onIconError"
                  />
                  <span class="cell-fallback" style="display: none"></span>
                  <span v-if="getSlotItem(-106)!.count > 1" class="cell-count">{{
                    getSlotItem(-106)!.count
                  }}</span>
                </template>
              </div>
            </div>
          </div>
        </div>
      </template>

      <!-- Ender Chest tab -->
      <template v-if="activeTab === 'enderchest'">
        <div class="section-label">Ender Chest</div>
        <div class="inv-grid">
          <div
            v-for="s in enderSlots"
            :key="s"
            :class="['inv-cell', { empty: !getEnderSlotItem(s) }]"
            :title="getEnderSlotItem(s) ? formatItemName(getEnderSlotItem(s)!.id) : ''"
          >
            <template v-if="getEnderSlotItem(s)">
              <img
                class="item-icon"
                :src="itemIconUrl(getEnderSlotItem(s)!.id)"
                :alt="getEnderSlotItem(s)!.name"
                @error="onIconError"
              />
              <span class="cell-fallback" style="display: none">{{
                formatItemName(getEnderSlotItem(s)!.id)
              }}</span>
              <span v-if="getEnderSlotItem(s)!.count > 1" class="cell-count">{{
                getEnderSlotItem(s)!.count
              }}</span>
            </template>
          </div>
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
  background: #0d0d0d;
  overflow-y: auto;
}

/* Header */
.panel-header {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 18px 20px;
  background: linear-gradient(135deg, #1a1a1a 0%, #222 100%);
  border-bottom: 1px solid #2a2a2a;
}

.back-btn {
  width: 32px;
  height: 32px;
  background: none;
  border: 1px solid #333;
  border-radius: 6px;
  color: #666;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.15s;
}

.back-btn:hover {
  color: #fff;
  border-color: #555;
  background: #2a2a2a;
}

.player-head {
  width: 56px;
  height: 56px;
  border-radius: 6px;
  image-rendering: pixelated;
  border: 2px solid #333;
  flex-shrink: 0;
}

.header-info {
  flex: 1;
  min-width: 0;
}

.player-title {
  margin: 0;
  font-size: 20px;
  color: #fff;
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
  background: #333;
  color: #666;
}

.online-badge.online {
  background: rgba(74, 222, 128, 0.15);
  color: #4ade80;
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
  color: #555;
  font-style: italic;
}

.last-refreshed {
  font-size: 11px;
  color: #444;
  white-space: nowrap;
}

.refresh-btn {
  width: 32px;
  height: 32px;
  background: none;
  border: 1px solid #333;
  border-radius: 6px;
  color: #666;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all 0.15s;
}

.refresh-btn:hover:not(:disabled) {
  color: #f97316;
  border-color: #f97316;
  background: rgba(249, 115, 22, 0.08);
}

.refresh-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

/* Body */
.panel-body {
  padding: 18px 20px;
}

.state-msg {
  color: #555;
  font-style: italic;
  text-align: center;
  padding: 40px 20px;
}

.state-msg.err {
  color: #ef4444;
}

/* Tab bar */
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
  color: #666;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.tab-btn:hover {
  color: #aaa;
  border-color: #444;
}

.tab-btn.active {
  background: #1a1a1a;
  color: #f97316;
  border-color: #f97316;
}

/* Stats grid */
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
  color: #555;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.stat-value {
  font-size: 13px;
  color: #ccc;
  font-weight: 500;
}

.stat-item.disabled {
  opacity: 0.4;
}

.stat-value.muted {
  font-size: 11px;
  font-style: italic;
  color: #555;
}

/* Inventory layout */
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
  color: #555;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 6px;
}

/* 9-column grid */
.inv-grid {
  display: grid;
  grid-template-columns: repeat(9, 1fr);
  gap: 3px;
}

/* Inventory cell */
.inv-cell {
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  border-radius: 3px;
  aspect-ratio: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.inv-cell.empty {
  background: #131313;
  border-color: #1e1e1e;
}

/* Item icon */
.item-icon {
  width: 70%;
  height: 70%;
  object-fit: contain;
  image-rendering: pixelated;
}

/* Text fallback when icon fails */
.cell-fallback {
  font-size: 9px;
  color: #888;
  text-align: center;
  line-height: 1.1;
  word-break: break-word;
  padding: 2px;
}

/* Stack count */
.cell-count {
  position: absolute;
  bottom: 0;
  right: 2px;
  font-size: 10px;
  font-weight: 700;
  color: #fff;
  text-shadow:
    1px 1px 0 #000,
    -1px -1px 0 #000,
    1px -1px 0 #000,
    -1px 1px 0 #000;
  line-height: 1;
}

/* Equipment column */
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

.equip-cell {
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  border-radius: 3px;
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.equip-cell.empty {
  background: #131313;
  border-color: #1e1e1e;
}

/* Header actions */
.header-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 6px;
}

.act-btn {
  padding: 4px 10px;
  border: 1px solid #333;
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
  border-color: #333;
}

.act-btn.neutral:hover:not(:disabled) {
  color: #fff;
  border-color: #555;
  background: #2a2a2a;
}

.act-btn.warn {
  color: #f59e0b;
  border-color: #f59e0b44;
}

.act-btn.warn:hover:not(:disabled) {
  background: rgba(245, 158, 11, 0.1);
}

.act-btn.danger {
  color: #ef4444;
  border-color: #ef444444;
}

.act-btn.danger:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.1);
}
</style>
