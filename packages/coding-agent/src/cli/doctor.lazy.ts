/** Loads `rapid-pi doctor` only when it is run (see doctor.ts). */
export const loadDoctor = () => import("./doctor.ts");
