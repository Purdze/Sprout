export interface Plugin {
  name: string;
  version: string;
  enabled: boolean;
}

export interface CloudflareZone {
  id: string;
  name: string;
}

export interface CloudflareSrvData {
  service: string;
  proto: string;
  name: string;
  priority: number;
  weight: number;
  port: number;
  target: string;
}

export interface CloudflareDnsRecord {
  id: string;
  record_type: string;
  name: string;
  content: string;
  data: CloudflareSrvData | null;
}

export interface CreateSrvForm {
  subdomain: string;
  target: string;
  port: number;
}

export interface SavedCommand {
  name: string;
  command: string;
}

export interface Server {
  id: string;
  name: string;
  path: string;
  logs: string[];
  status: 'stopped' | 'running' | 'starting';
  cpu: number;
  memory: number;
  players: number;
  maxPlayers: number;
  tps: number;
  cpuHistory: number[];
  memoryHistory: number[];
  tpsHistory: number[];
  playerList: string[];
  plugins: Plugin[];
  configFiles: string[];
  configContent: string;
  savedCommands: SavedCommand[];
}

export interface InventorySlot {
  slot: number;
  id: string;
  count: number;
  name: string;
}

export interface RconConfig {
  enabled: boolean;
  port: number;
  password: string;
}

export interface PlayerDetails {
  name: string;
  uuid: string;
  health: number;
  maxHealth: number;
  food: number;
  xpLevel: number;
  gameMode: string;
  inventory: InventorySlot[];
  enderChest: InventorySlot[];
  dimension: string;
  posX: number;
  posY: number;
  posZ: number;
  lastSlept: string | null;
  lastDeath: string | null;
  playtimeTicks: number;
  deaths: number;
  playerKills: number;
  mobKills: number;
  itemsPickedUp: number;
  itemsUsed: number;
  distanceCm: number;
  isOp: boolean;
}
