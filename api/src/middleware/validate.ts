const MD5_REGEX = /^[a-f0-9]{32}$/i;
const SHA256_REGEX = /^[a-f0-9]{64}$/i;

export function isValidMd5Hash(hash: string): boolean {
  return MD5_REGEX.test(hash);
}

export function isValidSha256Hash(hash: string): boolean {
  return SHA256_REGEX.test(hash);
}

export function isValidHash(hash: string): boolean {
  return isValidMd5Hash(hash) || isValidSha256Hash(hash);
}

export function sanitizePagination(
  limit: string | undefined,
  offset: string | undefined,
  maxLimit = 1000,
): { limit: number; offset: number } {
  const parsedLimit = Math.min(Math.max(1, Number(limit) || 100), maxLimit);
  const parsedOffset = Math.max(0, Number(offset) || 0);
  return { limit: parsedLimit, offset: parsedOffset };
}
