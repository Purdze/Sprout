<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { getVersion, getTauriVersion } from '@tauri-apps/api/app';
import { open } from '@tauri-apps/plugin-shell';

defineProps<{
  show: boolean;
  updateAvailable: boolean;
  checkingUpdate: boolean;
  upToDate: boolean;
}>();

const emit = defineEmits<{
  'update:show': [value: boolean];
  checkUpdate: [];
  openUpdate: [];
}>();

const appVersion = ref('');
const tauriVersion = ref('');
const commitHash = __COMMIT_HASH__;

onMounted(async () => {
  appVersion.value = await getVersion();
  tauriVersion.value = await getTauriVersion();
});
</script>

<template>
  <div v-if="show" class="dialog-overlay" @click="emit('update:show', false)">
    <div class="about compact-dialog" @click.stop>
      <div class="top">
        <span class="name">Sprout</span>
        <span class="meta">
          {{ appVersion }} &middot; tauri {{ tauriVersion }} &middot;
          <span class="mono">{{ commitHash }}</span>
        </span>
      </div>

      <div class="actions">
        <button class="link" @click="open('https://github.com/Purdze/Sprout')">GitHub</button>
        <button class="link" @click="open('https://discord.gg/qsRhJUP4q5')">Discord</button>
        <span class="spacer" />
        <button v-if="updateAvailable" class="update-link available" @click="emit('openUpdate')">
          Update available
        </button>
        <span v-else-if="upToDate" class="up-to-date">Up to date</span>
        <button v-else class="update-link" :disabled="checkingUpdate" @click="emit('checkUpdate')">
          <svg
            v-if="checkingUpdate"
            class="spin"
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
          >
            <path d="M21 12a9 9 0 1 1-6.219-8.56" />
          </svg>
          {{ checkingUpdate ? 'Checking' : 'Check for updates' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.top {
  margin-bottom: 12px;
}

.name {
  display: block;
  font-weight: 700;
  font-size: 14px;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.mono {
  font-family: var(--font-mono);
  font-size: 11px;
}

.link {
  color: var(--text-tertiary);
}

.link:hover {
  color: var(--text-primary);
}

.update-link {
  display: flex;
  align-items: center;
  gap: 5px;
}

.update-link:not(.available) {
  color: var(--text-faint);
}

.update-link:not(.available):hover:not(:disabled) {
  color: var(--text-secondary);
}

.update-link.available {
  color: var(--color-info);
}

.update-link.available:hover {
  filter: brightness(1.2);
}

.up-to-date {
  color: var(--color-success);
  font-size: 12px;
}

.spin {
  animation: spin 0.8s linear infinite;
}
</style>
