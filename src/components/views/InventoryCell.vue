<script setup lang="ts">
import { MINECRAFT_ASSET_BASE, formatMinecraftId } from '../../utils/minecraft';
import type { InventorySlot } from '../../types';

defineProps<{
  item: InventorySlot | null;
  variant?: 'cell' | 'equip';
}>();

function itemIconUrl(id: string): string {
  const name = id.replace('minecraft:', '');
  return `${MINECRAFT_ASSET_BASE}/item/${name}.png`;
}

function onIconError(e: Event) {
  const img = e.target as HTMLImageElement;
  const src = img.src;
  if (src.includes('/item/')) {
    img.src = src.replace('/item/', '/block/');
  } else {
    img.style.display = 'none';
    const fallback = img.nextElementSibling as HTMLElement | null;
    if (fallback) fallback.style.display = '';
  }
}
</script>

<template>
  <div
    :class="[variant === 'equip' ? 'equip-cell' : 'inv-cell', { empty: !item }]"
    :title="item ? formatMinecraftId(item.id) : ''"
  >
    <template v-if="item">
      <img class="item-icon" :src="itemIconUrl(item.id)" :alt="item.name" @error="onIconError" />
      <span class="cell-fallback" style="display: none">{{ formatMinecraftId(item.id) }}</span>
      <span v-if="item.count > 1" class="cell-count">{{ item.count }}</span>
    </template>
  </div>
</template>

<style scoped>
.inv-cell,
.equip-cell {
  background: var(--bg-dark);
  border: 1px solid #2a2a2a;
  border-radius: 3px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.inv-cell {
  aspect-ratio: 1;
}
.equip-cell {
  flex: 1;
}

.inv-cell.empty,
.equip-cell.empty {
  background: #131313;
  border-color: #1e1e1e;
}

.item-icon {
  width: 70%;
  height: 70%;
  object-fit: contain;
  image-rendering: pixelated;
}

.cell-fallback {
  font-size: 9px;
  color: var(--text-tertiary);
  text-align: center;
  line-height: 1.1;
  word-break: break-word;
  padding: 2px;
}

.cell-count {
  position: absolute;
  bottom: 0;
  right: 2px;
  font-size: 10px;
  font-weight: 700;
  color: var(--text-primary);
  text-shadow:
    1px 1px 0 #000,
    -1px -1px 0 #000,
    1px -1px 0 #000,
    -1px 1px 0 #000;
  line-height: 1;
}
</style>
