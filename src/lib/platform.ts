/**
 * The desktop webview reports a macOS user agent, so the UI can pick the
 * physical layout and key names that match the host keyboard without another
 * IPC round trip.
 */
export const IS_MAC = typeof navigator !== 'undefined' && /macintosh|mac os x/i.test(navigator.userAgent)
