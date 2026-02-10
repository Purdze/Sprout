<script setup lang="ts">
import { ref } from 'vue';
import { emit as tauriEmit } from '@tauri-apps/api/event';
import { WebviewWindow, getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { open } from '@tauri-apps/plugin-shell';
import type { Server } from '../types';

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
  <div class="tab-bar" @dragover="onTabDragOver" @dragenter="onTabDragEnter" @drop="onTabDrop">
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
    <button class="icon-btn donate-btn" title="Donate" @click="open('https://paypal.me/richarddm')">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" stroke="none">
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
  </div>
</template>

<style scoped>
.tab-bar {
  display: flex;
  background: var(--bg-medium);
  border-bottom: 1px solid var(--bg-light);
  padding: 8px 8px 0;
  gap: 4px;
}

.tab {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: var(--bg-dark);
  border: none;
  border-radius: 8px 8px 0 0;
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 14px;
}

.tab.active {
  background: var(--bg-dark);
  color: var(--text-primary);
  border: 1px solid var(--bg-light);
  border-bottom: 1px solid var(--bg-dark);
  margin-bottom: -1px;
}

.tab:hover:not(.active) {
  color: #aaa;
}

.tab.dragging {
  opacity: 0.5;
}

.tab[draggable='true'] {
  cursor: grab;
}

.tab[draggable='true']:active {
  cursor: grabbing;
}

.add-tab {
  padding: 8px 12px;
  font-size: 18px;
  color: var(--text-muted);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-muted);
}

.status-dot.running {
  background: var(--color-success);
}

.status-dot.starting {
  background: var(--color-warning-light);
}

.status-dot.stopped {
  background: var(--text-muted);
}

.close-btn {
  margin-left: 4px;
  opacity: 0.5;
  font-size: 16px;
}

.close-btn:hover {
  opacity: 1;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 18px;
  cursor: pointer;
  padding: 8px;
  transition: color 0.2s;
}

.donate-btn {
  margin-left: auto;
}

.donate-btn:hover {
  color: #e55b5b;
}

.discord-btn:hover {
  color: #5865f2;
}
</style>
