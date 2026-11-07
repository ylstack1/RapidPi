import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { parseArgs } from "../src/cli/args.ts";
import { SettingsManager } from "../src/core/settings-manager.ts";
import { main } from "../src/main.ts";
import { getPiBoltPackageFlagError, handleConfigCommand, handlePackageCommand } from "../src/package-manager-cli.ts";

const build = vi.hoisted(() => ({ rapidPi: true }));
vi.mock("../src/rapid-pi.ts", async (importOriginal) => ({
	...(await importOriginal<object>()),
	get RAPIDPI() {
		return build.rapidPi ? { version: "0.8.4", variant: "arm64", jit: false } : undefined;
	},
}));

let originalExitCode: typeof process.exitCode;

beforeEach(() => {
	originalExitCode = process.exitCode;
	process.exitCode = undefined;
	vi.spyOn(console, "error").mockImplementation(() => {});
	vi.spyOn(console, "log").mockImplementation(() => {});
	vi.spyOn(SettingsManager, "create").mockImplementation(() => {
		throw new Error("Preflight must not access settings");
	});
});

afterEach(() => {
	process.exitCode = originalExitCode;
	vi.restoreAllMocks();
});

// #28: package selector preflight must reuse startup argument consumption, not scan option values.
describe.each([true, false])("getPiBoltPackageFlagError (Rapid Pi=%s)", (rapidPi) => {
	beforeEach(() => {
		build.rapidPi = rapidPi;
	});

	test.each([
		"--name",
		"-n",
		"--provider",
		"--model",
		"--api-key",
		"--system-prompt",
		"--append-system-prompt",
		"--session",
		"--session-id",
		"--fork",
		"--session-dir",
		"--models",
		"--tools",
		"-t",
		"--exclude-tools",
		"-xt",
		"--thinking",
		"--export",
		"--extension",
		"-e",
		"--skill",
		"--prompt-template",
		"--theme",
	])("preserves selector-like values of %s", (option) => {
		for (const value of ["--rapid-pi", "--rapid-pi=true", "--rapid-pi="]) {
			const args = [option, value, "--version"];
			expect(parseArgs(args).unknownFlags.has("rapid-pi")).toBe(false);
			expect(getPiBoltPackageFlagError(args)).toBeUndefined();
		}
	});

	test("preserves the reported session-name example and repeated literal prompts", () => {
		const args = [
			"--name",
			"--rapid-pi",
			"--system-prompt",
			"--rapid-pi=false",
			"--append-system-prompt",
			"--rapid-pi",
			"--append-system-prompt",
			"--rapid-pi=true",
			"--version",
		];
		expect(parseArgs(args)).toMatchObject({
			name: "--rapid-pi",
			systemPrompt: "--rapid-pi=false",
			appendSystemPrompt: ["--rapid-pi", "--rapid-pi=true"],
			version: true,
			diagnostics: [],
		});
		expect(getPiBoltPackageFlagError(args)).toBeUndefined();
	});

	test.each([
		["--", "--rapid-pi", "--rapid-pi=true"],
		["--name", "--rapid-pi", "--", "--rapid-pi=true"],
		["--print", "--", "--rapid-pi"],
		["--print", "---literal --rapid-pi"],
		["@--rapid-pi"],
		["--rapid-pi-other"],
	])("does not treat positional or unrelated tokens as selectors: %j", (...args) => {
		expect(getPiBoltPackageFlagError(args)).toBeUndefined();
	});

	// #28: real selectors still fail at main's preflight, before settings reads or migrations.
	test.each([
		["--rapid-pi"],
		["--rapid-pi=true"],
		["--rapid-pi=", "--rapid-pi"],
		["--rapid-pi", "--rapid-pi=false"],
		["--name", "--rapid-pi=true", "--rapid-pi"],
		["--name", "--", "--rapid-pi"],
		["--mode", "--rapid-pi"],
		["--use-theme", "--rapid-pi"],
		["--tui-mode", "--rapid-pi"],
		["--list-models", "--rapid-pi"],
		["--print", "--rapid-pi"],
		["--custom-extension-flag", "--rapid-pi"],
		["auth", "--rapid-pi"],
		["mcp", "--rapid-pi"],
	])("rejects actual unsupported selector tokens: %j", async (...args) => {
		const expected = rapidPi
			? "--rapid-pi is only supported by install, remove, and uninstall"
			: "--rapid-pi requires a Rapid Pi build";
		expect(getPiBoltPackageFlagError(args)).toBe(expected);
		await main(args);
		expect(process.exitCode).toBe(1);
		expect(console.error).toHaveBeenCalledWith(expect.stringContaining(expected));
		expect(SettingsManager.create).not.toHaveBeenCalled();
	});

	// #28: startup value handling must not relax package-command selector validation or help precedence.
	test.each(["install", "remove", "uninstall"])("preserves %s selector grammar", async (command) => {
		const args = [command, "npm:fixture", "--rapid-pi", "--help"];
		expect(getPiBoltPackageFlagError(args)).toBe(rapidPi ? undefined : "--rapid-pi requires a Rapid Pi build");
		expect(await handlePackageCommand(args)).toBe(true);
		expect(process.exitCode).toBe(rapidPi ? undefined : 1);
		expect(SettingsManager.create).not.toHaveBeenCalled();
	});

	test.each([
		["install", "npm:fixture", "--rapid-pi=true"],
		["install", "npm:fixture", "--rapid-pi", "--rapid-pi=false"],
		["remove", "npm:fixture", "--rapid-pi=", "--rapid-pi"],
		["update", "--extension", "--rapid-pi=true"],
	])("rejects every valued package selector: %j", async (...args) => {
		expect(getPiBoltPackageFlagError(args)).toBe(
			rapidPi ? "--rapid-pi does not take a value" : "--rapid-pi requires a Rapid Pi build",
		);
		await handlePackageCommand(args);
		expect(process.exitCode).toBe(1);
		expect(SettingsManager.create).not.toHaveBeenCalled();
	});

	test.each(["list", "update", "config"])("rejects unsupported %s selectors before settings", async (command) => {
		const args = command === "update" ? [command, "--extension", "--rapid-pi"] : [command, "--rapid-pi"];
		expect(getPiBoltPackageFlagError(args)).toBe(
			rapidPi
				? "--rapid-pi is only supported by install, remove, and uninstall"
				: "--rapid-pi requires a Rapid Pi build",
		);
		if (command === "config") await handleConfigCommand(args);
		else await handlePackageCommand(args);
		expect(process.exitCode).toBe(1);
		expect(SettingsManager.create).not.toHaveBeenCalled();
	});

	test("preserves the package preflight cutoff without adding package end-of-options support", async () => {
		const args = ["install", "npm:fixture", "--", "--rapid-pi=true"];
		expect(getPiBoltPackageFlagError(args)).toBeUndefined();
		await handlePackageCommand(args);
		expect(process.exitCode).toBe(1);
		expect(console.error).toHaveBeenCalledWith(expect.stringContaining('Unknown option -- for "install".'));
		expect(SettingsManager.create).not.toHaveBeenCalled();
	});
});
