import { writable } from 'svelte/store';

export type ViewId = 'overview' | 'processes' | 'network' | 'containers' | 'alerts';

export interface ViewDef {
  id: ViewId;
  label: string;
  shortcut: string;
}

export const views: ViewDef[] = [
  { id: 'overview', label: 'Overview', shortcut: '1' },
  { id: 'processes', label: 'Processes', shortcut: '2' },
  { id: 'network', label: 'Network', shortcut: '3' },
  { id: 'containers', label: 'Containers', shortcut: '4' },
  { id: 'alerts', label: 'Alerts', shortcut: '5' },
];

export const activeView = writable<ViewId>('overview');

export function setView(view: ViewId): void {
  activeView.set(view);
}
