// Input validation utilities

/**
 * Maximum allowed code input size (50KB)
 */
export const MAX_CODE_SIZE = 50 * 1024; // 50KB in bytes

/**
 * Validate code input size
 */
export function validateCodeSize(code: string): { valid: boolean; size: number; maxSize: number; error?: string } {
  const size = new Blob([code]).size;
  const maxSize = MAX_CODE_SIZE;
  
  if (size > maxSize) {
    return {
      valid: false,
      size,
      maxSize,
      error: `Code size (${formatBytes(size)}) exceeds maximum allowed size (${formatBytes(maxSize)})`
    };
  }
  
  return { valid: true, size, maxSize };
}

/**
 * Format bytes to human-readable string
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

/**
 * Truncate code to maximum size if needed
 */
export function truncateCode(code: string, maxSize: number = MAX_CODE_SIZE): string {
  if (new Blob([code]).size <= maxSize) {
    return code;
  }
  
  // Binary search for truncation point
  let low = 0;
  let high = code.length;
  let result = '';
  
  while (low < high) {
    const mid = Math.floor((low + high) / 2);
    const testCode = code.substring(0, mid);
    const size = new Blob([testCode]).size;
    
    if (size <= maxSize) {
      result = testCode;
      low = mid + 1;
    } else {
      high = mid;
    }
  }
  
  return result;
}

