// apps/website/src/features/content/utils/dates.ts

/**
 * Format a date to a localized date string
 */
export function formatContentDate(date: string | Date | undefined | null): string | null {
  if (!date) return null;
  
  if (typeof date === 'string') {
    return new Date(date).toLocaleDateString();
  }
  
  return date.toLocaleDateString();
}

/**
 * Format a date to an ISO string
 */
export function formatContentDateISO(date: string | Date | undefined | null): string | null {
  if (!date) return null;
  
  if (typeof date === 'string') {
    return new Date(date).toISOString();
  }
  
  return date.toISOString();
}





