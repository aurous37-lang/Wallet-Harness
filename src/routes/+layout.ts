// This is a Tauri application. There is no Node server at runtime — the
// built static files are loaded directly into the Tauri webview. Disable
// SSR and prerendering at the root.
export const ssr = false;
export const prerender = false;
export const trailingSlash = 'never';
