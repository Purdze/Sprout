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
const saveInterval = ref(300);

async function fetchInventory() {
  const result = await invoke<PlayerDetails>('get_player_inventory', {
    id: serverId.value,
    path: serverPath.value,
    playerName: props.playerName,
  });
  player.value = result;
}

async function refresh() {
  try {
    await fetchInventory();
  } catch (e) {
    error.value = String(e);
  }
}

function formatInterval(seconds: number): string {
  if (seconds >= 60) {
    const mins = Math.round(seconds / 60);
    return `${mins} minute${mins !== 1 ? 's' : ''}`;
  }
  return `${seconds} second${seconds !== 1 ? 's' : ''}`;
}

async function loadPlayer() {
  if (!props.playerName) return;
  loading.value = true;
  error.value = '';
  try {
    await fetchInventory();
    try {
      saveInterval.value = await invoke<number>('get_save_interval', {
        path: serverPath.value,
      });
    } catch {
      saveInterval.value = 300;
    }
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
            ><span class="pstat-icon">&#9829;</span> {{ player.health.toFixed(0) }}/{{
              player.maxHealth.toFixed(0)
            }}</span
          >
          <span class="pstat food"
            ><span class="pstat-icon">&#9830;</span> {{ player.food }}/20</span
          >
          <span class="pstat xp"
            ><span class="pstat-icon">&#9733;</span> Lvl {{ player.xpLevel }}</span
          >
          <span class="pstat mode">{{ player.gameMode }}</span>
        </div>
      </div>
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
      <div class="inv-layout">
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
          <div class="section-label">Armor</div>
          <div
            v-for="(slot, i) in armorSlots"
            :key="slot"
            :class="['equip-cell', { empty: !getSlotItem(slot) }]"
          >
            <span class="equip-slot-label">{{
              ['Helmet', 'Chestplate', 'Leggings', 'Boots'][i]
            }}</span>
            <template v-if="getSlotItem(slot)">
              <div class="equip-item-row">
                <img
                  class="item-icon-sm"
                  :src="itemIconUrl(getSlotItem(slot)!.id)"
                  :alt="getSlotItem(slot)!.name"
                  @error="onIconError"
                />
                <span class="cell-fallback" style="display: none"></span>
                <span class="equip-item">{{ formatItemName(getSlotItem(slot)!.id) }}</span>
              </div>
            </template>
          </div>
          <div class="section-label offhand-label">Offhand</div>
          <div :class="['equip-cell', { empty: !getSlotItem(-106) }]">
            <span class="equip-slot-label">Offhand</span>
            <template v-if="getSlotItem(-106)">
              <div class="equip-item-row">
                <img
                  class="item-icon-sm"
                  :src="itemIconUrl(getSlotItem(-106)!.id)"
                  :alt="getSlotItem(-106)!.name"
                  @error="onIconError"
                />
                <span class="cell-fallback" style="display: none"></span>
                <span class="equip-item">{{ formatItemName(getSlotItem(-106)!.id) }}</span>
              </div>
            </template>
          </div>
        </div>
      </div>

      <div class="save-notice">
        Inventory data saves every {{ formatInterval(saveInterval) }}
        <span class="notice-src">(features.toml)</span>
      </div>

      <!-- Actions -->
      <div class="inv-actions">
        <button
          class="act-btn kick"
          :disabled="serverStatus !== 'running' || !isOnline"
          @click="
            emit('command', `kick ${playerName}`);
            emit('back');
          "
        >
          Kick
        </button>
        <button
          class="act-btn ban"
          :disabled="serverStatus !== 'running' || !isOnline"
          @click="
            emit('command', `ban ${playerName}`);
            emit('back');
          "
        >
          Ban
        </button>
      </div>
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

.pstat-icon {
  font-size: 12px;
}

.pstat.health .pstat-icon {
  color: #ef4444;
}

.pstat.food .pstat-icon {
  color: #f59e0b;
}

.pstat.xp .pstat-icon {
  color: #4ade80;
}

.pstat.mode {
  color: #555;
  font-style: italic;
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

/* Inventory layout */
.inv-layout {
  display: flex;
  gap: 20px;
  align-items: flex-start;
}

.inv-main {
  flex: 1;
  min-width: 0;
}

.section-label {
  font-size: 11px;
  color: #555;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 6px;
}

.offhand-label {
  margin-top: 10px;
}

/* 9-column grid */
.inv-grid {
  display: grid;
  grid-template-columns: repeat(9, 1fr);
  gap: 4px;
  margin-bottom: 12px;
}

/* Inventory cell */
.inv-cell {
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  border-radius: 4px;
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

.item-icon-sm {
  width: 20px;
  height: 20px;
  object-fit: contain;
  image-rendering: pixelated;
  flex-shrink: 0;
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
  bottom: 1px;
  right: 3px;
  font-size: 11px;
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
  width: 140px;
  flex-shrink: 0;
}

.equip-cell {
  background: #1a1a1a;
  border: 1px solid #2a2a2a;
  border-radius: 4px;
  padding: 8px 10px;
  margin-bottom: 4px;
}

.equip-cell.empty {
  background: #131313;
  border-color: #1e1e1e;
}

.equip-slot-label {
  display: block;
  font-size: 10px;
  color: #555;
}

.equip-item-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 3px;
}

.equip-item {
  font-size: 12px;
  color: #ccc;
}

/* Save Notice */
.save-notice {
  font-size: 11px;
  color: #444;
  margin-top: 14px;
  padding: 8px 10px;
  background: #161616;
  border-radius: 6px;
  border: 1px solid #222;
}

.notice-src {
  color: #555;
}

/* Actions */
.inv-actions {
  display: flex;
  gap: 8px;
  margin-top: 14px;
  padding-top: 14px;
  border-top: 1px solid #2a2a2a;
}

.act-btn {
  padding: 7px 18px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  font-weight: 500;
  transition: opacity 0.15s;
}

.act-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.act-btn:hover:not(:disabled) {
  opacity: 0.85;
}

.act-btn.kick {
  background: #f59e0b;
  color: #000;
}

.act-btn.ban {
  background: #ef4444;
  color: #fff;
}
</style>
