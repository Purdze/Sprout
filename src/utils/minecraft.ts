export const MINECRAFT_ASSET_BASE =
  'https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.21.4/assets/minecraft/textures';

export function getPlayerAvatarUrl(name: string, size: number): string {
  return `https://mc-heads.net/avatar/${name}/${size}`;
}

export function formatMinecraftId(id: string): string {
  return id
    .replace('minecraft:', '')
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

export function formatDimension(dim: string): string {
  const name = dim.replace('minecraft:', '');
  switch (name) {
    case 'overworld':
      return 'Overworld';
    case 'the_nether':
      return 'The Nether';
    case 'the_end':
      return 'The End';
    default:
      return formatMinecraftId(dim);
  }
}
