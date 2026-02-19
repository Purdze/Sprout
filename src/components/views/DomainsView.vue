<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-shell';
import type { CloudflareZone, CloudflareDnsRecord, CreateSrvForm } from '../../types';

defineProps<{
  server: import('../../types').Server;
}>();

const token = ref('');
const tokenInput = ref('');
const tokenError = ref('');
const tokenLoading = ref(false);

const zones = ref<CloudflareZone[]>([]);
const selectedZone = ref<CloudflareZone | null>(null);
const zonesLoading = ref(false);

const records = ref<CloudflareDnsRecord[]>([]);
const recordsLoading = ref(false);

const showCreateForm = ref(false);
const createForm = ref<CreateSrvForm>({ subdomain: '', target: '', port: 25565 });
const createLoading = ref(false);
const createError = ref('');

async function connectToken() {
  tokenError.value = '';
  tokenLoading.value = true;
  try {
    await invoke('verify_and_save_cf_token', { token: tokenInput.value });
    token.value = tokenInput.value;
    tokenInput.value = '';
    await loadZones();
  } catch (e: any) {
    tokenError.value = e.toString();
  } finally {
    tokenLoading.value = false;
  }
}

async function disconnect() {
  try {
    await invoke('delete_cf_token');
  } catch {
    // ignored
  }
  token.value = '';
  zones.value = [];
  selectedZone.value = null;
  records.value = [];
}

async function loadZones() {
  zonesLoading.value = true;
  try {
    zones.value = await invoke('list_cf_zones', { token: token.value });
  } catch {
    zones.value = [];
  } finally {
    zonesLoading.value = false;
  }
}

async function selectZone(zone: CloudflareZone) {
  selectedZone.value = zone;
  showCreateForm.value = false;
  recordsLoading.value = true;
  try {
    records.value = await invoke('list_cf_srv_records', {
      token: token.value,
      zoneId: zone.id,
    });
  } catch {
    records.value = [];
  } finally {
    recordsLoading.value = false;
  }
}

async function createRecord() {
  if (!selectedZone.value) return;
  createError.value = '';
  createLoading.value = true;
  try {
    const record: CloudflareDnsRecord = await invoke('create_cf_srv_record', {
      token: token.value,
      zoneId: selectedZone.value.id,
      zoneName: selectedZone.value.name,
      subdomain: createForm.value.subdomain,
      target: createForm.value.target,
      port: createForm.value.port,
    });
    records.value.push(record);
    showCreateForm.value = false;
    createForm.value = { subdomain: '', target: '', port: 25565 };
  } catch (e: any) {
    createError.value = e.toString();
  } finally {
    createLoading.value = false;
  }
}

async function deleteRecord(record: CloudflareDnsRecord) {
  if (!selectedZone.value) return;
  try {
    await invoke('delete_cf_dns_record', {
      token: token.value,
      zoneId: selectedZone.value.id,
      recordId: record.id,
    });
    records.value = records.value.filter((r) => r.id !== record.id);
  } catch {
    // ignored
  }
}

onMounted(async () => {
  try {
    const saved: string = await invoke('load_cf_token');
    if (saved) {
      token.value = saved;
      await loadZones();
    }
  } catch {
    // ignored
  }
});
</script>

<template>
  <div class="domains-view">
    <div v-if="!token" class="domains-connect">
      <div class="connect-card">
        <FontAwesomeIcon :icon="['fab', 'cloudflare']" class="cf-icon" />
        <h2>Connect Cloudflare</h2>
        <p>Enter your Cloudflare API token to manage DNS records for your Minecraft server.</p>
        <div class="permissions-box">
          <span class="permissions-title">Required token permissions</span>
          <div class="permission-row">
            <span class="perm-path"
              >Zone <span class="perm-arrow">→</span> Zone
              <span class="perm-arrow">→</span> Read</span
            >
          </div>
          <div class="permission-row">
            <span class="perm-path"
              >Zone <span class="perm-arrow">→</span> DNS
              <span class="perm-arrow">→</span> Edit</span
            >
          </div>
          <div class="permission-row">
            <span class="perm-label">Zone Resources:</span>
            <span class="perm-value">All zones</span>
            <span class="perm-note">or specific zone for a single domain</span>
          </div>
        </div>
        <p class="connect-hint">
          <span class="connect-link" @click="open('https://dash.cloudflare.com/profile/api-tokens')"
            >Create a token on Cloudflare</span
          >
        </p>
        <div class="connect-form">
          <input
            v-model="tokenInput"
            type="password"
            placeholder="Cloudflare API Token"
            class="connect-input"
            @keyup.enter="connectToken"
          />
          <button class="connect-btn" :disabled="!tokenInput || tokenLoading" @click="connectToken">
            {{ tokenLoading ? 'Verifying...' : 'Connect' }}
          </button>
        </div>
        <div v-if="tokenError" class="connect-error">{{ tokenError }}</div>
      </div>
    </div>

    <template v-else>
      <div class="domains-sidebar">
        <div class="domains-sidebar-header">
          <span class="sidebar-title">Zones</span>
          <button class="disconnect-btn" @click="disconnect">Disconnect</button>
        </div>
        <div class="domains-zone-list">
          <div v-if="zonesLoading" class="zones-loading">Loading zones...</div>
          <div
            v-for="zone in zones"
            :key="zone.id"
            :class="['zone-item', { active: selectedZone?.id === zone.id }]"
            @click="selectZone(zone)"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <circle cx="12" cy="12" r="10" />
              <path
                d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"
              />
            </svg>
            <span>{{ zone.name }}</span>
          </div>
          <div v-if="!zonesLoading && zones.length === 0" class="zones-empty">No zones found</div>
        </div>
      </div>

      <div class="domains-main">
        <div v-if="!selectedZone" class="domains-empty">Select a zone to manage DNS records</div>
        <template v-else>
          <div class="domains-main-header">
            <span class="main-title">SRV Records — {{ selectedZone.name }}</span>
            <button class="add-record-btn" @click="showCreateForm = !showCreateForm">
              {{ showCreateForm ? 'Cancel' : '+ Add Record' }}
            </button>
          </div>

          <div v-if="showCreateForm" class="create-form">
            <div class="form-row">
              <div class="form-group">
                <label>Subdomain <span class="hint">(leave empty for root)</span></label>
                <div class="subdomain-input">
                  <input
                    v-model="createForm.subdomain"
                    type="text"
                    placeholder="play"
                    class="form-input subdomain-field"
                  />
                  <span class="subdomain-suffix">.{{ selectedZone.name }}</span>
                </div>
              </div>
              <div class="form-group">
                <label>Target (IP/hostname)</label>
                <input
                  v-model="createForm.target"
                  type="text"
                  placeholder="mc.example.com"
                  class="form-input"
                />
              </div>
              <div class="form-group form-group-small">
                <label>Port</label>
                <input v-model.number="createForm.port" type="number" class="form-input" />
              </div>
              <button
                class="create-btn"
                :disabled="!createForm.target || createLoading"
                @click="createRecord"
              >
                {{ createLoading ? 'Creating...' : 'Create' }}
              </button>
            </div>
            <div v-if="createError" class="create-error">{{ createError }}</div>
          </div>

          <div class="records-list">
            <div v-if="recordsLoading" class="records-loading">Loading records...</div>
            <div v-else-if="records.length === 0" class="records-empty">
              No SRV records found. Click "+ Add Record" to create one.
            </div>
            <div v-for="record in records" :key="record.id" class="record-item">
              <div class="record-info">
                <div class="record-name">{{ record.name }}</div>
                <div class="record-content">{{ record.content }}</div>
                <div v-if="record.data" class="record-details">
                  Port: {{ record.data.port }} · Target: {{ record.data.target }} · Priority:
                  {{ record.data.priority }} · Weight: {{ record.data.weight }}
                </div>
              </div>
              <button class="delete-btn" @click="deleteRecord(record)">Delete</button>
            </div>
          </div>
        </template>
      </div>
    </template>
  </div>
</template>

<style scoped>
.domains-view {
  flex: 1;
  display: flex;
  overflow: hidden;
  background: var(--bg-darkest);
}

.domains-connect {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.connect-card {
  background: var(--bg-dark);
  border: 1px solid var(--bg-light);
  border-radius: 8px;
  padding: 32px;
  max-width: 420px;
  width: 100%;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.cf-icon {
  font-size: 48px;
  color: #f6821f;
}

.connect-card h2 {
  color: var(--text-primary);
  font-size: 18px;
  margin: 0;
}

.connect-card p {
  color: var(--text-tertiary);
  font-size: 13px;
  margin: 0;
  line-height: 1.5;
}

.permissions-box {
  width: 100%;
  background: var(--bg-medium);
  border: 1px solid var(--bg-light);
  border-radius: 6px;
  padding: 10px 14px;
  text-align: center;
}

.permissions-title {
  color: var(--text-tertiary);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  display: block;
  margin-bottom: 8px;
}

.permission-row {
  padding: 4px 0;
  font-size: 13px;
}

.perm-path {
  color: var(--text-secondary);
  font-family: var(--font-mono);
}

.perm-arrow {
  color: var(--text-dim);
  margin: 0 2px;
}

.perm-label {
  color: var(--text-tertiary);
}

.perm-value {
  color: var(--text-secondary);
  font-family: var(--font-mono);
}

.perm-note {
  color: var(--text-dim);
  font-size: 11px;
  font-style: italic;
}

.connect-hint {
  color: var(--text-muted) !important;
  font-size: 12px !important;
}

.connect-link {
  color: var(--color-primary);
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 2px;
}

.connect-link:hover {
  color: #fb923c;
}

.connect-form {
  display: flex;
  gap: 8px;
  width: 100%;
  margin-top: 4px;
}

.connect-input {
  flex: 1;
  padding: 8px 12px;
  background: var(--bg-medium);
  border: 1px solid var(--bg-light);
  border-radius: 4px;
  color: var(--text-primary);
  font-size: 13px;
}

.connect-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.connect-btn {
  padding: 8px 16px;
  background: var(--color-primary);
  border: none;
  border-radius: 4px;
  color: var(--text-primary);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
}

.connect-btn:hover {
  background: var(--color-primary-hover);
}

.connect-btn:disabled {
  background: var(--bg-light);
  color: var(--text-muted);
  cursor: not-allowed;
}

.connect-error {
  color: var(--color-danger);
  font-size: 12px;
  text-align: left;
  width: 100%;
}

.domains-sidebar {
  width: 200px;
  background: var(--bg-dark);
  border-right: 1px solid var(--bg-light);
  display: flex;
  flex-direction: column;
}

.domains-sidebar-header {
  padding: 8px;
  border-bottom: 1px solid var(--bg-light);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.sidebar-title {
  color: var(--text-tertiary);
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.disconnect-btn {
  padding: 4px 8px;
  background: transparent;
  border: 1px solid var(--bg-hover);
  border-radius: 3px;
  color: var(--text-tertiary);
  font-size: 11px;
  cursor: pointer;
}

.disconnect-btn:hover {
  border-color: var(--color-danger);
  color: var(--color-danger);
}

.domains-zone-list {
  flex: 1;
  overflow-y: auto;
}

.zone-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 13px;
}

.zone-item:hover {
  background: var(--bg-medium);
  color: var(--text-secondary);
}

.zone-item.active {
  background: var(--bg-light);
  color: var(--text-primary);
}

.zones-loading,
.zones-empty {
  padding: 12px;
  color: var(--text-faint);
  font-size: 12px;
  font-style: italic;
}

.domains-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.domains-empty {
  color: var(--text-faint);
  font-style: italic;
  text-align: center;
  padding: 40px;
}

.domains-main-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  background: var(--bg-dark);
  border-bottom: 1px solid var(--bg-light);
}

.main-title {
  color: var(--text-primary);
  font-size: 14px;
}

.add-record-btn {
  padding: 6px 14px;
  background: var(--color-primary);
  border: none;
  border-radius: 4px;
  color: var(--text-primary);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.add-record-btn:hover {
  background: var(--color-primary-hover);
}

.create-form {
  padding: 12px;
  background: var(--bg-dark);
  border-bottom: 1px solid var(--bg-light);
}

.form-row {
  display: flex;
  gap: 12px;
  align-items: flex-end;
}

.form-group {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-group-small {
  flex: 0 0 80px;
}

.form-group label {
  color: var(--text-tertiary);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.input-hint {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.hint {
  color: var(--text-dim);
  font-size: 11px;
}

.subdomain-input {
  display: flex;
  align-items: center;
  background: var(--bg-medium);
  border: 1px solid var(--bg-light);
  border-radius: 4px;
}

.subdomain-input:focus-within {
  border-color: var(--color-primary);
}

.subdomain-field {
  border: none !important;
  background: transparent !important;
  border-radius: 0 !important;
  flex: 1;
  min-width: 0;
}

.subdomain-field:focus {
  outline: none;
}

.subdomain-suffix {
  color: var(--text-dim);
  font-size: 13px;
  padding-right: 10px;
  white-space: nowrap;
}

.form-input {
  padding: 6px 10px;
  background: var(--bg-medium);
  border: 1px solid var(--bg-light);
  border-radius: 4px;
  color: var(--text-primary);
  font-size: 13px;
}

.form-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.create-btn {
  padding: 6px 14px;
  background: var(--color-success);
  border: none;
  border-radius: 4px;
  color: #000;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  align-self: flex-end;
}

.create-btn:hover {
  background: var(--color-success-hover);
}

.create-btn:disabled {
  background: var(--bg-light);
  color: var(--text-muted);
  cursor: not-allowed;
}

.create-error {
  color: var(--color-danger);
  font-size: 12px;
  margin-top: 8px;
}

.records-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.records-loading,
.records-empty {
  color: var(--text-faint);
  font-style: italic;
  padding: 24px;
  text-align: center;
  font-size: 13px;
}

.record-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  background: var(--bg-dark);
  border: 1px solid var(--bg-light);
  border-radius: 6px;
  margin-bottom: 8px;
}

.record-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.record-name {
  color: var(--text-primary);
  font-size: 13px;
  font-family: var(--font-mono);
}

.record-content {
  color: var(--text-tertiary);
  font-size: 12px;
  font-family: var(--font-mono);
}

.record-details {
  color: var(--text-dim);
  font-size: 11px;
}

.delete-btn {
  padding: 4px 10px;
  background: transparent;
  border: 1px solid var(--bg-hover);
  border-radius: 3px;
  color: var(--text-tertiary);
  font-size: 11px;
  cursor: pointer;
  flex-shrink: 0;
  margin-left: 12px;
}

.delete-btn:hover {
  border-color: var(--color-danger);
  color: var(--color-danger);
}
</style>
