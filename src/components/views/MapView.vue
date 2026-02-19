<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getPlayerAvatarUrl } from '../../utils/minecraft';
import type {
  Server,
  MapRegionInfo,
  MapTileResponse,
  MapPlayerMarker,
  MapMarker,
} from '../../types';

const props = defineProps<{
  server: Server;
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const dimension = ref<'overworld' | 'nether' | 'end'>('overworld');
const regions = ref<MapRegionInfo[]>([]);
const players = ref<MapPlayerMarker[]>([]);
const markers = ref<MapMarker[]>([]);
const loading = ref(false);
const error = ref('');
const mouseCoords = ref({ x: 0, z: 0 });
const zoomLevel = ref(1);

const markerPopup = ref<{
  visible: boolean;
  x: number;
  y: number;
  worldX: number;
  worldZ: number;
  name: string;
  color: string;
  editingId: string | null;
}>({
  visible: false,
  x: 0,
  y: 0,
  worldX: 0,
  worldZ: 0,
  name: '',
  color: '#f97316',
  editingId: null,
});

const PRESET_COLORS = ['#f97316', '#ef4444', '#22c55e', '#3b82f6', '#a855f7', '#eab308'];

let offsetX = 0;
let offsetZ = 0;
let dragging = false;
let hasDragged = false;
let dragStartX = 0;
let dragStartZ = 0;
let dragOffsetX = 0;
let dragOffsetZ = 0;
let animFrameId = 0;

const tileCache = new Map<string, HTMLImageElement>();
const tileLoading = new Set<string>();

const headCache = new Map<string, HTMLImageElement>();
const headLoading = new Set<string>();
const headFailed = new Set<string>();
const HEAD_SIZE = 24; // px

function loadPlayerHead(name: string) {
  if (headCache.has(name) || headLoading.has(name) || headFailed.has(name)) return;
  headLoading.add(name);
  const img = new Image();
  img.src = getPlayerAvatarUrl(name, HEAD_SIZE);
  img.onload = () => {
    headCache.set(name, img);
    headLoading.delete(name);
  };
  img.onerror = () => {
    headLoading.delete(name);
    headFailed.add(name);
  };
}

const TILE_SIZE = 512; // px per region tile

function tileKey(rx: number, rz: number): string {
  return `${rx},${rz}`;
}

async function loadRegions() {
  loading.value = true;
  error.value = '';
  try {
    regions.value = await invoke('list_map_regions', {
      path: props.server.path,
      dimension: dimension.value,
    });
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function loadPlayers() {
  try {
    players.value = await invoke('get_map_players', {
      id: props.server.id,
      path: props.server.path,
    });
  } catch {
    // silently ignore player loading errors
  }
}

async function loadMarkers() {
  try {
    markers.value = await invoke('load_map_markers', { id: props.server.id });
  } catch {
    // silently ignore marker loading errors
  }
}

async function loadTile(rx: number, rz: number) {
  const key = tileKey(rx, rz);
  if (tileCache.has(key) || tileLoading.has(key)) return;
  tileLoading.add(key);
  try {
    const resp: MapTileResponse = await invoke('get_map_tile', {
      path: props.server.path,
      dimension: dimension.value,
      regionX: rx,
      regionZ: rz,
    });
    const img = new Image();
    img.src = `data:image/png;base64,${resp.imageBase64}`;
    await new Promise<void>((resolve, reject) => {
      img.onload = () => resolve();
      img.onerror = () => reject();
    });
    tileCache.set(key, img);
  } catch {
    // tile failed to load
  } finally {
    tileLoading.delete(key);
  }
}

function worldToCanvas(worldX: number, worldZ: number): { cx: number; cz: number } {
  return {
    cx: (worldX + offsetX) * zoomLevel.value,
    cz: (worldZ + offsetZ) * zoomLevel.value,
  };
}

function canvasToWorld(cx: number, cz: number): { wx: number; wz: number } {
  return {
    wx: cx / zoomLevel.value - offsetX,
    wz: cz / zoomLevel.value - offsetZ,
  };
}

function drawLabel(ctx: CanvasRenderingContext2D, text: string, cx: number, cy: number) {
  ctx.fillStyle = '#fff';
  ctx.font = '11px sans-serif';
  ctx.textAlign = 'center';
  ctx.shadowColor = 'rgba(0,0,0,0.7)';
  ctx.shadowBlur = 3;
  ctx.fillText(text, cx, cy);
  ctx.shadowBlur = 0;
}

function drawDiamond(
  ctx: CanvasRenderingContext2D,
  cx: number,
  cz: number,
  size: number,
  fillColor: string
) {
  ctx.save();
  ctx.beginPath();
  ctx.moveTo(cx, cz - size);
  ctx.lineTo(cx + size * 0.7, cz);
  ctx.lineTo(cx, cz + size);
  ctx.lineTo(cx - size * 0.7, cz);
  ctx.closePath();
  ctx.fillStyle = fillColor;
  ctx.shadowColor = 'rgba(0,0,0,0.5)';
  ctx.shadowBlur = 4;
  ctx.fill();
  ctx.shadowBlur = 0;
  ctx.strokeStyle = '#fff';
  ctx.lineWidth = 1.5;
  ctx.stroke();
  ctx.restore();
}

function render() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const w = canvas.width;
  const h = canvas.height;
  const zoom = zoomLevel.value;
  const tilePixels = TILE_SIZE * zoom;

  ctx.clearRect(0, 0, w, h);
  ctx.fillStyle = '#1a1a1a';
  ctx.fillRect(0, 0, w, h);

  const topLeft = canvasToWorld(0, 0);
  const bottomRight = canvasToWorld(w, h);

  const minRX = Math.floor(topLeft.wx / TILE_SIZE) - 1;
  const maxRX = Math.floor(bottomRight.wx / TILE_SIZE) + 1;
  const minRZ = Math.floor(topLeft.wz / TILE_SIZE) - 1;
  const maxRZ = Math.floor(bottomRight.wz / TILE_SIZE) + 1;

  for (const region of regions.value) {
    if (
      region.regionX < minRX ||
      region.regionX > maxRX ||
      region.regionZ < minRZ ||
      region.regionZ > maxRZ
    )
      continue;

    const key = tileKey(region.regionX, region.regionZ);
    const img = tileCache.get(key);

    const worldX = region.regionX * TILE_SIZE;
    const worldZ = region.regionZ * TILE_SIZE;
    const { cx, cz } = worldToCanvas(worldX, worldZ);

    if (img) {
      // Floor position and ceil size to eliminate sub-pixel gaps between tiles
      const dx = Math.floor(cx);
      const dy = Math.floor(cz);
      const dw = Math.ceil(cx + tilePixels) - dx;
      const dh = Math.ceil(cz + tilePixels) - dy;
      ctx.drawImage(img, dx, dy, dw, dh);
    } else {
      // Show placeholder and start loading
      ctx.fillStyle = '#252525';
      ctx.fillRect(cx, cz, tilePixels, tilePixels);
      ctx.fillStyle = '#444';
      ctx.font = '12px monospace';
      ctx.textAlign = 'center';
      ctx.fillText(
        tileLoading.has(key) ? 'Loading...' : `r.${region.regionX}.${region.regionZ}`,
        cx + tilePixels / 2,
        cz + tilePixels / 2
      );
      loadTile(region.regionX, region.regionZ);
    }
  }

  const dimMarkers = markers.value.filter((m) => m.dimension === dimension.value);
  for (const marker of dimMarkers) {
    const { cx, cz } = worldToCanvas(marker.x, marker.z);
    if (cx < -40 || cx > w + 40 || cz < -40 || cz > h + 40) continue;

    drawDiamond(ctx, cx, cz, 8, marker.color);
    drawLabel(ctx, marker.name, cx, cz - 12);
  }

  const dimPlayers = players.value.filter((p) => p.dimension === dimension.value);
  for (const player of dimPlayers) {
    const { cx, cz } = worldToCanvas(player.x, player.z);
    if (cx < -40 || cx > w + 40 || cz < -40 || cz > h + 40) continue;

    const headImg = headCache.get(player.name);
    if (headImg) {
      const half = HEAD_SIZE / 2;
      ctx.save();
      ctx.shadowColor = 'rgba(0,0,0,0.5)';
      ctx.shadowBlur = 4;
      ctx.strokeStyle = '#fff';
      ctx.lineWidth = 2;
      ctx.strokeRect(cx - half - 1, cz - half - 1, HEAD_SIZE + 2, HEAD_SIZE + 2);
      ctx.shadowBlur = 0;
      ctx.drawImage(headImg, cx - half, cz - half, HEAD_SIZE, HEAD_SIZE);
      ctx.restore();
    } else {
      loadPlayerHead(player.name);
      ctx.beginPath();
      ctx.arc(cx, cz, 5, 0, Math.PI * 2);
      ctx.fillStyle = '#f97316';
      ctx.fill();
      ctx.strokeStyle = '#fff';
      ctx.lineWidth = 1.5;
      ctx.stroke();
    }

    drawLabel(ctx, player.name, cx, cz - HEAD_SIZE / 2 - 4);
  }

  animFrameId = requestAnimationFrame(render);
}

function onMouseDown(e: MouseEvent) {
  if (e.button !== 0) return;
  dragging = true;
  hasDragged = false;
  dragStartX = e.clientX;
  dragStartZ = e.clientY;
  dragOffsetX = offsetX;
  dragOffsetZ = offsetZ;
}

function onMouseMove(e: MouseEvent) {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const cx = e.clientX - rect.left;
  const cz = e.clientY - rect.top;
  const world = canvasToWorld(cx, cz);
  mouseCoords.value = { x: Math.round(world.wx), z: Math.round(world.wz) };

  if (dragging) {
    const dx = e.clientX - dragStartX;
    const dz = e.clientY - dragStartZ;
    if (Math.abs(dx) > 3 || Math.abs(dz) > 3) hasDragged = true;
    offsetX = dragOffsetX + dx / zoomLevel.value;
    offsetZ = dragOffsetZ + dz / zoomLevel.value;
  }
}

function findMarkerAtCanvas(canvasX: number, canvasY: number): MapMarker | null {
  const dimMarkers = markers.value.filter((m) => m.dimension === dimension.value);
  for (const marker of dimMarkers) {
    const { cx, cz } = worldToCanvas(marker.x, marker.z);
    const dx = canvasX - cx;
    const dz = canvasY - cz;
    if (Math.sqrt(dx * dx + dz * dz) <= 10) return marker;
  }
  return null;
}

function onMouseUp(e: MouseEvent) {
  const wasDragging = dragging;
  dragging = false;
  if (wasDragging && !hasDragged && e.button === 0 && !markerPopup.value.visible) {
    const canvas = canvasRef.value;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const cx = e.clientX - rect.left;
    const cz = e.clientY - rect.top;
    const hit = findMarkerAtCanvas(cx, cz);
    if (hit) {
      openEditPopup(hit, e.clientX, e.clientY);
    }
  }
}

function onContextMenu(e: MouseEvent) {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const cx = e.clientX - rect.left;
  const cz = e.clientY - rect.top;
  const world = canvasToWorld(cx, cz);

  // Check if right-clicking an existing marker
  const hit = findMarkerAtCanvas(cx, cz);
  if (hit) {
    openEditPopup(hit, e.clientX, e.clientY);
    return;
  }

  const wrapper = canvas.parentElement;
  if (!wrapper) return;
  const wrapperRect = wrapper.getBoundingClientRect();

  markerPopup.value = {
    visible: true,
    x: e.clientX - wrapperRect.left,
    y: e.clientY - wrapperRect.top,
    worldX: Math.round(world.wx),
    worldZ: Math.round(world.wz),
    name: '',
    color: '#f97316',
    editingId: null,
  };
}

function openEditPopup(marker: MapMarker, clientX: number, clientY: number) {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const wrapper = canvas.parentElement;
  if (!wrapper) return;
  const wrapperRect = wrapper.getBoundingClientRect();

  markerPopup.value = {
    visible: true,
    x: clientX - wrapperRect.left,
    y: clientY - wrapperRect.top,
    worldX: marker.x,
    worldZ: marker.z,
    name: marker.name,
    color: marker.color,
    editingId: marker.id,
  };
}

async function saveMarker() {
  const popup = markerPopup.value;
  if (!popup.name.trim()) return;

  const marker: MapMarker = {
    id: popup.editingId || crypto.randomUUID(),
    name: popup.name.trim(),
    x: popup.worldX,
    z: popup.worldZ,
    dimension: dimension.value,
    color: popup.color,
  };

  try {
    await invoke('save_map_marker', { id: props.server.id, marker });
    await loadMarkers();
  } catch {
    // silently ignore save errors
  }
  closePopup();
}

async function deleteMarker() {
  const popup = markerPopup.value;
  if (!popup.editingId) return;
  try {
    await invoke('delete_map_marker', { id: props.server.id, markerId: popup.editingId });
    await loadMarkers();
  } catch {
    // silently ignore delete errors
  }
  closePopup();
}

function closePopup() {
  markerPopup.value.visible = false;
}

function onWheel(e: WheelEvent) {
  e.preventDefault();
  const canvas = canvasRef.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const cx = e.clientX - rect.left;
  const cz = e.clientY - rect.top;

  // World position under cursor before zoom
  const worldBefore = canvasToWorld(cx, cz);

  const factor = e.deltaY < 0 ? 1.15 : 1 / 1.15;
  zoomLevel.value = Math.max(0.1, Math.min(8, zoomLevel.value * factor));

  // World position under cursor after zoom — adjust offset to keep it fixed
  const worldAfter = canvasToWorld(cx, cz);
  offsetX += worldAfter.wx - worldBefore.wx;
  offsetZ += worldAfter.wz - worldBefore.wz;
}

function goToOrigin() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  offsetX = canvas.width / 2 / zoomLevel.value;
  offsetZ = canvas.height / 2 / zoomLevel.value;
}

let needsInitialCenter = true;

function resizeCanvas() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const parent = canvas.parentElement;
  if (!parent) return;
  const w = parent.clientWidth;
  const h = parent.clientHeight;
  if (w === 0 || h === 0) return;
  canvas.width = w;
  canvas.height = h;
  if (needsInitialCenter) {
    needsInitialCenter = false;
    goToOrigin();
  }
}

async function refresh() {
  tileCache.clear();
  tileLoading.clear();
  await Promise.all([loadRegions(), loadPlayers(), loadMarkers()]);
}

watch(dimension, refresh);

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  resizeCanvas();
  loadRegions();
  loadPlayers();
  loadMarkers();
  animFrameId = requestAnimationFrame(render);
  window.addEventListener('resize', resizeCanvas);
  const wrapper = canvasRef.value?.parentElement;
  if (wrapper) {
    resizeObserver = new ResizeObserver(() => resizeCanvas());
    resizeObserver.observe(wrapper);
  }
});

onUnmounted(() => {
  cancelAnimationFrame(animFrameId);
  window.removeEventListener('resize', resizeCanvas);
  resizeObserver?.disconnect();
});
</script>

<template>
  <div class="map-view">
    <div class="map-toolbar">
      <div class="dimension-tabs">
        <button
          :class="['dim-tab', { active: dimension === 'overworld' }]"
          @click="dimension = 'overworld'"
        >
          Overworld
        </button>
        <button
          :class="['dim-tab', { active: dimension === 'nether' }]"
          @click="dimension = 'nether'"
        >
          Nether
        </button>
        <button :class="['dim-tab', { active: dimension === 'end' }]" @click="dimension = 'end'">
          End
        </button>
      </div>

      <div class="map-info">
        <span class="map-coord">X: {{ mouseCoords.x }}</span>
        <span class="map-coord">Z: {{ mouseCoords.z }}</span>
        <span class="map-zoom">{{ Math.round(zoomLevel * 100) }}%</span>
        <span class="map-region-count">{{ regions.length }} regions</span>
      </div>

      <div class="map-actions">
        <button class="map-btn" @click="goToOrigin">Origin</button>
        <button class="map-btn" @click="refresh">Refresh</button>
      </div>
    </div>

    <div class="map-canvas-wrapper">
      <div v-if="loading && regions.length === 0" class="map-overlay">
        <span class="map-loading">Loading map data...</span>
      </div>
      <div v-else-if="error" class="map-overlay">
        <span class="map-error">{{ error }}</span>
      </div>
      <div v-else-if="!loading && regions.length === 0" class="map-overlay">
        <span class="map-empty">No world data found</span>
        <span class="map-empty-hint">Start the server and explore to generate chunks</span>
      </div>
      <canvas
        ref="canvasRef"
        class="map-canvas"
        @mousedown="onMouseDown"
        @mousemove="onMouseMove"
        @mouseup="onMouseUp"
        @mouseleave="dragging = false"
        @wheel="onWheel"
        @contextmenu.prevent="onContextMenu"
      />

      <div
        v-if="markerPopup.visible"
        class="marker-popup"
        :style="{ left: markerPopup.x + 'px', top: markerPopup.y + 'px' }"
      >
        <div class="marker-popup-header">
          {{ markerPopup.editingId ? 'Edit Marker' : 'New Marker' }}
        </div>
        <div class="marker-popup-coords">
          X: {{ markerPopup.worldX }}, Z: {{ markerPopup.worldZ }}
        </div>
        <input
          v-model="markerPopup.name"
          class="marker-popup-input"
          placeholder="Marker name..."
          maxlength="32"
          @keyup.enter="saveMarker"
        />
        <div class="marker-popup-colors">
          <button
            v-for="c in PRESET_COLORS"
            :key="c"
            class="marker-color-btn"
            :class="{ active: markerPopup.color === c }"
            :style="{ background: c }"
            @click="markerPopup.color = c"
          />
        </div>
        <div class="marker-popup-actions">
          <button class="marker-btn save" @click="saveMarker">Save</button>
          <button class="marker-btn cancel" @click="closePopup">Cancel</button>
          <button v-if="markerPopup.editingId" class="marker-btn delete" @click="deleteMarker">
            Delete
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.map-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-dark);
}

.map-toolbar {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 12px;
  background: var(--bg-medium);
  border-bottom: 1px solid var(--bg-light);
  flex-shrink: 0;
}

.dimension-tabs {
  display: flex;
  gap: 2px;
  background: var(--bg-dark);
  border-radius: 6px;
  padding: 2px;
}

.dim-tab {
  padding: 5px 12px;
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 12px;
  border-radius: 4px;
  transition: all 0.15s;
}

.dim-tab:hover {
  color: var(--text-secondary);
}

.dim-tab.active {
  background: var(--bg-light);
  color: var(--color-primary);
}

.map-info {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-left: auto;
}

.map-coord {
  font-size: 12px;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
}

.map-zoom {
  font-size: 12px;
  color: var(--text-muted);
}

.map-region-count {
  font-size: 11px;
  color: var(--text-dim);
}

.map-actions {
  display: flex;
  gap: 6px;
}

.map-btn {
  padding: 4px 10px;
  background: var(--bg-light);
  border: none;
  border-radius: 4px;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 12px;
}

.map-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.map-canvas-wrapper {
  flex: 1;
  position: relative;
  min-height: 0;
  overflow: hidden;
}

.map-canvas {
  display: block;
  cursor: grab;
}

.map-canvas:active {
  cursor: grabbing;
}

.map-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  z-index: 1;
  pointer-events: none;
}

.map-loading {
  color: var(--text-tertiary);
  font-size: 14px;
}

.map-error {
  color: var(--color-danger);
  font-size: 14px;
}

.map-empty {
  color: var(--text-muted);
  font-size: 16px;
}

.map-empty-hint {
  color: var(--text-faint);
  font-size: 12px;
}

.marker-popup {
  position: absolute;
  z-index: 10;
  background: #2a2a2a;
  border: 1px solid var(--bg-hover);
  border-radius: 8px;
  padding: 12px;
  min-width: 200px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
  transform: translate(8px, 8px);
}

.marker-popup-header {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.marker-popup-coords {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  margin-bottom: 8px;
}

.marker-popup-input {
  width: 100%;
  padding: 6px 8px;
  background: var(--bg-dark);
  border: 1px solid var(--bg-hover);
  border-radius: 4px;
  color: #eee;
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
}

.marker-popup-input:focus {
  border-color: var(--color-primary);
}

.marker-popup-colors {
  display: flex;
  gap: 6px;
  margin-top: 8px;
}

.marker-color-btn {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: border-color 0.15s;
}

.marker-color-btn:hover {
  border-color: var(--text-tertiary);
}

.marker-color-btn.active {
  border-color: var(--text-primary);
}

.marker-popup-actions {
  display: flex;
  gap: 6px;
  margin-top: 10px;
}

.marker-btn {
  padding: 4px 12px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}

.marker-btn.save {
  background: var(--color-primary);
  color: var(--text-primary);
}

.marker-btn.save:hover {
  background: var(--color-primary-hover);
}

.marker-btn.cancel {
  background: var(--bg-light);
  color: var(--text-secondary);
}

.marker-btn.cancel:hover {
  background: var(--bg-hover);
}

.marker-btn.delete {
  background: var(--color-danger-hover);
  color: var(--text-primary);
  margin-left: auto;
}

.marker-btn.delete:hover {
  background: #b91c1c;
}
</style>
