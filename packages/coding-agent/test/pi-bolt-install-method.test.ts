import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import {
	rapidPiInstallDir,
	rapidPiInstallerProcess,
	rapidPiInstallMethod,
	rapidPiUpdateEnvironment,
} from "../src/rapid-pi.ts";

const originalExecPath = process.execPath;
const originalNpm = process.env.RAPIDPI_NPM;

afterEach(() => {
	Object.defineProperty(process, "execPath", { configurable: true, writable: true, value: originalExecPath });
	if (originalNpm === undefined) delete process.env.RAPIDPI_NPM;
	else process.env.RAPIDPI_NPM = originalNpm;
});

function setExecPath(value: string): void {
	Object.defineProperty(process, "execPath", { configurable: true, writable: true, value });
}

describe("rapidPiInstallMethod", () => {
	it("tells an executable the npm package downloaded, on every platform", () => {
		delete process.env.RAPIDPI_NPM;
		setExecPath("/home/me/.rapid-pi/npm/0.7.0/rapid-pi-linux-x64/pi");
		expect(rapidPiInstallMethod()).toBe("npm");
		setExecPath("/Users/me/.rapid-pi/npm/0.7.0/rapid-pi-darwin-arm64/pi");
		expect(rapidPiInstallMethod()).toBe("npm");
		setExecPath("C:\\Users\\me\\.rapid-pi\\npm\\0.7.0\\rapid-pi-win32-x64\\rapid-pi.exe");
		expect(rapidPiInstallMethod()).toBe("npm");
		// Windows: the package's install script puts the executable in its own bin folder (npm/install.cjs).
		setExecPath("C:\\Users\\me\\AppData\\Roaming\\npm\\node_modules\\rapid-pi\\bin\\rapid-pi.exe");
		expect(rapidPiInstallMethod()).toBe("npm");
	});

	it("tells an executable the installer put in place", () => {
		delete process.env.RAPIDPI_NPM;
		setExecPath("/home/me/.rapid-pi/rapid-pi-linux-x64/pi");
		expect(rapidPiInstallMethod()).toBe("installer");
		setExecPath("C:\\Users\\me\\.rapid-pi\\rapid-pi-win32-x64\\rapid-pi.exe");
		expect(rapidPiInstallMethod()).toBe("installer");
	});
});

describe("rapidPiUpdateEnvironment", () => {
	const saved = { ...process.env };
	afterEach(() => {
		for (const name of ["RAPIDPI_INSTALL", "RAPIDPI_DOWNLOAD_BASE", "RAPIDPI_SOURCE", "RAPIDPI_VERSION"]) {
			if (saved[name] === undefined) delete process.env[name];
			else process.env[name] = saved[name];
		}
	});

	it("updates the installation where it is, to the version decided on, from the signed release", () => {
		const root = process.platform === "win32" ? "D:\\Tools\\Pi" : "/opt/tools/pi";
		setExecPath(join(root, `rapid-pi-${process.platform}-x64`, process.platform === "win32" ? "rapid-pi.exe" : "pi"));
		process.env.RAPIDPI_INSTALL = join(root, "elsewhere");
		process.env.RAPIDPI_DOWNLOAD_BASE = "https://mirror.example";
		process.env.RAPIDPI_SOURCE = "github";
		process.env.RAPIDPI_VERSION = "bolt-v0.1.0";
		process.env.RAPIDPI_INSTALLER_NO_MAIN = "1";
		const env = rapidPiUpdateEnvironment("0.8.0");
		delete process.env.RAPIDPI_INSTALLER_NO_MAIN;
		expect(env.RAPIDPI_INSTALLER_NO_MAIN).toBeUndefined();
		expect(env.RAPIDPI_INSTALL).toBe(root);
		expect(env.RAPIDPI_VERSION).toBe("bolt-v0.8.0");
		expect(env.RAPIDPI_DOWNLOAD_BASE).toBeUndefined();
		expect(env.RAPIDPI_SOURCE).toBeUndefined();
		// PATH stays as the user left it (on Windows the installer would add this folder without asking).
		expect(env.RAPIDPI_NO_PATH).toBe("1");
	});

	it("falls back to RAPIDPI_INSTALL, then ~/.rapid-pi, for an executable outside a variant's folder", () => {
		setExecPath(process.platform === "win32" ? "C:\\bin\\rapid-pi.exe" : "/usr/local/bin/pi");
		delete process.env.RAPIDPI_INSTALL;
		expect(rapidPiInstallDir()).toBeUndefined();
		expect(rapidPiUpdateEnvironment("0.8.0").RAPIDPI_INSTALL).toBe(join(homedir(), ".rapid-pi"));
	});
});

describe("rapidPiInstallerProcess", () => {
	it("runs the installer for this platform, with an exit status that says whether it installed", () => {
		const { command, args } = rapidPiInstallerProcess();
		if (process.platform === "win32") {
			expect(command.toLowerCase()).toMatch(/\\system32\\windowspowershell\\v1\.0\\powershell\.exe$/);
			expect(args.at(-1)).toBe(
				"[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor 3072; try { $installer = irm https://rapid-pi.opensec.in/install.ps1 -ErrorAction Stop } catch { Write-Error $_; exit 1 }; iex $installer; exit $LASTEXITCODE",
			);
		} else {
			expect(command).toBe("sh");
			expect(args[0]).toBe("-c");
			expect(args[1]).toContain("curl -fsSL https://rapid-pi.opensec.in/install.sh");
			expect(args[1]).toContain("wget -qO- https://rapid-pi.opensec.in/install.sh");
		}
	});

	it.skipIf(process.platform === "win32")(
		"downloads with wget where there is no curl; a failed download fails",
		() => {
			const { args } = rapidPiInstallerProcess();
			const bin = mkdtempSync(join(tmpdir(), "rapid-pi-installer-"));
			try {
				// A PATH with only sh and a wget that prints a tiny installer: no curl.
				symlinkSync("/bin/sh", join(bin, "sh"));
				const env = { PATH: bin };
				writeFileSync(join(bin, "wget"), "#!/bin/sh\nprintf 'echo installed-by-wget\\n'\n", { mode: 0o755 });
				const withWget = spawnSync("/bin/sh", args, { env, encoding: "utf8" });
				expect(withWget.status).toBe(0);
				expect(withWget.stdout.trim()).toBe("installed-by-wget");
				writeFileSync(join(bin, "wget"), "#!/bin/sh\nexit 4\n", { mode: 0o755 });
				expect(spawnSync("/bin/sh", args, { env, encoding: "utf8" }).status).toBe(1);
				rmSync(join(bin, "wget"));
				const neither = spawnSync("/bin/sh", args, { env, encoding: "utf8" });
				expect(neither.status).toBe(1);
				expect(neither.stderr).toContain("curl or wget is needed");
			} finally {
				rmSync(bin, { recursive: true, force: true });
			}
		},
	);
});
