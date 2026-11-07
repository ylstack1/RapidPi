import { homedir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, test } from "vitest";
import { rapidPiUpdateEnvironment } from "../src/rapid-pi.ts";

const realExecPath = process.execPath;
const realInstall = process.env.RAPIDPI_INSTALL;

function runningFrom(execPath: string): void {
	Object.defineProperty(process, "execPath", { value: execPath, configurable: true });
}

afterEach(() => {
	runningFrom(realExecPath);
	if (realInstall === undefined) delete process.env.RAPIDPI_INSTALL;
	else process.env.RAPIDPI_INSTALL = realInstall;
});

describe("rapid-pi update installs where Rapid Pi is", () => {
	test("an installation in the home directory", () => {
		delete process.env.RAPIDPI_INSTALL;
		runningFrom(join(homedir(), ".rapid-pi", "rapid-pi-linux-x64", "pi"));
		expect(rapidPiUpdateEnvironment("0.8.0").RAPIDPI_INSTALL).toBe(join(homedir(), ".rapid-pi"));
	});

	test("an installation outside it, such as /opt/rapid-pi", () => {
		delete process.env.RAPIDPI_INSTALL;
		runningFrom("/opt/rapid-pi/rapid-pi-darwin-arm64/pi-bin");
		expect(rapidPiUpdateEnvironment("0.8.0").RAPIDPI_INSTALL).toBe("/opt/rapid-pi");
	});

	test("a build run from its output folder gets the default installation", () => {
		delete process.env.RAPIDPI_INSTALL;
		runningFrom("/home/me/Rapid Pi/out/rapid-pi/pi");
		expect(rapidPiUpdateEnvironment("0.8.0").RAPIDPI_INSTALL).toBe(join(homedir(), ".rapid-pi"));
	});

	// The installation that is running is the one updated: a RAPIDPI_INSTALL left in the environment would put a second copy
	// elsewhere and leave this one as it was. It is used for an executable that is not in an installation.
	test("the running installation wins over RAPIDPI_INSTALL", () => {
		process.env.RAPIDPI_INSTALL = "/srv/tools/rapid-pi";
		runningFrom("/opt/rapid-pi/rapid-pi-linux-x64/pi");
		expect(rapidPiUpdateEnvironment("0.8.0").RAPIDPI_INSTALL).toBe("/opt/rapid-pi");
	});

	test("RAPIDPI_INSTALL, for a build run from its output folder", () => {
		process.env.RAPIDPI_INSTALL = "/srv/tools/rapid-pi";
		runningFrom("/home/me/Rapid Pi/out/rapid-pi/pi");
		expect(rapidPiUpdateEnvironment("0.8.0").RAPIDPI_INSTALL).toBe("/srv/tools/rapid-pi");
	});
});
