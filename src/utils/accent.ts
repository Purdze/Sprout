export const ACCENT_COLORS = [
  { name: 'Orange', value: '#f97316', hover: '#fb923c' },
  { name: 'Blue', value: '#3b82f6', hover: '#60a5fa' },
  { name: 'Green', value: '#22c55e', hover: '#4ade80' },
  { name: 'Purple', value: '#a855f7', hover: '#c084fc' },
  { name: 'Rose', value: '#f43f5e', hover: '#fb7185' },
  { name: 'White', value: '#a1a1aa', hover: '#d4d4d8' },
] as const;

export type AccentColor = (typeof ACCENT_COLORS)[number];

const PRESET_MAP = new Map<string, AccentColor>(ACCENT_COLORS.map((c) => [c.value, c]));

export function applyAccent(colorValue: string | null) {
  const root = document.documentElement;
  const preset = colorValue ? PRESET_MAP.get(colorValue) : null;
  if (preset) {
    root.style.setProperty('--accent', preset.value);
    root.style.setProperty('--accent-hover', preset.hover);
    root.style.setProperty('--accent-muted', preset.value + '1f');
    root.style.setProperty('--accent-ring', preset.value + '59');
  } else {
    root.style.removeProperty('--accent');
    root.style.removeProperty('--accent-hover');
    root.style.removeProperty('--accent-muted');
    root.style.removeProperty('--accent-ring');
  }
}
