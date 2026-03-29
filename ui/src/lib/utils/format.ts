/**
 * Format bytes to human-readable string
 * e.g., 1536000000 -> "1.5 GB"
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';

  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const value = bytes / Math.pow(k, i);

  if (i === 0) {
    return `${bytes} B`;
  }

  return `${value.toFixed(i > 2 ? 1 : 0)} ${units[i]}`;
}

/**
 * Format bytes to GB with 1 decimal
 * e.g., 1536000000 -> "1.5"
 */
export function formatBytesToGB(bytes: number): string {
  const gb = bytes / (1024 * 1024 * 1024);
  return gb.toFixed(1);
}

/**
 * Format percentage with 1 decimal
 * e.g., 85.234 -> "85.2%"
 */
export function formatPercent(value: number): string {
  return `${value.toFixed(1)}%`;
}

/**
 * Format duration in seconds to human-readable string
 * e.g., 9000 -> "2h 30m", 280800 -> "3d 6h"
 */
export function formatDuration(seconds: number): string {
  if (seconds < 60) {
    return `${Math.floor(seconds)}s`;
  }

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes}m`;
  }

  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  if (hours < 24) {
    return remainingMinutes > 0 ? `${hours}h ${remainingMinutes}m` : `${hours}h`;
  }

  const days = Math.floor(hours / 24);
  const remainingHours = hours % 24;
  if (remainingHours > 0) {
    return `${days}d ${remainingHours}h`;
  }
  return `${days}d`;
}

/**
 * Format uptime in seconds to days, hours, minutes
 * e.g., 86400 -> "1 day, 0 hours, 0 minutes"
 */
export function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  const parts: string[] = [];
  if (days > 0) parts.push(`${days} day${days > 1 ? 's' : ''}`);
  if (hours > 0) parts.push(`${hours} hour${hours > 1 ? 's' : ''}`);
  if (minutes > 0 || parts.length === 0) parts.push(`${minutes} min${minutes > 1 ? 's' : ''}`);

  return parts.join(', ');
}

/**
 * Get color for percentage value
 * Returns CSS variable name based on value
 */
export function getPercentColor(value: number): string {
  if (value >= 80) return 'var(--danger)';
  if (value >= 60) return 'var(--warning)';
  return 'var(--success)';
}

/**
 * Format number with commas
 * e.g., 1234567 -> "1,234,567"
 */
export function formatNumber(num: number): string {
  return num.toLocaleString('en-US');
}

/**
 * Truncate string to max length with ellipsis
 */
export function truncate(str: string, maxLength: number): string {
  if (str.length <= maxLength) return str;
  return str.slice(0, maxLength - 3) + '...';
}

/**
 * Get relative time string
 * e.g., timestamp -> "2 minutes ago"
 */
export function getRelativeTime(timestamp: string | number | Date): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffSec < 10) return 'just now';
  if (diffSec < 60) return `${diffSec}s ago`;
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHour < 24) return `${diffHour}h ago`;
  return `${diffDay}d ago`;
}
