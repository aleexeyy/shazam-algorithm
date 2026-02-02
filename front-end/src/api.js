function normalizePath(path) {
  if (!path) return "/";
  return path.startsWith("/") ? path : `/${path}`;
}

export function apiUrl(path) {
  const base = import.meta.env.VITE_API_URL;
  const normalized = normalizePath(path);
  if (!base) return normalized;
  return new URL(normalized, base).toString();
}

