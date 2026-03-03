<script setup lang="ts">
import { ref, computed } from 'vue';

const props = defineProps<{
  data: Record<string, unknown>;
  disabled: boolean;
  path: string[];
}>();

const emit = defineEmits<{
  'update:data': [value: Record<string, unknown>];
}>();

const collapsed = ref(new Set<string>());

const SLIDER_RANGES: Record<string, [number, number]> = {
  view_distance: [2, 32],
  simulation_distance: [2, 32],
  max_players: [1, 1000],
  port: [1, 65535],
  max_tick_time: [-1, 600000],
  op_permission_level: [1, 4],
  rate_limit: [0, 1000],
  compression_threshold: [-1, 65535],
};

function toggleCollapsed(key: string) {
  const k = [...props.path, key].join('.');
  if (collapsed.value.has(k)) {
    collapsed.value.delete(k);
  } else {
    collapsed.value.add(k);
  }
}

function isCollapsed(key: string): boolean {
  return collapsed.value.has([...props.path, key].join('.'));
}

function getSliderRange(key: string): [number, number] | null {
  const lower = key.toLowerCase().replace(/-/g, '_');
  return SLIDER_RANGES[lower] || null;
}

function updateValue(key: string, value: unknown) {
  const clone = structuredClone(props.data);
  clone[key] = value;
  emit('update:data', clone);
}

function updateArrayItem(key: string, index: number, value: unknown) {
  const arr = [...(props.data[key] as unknown[])];
  arr[index] = value;
  updateValue(key, arr);
}

const entries = computed(() => Object.entries(props.data));
</script>

<template>
  <div class="visual-editor">
    <div v-for="[key, value] in entries" :key="key" class="editor-field">
      <template v-if="typeof value === 'boolean'">
        <div class="field-row">
          <label class="field-label">{{ key }}</label>
          <button
            :class="['toggle-switch', { on: value }]"
            :disabled="disabled"
            @click="updateValue(key, !value)"
          >
            <span class="toggle-knob" />
          </button>
        </div>
      </template>

      <template v-else-if="typeof value === 'number'">
        <div class="field-row">
          <label class="field-label">{{ key }}</label>
          <div class="number-control">
            <input
              type="number"
              class="number-input"
              :value="value"
              :disabled="disabled"
              @input="updateValue(key, Number(($event.target as HTMLInputElement).value))"
            />
            <input
              v-if="getSliderRange(key)"
              type="range"
              class="number-slider"
              :min="getSliderRange(key)![0]"
              :max="getSliderRange(key)![1]"
              :value="value"
              :disabled="disabled"
              @input="updateValue(key, Number(($event.target as HTMLInputElement).value))"
            />
          </div>
        </div>
      </template>

      <template v-else-if="typeof value === 'string'">
        <div class="field-row">
          <label class="field-label">{{ key }}</label>
          <input
            type="text"
            class="text-input"
            :value="value"
            :disabled="disabled"
            @input="updateValue(key, ($event.target as HTMLInputElement).value)"
          />
        </div>
      </template>

      <template v-else-if="value === null">
        <div class="field-row">
          <label class="field-label">{{ key }}</label>
          <span class="null-label">(null)</span>
        </div>
      </template>

      <template v-else-if="Array.isArray(value)">
        <div class="section-header" @click="toggleCollapsed(key)">
          <svg
            :class="['disclosure', { open: !isCollapsed(key) }]"
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="currentColor"
          >
            <path d="M8 5l8 7-8 7z" />
          </svg>
          <span class="field-label">{{ key }}</span>
          <span class="section-badge">{{ (value as unknown[]).length }} items</span>
        </div>
        <div v-if="!isCollapsed(key)" class="section-body">
          <template v-for="(item, i) in (value as unknown[])" :key="i">
            <div v-if="typeof item === 'object' && item !== null && !Array.isArray(item)" class="nested-item">
              <span class="array-index">[{{ i }}]</span>
              <VisualConfigEditor
                :data="(item as Record<string, unknown>)"
                :disabled="disabled"
                :path="[...path, key, String(i)]"
                @update:data="(v) => updateArrayItem(key, i, v)"
              />
            </div>
            <div v-else class="field-row array-primitive">
              <span class="array-index">[{{ i }}]</span>
              <input
                v-if="typeof item === 'string'"
                type="text"
                class="text-input"
                :value="item"
                :disabled="disabled"
                @input="updateArrayItem(key, i, ($event.target as HTMLInputElement).value)"
              />
              <input
                v-else-if="typeof item === 'number'"
                type="number"
                class="number-input"
                :value="item"
                :disabled="disabled"
                @input="updateArrayItem(key, i, Number(($event.target as HTMLInputElement).value))"
              />
              <button
                v-else-if="typeof item === 'boolean'"
                :class="['toggle-switch', { on: item }]"
                :disabled="disabled"
                @click="updateArrayItem(key, i, !item)"
              >
                <span class="toggle-knob" />
              </button>
              <span v-else class="null-label">{{ String(item) }}</span>
            </div>
          </template>
        </div>
      </template>

      <template v-else-if="typeof value === 'object' && value !== null">
        <div class="section-header" @click="toggleCollapsed(key)">
          <svg
            :class="['disclosure', { open: !isCollapsed(key) }]"
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="currentColor"
          >
            <path d="M8 5l8 7-8 7z" />
          </svg>
          <span class="field-label">{{ key }}</span>
          <span class="section-badge">{{ Object.keys(value).length }} keys</span>
        </div>
        <div v-if="!isCollapsed(key)" class="section-body">
          <VisualConfigEditor
            :data="(value as Record<string, unknown>)"
            :disabled="disabled"
            :path="[...path, key]"
            @update:data="(v) => updateValue(key, v)"
          />
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.visual-editor {
  display: flex;
  flex-direction: column;
}

.editor-field {
  border-bottom: 1px solid var(--border-subtle);
}

.editor-field:last-child {
  border-bottom: none;
}

.field-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 14px;
  gap: 12px;
  min-height: 38px;
}

.field-row.array-primitive {
  padding-left: 28px;
}

.field-label {
  color: var(--text-secondary);
  font-size: 12.5px;
  font-family: var(--font-mono);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.toggle-switch {
  position: relative;
  width: 36px;
  height: 20px;
  background: var(--bg-active);
  border: 1px solid var(--border-default);
  border-radius: 10px;
  cursor: pointer;
  transition: all var(--transition-fast);
  flex-shrink: 0;
  padding: 0;
}

.toggle-switch.on {
  background: var(--color-success);
  border-color: var(--color-success);
}

.toggle-switch:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  background: var(--text-primary);
  border-radius: 50%;
  transition: transform var(--transition-fast);
}

.toggle-switch.on .toggle-knob {
  transform: translateX(16px);
}

.number-control {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

.number-input {
  width: 90px;
  padding: 4px 8px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 12px;
  text-align: right;
}

.number-input:disabled {
  color: var(--text-faint);
  cursor: not-allowed;
}

.number-slider {
  width: 100px;
  accent-color: var(--accent);
}

.number-slider:disabled {
  opacity: 0.4;
}

.text-input {
  flex: 1;
  min-width: 120px;
  max-width: 320px;
  padding: 4px 8px;
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 12px;
}

.text-input:disabled {
  color: var(--text-faint);
  cursor: not-allowed;
}

.null-label {
  color: var(--text-faint);
  font-size: 12px;
  font-style: italic;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  cursor: pointer;
  user-select: none;
  transition: background var(--transition-fast);
}

.section-header:hover {
  background: var(--bg-hover);
}

.disclosure {
  color: var(--text-faint);
  transition: transform var(--transition-fast);
  flex-shrink: 0;
}

.disclosure.open {
  transform: rotate(90deg);
}

.section-badge {
  font-size: 10px;
  color: var(--text-faint);
  background: var(--bg-surface);
  padding: 1px 6px;
  border-radius: 4px;
  margin-left: 4px;
}

.section-body {
  padding-left: 16px;
  border-left: 1px solid var(--border-subtle);
  margin-left: 20px;
}

.array-index {
  color: var(--text-faint);
  font-size: 11px;
  font-family: var(--font-mono);
  flex-shrink: 0;
  min-width: 28px;
}

.nested-item {
  padding: 4px 14px 4px 28px;
}

.nested-item .array-index {
  display: block;
  margin-bottom: 4px;
}
</style>
