<script setup lang="ts">
import { ref } from 'vue';
import { emit as tauriEmit } from '@tauri-apps/api/event';
import { WebviewWindow, getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { open } from '@tauri-apps/plugin-shell';
import type { Server } from '../types';

const appWindow = getCurrentWebviewWindow();
const isMaximized = ref(false);

async function checkMaximized() {
  isMaximized.value = await appWindow.isMaximized();
}

checkMaximized();

function minimize() {
  appWindow.minimize();
}

function toggleMaximize() {
  appWindow.toggleMaximize();
  // Small delay to let the OS finish the resize
  setTimeout(checkMaximized, 50);
}

function closeWindow() {
  appWindow.close();
}

const props = defineProps<{
  servers: Server[];
  activeTab: number;
  currentWindowLabel: string;
}>();

const emit = defineEmits<{
  'update:activeTab': [index: number];
  remove: [index: number];
  add: [];
  serverRemoved: [serverId: string];
}>();

const draggingTab = ref<number | null>(null);
let dragStartX = 0;
let dragStartY = 0;

function onTabDragStart(event: DragEvent, index: number) {
  draggingTab.value = index;
  dragStartX = event.screenX;
  dragStartY = event.screenY;
  const data = {
    server: props.servers[index],
    sourceWindow: props.currentWindowLabel,
  };
  event.dataTransfer!.setData('text/plain', JSON.stringify(data));
  event.dataTransfer!.effectAllowed = 'move';
  document.body.classList.add('dragging');
}

async function onTabDragEnd(event: DragEvent, index: number) {
  const dragDistance = Math.sqrt(
    Math.pow(event.screenX - dragStartX, 2) + Math.pow(event.screenY - dragStartY, 2)
  );

  if (dragDistance > 100) {
    const server = props.servers[index];

    await tauriEmit('check-drop-target', {
      x: event.screenX,
      y: event.screenY,
      server: JSON.stringify(server),
      sourceWindow: props.currentWindowLabel,
    });

    await new Promise((resolve) => setTimeout(resolve, 150));

    if (props.servers.find((s) => s.id === server.id) && props.servers.length > 1) {
      const windowLabel = `server-${Date.now()}`;

      try {
        new WebviewWindow(windowLabel, {
          url: `/?serverId=${server.id}`,
          title: server.name,
          width: 1200,
          height: 800,
          x: event.screenX - 100,
          y: event.screenY - 50,
          decorations: false,
        });

        emit('serverRemoved', server.id);

        if (props.servers.length === 1 && props.currentWindowLabel !== 'main') {
          getCurrentWebviewWindow().close();
        }
      } catch (e) {
        console.error('Failed to create new window:', e);
      }
    }
  }

  draggingTab.value = null;
  document.body.classList.remove('dragging');
}

function onTabDragOver(event: DragEvent) {
  event.preventDefault();
  event.stopPropagation();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move';
  }
}

function onTabDragEnter(event: DragEvent) {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move';
  }
}

async function onTabDrop(event: DragEvent) {
  event.preventDefault();
  const data = event.dataTransfer?.getData('text/plain');
  if (data) {
    try {
      const { server, sourceWindow } = JSON.parse(data);
      if (sourceWindow !== props.currentWindowLabel) {
        await tauriEmit('transfer-server', {
          server: JSON.stringify(server),
          targetWindow: props.currentWindowLabel,
          sourceWindow: sourceWindow,
        });
      }
    } catch (e) {
      console.error('Failed to handle drop:', e);
    }
  }
}
</script>

<template>
  <div class="titlebar" @dragover="onTabDragOver" @dragenter="onTabDragEnter" @drop="onTabDrop">
    <div class="tab-area">
      <button
        v-for="(server, index) in servers"
        :key="server.id"
        :class="['tab', { active: activeTab === index, dragging: draggingTab === index }]"
        draggable="true"
        @click="emit('update:activeTab', index)"
        @dragstart="onTabDragStart($event, index)"
        @dragend="onTabDragEnd($event, index)"
      >
        <span :class="['status-dot', server.status]"></span>
        {{ server.name }}
        <span class="close-btn" @click.stop="emit('remove', index)">&times;</span>
      </button>
      <button class="tab add-tab" @click="emit('add')">+</button>
    </div>

    <div class="drag-region" data-tauri-drag-region></div>

    <div class="titlebar-right">
      <button
        class="icon-btn donate-btn"
        title="Donate"
        @click="open('https://github.com/sponsors/Purdze')"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" stroke="none">
          <path
            d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"
          />
        </svg>
      </button>
      <button
        class="icon-btn discord-btn"
        title="Join the Discord for support"
        @click="open('https://discord.gg/qsRhJUP4q5')"
      >
        <FontAwesomeIcon :icon="['fab', 'discord']" />
      </button>

      <div class="window-controls">
        <button class="win-btn" title="Minimize" @click="minimize">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect x="2" y="5.5" width="8" height="1" fill="currentColor" />
          </svg>
        </button>
        <button class="win-btn" title="Maximize" @click="toggleMaximize">
          <svg v-if="!isMaximized" width="12" height="12" viewBox="0 0 12 12">
            <rect
              x="2"
              y="2"
              width="8"
              height="8"
              rx="1"
              fill="none"
              stroke="currentColor"
              stroke-width="1.2"
            />
          </svg>
          <svg v-else width="12" height="12" viewBox="0 0 12 12">
            <rect
              x="3.5"
              y="0.5"
              width="7"
              height="7"
              rx="1"
              fill="none"
              stroke="currentColor"
              stroke-width="1.2"
            />
            <rect
              x="1.5"
              y="3.5"
              width="7"
              height="7"
              rx="1"
              fill="var(--bg-base)"
              stroke="currentColor"
              stroke-width="1.2"
            />
          </svg>
        </button>
        <button class="win-btn win-close" title="Close" @click="closeWindow">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <path
              d="M3 3l6 6M9 3l-6 6"
              stroke="currentColor"
              stroke-width="1.4"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  display: flex;
  align-items: center;
  background: var(--bg-base);
  border-bottom: 1px solid var(--border-subtle);
  padding: 0;
  user-select: none;
  height: 40px;
  flex-shrink: 0;
}

.tab-area {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 10px;
  height: 100%;
  flex-shrink: 0;
}

.drag-region {
  flex: 1;
  height: 100%;
  -webkit-app-region: drag;
}

.titlebar-right {
  display: flex;
  align-items: center;
  gap: 2px;
  padding-right: 2px;
  height: 100%;
  flex-shrink: 0;
}

/* Tabs */
.tab {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 14px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 12.5px;
  font-weight: 500;
  font-family: var(--font-ui);
  transition:
    color var(--transition-fast),
    background var(--transition-fast),
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
  position: relative;
}

.tab.active {
  background: var(--bg-surface);
  color: var(--text-primary);
  border-color: var(--border-default);
  box-shadow: var(--shadow-sm);
}

.tab:hover:not(.active) {
  color: var(--text-secondary);
  background: rgba(255, 255, 255, 0.04);
}

.tab.dragging {
  opacity: 0.4;
}

.tab[draggable='true'] {
  cursor: grab;
}

.tab[draggable='true']:active {
  cursor: grabbing;
}

.add-tab {
  width: 26px;
  height: 26px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  color: var(--text-faint);
  font-weight: 400;
  border: 1px dashed var(--border-default);
  border-radius: var(--radius-sm);
  gap: 0;
}

.add-tab:hover {
  color: var(--text-secondary);
  border-color: var(--border-strong);
  background: rgba(255, 255, 255, 0.04);
}

/* Status dot */
.status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--text-faint);
  flex-shrink: 0;
  transition: background var(--transition-normal);
}

.status-dot.running {
  background: var(--color-success);
  box-shadow: 0 0 6px rgba(74, 222, 128, 0.4);
  animation: pulse-glow 2s ease-in-out infinite;
}

.status-dot.starting {
  background: var(--color-warning);
  box-shadow: 0 0 6px rgba(251, 191, 36, 0.3);
  animation: pulse-glow 1s ease-in-out infinite;
}

.status-dot.stopped {
  background: var(--text-faint);
}

/* Tab close */
.close-btn {
  margin-left: 2px;
  opacity: 0;
  font-size: 14px;
  transition: opacity var(--transition-fast);
  line-height: 1;
}

.tab:hover .close-btn {
  opacity: 0.4;
}

.close-btn:hover {
  opacity: 1 !important;
}

/* Icon buttons */
.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: none;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  color: var(--text-faint);
  font-size: 14px;
  cursor: pointer;
  padding: 0;
  transition:
    color var(--transition-fast),
    background var(--transition-fast);
}

.icon-btn:hover {
  background: rgba(255, 255, 255, 0.06);
}

.donate-btn:hover {
  color: #f87171;
}

.discord-btn:hover {
  color: #7289da;
}

/* Window controls */
.window-controls {
  display: flex;
  align-items: center;
  height: 100%;
  margin-left: 4px;
}

.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 100%;
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  transition:
    background var(--transition-fast),
    color var(--transition-fast);
}

.win-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-primary);
}

.win-close:hover {
  background: var(--color-danger);
  color: #fff;
}
</style>
