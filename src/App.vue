<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, emit as tauriEmit, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import ServerTab from './components/ServerTab.vue';
import TabBar from './components/TabBar.vue';
import AddServerDialog from './components/AddServerDialog.vue';
import type { Server } from './types';

const servers = ref<Server[]>([]);
const activeTab = ref(0);
const showAddDialog = ref(false);
let unlistenLog: UnlistenFn | null = null;
let unlistenTransfer: UnlistenFn | null = null;
let statsInterval: ReturnType<typeof setInterval> | null = null;
const draggingTab = ref<number | null>(null);
const currentWindowLabel = getCurrentWebviewWindow().label;

/** Fills in runtime-only fields that aren't persisted or come from transfers. */
function withServerDefaults(partial: Partial<Server>): Server {
  return {
    id: partial.id || crypto.randomUUID(),
    name: partial.name || '',
    path: partial.path || '',
    status: partial.status || 'stopped',
    cpu: partial.cpu || 0,
    memory: partial.memory || 0,
    players: partial.players || 0,
    maxPlayers: partial.maxPlayers || 20,
    tps: partial.tps || 0,
    logs: partial.logs || [],
    cpuHistory: partial.cpuHistory || [],
    memoryHistory: partial.memoryHistory || [],
    tpsHistory: partial.tpsHistory || [],
    playerList: partial.playerList || [],
    plugins: partial.plugins || [],
    configFiles: partial.configFiles || [],
    configContent: partial.configContent || '',
  };
}

function createServer(name: string, path: string): Server {
  return withServerDefaults({ name, path });
}

async function addServer(name: string, path: string) {
  const server = createServer(name, path);
  servers.value.push(server);
  activeTab.value = servers.value.length - 1;
  await saveConfig();
}

function removeServerByIndex(index: number) {
  servers.value.splice(index, 1);
  if (activeTab.value >= servers.value.length) {
    activeTab.value = Math.max(0, servers.value.length - 1);
  }
}

function removeServer(index: number) {
  removeServerByIndex(index);
  saveConfig();
}

function handleServerRemoved(serverId: string) {
  const index = servers.value.findIndex((s) => s.id === serverId);
  if (index !== -1) {
    removeServerByIndex(index);
  }
}

async function saveConfig() {
  try {
    await invoke('save_config', { servers: servers.value });
  } catch (e) {
    console.error('Failed to save config:', e);
  }
}

async function loadConfig() {
  try {
    const config = await invoke<Server[]>('load_config');

    const urlParams = new URLSearchParams(window.location.search);
    const serverId = urlParams.get('serverId');

    let filteredConfig = config;
    if (serverId) {
      filteredConfig = config.filter((s) => s.id === serverId);
    }

    servers.value = filteredConfig.map((s) => withServerDefaults(s));
  } catch (e) {
    console.error('Failed to load config:', e);
  }
}

async function loadConfigFiles(server: Server, dir: string) {
  try {
    const files = await invoke<string[]>('list_config_files', { path: server.path, dir });
    server.configFiles = files;
  } catch (e) {
    console.error('Failed to list config files:', e);
    server.configFiles = [];
  }
}

async function loadConfigFile(server: Server, dir: string, file: string) {
  try {
    const content = await invoke<string>('read_config_file', { path: server.path, dir, file });
    server.configContent = content;
  } catch (e) {
    console.error('Failed to read config file:', e);
  }
}

async function saveConfigFile(server: Server, dir: string, file: string, content: string) {
  try {
    await invoke('save_config_file', { path: server.path, dir, file, content });
  } catch (e) {
    console.error('Failed to save config file:', e);
  }
}

function onBodyDrag(event: DragEvent) {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move';
  }
}

async function startServer(server: Server) {
  server.status = 'starting';
  try {
    await invoke('start_server', { id: server.id, path: server.path });
    server.status = 'running';
  } catch (e) {
    console.error('Failed to start server:', e);
    server.status = 'stopped';
  }
}

async function stopServer(server: Server) {
  try {
    await invoke('stop_server', { id: server.id });
    server.status = 'stopped';
    server.cpu = 0;
    server.memory = 0;
    server.players = 0;
    server.tps = 0;
    server.cpuHistory.length = 0;
    server.memoryHistory.length = 0;
    server.tpsHistory.length = 0;
    server.playerList.length = 0;
  } catch (e) {
    console.error('Failed to stop server:', e);
  }
}

async function sendCommand(server: Server, command: string) {
  try {
    await invoke('send_command', { id: server.id, command });
  } catch (e) {
    console.error('Failed to send command:', e);
  }
}

onMounted(async () => {
  await loadConfig();

  unlistenTransfer = await listen<{ server: string; targetWindow: string; sourceWindow: string }>(
    'transfer-server',
    async (event) => {
      if (event.payload.targetWindow === currentWindowLabel) {
        const server = JSON.parse(event.payload.server) as Server;
        if (!servers.value.find((s) => s.id === server.id)) {
          servers.value.push(withServerDefaults({ ...server, logs: [] }));
          activeTab.value = servers.value.length - 1;
        }
      }
      if (event.payload.sourceWindow === currentWindowLabel) {
        const server = JSON.parse(event.payload.server) as Server;
        const index = servers.value.findIndex((s) => s.id === server.id);
        if (index !== -1) {
          removeServerByIndex(index);
          if (servers.value.length === 0) {
            const currentWindow = getCurrentWebviewWindow();
            if (currentWindow.label !== 'main') {
              currentWindow.close();
            }
          }
        }
      }
    }
  );

  await listen<{ x: number; y: number; server: string; sourceWindow: string }>(
    'check-drop-target',
    async (event) => {
      if (event.payload.sourceWindow === currentWindowLabel) return;

      const currentWindow = getCurrentWebviewWindow();
      const position = await currentWindow.outerPosition();
      const size = await currentWindow.outerSize();

      const { x, y } = event.payload;
      if (
        x >= position.x &&
        x <= position.x + size.width &&
        y >= position.y &&
        y <= position.y + size.height
      ) {
        await tauriEmit('transfer-server', {
          server: event.payload.server,
          targetWindow: currentWindowLabel,
          sourceWindow: event.payload.sourceWindow,
        });
      }
    }
  );

  unlistenLog = await listen<{ id: string; log: string }>('server-log', (event) => {
    const server = servers.value.find((s) => s.id === event.payload.id);
    if (server) {
      server.logs.push(event.payload.log);
      if (server.logs.length > 1000) {
        server.logs.shift();
      }
      parseLogForStats(server, event.payload.log);
    }
  });

  statsInterval = setInterval(updateStats, 2000);
});

onUnmounted(() => {
  if (unlistenLog) unlistenLog();
  if (unlistenTransfer) unlistenTransfer();
  if (statsInterval) clearInterval(statsInterval);
});

const MAX_HISTORY = 60; // 2 minutes at 2s intervals

async function updateStats() {
  for (const server of servers.value) {
    if (server.status === 'running') {
      try {
        const [cpu, memory] = await invoke<[number, number]>('get_server_stats', { id: server.id });
        server.cpu = cpu;
        server.memory = memory / 1024 / 1024; // Convert to MB

        server.cpuHistory.push(cpu);
        server.memoryHistory.push(server.memory);
        server.tpsHistory.push(server.tps);

        if (server.cpuHistory.length > MAX_HISTORY) server.cpuHistory.shift();
        if (server.memoryHistory.length > MAX_HISTORY) server.memoryHistory.shift();
        if (server.tpsHistory.length > MAX_HISTORY) server.tpsHistory.shift();
      } catch {
        // Ignore stats errors
      }
    }
  }
}

function parseLogForStats(server: Server, log: string) {
  const joinMatch = log.match(/(\w+) joined the game/);
  const leaveMatch = log.match(/(\w+) left the game/);
  if (joinMatch) {
    const playerName = joinMatch[1];
    server.players = Math.min(server.players + 1, server.maxPlayers);
    if (!server.playerList.includes(playerName)) {
      server.playerList.push(playerName);
    }
  } else if (leaveMatch) {
    const playerName = leaveMatch[1];
    server.players = Math.max(server.players - 1, 0);
    server.playerList = server.playerList.filter((p) => p !== playerName);
  }

  // Parse TPS from logs (adjust pattern based on Pumpkin's output format)
  const tpsMatch = log.match(/TPS[:\s]+(\d+(?:\.\d+)?)/i);
  if (tpsMatch) {
    server.tps = parseFloat(tpsMatch[1]);
  }
}
</script>

<template>
  <div
    :class="['app', { 'dragging-active': draggingTab !== null }]"
    @dragover="onBodyDrag"
    @dragenter="onBodyDrag"
  >
    <TabBar
      :servers="servers"
      :active-tab="activeTab"
      :current-window-label="currentWindowLabel"
      @update:active-tab="activeTab = $event"
      @remove="removeServer"
      @add="showAddDialog = true"
      @server-removed="handleServerRemoved"
    />

    <div v-if="servers.length > 0" class="content">
      <ServerTab
        :server="servers[activeTab]"
        @start="startServer(servers[activeTab])"
        @stop="stopServer(servers[activeTab])"
        @command="(cmd) => sendCommand(servers[activeTab], cmd)"
        @clear="servers[activeTab].logs = []"
        @load-config-files="(dir) => loadConfigFiles(servers[activeTab], dir)"
        @load-config-file="(dir, file) => loadConfigFile(servers[activeTab], dir, file)"
        @save-config-file="
          (dir, file, content) => saveConfigFile(servers[activeTab], dir, file, content)
        "
      />
    </div>

    <div v-else class="empty-state">
      <h2>No Servers</h2>
      <p>Click the + button to add a server</p>
    </div>

    <AddServerDialog v-model:show="showAddDialog" @add="addServer" />
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-dark);
}

.app.dragging-active,
.app.dragging-active * {
  cursor: grabbing !important;
}

.content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}
</style>
