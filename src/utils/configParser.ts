import * as TOML from 'smol-toml';

export type ConfigFormat = 'toml' | 'json' | 'unknown';

export function detectFormat(filename: string): ConfigFormat {
  const lower = filename.toLowerCase();
  if (lower.endsWith('.toml')) return 'toml';
  if (lower.endsWith('.json')) return 'json';
  return 'unknown';
}

export function parseConfig(content: string, format: ConfigFormat): Record<string, unknown> | null {
  try {
    if (format === 'toml') return TOML.parse(content) as Record<string, unknown>;
    if (format === 'json') return JSON.parse(content);
    return null;
  } catch {
    return null;
  }
}

export function serializeConfig(data: Record<string, unknown>, format: ConfigFormat): string {
  if (format === 'toml') return TOML.stringify(data);
  if (format === 'json') return JSON.stringify(data, null, 2);
  return '';
}
