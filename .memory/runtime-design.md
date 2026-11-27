# Runtime Design — Ahead-of-Time Compiled Pi

## Executable layout

A Rapid Pi executable is a Bun single-file executable with two extra page-aligned sections appended:

```
┌──────────────────────────────┐
│ Bun runtime                  │  Bun + JavaScriptCore (AOT engine), ICU, statically linked
├──────────────────────────────┤
│ Bundled program              │  Pi's modules (Bun --compile embeds them, bytecode optional)
├──────────────────────────────┤
│ Static heap                  │  JS heap after Pi's modules loaded and evaluated (relocatable)
├──────────────────────────────┤
│ Code image                   │  Machine code for every function, regexps, metadata
├──────────────────────────────┤
│ Trailer                      │  Offsets, engine stamp, CPU features/options
└──────────────────────────────┘
```

## Build pipeline (`scripts/build-pi.sh`)

Runs `bun build --compile --bytecode` with these settings:

| Setting | Effect |
|---|---|
| `BUN_STATIC_HEAP=1` | Build the static heap |
| `BUN_AOT=1` | Compile every function AOT into the code image |
| `BUN_AOT_JIT=0` | Mark code as JIT off (default build) |
| `BUN_JSC_omitBytecodeFromStaticHeap=1` | Leave bytecode out of heap; run from machine code only |
| `BUN_JSC_useAOTLoopSplitting=1` | Give loops a fast copy |
| `--bytecode-order=profiles/pi-<ver>/bytecode.order` | Lay out code/heap in training-session first-use order |
| `BUN_JSC_aotRegExpsPath=profiles/pi-<ver>/regexps.txt` | AOT-compile runtime-built regexps |

Steps:
1. **Bundle** — resolve Pi's module graph from `dist/bun/cli.js` + worker entries, transpile and link
2. **Bytecode** — compile every module/function to JSC bytecode, prelink module graph
3. **Evaluate into heap** — load/evaluate Pi's modules in a VM, serialize heap as relocatable static heap
4. **Compile** — AOT compiler turns every function's bytecode into machine code, linked as one image
5. **Append** — heap, image, trailer appended to executable

## AOT compiler pipeline

Bytecode -> AOT graph (SSA, type inference, range analysis) -> Inlining / loop splitting / escape analysis -> B3 optimizer -> Air register allocation -> x86-64/ARM64 machine code

Key techniques:
- Whole-program type inference (locals, module bindings, constants)
- Guards for unprovable assumptions, with generic fallback paths (no deoptimization)
- Stubs for shared slow paths (property access, calls, arithmetic on unknown types)
- Inline caches for property access
- Intrinsics for hot built-ins (`Math.*`, `charCodeAt`, `push`/`pop`, `Map`/`Set`)
- Inlining of known small functions across the program
- Array callbacks (`forEach`, `map`, `filter`, `reduce`) inlined with callback
- Loop splitting: fast copy without slow paths, falls back to generic copy
- Number encoding: doubles boxed as int32 where possible
- Regular expressions compiled by Yarr into the same image (literals + training-recorded runtime patterns)

## Launch sequence

1. Bun reads the trailer and checks it against the engine:
   - Engine stamp must match exactly
   - CPU must have the instruction-set features the code was compiled for
   - Expected address range must be available
2. **Static heap mapped** at fixed address (Linux x86-64: `0x200000000000`; Linux ARM64: `0x1000000000`; macOS: linked address), copy-on-write from the executable
3. **Code image mapped** executable and registered with the VM
4. Pi's modules run, then `main()` — nothing parsed or compiled at launch; only top-level code and `node:*` imports run per start

If any check fails, falls back to bytecode (correct, slower start). Checks: engine stamp, CPU features, address space (`ulimit -v`), `BUN_AOT=0`.

## JIT off (default) vs JIT on

| | JIT off | JIT on (`--jit on`) |
|---|---|---|
| Pi's own code | AOT machine code | AOT machine code (no tier-up) |
| Run-time-loaded code (plugins, `eval`) | Interpreter | Interpreter -> baseline JIT -> DFG -> FTL |
| Memory | Lowest | Higher once JIT compiles anything |
| Best for | Pi with compiled-in plugins | Heavy use of run-time-loaded plugins |

## Platform ports

**Linux x86-64:** Rapid Pi's WebKit patch adds the x86-64 back end (upstream targets ARM64 only). Register convention for stubs on x86-64's smaller register file; all stub families ported (property access, calls, arithmetic, iteration, intrinsics, Map/Set, bit ops on doubles). Realms commit memory lazily. Address-space limits handled (static region/pools shrink or fall back). Workers share the static heap. Linked against glibc 2.17 sysroot with static ICU; portable (glibc 2.17+).

**Linux ARM64:** Same ARM64 code gen as macOS. Static region at 64 GB (`0x1000000000`) because many ARM64 kernels give only 512 GB (39-bit) or 4 TB (42-bit) address space. Heap/code image laid out in 64 KB pages (works with 4/16/64 KB kernel pages). One build for every ARM64 CPU (`BUN_AOT_CPU=baseline` — leaves out LSE, JSCVT, FP16, FRINTTS, SHA3, DotProd). Android/Termux: runs as Linux build with Termux's glibc; installer overwrites PT_INTERP with Termux's loader; returns ENOSYS for blocked syscalls via SIGSYS handler.

**macOS ARM64:** ARM64 code gen is upstream's. `__BUN` section of Mach-O starts on 16 KB page. Code signature (ad hoc) covers the executable so code image maps executable from file. `pi` is a small launcher (`scripts/lib/darwin-launcher.c`) that starts `pi-bin` beside it via `posix_spawn`. libSystem imports bound through a custom `libsystem/` re-export directory beside `pi-bin`. Hot pages preloaded via `F_RDADVISE`. Programs Pi starts run through `pi-spawn` helper, not `pi-bin`'s descendant. Linked against Xcode SDK, macOS 13+, `-mcpu=apple-m1`, system ICU.

**Windows x64:** Same steps via PowerShell scripts with Visual Studio environment. x64 build runs on Windows on ARM64 via x64 emulation.

## Limitations

- No JIT at run time for loaded code on default build (interpreted — compile plugins in or use JIT-on build)
- No type feedback; AOT can only use what it proves
- Each executable built for one Pi version and one engine (engine stamp checked at launch)
- Windows on ARM64 runs x64 build in emulation
