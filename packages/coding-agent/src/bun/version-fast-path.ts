// `pi --version` (or -v) alone is answered here, before the rest of Pi is loaded: its modules' top-level code, which runs at
// every start, is two thirds of what that command costs. Anything else on the command line goes to main() as before.
import { VERSION } from "../config.ts";
import { RAPIDPI } from "../rapid-pi.ts";

const args = process.argv.slice(2);
if (args.length === 1 && (args[0] === "--version" || args[0] === "-v")) {
	console.log(
		RAPIDPI
			? `${VERSION} (Rapid Pi ${RAPIDPI.version}, ${process.platform}-${RAPIDPI.variant}, JIT ${RAPIDPI.jit ? "on" : "off"})`
			: VERSION,
	);
	process.exit(0);
}
