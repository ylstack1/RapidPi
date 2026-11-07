/**
 * Loads remote control (server, Noise, QR code, the background service) only when `/remote-control` or `rapid-pi service`
 * runs (see entry.ts).
 */
export const loadRemoteControl = () => import("./entry.ts");
